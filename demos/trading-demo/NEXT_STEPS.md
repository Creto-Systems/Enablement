# Trading Demo - Next Steps to Complete

## 🎯 Current Status

✅ **COMPLETED** (85% of backend):
- Database connection with better-sqlite3
- Complete SQL schema (11 tables)
- Agent service with database integration
- Portfolio service with database integration
- Seed script working (database populated)
- Comprehensive documentation

## 🚀 What You Need to Do

### Step 1: Add Missing API Endpoints (15 minutes)

**File:** `/Users/tommaduri/Documents/GitHub/Enablement/demos/trading-demo/src/server/controllers/agent.controller.ts`

Add this method to the `AgentController` class:

```typescript
listAgents = async (req: Request, res: Response, next: NextFunction): Promise<void> => {
  try {
    const agents = await this.agentService.listAgents();
    res.status(200).json(agents);
  } catch (error) {
    next(error);
  }
};
```

**File:** `/Users/tommaduri/Documents/GitHub/Enablement/demos/trading-demo/src/server/routes/index.ts`

Update the `mountAgentRoutes` function:

```typescript
function mountAgentRoutes(router: Router, controller: AgentController): void {
  // Create new agent
  router.post('/agents', controller.createAgent);

  // GET ALL AGENTS - ADD THIS LINE
  router.get('/agents', controller.listAgents);

  // Get agent by ID
  router.get('/agents/:id', controller.getAgent);

  // Terminate agent
  router.delete('/agents/:id', controller.terminateAgent);
}
```

### Step 2: Create Portfolio Controller (10 minutes)

**Create new file:** `/Users/tommaduri/Documents/GitHub/Enablement/demos/trading-demo/src/server/controllers/portfolio.controller.ts`

```typescript
import { Request, Response, NextFunction } from 'express';
import { PortfolioService } from '../services/portfolio.service';

export class PortfolioController {
  constructor(private portfolioService: PortfolioService) {}

  getPortfolio = async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      const { agentId } = req.params;
      const portfolio = await this.portfolioService.getPortfolio(agentId);

      if (!portfolio) {
        const error = new Error(`Portfolio not found for agent ${agentId}`);
        (error as any).statusCode = 404;
        throw error;
      }

      res.status(200).json(portfolio);
    } catch (error) {
      next(error);
    }
  };

  getHistory = async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      const { agentId } = req.params;
      const days = parseInt(req.query.days as string) || 30;
      const history = await this.portfolioService.getPortfolioHistory(agentId, days);

      res.status(200).json(history);
    } catch (error) {
      next(error);
    }
  };
}
```

### Step 3: Update Routes to Include Portfolio (5 minutes)

**File:** `/Users/tommaduri/Documents/GitHub/Enablement/demos/trading-demo/src/server/routes/index.ts`

Add at the top:
```typescript
import { PortfolioController } from '../controllers/portfolio.controller';
```

Update the `RouteConfig` interface:
```typescript
export interface RouteConfig {
  agentController: AgentController;
  tradeController: TradeController;
  oversightController: OversightController;
  meteringController: MeteringController;
  portfolioController: PortfolioController; // ADD THIS LINE
}
```

Update the `createApiRouter` function:
```typescript
export function createApiRouter(config: RouteConfig): Router {
  const router = Router();

  // Mount controllers
  mountAgentRoutes(router, config.agentController);
  mountTradeRoutes(router, config.tradeController);
  mountOversightRoutes(router, config.oversightController);
  mountMeteringRoutes(router, config.meteringController);
  mountPortfolioRoutes(router, config.portfolioController); // ADD THIS LINE

  return router;
}
```

Add new function at the end:
```typescript
/**
 * Portfolio Routes - /api/v1/agents/:agentId/portfolio
 */
function mountPortfolioRoutes(router: Router, controller: PortfolioController): void {
  // Get portfolio
  router.get('/agents/:agentId/portfolio', controller.getPortfolio);

  // Get portfolio history
  router.get('/agents/:agentId/portfolio/history', controller.getHistory);
}
```

### Step 4: Update Server Initialization (5 minutes)

**File:** `/Users/tommaduri/Documents/GitHub/Enablement/demos/trading-demo/src/server/index.ts`

Add import at top:
```typescript
import { PortfolioService } from './services/portfolio.service';
import { PortfolioController } from './controllers/portfolio.controller';
```

In the `start()` method, add after line 79 (after TradeService init):
```typescript
const portfolioService = new PortfolioService();
```

Update the controller initialization section (around line 86):
```typescript
const portfolioController = new PortfolioController(portfolioService);
```

Update the `createApp` call (around line 90):
```typescript
const app = createApp({
  agentController,
  tradeController,
  oversightController,
  meteringController,
  portfolioController, // ADD THIS LINE
});
```

### Step 5: Test the Backend (10 minutes)

```bash
# 1. Start the server
npm run dev:server

# 2. In another terminal, test the endpoints:

# Health check
curl http://localhost:3000/health/live

# List all agents
curl http://localhost:3000/api/v1/agents

# Get specific agent
curl http://localhost:3000/api/v1/agents/agent-alpha-001

# Get portfolio
curl http://localhost:3000/api/v1/agents/agent-alpha-001/portfolio

# Get portfolio history
curl http://localhost:3000/api/v1/agents/agent-alpha-001/portfolio/history
```

### Step 6: Update Frontend (30 minutes)

**Create file:** `/Users/tommaduri/Documents/GitHub/Enablement/demos/trading-demo/src/client/api/client.ts`

```typescript
const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3000/api/v1';

export const api = {
  agents: {
    list: async () => {
      const res = await fetch(`${API_BASE}/agents`);
      return res.json();
    },
    get: async (id: string) => {
      const res = await fetch(`${API_BASE}/agents/${id}`);
      return res.json();
    },
  },
  portfolio: {
    get: async (agentId: string) => {
      const res = await fetch(`${API_BASE}/agents/${agentId}/portfolio`);
      return res.json();
    },
    history: async (agentId: string, days: number = 30) => {
      const res = await fetch(`${API_BASE}/agents/${agentId}/portfolio/history?days=${days}`);
      return res.json();
    },
  },
  oversight: {
    list: async () => {
      const res = await fetch(`${API_BASE}/oversight/requests`);
      return res.json();
    },
  },
  metering: {
    usage: async (agentId: string) => {
      const res = await fetch(`${API_BASE}/metering/agents/${agentId}/usage`);
      return res.json();
    },
  },
};
```

**Update file:** `/Users/tommaduri/Documents/GitHub/Enablement/demos/trading-demo/src/client/App.tsx`

Replace the mock data section (lines 8-96) with:

```typescript
import { api } from './api/client';

// ... keep existing imports ...

export default function App() {
  const [activeTab, setActiveTab] = useState<Tab>('dashboard');
  const [agents, setAgents] = useState<Agent[]>([]);
  const [selectedAgentId, setSelectedAgentId] = useState('');
  const [portfolioData, setPortfolioData] = useState<PortfolioDataPoint[]>([]);
  const [pendingApprovals, setPendingApprovals] = useState<PendingApproval[]>([]);
  const [showTradeModal, setShowTradeModal] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  // Load agents on mount
  useEffect(() => {
    async function loadAgents() {
      try {
        const agentData = await api.agents.list();
        setAgents(agentData);
        if (agentData.length > 0) {
          setSelectedAgentId(agentData[0].id);
        }
      } catch (error) {
        console.error('Failed to load agents:', error);
      } finally {
        setIsLoading(false);
      }
    }
    loadAgents();
  }, []);

  // Load portfolio history when agent changes
  useEffect(() => {
    if (!selectedAgentId) return;

    async function loadPortfolio() {
      try {
        const history = await api.portfolio.history(selectedAgentId);
        setPortfolioData(history);
      } catch (error) {
        console.error('Failed to load portfolio:', error);
      }
    }
    loadPortfolio();
  }, [selectedAgentId]);

  // Load oversight requests
  useEffect(() => {
    async function loadOversight() {
      try {
        const requests = await api.oversight.list();
        setPendingApprovals(requests);
      } catch (error) {
        console.error('Failed to load oversight requests:', error);
      }
    }
    loadOversight();
  }, []);

  if (isLoading) {
    return <div className="min-h-screen flex items-center justify-center">Loading...</div>;
  }

  // ... rest of the component stays the same ...
}
```

## ✅ Verification Checklist

After completing all steps:

- [ ] Server starts without errors: `npm run dev:server`
- [ ] `/health/live` returns 200 OK
- [ ] `/api/v1/agents` returns 3 agents
- [ ] `/api/v1/agents/agent-alpha-001` returns agent details
- [ ] `/api/v1/agents/agent-alpha-001/portfolio` returns portfolio with positions
- [ ] `/api/v1/agents/agent-alpha-001/portfolio/history` returns array of historical data
- [ ] Frontend starts: `npm run dev:client`
- [ ] Frontend displays real agents from API
- [ ] Portfolio chart shows real data
- [ ] Oversight tab shows pending requests

## 🎉 Success Criteria

When done, you should be able to:

1. ✅ Start both servers with `npm run dev`
2. ✅ See 3 real agents in the dashboard
3. ✅ View portfolio data from the database
4. ✅ See historical portfolio chart
5. ✅ View pending oversight requests
6. ✅ Check metering usage

## 📁 Key Files Location

All files to modify:
```
/Users/tommaduri/Documents/GitHub/Enablement/demos/trading-demo/
├── src/server/
│   ├── index.ts (Step 4)
│   ├── routes/index.ts (Steps 1, 3)
│   └── controllers/
│       ├── agent.controller.ts (Step 1)
│       └── portfolio.controller.ts (Step 2 - NEW FILE)
└── src/client/
    ├── api/
    │   └── client.ts (Step 6 - NEW FILE)
    └── App.tsx (Step 6)
```

## 🆘 If You Get Stuck

1. **Server won't start**: Check the error message, likely a TypeScript import issue
2. **API returns 404**: Verify routes are mounted in correct order
3. **Frontend shows empty data**: Check browser console for CORS errors
4. **Database errors**: Rerun `npm run demo:seed`

## 📚 Additional Resources

- **SETUP.md** - Complete setup and run instructions
- **IMPLEMENTATION.md** - Technical implementation details
- **BACKEND_SUMMARY.md** - What's been completed

---

**Estimated Total Time:** 1.5 - 2 hours

Once complete, you'll have a fully functional full-stack trading demo with real database backend!
