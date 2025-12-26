import React, { useState, useMemo } from 'react';
import { AgentCard, Agent } from './components/AgentCard';
import { PortfolioChart, PortfolioDataPoint } from './components/PortfolioChart';
import { TradeForm, TradeFormData } from './components/TradeForm';
import { UsageMeter } from './components/UsageMeter';
import { ApprovalCard, PendingApproval } from './components/ApprovalCard';
import {
  useAgents,
  usePendingApprovals,
  usePortfolioHistory,
  useUsageMetrics,
  useApproveRequest,
  useRejectRequest,
  useExecuteTrade,
} from './hooks/useApi';

// Demo user ID for API calls
const DEMO_USER_ID = 'demo-user-1';

// Fallback mock data when API is not available
const mockAgents: Agent[] = [
  {
    id: 'agent-1',
    name: 'Alpha Trader',
    status: 'active',
    budget: { total: 100000, used: 45000, remaining: 55000 },
    performance: { dailyPnL: 2340, totalPnL: 12500, winRate: 0.68 },
  },
  {
    id: 'agent-2',
    name: 'Safe Haven',
    status: 'active',
    budget: { total: 50000, used: 12000, remaining: 38000 },
    performance: { dailyPnL: 180, totalPnL: 1200, winRate: 0.72 },
  },
  {
    id: 'agent-3',
    name: 'Balanced Growth',
    status: 'paused',
    budget: { total: 75000, used: 35000, remaining: 40000 },
    performance: { dailyPnL: -420, totalPnL: 3900, winRate: 0.55 },
  },
];

// Generate portfolio history data (last 30 days)
const generatePortfolioData = (): PortfolioDataPoint[] => {
  const data: PortfolioDataPoint[] = [];
  const baseValue = 100000;
  let currentValue = baseValue;

  for (let i = 30; i >= 0; i--) {
    const date = new Date();
    date.setDate(date.getDate() - i);
    const change = (Math.random() - 0.4) * 0.03;
    currentValue *= (1 + change);
    data.push({
      timestamp: date.toISOString(),
      value: Math.round(currentValue * 100) / 100,
    });
  }
  return data;
};

const mockPortfolioData = generatePortfolioData();

const mockPendingApprovals: PendingApproval[] = [
  {
    id: 'req-1',
    agentId: 'agent-1',
    symbol: 'NVDA',
    side: 'buy',
    quantity: 200,
    price: 495.00,
    estimatedCost: 99000,
    timestamp: new Date(Date.now() - 30 * 60 * 1000).toISOString(),
    reasoning: 'NVDA shows strong momentum with AI chip demand increasing. Entry point aligns with 50-day moving average support.',
    riskAssessment: {
      score: 0.72,
      level: 'high',
      factors: [
        'Trade exceeds $50,000 threshold',
        'Single position would be 40% of portfolio',
        'High volatility sector (Technology)',
      ],
    },
  },
  {
    id: 'req-2',
    agentId: 'agent-2',
    symbol: 'AAPL',
    side: 'buy',
    quantity: 100,
    price: 185.00,
    estimatedCost: 18500,
    timestamp: new Date(Date.now() - 15 * 60 * 1000).toISOString(),
    reasoning: 'Adding to existing position ahead of earnings. Dividend yield attractive at current levels.',
    riskAssessment: {
      score: 0.35,
      level: 'low',
      factors: [
        'Position adds to diversified portfolio',
        'Low volatility blue-chip stock',
        'Within normal trade size range',
      ],
    },
  },
];

type Tab = 'dashboard' | 'portfolio' | 'oversight' | 'metering';

export default function App() {
  const [activeTab, setActiveTab] = useState<Tab>('dashboard');
  const [selectedAgentId, setSelectedAgentId] = useState(mockAgents[0].id);
  const [showTradeModal, setShowTradeModal] = useState(false);

  // API Hooks with fallback to mock data
  const { data: apiAgents, isLoading: agentsLoading } = useAgents(DEMO_USER_ID);
  const { data: apiApprovals, isLoading: approvalsLoading } = usePendingApprovals();
  const { data: portfolioHistory } = usePortfolioHistory(selectedAgentId);
  const { data: usageMetrics } = useUsageMetrics(selectedAgentId);

  // Mutations
  const executeTrade = useExecuteTrade(selectedAgentId);
  const approveRequest = useApproveRequest();
  const rejectRequest = useRejectRequest();

  // Use API data if available, otherwise fall back to mock data
  const agents = apiAgents || mockAgents;
  const pendingApprovals = apiApprovals || mockPendingApprovals;
  const chartData = portfolioHistory || mockPortfolioData;

  // Usage metrics with fallback
  const usage = useMemo(() => ({
    apiCalls: usageMetrics?.apiCalls || { used: 8500, total: 10000 },
    trades: usageMetrics?.trades || { used: 47, total: 100 },
    dataRequests: usageMetrics?.dataRequests || { used: 2100, total: 5000 },
    monthlyBudget: usageMetrics?.monthlyBudget || { used: 4250, total: 10000 },
  }), [usageMetrics]);

  const isLoading = executeTrade.isPending || approveRequest.isPending || rejectRequest.isPending;
  const selectedAgent = agents.find(a => a.id === selectedAgentId) || agents[0];

  const handleTradeSubmit = async (data: TradeFormData) => {
    try {
      await executeTrade.mutateAsync({
        symbol: data.symbol,
        side: data.side,
        quantity: data.quantity,
        price: data.price,
        orderType: 'limit',
      });
      setShowTradeModal(false);
      alert(`Trade submitted: ${data.side.toUpperCase()} ${data.quantity} ${data.symbol} @ $${data.price}`);
    } catch (error) {
      console.error('Trade failed:', error);
      alert('Trade submission failed. Please try again.');
    }
  };

  const handleApprove = async (approvalId: string, notes?: string) => {
    try {
      await approveRequest.mutateAsync({ requestId: approvalId, notes });
      alert(`Trade ${approvalId} approved!`);
    } catch (error) {
      console.error('Approval failed:', error);
      alert('Approval failed. Please try again.');
    }
  };

  const handleReject = async (approvalId: string, reason: string) => {
    try {
      await rejectRequest.mutateAsync({ requestId: approvalId, reason });
      alert(`Trade ${approvalId} rejected: ${reason}`);
    } catch (error) {
      console.error('Rejection failed:', error);
      alert('Rejection failed. Please try again.');
    }
  };

  return (
    <div className="min-h-screen bg-gray-100">
      {/* Header */}
      <header className="bg-white border-b border-gray-200 shadow-sm">
        <div className="max-w-7xl mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <h1 className="text-2xl font-bold text-blue-600">
                Creto Trading Demo
              </h1>
              <span className="px-2 py-1 text-xs bg-green-100 text-green-800 rounded-full font-medium">
                Live
              </span>
            </div>
            <nav className="flex gap-2">
              {(['dashboard', 'portfolio', 'oversight', 'metering'] as Tab[]).map((tab) => (
                <button
                  key={tab}
                  onClick={() => setActiveTab(tab)}
                  className={`px-4 py-2 rounded-lg transition-colors capitalize font-medium ${
                    activeTab === tab
                      ? 'bg-blue-600 text-white'
                      : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
                  }`}
                >
                  {tab}
                </button>
              ))}
            </nav>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 py-8">
        {activeTab === 'dashboard' && (
          <div className="space-y-8">
            {/* Summary Cards */}
            <section className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="bg-white rounded-lg p-6 shadow-sm">
                <h3 className="text-sm font-medium text-gray-500 mb-2">Total Portfolio Value</h3>
                <p className="text-3xl font-bold text-gray-900">
                  ${agents.reduce((sum, a) => sum + a.budget.total, 0).toLocaleString()}
                </p>
                <p className="text-sm text-green-600 mt-1">
                  +${agents.reduce((sum, a) => sum + a.performance.dailyPnL, 0).toLocaleString()} today
                </p>
              </div>
              <div className="bg-white rounded-lg p-6 shadow-sm">
                <h3 className="text-sm font-medium text-gray-500 mb-2">Pending Approvals</h3>
                <p className="text-3xl font-bold text-yellow-600">
                  {pendingApprovals.length}
                </p>
                <p className="text-sm text-gray-500 mt-1">
                  ${pendingApprovals.reduce((sum, a) => sum + a.estimatedCost, 0).toLocaleString()} total value
                </p>
              </div>
              <div className="bg-white rounded-lg p-6 shadow-sm">
                <h3 className="text-sm font-medium text-gray-500 mb-2">Active Agents</h3>
                <p className="text-3xl font-bold text-blue-600">
                  {agents.filter(a => a.status === 'active').length}
                </p>
                <p className="text-sm text-gray-500 mt-1">
                  {agents.length} total agents
                </p>
              </div>
            </section>

            {/* Agent Cards */}
            <section>
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-xl font-semibold text-gray-900">Trading Agents</h2>
                <button
                  onClick={() => setShowTradeModal(true)}
                  className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium"
                >
                  + New Trade
                </button>
              </div>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {agents.map((agent) => (
                  <AgentCard
                    key={agent.id}
                    agent={agent}
                    onSelect={setSelectedAgentId}
                    selected={selectedAgentId === agent.id}
                  />
                ))}
              </div>
            </section>
          </div>
        )}

        {activeTab === 'portfolio' && (
          <div className="space-y-6">
            <div className="bg-white rounded-lg p-6 shadow-sm">
              <h2 className="text-xl font-semibold text-gray-900 mb-4">
                Portfolio Performance - {selectedAgent.name}
              </h2>
              <PortfolioChart data={chartData} height={400} />
            </div>
          </div>
        )}

        {activeTab === 'oversight' && (
          <div className="space-y-6">
            <h2 className="text-xl font-semibold text-gray-900">Pending Approvals</h2>
            {pendingApprovals.length === 0 ? (
              <div className="text-center py-12 text-gray-500 bg-white rounded-lg">
                No pending approvals
              </div>
            ) : (
              <div className="space-y-4">
                {pendingApprovals.map((approval) => (
                  <ApprovalCard
                    key={approval.id}
                    approval={approval}
                    onApprove={handleApprove}
                    onReject={handleReject}
                    loading={isLoading}
                  />
                ))}
              </div>
            )}
          </div>
        )}

        {activeTab === 'metering' && (
          <div className="space-y-6">
            <h2 className="text-xl font-semibold text-gray-900">Usage & Metering - {selectedAgent.name}</h2>
            <div className="bg-white rounded-lg p-6 shadow-sm space-y-6">
              <UsageMeter
                used={usage.apiCalls.used}
                total={usage.apiCalls.total}
                label="API Calls"
                showWarning={true}
              />
              <UsageMeter
                used={usage.trades.used}
                total={usage.trades.total}
                label="Trades Executed"
                showWarning={true}
              />
              <UsageMeter
                used={usage.dataRequests.used}
                total={usage.dataRequests.total}
                label="Data Requests"
                showWarning={true}
              />
              <UsageMeter
                used={usage.monthlyBudget.used}
                total={usage.monthlyBudget.total}
                label="Monthly Budget ($)"
                showWarning={true}
              />
            </div>
          </div>
        )}
      </main>

      {/* Trade Modal */}
      {showTradeModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-md shadow-xl">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold text-gray-900">New Trade - {selectedAgent.name}</h3>
              <button
                onClick={() => setShowTradeModal(false)}
                className="text-gray-400 hover:text-gray-600 text-xl"
                aria-label="Close"
              >
                &times;
              </button>
            </div>
            <TradeForm
              agentId={selectedAgentId}
              onSubmit={handleTradeSubmit}
              loading={isLoading}
              maxTradeSize={50000}
            />
          </div>
        </div>
      )}
    </div>
  );
}
