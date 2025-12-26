import React, { useState } from 'react';
import { Patient, Encounter, ApprovalRequest } from '@shared/types';
import PatientCard from './components/PatientCard';
import SymptomInput from './components/SymptomInput';
import DiagnosisPanel from './components/DiagnosisPanel';
import ApprovalQueue from './components/ApprovalQueue';
import './App.css';

interface AppState {
  selectedPatient: Patient | null;
  currentEncounter: Encounter | null;
  approvalQueue: ApprovalRequest[];
  activeView: 'clinical' | 'approvals' | 'audit';
}

const App: React.FC = () => {
  const [state, setState] = useState<AppState>({
    selectedPatient: null,
    currentEncounter: null,
    approvalQueue: [],
    activeView: 'clinical'
  });

  const handlePatientSelect = (patient: Patient) => {
    setState(prev => ({ ...prev, selectedPatient: patient }));
  };

  const handleEncounterCreated = (encounter: Encounter) => {
    setState(prev => ({ ...prev, currentEncounter: encounter }));
  };

  const handleApprovalRequired = (request: ApprovalRequest) => {
    setState(prev => ({
      ...prev,
      approvalQueue: [...prev.approvalQueue, request]
    }));
  };

  return (
    <div className="app">
      <header className="app-header">
        <h1>Clinical Decision Support System</h1>
        <nav>
          <button
            className={state.activeView === 'clinical' ? 'active' : ''}
            onClick={() => setState(prev => ({ ...prev, activeView: 'clinical' }))}
          >
            Clinical Dashboard
          </button>
          <button
            className={state.activeView === 'approvals' ? 'active' : ''}
            onClick={() => setState(prev => ({ ...prev, activeView: 'approvals' }))}
          >
            Approval Queue ({state.approvalQueue.length})
          </button>
          <button
            className={state.activeView === 'audit' ? 'active' : ''}
            onClick={() => setState(prev => ({ ...prev, activeView: 'audit' }))}
          >
            Audit Trail
          </button>
        </nav>
      </header>

      <main className="app-main">
        {state.activeView === 'clinical' && (
          <div className="clinical-view">
            <aside className="patient-sidebar">
              <h2>Patient Selection</h2>
              <PatientCard
                patient={state.selectedPatient}
                onSelect={handlePatientSelect}
              />
            </aside>

            <section className="encounter-section">
              {state.selectedPatient ? (
                <>
                  <h2>Active Encounter</h2>
                  <SymptomInput
                    patientId={state.selectedPatient.id}
                    onEncounterCreated={handleEncounterCreated}
                  />

                  {state.currentEncounter && (
                    <DiagnosisPanel
                      encounter={state.currentEncounter}
                      onApprovalRequired={handleApprovalRequired}
                    />
                  )}
                </>
              ) : (
                <div className="placeholder">
                  <p>Select a patient to begin clinical encounter</p>
                </div>
              )}
            </section>
          </div>
        )}

        {state.activeView === 'approvals' && (
          <div className="approval-view">
            <ApprovalQueue
              approvalRequests={state.approvalQueue}
              onApprovalProcessed={(id, decision) => {
                setState(prev => ({
                  ...prev,
                  approvalQueue: prev.approvalQueue.filter(req => req.id !== id)
                }));
              }}
            />
          </div>
        )}

        {state.activeView === 'audit' && (
          <div className="audit-view">
            <h2>Audit Trail</h2>
            <p>Comprehensive audit log showing all clinical decisions and approvals</p>
            <div className="audit-placeholder">
              Audit trail functionality would display here in full implementation
            </div>
          </div>
        )}
      </main>

      <footer className="app-footer">
        <div className="compliance-badge">
          <span>HIPAA Compliant</span>
          <span>|</span>
          <span>Creto-Oversight Enabled</span>
          <span>|</span>
          <span>AI-Assisted</span>
        </div>
      </footer>
    </div>
  );
};

export default App;
