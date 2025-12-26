import React, { useState } from 'react';
import { Encounter, Symptom } from '@shared/types';
import { v4 as uuid } from 'uuid';

interface SymptomInputProps {
  patientId: string;
  onEncounterCreated: (encounter: Encounter) => void;
}

const SymptomInput: React.FC<SymptomInputProps> = ({ patientId, onEncounterCreated }) => {
  const [chiefComplaint, setChiefComplaint] = useState('');
  const [symptoms, setSymptoms] = useState<Partial<Symptom>[]>([]);
  const [currentSymptom, setCurrentSymptom] = useState({
    description: '',
    onset: '',
    duration: '',
    severity: 5,
    location: '',
    quality: ''
  });

  const handleAddSymptom = () => {
    if (!currentSymptom.description) return;

    const symptom: Partial<Symptom> = {
      id: uuid(),
      description: currentSymptom.description,
      onset: new Date(currentSymptom.onset || new Date()),
      duration: currentSymptom.duration,
      severity: currentSymptom.severity,
      characteristics: {
        location: currentSymptom.location,
        quality: currentSymptom.quality
      },
      redFlag: currentSymptom.severity >= 8 || isRedFlagSymptom(currentSymptom.description),
      recordedAt: new Date(),
      recordedBy: 'current-user-id'
    };

    setSymptoms([...symptoms, symptom]);
    setCurrentSymptom({
      description: '',
      onset: '',
      duration: '',
      severity: 5,
      location: '',
      quality: ''
    });
  };

  const isRedFlagSymptom = (description: string): boolean => {
    const redFlags = [
      'chest pain',
      'crushing pain',
      'shortness of breath',
      'severe headache',
      'altered mental status',
      'seizure'
    ];
    return redFlags.some(flag => description.toLowerCase().includes(flag));
  };

  const handleCreateEncounter = () => {
    if (!chiefComplaint || symptoms.length === 0) {
      alert('Please enter chief complaint and at least one symptom');
      return;
    }

    const encounter: Encounter = {
      id: uuid(),
      patientId,
      type: 'outpatient',
      status: 'active',
      chiefComplaint,
      symptoms: symptoms as Symptom[],
      diagnoses: [],
      treatments: [],
      approvalRequests: [],
      providerId: 'current-provider-id',
      facilityId: 'facility-1',
      startTime: new Date()
    };

    onEncounterCreated(encounter);
  };

  return (
    <div className="symptom-input">
      <h3>New Encounter</h3>

      <div className="form-group">
        <label>Chief Complaint</label>
        <input
          type="text"
          value={chiefComplaint}
          onChange={(e) => setChiefComplaint(e.target.value)}
          placeholder="e.g., Chest pain, Shortness of breath"
        />
      </div>

      <h4>Symptoms</h4>
      <div className="symptom-form">
        <div className="form-row">
          <div className="form-group">
            <label>Description *</label>
            <input
              type="text"
              value={currentSymptom.description}
              onChange={(e) => setCurrentSymptom({ ...currentSymptom, description: e.target.value })}
              placeholder="Describe the symptom"
            />
          </div>

          <div className="form-group">
            <label>Location</label>
            <input
              type="text"
              value={currentSymptom.location}
              onChange={(e) => setCurrentSymptom({ ...currentSymptom, location: e.target.value })}
              placeholder="e.g., Chest, Left arm"
            />
          </div>
        </div>

        <div className="form-row">
          <div className="form-group">
            <label>Onset</label>
            <input
              type="datetime-local"
              value={currentSymptom.onset}
              onChange={(e) => setCurrentSymptom({ ...currentSymptom, onset: e.target.value })}
            />
          </div>

          <div className="form-group">
            <label>Duration</label>
            <input
              type="text"
              value={currentSymptom.duration}
              onChange={(e) => setCurrentSymptom({ ...currentSymptom, duration: e.target.value })}
              placeholder="e.g., 2 hours, 3 days"
            />
          </div>
        </div>

        <div className="form-row">
          <div className="form-group">
            <label>Severity (1-10): {currentSymptom.severity}</label>
            <input
              type="range"
              min="1"
              max="10"
              value={currentSymptom.severity}
              onChange={(e) => setCurrentSymptom({ ...currentSymptom, severity: parseInt(e.target.value) })}
            />
          </div>

          <div className="form-group">
            <label>Quality</label>
            <select
              value={currentSymptom.quality}
              onChange={(e) => setCurrentSymptom({ ...currentSymptom, quality: e.target.value })}
            >
              <option value="">Select quality</option>
              <option value="sharp">Sharp</option>
              <option value="dull">Dull</option>
              <option value="burning">Burning</option>
              <option value="crushing">Crushing</option>
              <option value="aching">Aching</option>
            </select>
          </div>
        </div>

        <button onClick={handleAddSymptom} className="btn-add-symptom">
          Add Symptom
        </button>
      </div>

      {symptoms.length > 0 && (
        <div className="symptoms-list">
          <h5>Recorded Symptoms</h5>
          {symptoms.map((symptom, index) => (
            <div key={symptom.id} className={`symptom-item ${symptom.redFlag ? 'red-flag' : ''}`}>
              {symptom.redFlag && <span className="red-flag-badge">⚠️ RED FLAG</span>}
              <div className="symptom-description">{symptom.description}</div>
              <div className="symptom-details">
                <span>Severity: {symptom.severity}/10</span>
                {symptom.characteristics?.location && <span>Location: {symptom.characteristics.location}</span>}
                {symptom.duration && <span>Duration: {symptom.duration}</span>}
              </div>
              <button
                onClick={() => setSymptoms(symptoms.filter((_, i) => i !== index))}
                className="btn-remove"
              >
                Remove
              </button>
            </div>
          ))}
        </div>
      )}

      <div className="encounter-actions">
        <button
          onClick={handleCreateEncounter}
          className="btn-primary"
          disabled={!chiefComplaint || symptoms.length === 0}
        >
          Analyze Symptoms & Generate Recommendations
        </button>
      </div>
    </div>
  );
};

export default SymptomInput;
