import React, { useState } from 'react';
import TripForm from './components/TripForm';
import AgentCard from './components/AgentCard';
import MessagePanel from './components/MessagePanel';
import BudgetTracker from './components/BudgetTracker';
import ItineraryCard from './components/ItineraryCard';
import NetworkGraph from './components/NetworkGraph';
import { useTrip } from './hooks/useTrip';
import { useAgents } from './hooks/useAgents';
import { useMessages } from './hooks/useMessages';
import { useBudget } from './hooks/useBudget';
import type { CreateTripRequest } from '../shared/types';

type TabType = 'planning' | 'network' | 'messages' | 'budget';

function App() {
  const [activeTab, setActiveTab] = useState<TabType>('planning');
  const { trip, itinerary, createTrip, loading: tripLoading } = useTrip();
  const { agents, connections } = useAgents(trip?.id);
  const { messages, sendMessage } = useMessages(trip?.id);
  const { budgetAnalysis } = useBudget(trip?.id);

  const handleCreateTrip = async (request: CreateTripRequest) => {
    await createTrip(request);
  };

  const tabs: { id: TabType; label: string; icon: string }[] = [
    { id: 'planning', label: 'Trip Planning', icon: '✈️' },
    { id: 'network', label: 'Agent Network', icon: '🕸️' },
    { id: 'messages', label: 'Messages', icon: '🔒' },
    { id: 'budget', label: 'Budget', icon: '💰' },
  ];

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-white to-purple-50 dark:from-gray-900 dark:via-gray-800 dark:to-gray-900">
      {/* Header */}
      <header className="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
                Travel Demo
              </h1>
              <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
                Multi-Agent Trip Planner with E2E Encrypted Communication
              </p>
            </div>
            <div className="flex items-center gap-2">
              <div className="encrypted-badge">
                🔒 creto-messaging
              </div>
            </div>
          </div>
        </div>
      </header>

      {/* Tab Navigation */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 mt-6">
        <div className="border-b border-gray-200 dark:border-gray-700">
          <nav className="-mb-px flex space-x-8" aria-label="Tabs">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`
                  whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm flex items-center gap-2
                  ${
                    activeTab === tab.id
                      ? 'border-primary text-primary'
                      : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-300'
                  }
                `}
              >
                <span className="text-lg">{tab.icon}</span>
                {tab.label}
              </button>
            ))}
          </nav>
        </div>
      </div>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {activeTab === 'planning' && (
          <div className="space-y-8">
            {!trip ? (
              <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8">
                <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">
                  Plan Your Trip
                </h2>
                <TripForm onSubmit={handleCreateTrip} loading={tripLoading} />
              </div>
            ) : (
              <div className="space-y-6">
                {/* Trip Overview */}
                <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
                  <div className="flex items-start justify-between">
                    <div>
                      <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
                        {trip.destination}
                      </h2>
                      <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
                        {new Date(trip.startDate).toLocaleDateString()} -{' '}
                        {new Date(trip.endDate).toLocaleDateString()}
                      </p>
                    </div>
                    <div className="text-right">
                      <p className="text-sm text-gray-500 dark:text-gray-400">
                        {trip.travelerCount}{' '}
                        {trip.travelerCount === 1 ? 'Traveler' : 'Travelers'}
                      </p>
                      <p className="mt-1 text-lg font-semibold text-gray-900 dark:text-white">
                        {trip.budget.currency} {trip.budget.min.toLocaleString()} -{' '}
                        {trip.budget.max.toLocaleString()}
                      </p>
                    </div>
                  </div>
                </div>

                {/* Agents Grid */}
                <div>
                  <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                    Active Agents
                  </h3>
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                    {agents.map((agent) => (
                      <AgentCard key={agent.id} agent={agent} />
                    ))}
                  </div>
                </div>

                {/* Itinerary */}
                {itinerary && itinerary.status !== 'building' && (
                  <div>
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                      Your Itinerary
                    </h3>
                    <ItineraryCard itinerary={itinerary} />
                  </div>
                )}
              </div>
            )}
          </div>
        )}

        {activeTab === 'network' && trip && (
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8">
            <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">
              Agent Communication Network
            </h2>
            <NetworkGraph agents={agents} connections={connections} />
          </div>
        )}

        {activeTab === 'messages' && trip && (
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8">
            <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6 flex items-center gap-2">
              Encrypted Agent Messages
              <span className="encrypted-badge">E2E Encrypted</span>
            </h2>
            <MessagePanel messages={messages} onSend={sendMessage} />
          </div>
        )}

        {activeTab === 'budget' && trip && budgetAnalysis && (
          <div className="space-y-6">
            <BudgetTracker budget={budgetAnalysis} />
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
