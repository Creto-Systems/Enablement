import type {
  Agent,
  TripConstraints,
  Itinerary,
  FlightBooking,
  HotelBooking,
  Activity,
  BudgetAnalysis,
  Conflict,
  AgentMessage,
} from '@shared/types';
import { FlightAgent } from '../agents/FlightAgent.js';
import { HotelAgent } from '../agents/HotelAgent.js';
import { ActivityAgent } from '../agents/ActivityAgent.js';
import { BudgetAgent } from '../agents/BudgetAgent.js';
import { MessageBroker } from './MessageBroker.js';
import { StateManager } from './StateManager.js';
import { ConflictResolver } from './ConflictResolver.js';
import { generateId } from '../utils/encryption.js';

/**
 * Agent Coordinator - Orchestrates multi-agent trip planning
 */
export class AgentCoordinator {
  private agents: Map<string, Agent> = new Map();
  private messageBroker: MessageBroker;
  private stateManager: StateManager;
  private conflictResolver: ConflictResolver;
  private tripId: string;

  // Specialized agent instances
  private flightAgent?: FlightAgent;
  private hotelAgent?: HotelAgent;
  private activityAgent?: ActivityAgent;
  private budgetAgent?: BudgetAgent;

  // Planning results
  private results: {
    flights?: FlightBooking[];
    hotels?: HotelBooking[];
    activities?: Activity[];
    budget?: BudgetAnalysis;
  } = {};

  private eventCallback?: (event: CoordinatorEvent) => void;

  constructor(tripId: string) {
    this.tripId = tripId;
    this.messageBroker = new MessageBroker();
    this.stateManager = new StateManager(tripId);
    this.conflictResolver = new ConflictResolver();
  }

  /**
   * Set event callback for real-time updates
   */
  setEventCallback(callback: (event: CoordinatorEvent) => void): void {
    this.eventCallback = callback;
  }

  /**
   * Emit event to callback
   */
  private emitEvent(event: CoordinatorEvent): void {
    if (this.eventCallback) {
      this.eventCallback(event);
    }
  }

  /**
   * Initialize and spawn all agents
   */
  async initializeAgents(): Promise<void> {
    // Create agents
    this.flightAgent = new FlightAgent(this.tripId);
    this.hotelAgent = new HotelAgent(this.tripId);
    this.activityAgent = new ActivityAgent(this.tripId);
    this.budgetAgent = new BudgetAgent(this.tripId);

    const allAgents = [
      this.flightAgent,
      this.hotelAgent,
      this.activityAgent,
      this.budgetAgent,
    ];

    // Initialize all agents in parallel
    await Promise.all(allAgents.map((agent) => agent.initialize()));

    // Register with message broker
    for (const agent of allAgents) {
      this.messageBroker.registerAgent(
        agent.id,
        agent.publicKey,
        (agent as any).privateKey // Access private key for broker
      );

      this.messageBroker.registerHandler(agent.id, (message) =>
        agent.handleMessage(message)
      );

      agent.setMessageBroker(this.messageBroker);

      this.agents.set(agent.id, agent.getInfo());
    }

    // Emit agent status
    this.emitEvent({
      type: 'agents:initialized',
      data: Array.from(this.agents.values()),
    });
  }

  /**
   * Start trip planning process
   */
  async planTrip(constraints: TripConstraints): Promise<Itinerary> {
    if (!this.flightAgent || !this.hotelAgent || !this.activityAgent || !this.budgetAgent) {
      throw new Error('Agents not initialized');
    }

    // Update state
    this.stateManager.update('constraints', constraints);

    // Broadcast start planning to all agents
    const startMessage: AgentMessage = {
      id: generateId(),
      from: 'coordinator',
      to: 'all',
      type: 'request',
      payload: {
        action: 'startPlanning',
        data: { constraints },
      },
      encrypted: false,
      timestamp: Date.now(),
      correlationId: generateId(),
    };

    // Notify budget agent first
    await this.budgetAgent.handleMessage({
      ...startMessage,
      to: this.budgetAgent.id,
    });

    // Execute agents in parallel
    this.emitEvent({
      type: 'planning:started',
      data: { tripId: this.tripId },
    });

    const [flights, hotels, activities] = await Promise.all([
      this.executeFlightAgent(constraints),
      this.executeHotelAgent(constraints),
      this.executeActivityAgent(constraints),
    ]);

    // Store results
    this.results = { flights, hotels, activities };

    // Analyze budget
    await this.executeBudgetAgent();

    // Build itinerary
    const itinerary = this.buildItinerary();

    // Detect conflicts
    const conflicts = this.conflictResolver.detectConflicts(itinerary);

    if (conflicts.length > 0) {
      this.emitEvent({
        type: 'conflicts:detected',
        data: conflicts,
      });

      // Resolve conflicts
      const resolved = this.conflictResolver.resolveConflicts(
        conflicts,
        itinerary
      );
      itinerary.conflicts = conflicts;

      this.emitEvent({
        type: 'conflicts:resolved',
        data: conflicts,
      });
    }

    this.emitEvent({
      type: 'planning:complete',
      data: itinerary,
    });

    return itinerary;
  }

  /**
   * Execute flight agent search
   */
  private async executeFlightAgent(
    constraints: TripConstraints
  ): Promise<FlightBooking[]> {
    if (!this.flightAgent) throw new Error('Flight agent not initialized');

    this.emitEvent({
      type: 'agent:working',
      data: { agentId: this.flightAgent.id, type: 'flight' },
    });

    const flights = await this.flightAgent.search(constraints);

    this.emitEvent({
      type: 'agent:completed',
      data: { agentId: this.flightAgent.id, type: 'flight' },
    });

    this.emitEvent({
      type: 'itinerary:update',
      data: { category: 'flights', items: flights },
    });

    return flights;
  }

  /**
   * Execute hotel agent search
   */
  private async executeHotelAgent(
    constraints: TripConstraints
  ): Promise<HotelBooking[]> {
    if (!this.hotelAgent) throw new Error('Hotel agent not initialized');

    this.emitEvent({
      type: 'agent:working',
      data: { agentId: this.hotelAgent.id, type: 'hotel' },
    });

    const hotels = await this.hotelAgent.search(constraints);

    this.emitEvent({
      type: 'agent:completed',
      data: { agentId: this.hotelAgent.id, type: 'hotel' },
    });

    this.emitEvent({
      type: 'itinerary:update',
      data: { category: 'hotels', items: hotels },
    });

    return hotels;
  }

  /**
   * Execute activity agent search
   */
  private async executeActivityAgent(
    constraints: TripConstraints
  ): Promise<Activity[]> {
    if (!this.activityAgent) throw new Error('Activity agent not initialized');

    this.emitEvent({
      type: 'agent:working',
      data: { agentId: this.activityAgent.id, type: 'activity' },
    });

    const activities = await this.activityAgent.search(constraints);

    this.emitEvent({
      type: 'agent:completed',
      data: { agentId: this.activityAgent.id, type: 'activity' },
    });

    this.emitEvent({
      type: 'itinerary:update',
      data: { category: 'activities', items: activities },
    });

    return activities;
  }

  /**
   * Execute budget agent analysis
   */
  private async executeBudgetAgent(): Promise<void> {
    if (!this.budgetAgent) throw new Error('Budget agent not initialized');

    // Budget agent already received cost updates
    // Trigger final analysis
    const message: AgentMessage = {
      id: generateId(),
      from: 'coordinator',
      to: this.budgetAgent.id,
      type: 'request',
      payload: {
        action: 'analyzeComplete',
      },
      encrypted: false,
      timestamp: Date.now(),
      correlationId: generateId(),
    };

    await this.budgetAgent.handleMessage(message);
  }

  /**
   * Build final itinerary from results
   */
  private buildItinerary(): Itinerary {
    const { flights = [], hotels = [], activities = [] } = this.results;

    const totalCost =
      (flights[0]?.price || 0) +
      (hotels[0]?.totalPrice || 0) +
      activities.reduce((sum, a) => sum + a.price, 0);

    return {
      id: generateId(),
      tripId: this.tripId,
      flights,
      hotels,
      activities,
      totalCost,
      optimizationScore: 85, // Mock score
      conflicts: [],
      status: 'ready',
    };
  }

  /**
   * Get all agent statuses
   */
  getAgentStatuses(): Agent[] {
    return Array.from(this.agents.values());
  }

  /**
   * Get specific agent status
   */
  getAgentStatus(agentId: string): Agent | undefined {
    return this.agents.get(agentId);
  }

  /**
   * Destroy all agents and cleanup
   */
  async destroy(): Promise<void> {
    const agents = [
      this.flightAgent,
      this.hotelAgent,
      this.activityAgent,
      this.budgetAgent,
    ];

    await Promise.all(
      agents.filter(Boolean).map((agent) => agent!.destroy())
    );

    this.agents.clear();
    this.messageBroker.clear();
  }
}

// Event types
export type CoordinatorEvent =
  | { type: 'agents:initialized'; data: Agent[] }
  | { type: 'planning:started'; data: { tripId: string } }
  | { type: 'agent:working'; data: { agentId: string; type: string } }
  | { type: 'agent:completed'; data: { agentId: string; type: string } }
  | { type: 'itinerary:update'; data: { category: string; items: any[] } }
  | { type: 'conflicts:detected'; data: Conflict[] }
  | { type: 'conflicts:resolved'; data: Conflict[] }
  | { type: 'planning:complete'; data: Itinerary };
