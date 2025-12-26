# Phase 3: Architecture - Travel Demo Multi-Agent Trip Planner

## 1. System Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Client Layer (React)                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │ TripWizard   │  │ AgentPanel   │  │ ItineraryView│             │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘             │
│         │                  │                  │                      │
│         └──────────────────┴──────────────────┘                      │
│                            │                                         │
│                  ┌─────────▼─────────┐                              │
│                  │   State Manager   │                              │
│                  │    (Zustand)      │                              │
│                  └─────────┬─────────┘                              │
└────────────────────────────┼──────────────────────────────────────┘
                             │ WebSocket (Socket.IO)
                             │
┌────────────────────────────▼──────────────────────────────────────┐
│                      Server Layer (Node.js)                        │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │               Agent Coordinator                              │  │
│  │  - Agent lifecycle management                                │  │
│  │  - Message routing and encryption (creto-messaging)          │  │
│  │  - State consistency management                              │  │
│  │  - Conflict detection and resolution                         │  │
│  └─────────────┬───────────────────────────────────────────────┘  │
│                │                                                    │
│       ┌────────┴─────────┬──────────────┬──────────────┐          │
│       │                  │              │              │          │
│  ┌────▼─────┐  ┌────────▼───┐  ┌───────▼────┐  ┌──────▼─────┐   │
│  │  Flight  │  │   Hotel    │  │  Activity  │  │   Budget   │   │
│  │  Agent   │  │   Agent    │  │   Agent    │  │   Agent    │   │
│  └────┬─────┘  └────────┬───┘  └───────┬────┘  └──────┬─────┘   │
│       │                 │              │              │          │
│       └─────────────────┴──────────────┴──────────────┘          │
│                         │                                         │
│               ┌─────────▼─────────┐                               │
│               │  Message Broker   │                               │
│               │  (E2EE Queue)     │                               │
│               └───────────────────┘                               │
└───────────────────────────────────────────────────────────────────┘
```

## 2. Component Architecture

### 2.1 Client Components

#### TripWizard Component
```
TripWizard
├── StepIndicator
├── DestinationInput
├── DateRangePicker
├── BudgetSlider
├── PreferencesForm
│   ├── ActivityTypeSelector
│   ├── AccommodationSelector
│   └── PaceSelector
└── ActionButtons
```

**Responsibilities:**
- Collect user trip requirements
- Validate inputs
- Initiate trip planning
- Progress indication

**State:**
```typescript
interface WizardState {
  currentStep: number;
  formData: TripRequest;
  validation: ValidationErrors;
  isSubmitting: boolean;
}
```

#### AgentPanel Component
```
AgentPanel
├── AgentCard (x4)
│   ├── AgentAvatar
│   ├── StatusIndicator
│   ├── ProgressBar
│   └── MessageLog
└── CoordinationGraph
```

**Responsibilities:**
- Display agent status in real-time
- Show agent communication flow
- Visualize work progress
- Message transparency

**State:**
```typescript
interface AgentPanelState {
  agents: Map<AgentId, AgentStatus>;
  messages: AgentMessage[];
  coordinationGraph: GraphData;
}
```

#### ItineraryView Component
```
ItineraryView
├── Timeline
│   ├── FlightSegment
│   ├── HotelSegment
│   └── ActivitySegment
├── MapView
├── BudgetSummary
└── ItemActions
    ├── AcceptButton
    ├── RejectButton
    └── AlternativesButton
```

**Responsibilities:**
- Display itinerary timeline
- Show map with locations
- Enable item interactions
- Real-time updates

**State:**
```typescript
interface ItineraryState {
  itinerary: Itinerary;
  selectedItem: string | null;
  viewMode: 'timeline' | 'map' | 'list';
  pendingChanges: ItemChange[];
}
```

#### BudgetTracker Component
```
BudgetTracker
├── BudgetGauge
├── CategoryBreakdown
│   ├── FlightCosts
│   ├── HotelCosts
│   └── ActivityCosts
├── OptimizationSuggestions
└── AlertBanner
```

**Responsibilities:**
- Real-time budget monitoring
- Cost breakdown visualization
- Display optimization suggestions
- Budget alerts

**State:**
```typescript
interface BudgetState {
  totalCost: number;
  budget: Budget;
  breakdown: CostBreakdown;
  alerts: BudgetAlert[];
  suggestions: OptimizationSuggestion[];
}
```

### 2.2 Server Components

#### Agent Coordinator
```
AgentCoordinator
├── AgentRegistry
├── MessageRouter
├── StateManager
├── ConflictResolver
└── PerformanceMonitor
```

**Responsibilities:**
- Spawn and manage agent lifecycle
- Route encrypted messages between agents
- Maintain consistent trip state
- Detect and resolve conflicts
- Monitor agent performance

**API:**
```typescript
class AgentCoordinator {
  // Agent management
  spawnAgent(type: AgentType): Agent
  destroyAgent(agentId: string): void
  getAgent(agentId: string): Agent

  // Messaging
  routeMessage(message: EncryptedMessage): void
  broadcastMessage(message: Message, recipients: string[]): void

  // State management
  getState(): TripState
  updateState(path: string, value: any): void

  // Coordination
  detectConflicts(itinerary: Itinerary): Conflict[]
  resolveConflicts(conflicts: Conflict[]): Itinerary

  // Monitoring
  getAgentMetrics(): AgentMetrics[]
}
```

#### Flight Agent
```
FlightAgent extends BaseAgent
├── SearchEngine
├── RankingAlgorithm
├── ResultFormatter
└── MessageHandler
```

**Responsibilities:**
- Search flight options (mock data)
- Rank by price, duration, stops
- Format results for coordinator
- Communicate via creto-messaging

**Search Logic:**
```typescript
class FlightAgent extends BaseAgent {
  async search(constraints: TripConstraints): Promise<FlightResult[]> {
    // Generate flight options
    const routes = this.generateRoutes(constraints);
    const flights = await this.searchFlights(routes);

    // Rank results
    const ranked = this.rankFlights(flights, constraints);

    // Send to coordinator
    await this.sendResults('coordinator', ranked);

    // Notify budget agent
    await this.notifyBudget('flights', ranked[0].price);

    return ranked;
  }
}
```

#### Hotel Agent
```
HotelAgent extends BaseAgent
├── LocationClusterAnalyzer
├── HotelSearchEngine
├── ScoringAlgorithm
└── MessageHandler
```

**Responsibilities:**
- Identify optimal hotel locations
- Search hotels by clusters
- Score based on preferences
- Ensure diverse options

**Search Logic:**
```typescript
class HotelAgent extends BaseAgent {
  async search(constraints: TripConstraints): Promise<HotelResult[]> {
    // Identify location clusters
    const clusters = await this.analyzeClusters(constraints.destination);

    // Search hotels in each cluster
    const hotels = await this.searchHotels(clusters, constraints);

    // Score and rank
    const ranked = this.rankHotels(hotels, constraints);

    // Send results
    await this.sendResults('coordinator', ranked);
    await this.notifyBudget('hotels', ranked[0].totalPrice);

    return ranked;
  }
}
```

#### Activity Agent
```
ActivityAgent extends BaseAgent
├── InterestMatcher
├── ScheduleBuilder
├── DiversityOptimizer
└── MessageHandler
```

**Responsibilities:**
- Match activities to interests
- Build daily schedules
- Ensure activity diversity
- Optimize pacing

**Search Logic:**
```typescript
class ActivityAgent extends BaseAgent {
  async search(constraints: TripConstraints): Promise<ActivitySchedule[]> {
    // Search by interests
    const activities = await this.searchByInterests(
      constraints.preferences.activityTypes,
      constraints.destination
    );

    // Build daily schedule
    const schedule = this.buildSchedule(
      activities,
      constraints.startDate,
      constraints.endDate,
      constraints.preferences.pace
    );

    // Send results
    await this.sendResults('coordinator', schedule);
    await this.notifyBudget('activities', this.calculateTotal(schedule));

    return schedule;
  }
}
```

#### Budget Agent
```
BudgetAgent extends BaseAgent
├── CostAggregator
├── ConstraintValidator
├── OptimizationEngine
└── MessageHandler
```

**Responsibilities:**
- Aggregate costs from all agents
- Validate against budget
- Generate optimizations
- Send real-time alerts

**Analysis Logic:**
```typescript
class BudgetAgent extends BaseAgent {
  async analyze(costs: CostData, budget: Budget): Promise<BudgetAnalysis> {
    // Aggregate costs
    const total = this.aggregateCosts(costs);

    // Validate constraints
    const overBudget = total > budget.max;
    const underBudget = total < budget.min;

    // Generate suggestions
    let suggestions = [];
    if (overBudget) {
      suggestions = this.generateReductions(costs, total - budget.max);
    } else if (underBudget) {
      suggestions = this.generateUpgrades(costs, budget.max - total);
    }

    // Send analysis
    const analysis = { total, overBudget, suggestions };
    await this.sendResults('coordinator', analysis);

    return analysis;
  }
}
```

### 2.3 Message Broker Design

#### Creto-Messaging Integration
```
MessageBroker
├── EncryptionService (creto-messaging)
│   ├── KeyManager (Ed25519)
│   ├── Encryptor (AES-256-GCM)
│   └── Signer (Ed25519)
├── MessageQueue (Priority-based)
│   ├── CriticalQueue
│   ├── HighQueue
│   ├── MediumQueue
│   └── LowQueue
├── Router
└── DeadLetterQueue
```

**Message Flow:**
```
1. Agent creates message
2. Broker signs with sender's private key
3. Broker encrypts with recipient's public key
4. Message queued by priority
5. Router delivers to recipient
6. Recipient decrypts with private key
7. Recipient verifies signature
8. Message processed
```

**Encryption Schema:**
```typescript
interface EncryptedMessage {
  envelope: {
    from: string;
    to: string;
    timestamp: number;
    correlationId: string;
  };
  payload: string; // Encrypted JSON
  signature: string; // Ed25519 signature
  nonce: string; // For AES-GCM
}

interface DecryptedPayload {
  type: 'request' | 'response' | 'notification' | 'error';
  action: string;
  data: any;
  constraints?: Constraints;
}
```

## 3. Data Flow Diagrams

### 3.1 Trip Planning Flow
```
User Input → Client State → WebSocket → Server Coordinator
                                              ↓
                                    Spawn Agents (4x)
                                              ↓
                                    Broadcast Constraints (E2EE)
                                              ↓
                        ┌────────────────────┴───────────────────┐
                        ↓                    ↓                   ↓
                  Flight Agent         Hotel Agent       Activity Agent
                        ↓                    ↓                   ↓
                   Search Flights      Search Hotels      Search Activities
                        ↓                    ↓                   ↓
                        └────────────────────┬───────────────────┘
                                             ↓
                                       Budget Agent
                                             ↓
                                    Aggregate & Validate
                                             ↓
                                        Coordinator
                                             ↓
                                    Detect Conflicts
                                             ↓
                                    Resolve Conflicts
                                             ↓
                                    Build Itinerary
                                             ↓
                                    WebSocket → Client
                                             ↓
                                    Display to User
```

### 3.2 Secure Message Flow
```
Sender Agent
    ↓
Create Message {type, action, data}
    ↓
Sign with Private Key (Ed25519)
    ↓
Encrypt with Recipient Public Key (AES-256-GCM)
    ↓
Message Broker
    ↓
Queue by Priority
    ↓
Route to Recipient
    ↓
Recipient Agent
    ↓
Decrypt with Private Key
    ↓
Verify Signature
    ↓
Process Message
    ↓
Send Response (repeat flow)
```

### 3.3 State Update Flow
```
Agent Updates Itinerary Item
    ↓
Acquire Lock on State Path
    ↓
Update State Version
    ↓
Release Lock
    ↓
Broadcast State Change (E2EE)
    ↓
Other Agents Receive Update
    ↓
Sync Local State
    ↓
Coordinator Validates Consistency
    ↓
Stream Update to Client (WebSocket)
    ↓
Client UI Re-renders
```

## 4. Technology Stack Details

### 4.1 Frontend Stack
```yaml
Framework: React 18.2+
Language: TypeScript 5.3+
Build Tool: Vite 5.0+
State Management: Zustand 4.4+
WebSocket: Socket.IO Client 4.6+
Styling: TailwindCSS 3.4+
UI Components: Headless UI
Charts: Recharts
Maps: Leaflet (optional)
Testing:
  - Vitest
  - React Testing Library
  - Mock Service Worker
```

### 4.2 Backend Stack
```yaml
Runtime: Node.js 20+
Framework: Express 4.18+
Language: TypeScript 5.3+
WebSocket: Socket.IO 4.6+
Messaging: creto-messaging (E2EE)
Encryption: Ed25519, AES-256-GCM
State: In-memory (Map-based)
Testing:
  - Vitest
  - Supertest
  - Socket.IO Mock
```

### 4.3 Development Tools
```yaml
Package Manager: npm
Linting: ESLint 8+
Formatting: Prettier
Type Checking: tsc --noEmit
Git Hooks: Husky + lint-staged
```

## 5. Security Architecture

### 5.1 Creto-Messaging Protocol

**Key Generation:**
```typescript
// Each agent gets Ed25519 keypair
const keypair = await generateEd25519KeyPair();
agent.publicKey = keypair.publicKey;
agent.privateKey = keypair.privateKey;

// Store public keys in registry
coordinator.registerPublicKey(agent.id, agent.publicKey);
```

**Message Encryption:**
```typescript
async function encryptMessage(
  message: Message,
  recipientPublicKey: string,
  senderPrivateKey: string
): Promise<EncryptedMessage> {
  // Serialize message
  const payload = JSON.stringify(message);

  // Sign with sender's private key
  const signature = await signEd25519(payload, senderPrivateKey);

  // Encrypt with recipient's public key
  const { encrypted, nonce } = await encryptAES256GCM(
    payload,
    recipientPublicKey
  );

  return {
    envelope: {
      from: message.from,
      to: message.to,
      timestamp: Date.now(),
      correlationId: message.correlationId
    },
    payload: encrypted,
    signature,
    nonce
  };
}
```

**Message Decryption:**
```typescript
async function decryptMessage(
  encrypted: EncryptedMessage,
  recipientPrivateKey: string,
  senderPublicKey: string
): Promise<Message> {
  // Decrypt payload
  const payload = await decryptAES256GCM(
    encrypted.payload,
    encrypted.nonce,
    recipientPrivateKey
  );

  // Verify signature
  const valid = await verifyEd25519(
    payload,
    encrypted.signature,
    senderPublicKey
  );

  if (!valid) {
    throw new Error('Invalid message signature');
  }

  return JSON.parse(payload);
}
```

### 5.2 Security Guarantees

1. **End-to-End Encryption:** All agent messages encrypted, coordinator cannot read
2. **Message Integrity:** Ed25519 signatures prevent tampering
3. **Perfect Forward Secrecy:** Each message uses unique nonce
4. **Non-Repudiation:** Signatures prove message origin
5. **Audit Trail:** All encrypted messages logged with metadata

## 6. Scalability Considerations

### 6.1 Horizontal Scaling
```
Load Balancer
    ↓
┌───────┬───────┬───────┐
│Server1│Server2│Server3│
└───┬───┴───┬───┴───┬───┘
    │       │       │
    └───────┴───────┘
            ↓
    Shared State Store
      (Future: Redis)
```

### 6.2 Agent Pooling
```typescript
class AgentPool {
  private pools: Map<AgentType, Agent[]>;

  async acquireAgent(type: AgentType): Promise<Agent> {
    let pool = this.pools.get(type);

    // Reuse idle agent
    let agent = pool.find(a => a.status === 'idle');

    if (!agent) {
      // Spawn new if pool not full
      agent = await this.spawnAgent(type);
      pool.push(agent);
    }

    agent.status = 'working';
    return agent;
  }

  releaseAgent(agent: Agent): void {
    agent.status = 'idle';
    // Agent remains in pool for reuse
  }
}
```

## 7. Monitoring and Observability

### 7.1 Metrics Collection
```typescript
interface SystemMetrics {
  agents: {
    [agentId: string]: {
      responseTime: number[];
      successRate: number;
      messagesProcessed: number;
      errorsCount: number;
    }
  };
  messaging: {
    totalMessages: number;
    encryptionTime: number[];
    routingTime: number[];
    queueDepth: number;
  };
  trips: {
    activeSessions: number;
    avgPlanningTime: number;
    completionRate: number;
  };
}
```

### 7.2 Health Checks
```typescript
async function healthCheck(): Promise<HealthStatus> {
  return {
    status: 'healthy',
    timestamp: Date.now(),
    checks: {
      agents: await checkAgentHealth(),
      messaging: await checkMessageBroker(),
      websocket: await checkWebSocketServer(),
      memory: process.memoryUsage()
    }
  };
}
```

## 8. Deployment Architecture

### 8.1 Development
```
localhost:5173  → Vite Dev Server (Client)
localhost:3000  → Express Server (API + WebSocket)
```

### 8.2 Production (Future)
```
CDN → Static React Build
    ↓
Load Balancer → Node.js Servers (multiple instances)
                    ↓
                Redis (State + Sessions)
                    ↓
                PostgreSQL (Persistence)
```

## 9. API Design

### 9.1 REST Endpoints
```typescript
// Trip Management
POST   /api/trips
  Body: TripRequest
  Response: { tripId: string }

GET    /api/trips/:id
  Response: Trip

PUT    /api/trips/:id
  Body: Partial<Trip>
  Response: Trip

// Itinerary
GET    /api/trips/:id/itinerary
  Response: Itinerary

PUT    /api/trips/:id/itinerary/items/:itemId
  Body: { status: 'accepted' | 'rejected' }
  Response: Itinerary

// Agents
GET    /api/trips/:id/agents
  Response: AgentStatus[]
```

### 9.2 WebSocket Events
```typescript
// Client → Server
interface ClientEvents {
  'trip:start': (tripId: string) => void;
  'trip:update': (tripId: string, updates: Partial<Trip>) => void;
  'item:accept': (tripId: string, itemId: string) => void;
  'item:reject': (tripId: string, itemId: string) => void;
  'request:alternative': (tripId: string, category: string) => void;
}

// Server → Client
interface ServerEvents {
  'agent:status': (agentId: string, status: AgentStatus) => void;
  'itinerary:update': (update: ItineraryUpdate) => void;
  'budget:alert': (alert: BudgetAlert) => void;
  'conflict:detected': (conflict: Conflict) => void;
  'planning:complete': (itinerary: Itinerary) => void;
  'error': (error: ErrorEvent) => void;
}
```

## 10. File Structure

```
travel-demo/
├── src/
│   ├── client/
│   │   ├── components/
│   │   │   ├── TripWizard/
│   │   │   │   ├── TripWizard.tsx
│   │   │   │   ├── StepIndicator.tsx
│   │   │   │   ├── DestinationInput.tsx
│   │   │   │   ├── DateRangePicker.tsx
│   │   │   │   ├── BudgetSlider.tsx
│   │   │   │   └── PreferencesForm.tsx
│   │   │   ├── AgentPanel/
│   │   │   │   ├── AgentPanel.tsx
│   │   │   │   ├── AgentCard.tsx
│   │   │   │   └── CoordinationGraph.tsx
│   │   │   ├── ItineraryView/
│   │   │   │   ├── ItineraryView.tsx
│   │   │   │   ├── Timeline.tsx
│   │   │   │   ├── FlightSegment.tsx
│   │   │   │   ├── HotelSegment.tsx
│   │   │   │   └── ActivitySegment.tsx
│   │   │   └── BudgetTracker/
│   │   │       ├── BudgetTracker.tsx
│   │   │       ├── BudgetGauge.tsx
│   │   │       └── CategoryBreakdown.tsx
│   │   ├── hooks/
│   │   │   ├── useWebSocket.ts
│   │   │   ├── useTripPlanner.ts
│   │   │   └── useAgentStatus.ts
│   │   ├── store/
│   │   │   └── tripStore.ts
│   │   ├── types/
│   │   │   └── index.ts
│   │   ├── utils/
│   │   │   └── formatting.ts
│   │   ├── App.tsx
│   │   └── main.tsx
│   ├── server/
│   │   ├── agents/
│   │   │   ├── BaseAgent.ts
│   │   │   ├── FlightAgent.ts
│   │   │   ├── HotelAgent.ts
│   │   │   ├── ActivityAgent.ts
│   │   │   └── BudgetAgent.ts
│   │   ├── services/
│   │   │   ├── AgentCoordinator.ts
│   │   │   ├── MessageBroker.ts
│   │   │   ├── StateManager.ts
│   │   │   └── ConflictResolver.ts
│   │   ├── types/
│   │   │   └── index.ts
│   │   ├── utils/
│   │   │   ├── encryption.ts
│   │   │   └── mockData.ts
│   │   ├── routes/
│   │   │   └── api.ts
│   │   └── index.ts
│   └── shared/
│       └── types.ts
├── tests/
│   ├── unit/
│   │   ├── agents/
│   │   ├── components/
│   │   └── services/
│   └── integration/
│       └── tripPlanning.test.ts
├── docs/
│   └── sparc/
│       ├── 01-specification.md
│       ├── 02-pseudocode.md
│       ├── 03-architecture.md
│       ├── 04-refinement.md
│       └── 05-completion.md
├── package.json
├── tsconfig.json
├── vite.config.ts
└── README.md
```

## Next Steps

Architecture design complete. **Proceed to Phase 4: Refinement** for TDD implementation of all components.

Key architectural decisions:
- ✅ Multi-agent coordination via creto-messaging
- ✅ Real-time WebSocket communication
- ✅ Component-based React architecture
- ✅ Priority-based message queue
- ✅ Conflict detection and resolution
- ✅ In-memory state for demo simplicity
- ✅ Comprehensive security with E2EE
