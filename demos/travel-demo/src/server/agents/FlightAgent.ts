import { BaseAgent } from './BaseAgent.js';
import type {
  AgentMessage,
  TripConstraints,
  FlightBooking,
} from '@shared/types';
import { generateFlights } from '../utils/mockData.js';

/**
 * Flight Agent - Searches and ranks flight options
 */
export class FlightAgent extends BaseAgent {
  constructor(tripId: string) {
    super('flight', tripId);
  }

  protected async processMessage(message: AgentMessage): Promise<void> {
    const { action, data } = message.payload;

    switch (action) {
      case 'startPlanning':
        await this.handleStartPlanning(data as { constraints: TripConstraints });
        break;

      case 'requestAlternative':
        await this.handleAlternativeRequest(message.from);
        break;

      default:
        console.log(`FlightAgent: Unknown action ${action}`);
    }
  }

  private async handleStartPlanning(data: {
    constraints: TripConstraints;
  }): Promise<void> {
    this.updateStatus('working');

    // Simulate search delay
    await new Promise((resolve) => setTimeout(resolve, 800));

    const results = await this.search(data.constraints);

    this.updateStatus('completed');
  }

  private async handleAlternativeRequest(requesterId: string): Promise<void> {
    this.updateStatus('working');

    // Generate alternative options
    await new Promise((resolve) => setTimeout(resolve, 500));

    this.updateStatus('completed');
  }

  async search(constraints: TripConstraints): Promise<FlightBooking[]> {
    // Generate mock flight data
    const flights = generateFlights(constraints, this.id);

    // Select top 3-5 options
    const topFlights = flights.slice(0, 5);

    // Send results to coordinator
    await this.sendResults('coordinator', {
      flights: topFlights,
      category: 'flights',
    });

    // Notify budget agent of cheapest option cost
    if (topFlights.length > 0) {
      await this.notifyBudget('flights', topFlights[0].price);
    }

    return topFlights;
  }

  /**
   * Rank flights by composite score
   */
  private rankFlights(
    flights: FlightBooking[],
    constraints: TripConstraints
  ): FlightBooking[] {
    return flights.sort((a, b) => {
      const scoreA = this.calculateFlightScore(a, constraints);
      const scoreB = this.calculateFlightScore(b, constraints);
      return scoreB - scoreA; // Higher score first
    });
  }

  /**
   * Calculate flight score based on multiple factors
   */
  private calculateFlightScore(
    flight: FlightBooking,
    constraints: TripConstraints
  ): number {
    // Normalize price (inverse - lower is better)
    const maxPrice = constraints.budget.max * 0.4;
    const priceScore = 1 - flight.price / maxPrice;

    // Normalize duration (inverse - shorter is better)
    const durationScore = 1 - flight.duration / 1200; // 20 hours max

    // Stops score (fewer is better)
    const stopsScore = 1 - flight.stops / 3;

    // Weighted combination
    return priceScore * 0.4 + durationScore * 0.3 + stopsScore * 0.3;
  }
}
