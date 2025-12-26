import React from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { Provider } from 'react-redux';
import { store } from './store';
import Dashboard from './pages/Dashboard';
import Projects from './pages/Projects';
import TimeTracker from './pages/TimeTracker';
import Invoicing from './pages/Invoicing';
import Navigation from './components/shared/Navigation';

const App: React.FC = () => {
  return (
    <Provider store={store}>
      <Router>
        <div className="min-h-screen bg-gray-50">
          <Navigation />
          <main className="container mx-auto px-4 py-8">
            <Routes>
              <Route path="/" element={<Navigate to="/dashboard" replace />} />
              <Route path="/dashboard" element={<Dashboard />} />
              <Route path="/projects" element={<Projects />} />
              <Route path="/time" element={<TimeTracker />} />
              <Route path="/invoicing" element={<Invoicing />} />
            </Routes>
          </main>
        </div>
      </Router>
    </Provider>
  );
};

export default App;
