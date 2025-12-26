# Trading Demo - Setup & Run Guide

## Quick Start

### 1. Install Dependencies
```bash
npm install
```

### 2. Seed Database
```bash
npm run demo:seed
```

This creates a SQLite database at `./data/trading-demo.db` with:
- 3 demo trading agents (Alpha Trader, Safe Haven, Balanced Growth)
- Portfolio positions for each agent
- Historical trades
- 1 pending oversight request
- Usage metrics
- 30 days of portfolio history

### 3. Start Development Servers

#### Option A: Start Everything (Recommended)
```bash
npm run dev
```

This starts both:
- Backend API server on http://localhost:3000
- Frontend Vite dev server on http://localhost:5173

#### Option B: Start Individually
```bash
# Terminal 1 - Backend
npm run dev:server

# Terminal 2 - Frontend
npm run dev:client
```

### 4. Access the Application

Frontend: http://localhost:5173

API Endpoints:
- Health Check: http://localhost:3000/health/live
- API Docs: http://localhost:3000/api
- Agents: http://localhost:3000/api/v1/agents
- Trades: http://localhost:3000/api/v1/agents/{agentId}/trades
- Oversight: http://localhost:3000/api/v1/oversight/requests
- Metering: http://localhost:3000/api/v1/metering/agents/{agentId}/usage

## Database Schema

The SQLite database includes these tables:
- `agents` - Trading agents
- `portfolios` - Agent portfolios
- `positions` - Stock positions
- `trades` - Trade history
- `oversight_requests` - Approval requests
- `approval_decisions` - Approval history
- `usage_metrics` - API usage tracking
- `quota_configs` - Quota configurations
- `billable_events` - Billing events
- `oversight_policies` - Approval policies
- `portfolio_history` - Historical data for charts

## API Examples

### List All Agents
```bash
curl http://localhost:3000/api/v1/agents
```

### Get Agent Details
```bash
curl http://localhost:3000/api/v1/agents/agent-alpha
```

### Get Agent Portfolio
```bash
curl http://localhost:3000/api/v1/agents/agent-alpha/portfolio
```

### Get Pending Oversight Requests
```bash
curl http://localhost:3000/api/v1/oversight/requests
```

### Get Usage Metrics
```bash
curl http://localhost:3000/api/v1/metering/agents/agent-alpha/usage
```

## Demo Data

### Agents
1. **Alpha Trader** (agent-alpha)
   - Strategy: Momentum
   - Budget: $100,000
   - Risk: 7/10
   - Positions: AAPL, NVDA, MSFT

2. **Safe Haven** (agent-beta)
   - Strategy: Conservative
   - Budget: $50,000
   - Risk: 3/10
   - Positions: JNJ, PG

3. **Balanced Growth** (agent-gamma)
   - Strategy: Balanced
   - Budget: $75,000
   - Risk: 5/10
   - Positions: GOOGL, AMZN

### Pending Oversight Request
- Agent: Alpha Trader
- Trade: BUY 200 NVDA @ $495
- Total Value: $99,000
- Risk Level: HIGH
- Reason: Exceeds $50K approval threshold

## Development

### Run Tests
```bash
npm test                 # All tests
npm run test:unit        # Unit tests only
npm run test:services    # Service tests
npm run test:e2e         # End-to-end tests
```

### Type Checking
```bash
npm run typecheck
```

### Linting
```bash
npm run lint
npm run lint:fix
```

### Reset Demo
```bash
npm run demo:reset
```

## Production Build

### Build
```bash
npm run build
```

This creates:
- `dist/server/` - Compiled backend
- `dist/client/` - Static frontend assets

### Run Production
```bash
NODE_ENV=production node dist/server/index.js
```

The production server serves both API and frontend from port 3000.

## Environment Variables

Create `.env` file:

```env
# Server
PORT=3000
HOST=0.0.0.0
NODE_ENV=development

# Database
DB_PATH=./data/trading-demo.db

# Frontend (development)
CLIENT_URL=http://localhost:5173

# gRPC Services (optional)
METERING_SERVICE_URL=localhost:50051
OVERSIGHT_SERVICE_URL=localhost:50052
```

## Troubleshooting

### Database Not Found
```bash
npm run demo:seed
```

### Port Already in Use
Change `PORT` in `.env` or:
```bash
PORT=3001 npm run dev:server
```

### Frontend Can't Connect to API
Check `VITE_API_URL` in frontend or CORS settings in `src/server/app.ts`

### Clear All Data
```bash
rm -rf data/
npm run demo:seed
```

## Architecture

```
┌─────────────────────────────────────────┐
│          Frontend (React)               │
│  - Dashboard, Portfolio, Oversight      │
│  - Real-time WebSocket updates          │
└──────────────┬──────────────────────────┘
               │ HTTP/WS
┌──────────────▼──────────────────────────┐
│       Express API Server                │
│  - RESTful endpoints                    │
│  - WebSocket hub                        │
│  - Controllers & Services               │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│       SQLite Database                    │
│  - Agents, Portfolios, Trades           │
│  - Oversight, Metering                  │
└─────────────────────────────────────────┘
```

## Next Steps

1. **Test the Demo**: Navigate through all tabs (Dashboard, Portfolio, Oversight, Metering)
2. **Submit Trades**: Click "New Trade" button
3. **Approve/Reject**: Review pending oversight requests
4. **Monitor Usage**: Check metering quotas

The demo is now fully functional with a real database backend!
