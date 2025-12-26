# Travel Demo - Multi-Agent Trip Planner

A demonstration of **multi-agent coordination** with **secure agent-to-agent communication** using creto-messaging (E2EE). Four specialized AI agents collaborate in real-time to plan optimal travel itineraries.

## 🎯 Demo Purpose

Showcase secure multi-agent systems where:
- **4 Specialized Agents** coordinate to plan trips (Flight, Hotel, Activity, Budget)
- **Creto-Messaging** provides end-to-end encrypted agent communication
- **Real-time Updates** stream planning progress to users via WebSocket
- **Conflict Resolution** automatically detects and resolves itinerary issues
- **Budget Optimization** ensures cost-effective recommendations

## 🏗️ Architecture

```
┌─────────────┐     WebSocket      ┌──────────────────┐
│   React     │ ◄────────────────► │  Agent           │
│   Client    │                    │  Coordinator     │
└─────────────┘                    └────────┬─────────┘
                                            │
                   ┌────────────────────────┼────────────────────┐
                   │ Creto-Messaging (E2EE) │                    │
        ┌──────────▼─────┐  ┌──────▼──────┐  ┌────▼─────┐  ┌───▼──────┐
        │ Flight Agent   │  │ Hotel Agent │  │ Activity │  │  Budget  │
        │                │  │             │  │  Agent   │  │  Agent   │
        └────────────────┘  └─────────────┘  └──────────┘  └──────────┘
```

## ✨ Key Features

### 1. Multi-Agent Coordination
- **FlightAgent**: Searches flights, ranks by price/duration/stops
- **HotelAgent**: Finds hotels, optimizes location and amenities
- **ActivityAgent**: Plans activities, ensures no schedule conflicts
- **BudgetAgent**: Monitors costs, provides optimization suggestions

### 2. Secure Communication (Creto-Messaging)
- **Ed25519** signatures for message authentication
- **AES-256-GCM** encryption for confidentiality
- **Perfect Forward Secrecy** with unique nonces per message
- **Audit Trail** of all encrypted agent communications

### 3. Real-Time Collaboration
- **WebSocket Streaming**: Live updates as agents work
- **Priority Queue**: Critical messages processed first
- **State Synchronization**: Consistent trip state across agents
- **Event-Driven**: Immediate UI updates

### 4. Intelligent Features
- **Conflict Detection**: Time/location/budget conflicts identified
- **Automatic Resolution**: Conflicts resolved or marked for review
- **Budget Optimization**: Suggestions to save or upgrade
- **Preference Matching**: Activities match user interests

## 🚀 Quick Start

### Prerequisites
- Node.js 20+
- npm or pnpm

### Installation

```bash
# Clone or navigate to demo directory
cd /Users/tommaduri/Documents/GitHub/Enablement/demos/travel-demo

# Install dependencies
npm install

# Start development servers (client + server)
npm run dev
```

The demo will be available at:
- **Client**: http://localhost:5173
- **API**: http://localhost:3000

### 5-Minute Demo Script

1. **Start Planning**
   - Open http://localhost:5173
   - Enter destination: "Paris"
   - Dates: Next month, 5 days
   - Budget: $3000
   - Preferences: Culture, Food, Relaxation
   - Click "Start Planning"

2. **Watch Agents Work**
   - **Agent Panel** shows real-time status
   - **Messages** display encrypted communication
   - **Progress bars** update as agents search
   - Results appear within 2-5 seconds

3. **Review Itinerary**
   - **Timeline** shows complete trip schedule
   - **Flights**: 3-5 ranked options
   - **Hotels**: 3-5 recommendations with ratings
   - **Activities**: Daily schedule with no conflicts
   - **Budget**: Total cost with breakdown

4. **Budget Insights**
   - View **cost breakdown** by category
   - Check **optimization suggestions** if over budget
   - See **upgrade options** if under budget
   - Monitor **real-time budget alerts**

5. **Conflict Resolution** (if any)
   - System detects time/location conflicts
   - View **conflict details** and severity
   - See **automatic resolutions** applied
   - Manual review for complex conflicts

## 📁 Project Structure

```
travel-demo/
├── src/
│   ├── client/              # React frontend
│   │   ├── components/      # UI components
│   │   ├── hooks/           # Custom React hooks
│   │   ├── store/           # Zustand state management
│   │   └── App.tsx          # Main app component
│   ├── server/              # Node.js backend
│   │   ├── agents/          # Agent implementations
│   │   ├── services/        # Core services
│   │   │   ├── AgentCoordinator.ts
│   │   │   ├── MessageBroker.ts
│   │   │   ├── StateManager.ts
│   │   │   └── ConflictResolver.ts
│   │   ├── utils/
│   │   │   ├── encryption.ts  # Creto-messaging
│   │   │   └── mockData.ts    # Demo data
│   │   └── index.ts         # Server entry
│   └── shared/
│       └── types.ts         # Shared TypeScript types
├── tests/                   # Test suites
├── docs/
│   └── sparc/               # SPARC methodology docs
│       ├── 01-specification.md
│       ├── 02-pseudocode.md
│       ├── 03-architecture.md
│       ├── 04-refinement.md
│       └── 05-completion.md
├── package.json
└── README.md
```

## 🧪 Testing

```bash
# Run all tests
npm test

# Run with coverage
npm run test:coverage

# Run with UI
npm run test:ui

# Type checking
npm run typecheck

# Linting
npm run lint
```

**Coverage Target**: 90%+

## 🔒 Security Features

### Creto-Messaging Implementation

**Message Encryption Flow:**
```typescript
// 1. Agent creates message
const message = { action: 'searchFlights', data: constraints };

// 2. Sign with Ed25519 private key
const signature = await signMessage(message, senderPrivateKey);

// 3. Encrypt with recipient's public key (AES-256-GCM)
const encrypted = await encryptMessage(message, recipientPublicKey);

// 4. Send through message broker
await broker.send(encrypted, 'high');

// 5. Recipient decrypts and verifies
const decrypted = await decryptMessage(encrypted, recipientPrivateKey);
const valid = await verifySignature(decrypted, signature, senderPublicKey);
```

**Security Guarantees:**
- ✅ End-to-end encryption (coordinator cannot read agent messages)
- ✅ Message integrity (signatures prevent tampering)
- ✅ Non-repudiation (proof of message origin)
- ✅ Forward secrecy (unique nonce per message)

## 🎨 UI Components

### TripWizard
Multi-step form for trip requirements:
- Destination input
- Date range picker
- Budget slider
- Preference selection (activities, accommodation, pace)

### AgentPanel
Real-time agent status display:
- Individual agent cards with status indicators
- Message log showing encrypted communication
- Coordination graph visualizing agent interactions

### ItineraryView
Comprehensive trip display:
- Timeline of all activities
- Flight and hotel segments
- Interactive map (optional)
- Accept/reject individual items

### BudgetTracker
Financial overview:
- Gauge showing budget utilization
- Category breakdown (flights, hotels, activities)
- Optimization suggestions
- Real-time alerts

## 📊 SPARC Methodology

This demo was built using the **SPARC** (Specification, Pseudocode, Architecture, Refinement, Completion) methodology:

1. **Phase 1: Specification** - 15+ functional requirements, 10+ non-functional requirements, 15+ user stories
2. **Phase 2: Pseudocode** - Algorithms for coordination, search, budget optimization, conflict resolution
3. **Phase 3: Architecture** - Component design, data flows, security architecture
4. **Phase 4: Refinement** - TDD implementation with 90%+ coverage
5. **Phase 5: Completion** - Integration tests, demo data, documentation

See `docs/sparc/` for complete methodology documentation.

## 🛠️ Technology Stack

**Frontend:**
- React 18 + TypeScript 5.3
- Vite (build tool)
- TailwindCSS (styling)
- Zustand (state management)
- Socket.IO Client (WebSocket)

**Backend:**
- Node.js 20 + TypeScript 5.3
- Express (HTTP server)
- Socket.IO (WebSocket server)
- @noble/ed25519 (Ed25519 signatures)
- Native crypto (AES-256-GCM)

**Testing:**
- Vitest (unit/integration tests)
- React Testing Library
- @vitest/coverage-v8

## 🎯 Demo Highlights

### What to Showcase

1. **Multi-Agent Intelligence**
   - 4 agents working in parallel
   - Each with specialized search algorithms
   - Coordinated through message broker

2. **Secure Communication**
   - All agent messages encrypted
   - View encrypted payloads in agent panel
   - Signature verification on every message

3. **Real-Time Coordination**
   - Live status updates
   - Progressive itinerary building
   - Instant conflict detection

4. **Budget Intelligence**
   - Automatic cost tracking
   - Smart optimization suggestions
   - Alert system for budget overruns

5. **Conflict Resolution**
   - Time conflict detection
   - Location-based travel time calculation
   - Automatic rescheduling

## 📝 API Documentation

### REST Endpoints

```
POST   /api/trips              - Create trip and start planning
GET    /api/trips/:id          - Get trip details
GET    /api/trips/:id/agents   - Get agent statuses
PUT    /api/trips/:id/items/:itemId - Update item status
```

### WebSocket Events

**Client → Server:**
- `trip:start` - Begin trip planning
- `item:accept` - Accept recommendation
- `item:reject` - Reject recommendation
- `request:alternative` - Request alternative options

**Server → Client:**
- `agent:status` - Agent status update
- `itinerary:update` - New itinerary item
- `budget:alert` - Budget threshold alert
- `conflict:detected` - Conflict identified
- `planning:complete` - All agents finished

## 🔧 Configuration

### Environment Variables

Create `.env` file (optional for demo):

```env
PORT=3000
NODE_ENV=development
ENCRYPTION_ENABLED=true
```

## 🐛 Troubleshooting

**WebSocket connection fails:**
- Ensure both client (5173) and server (3000) are running
- Check proxy configuration in `vite.config.ts`

**Encryption errors:**
- Verify @noble/ed25519 is installed
- Check Node.js version (20+ required)

**Agents not responding:**
- Check console for errors
- Verify agents initialized before planning
- Review message broker queue status

## 📚 Further Reading

- **Creto-Messaging**: Secure multi-agent communication patterns
- **SPARC Methodology**: Systematic development approach
- **Multi-Agent Systems**: Coordination and consensus algorithms
- **WebSocket Architecture**: Real-time bidirectional communication

## 🤝 Contributing

This is a demonstration project. For production use:
- Replace mock data with real API integrations
- Add user authentication
- Implement persistent storage (PostgreSQL/MongoDB)
- Add payment processing
- Deploy with Docker + Kubernetes
- Scale with Redis for state management

## 📄 License

MIT License - See LICENSE file for details

## 🙋 Support

For questions or issues:
- Review SPARC documentation in `docs/sparc/`
- Check test files for usage examples
- Examine agent implementations in `src/server/agents/`

---

**Built with SPARC methodology | Secure by design | Real-time by default**
