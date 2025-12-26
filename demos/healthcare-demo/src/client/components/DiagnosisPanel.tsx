import React, { useState, useEffect } from 'react';
import { Encounter, Diagnosis, Treatment, ApprovalRequest } from '@shared/types';

interface DiagnosisPanelProps {
  encounter: Encounter;
  onApprovalRequired: (request: ApprovalRequest) => void;
}

const DiagnosisPanel: React.FC<DiagnosisPanelProps> = ({ encounter, onApprovalRequired }) => {
  const [diagnoses, setDiagnoses] = useState<Diagnosis[]>([]);
  const [treatments, setTreatments] = useState<Treatment[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Simulate AI diagnosis generation
    generateDiagnoses();
  }, [encounter]);

  const generateDiagnoses = async () => {
    setLoading(true);

    // Simulate API call delay
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Mock diagnoses based on symptoms
    const mockDiagnoses: Diagnosis[] = [
      {
        id: 'd1',
        encounterId: encounter.id,
        icd10Code: 'I24.9',
        description: 'Acute Coronary Syndrome',
        type: 'differential',
        probability: 75,
        confidenceScore: 82,
        evidenceBasis: ['Patient presents with chest pain', 'Crushing quality', 'Radiation to left arm'],
        suggestedBy: 'ai',
        suggestedAt: new Date(),
        reasoning: 'Diagnosis suggested based on presenting symptoms: chest pain, crushing pain, shortness of breath. This is a potentially emergent condition requiring immediate evaluation.',
        clinicalGuidelines: ['https://www.acc.org/guidelines'],
        recommendedTests: ['Troponin', 'ECG', 'Chest X-Ray'],
        status: 'suggested'
      },
      {
        id: 'd2',
        encounterId: encounter.id,
        icd10Code: 'J18.9',
        description: 'Pneumonia, unspecified organism',
        type: 'differential',
        probability: 45,
        confidenceScore: 65,
        evidenceBasis: ['Shortness of breath', 'Possible cough'],
        suggestedBy: 'ai',
        suggestedAt: new Date(),
        reasoning: 'Diagnosis suggested based on respiratory symptoms.',
        clinicalGuidelines: ['https://www.idsociety.org/guidelines'],
        recommendedTests: ['Chest X-Ray', 'CBC', 'Blood cultures'],
        status: 'suggested'
      }
    ];

    setDiagnoses(mockDiagnoses);
    setLoading(false);
  };

  const handleConfirmDiagnosis = async (diagnosis: Diagnosis) => {
    // Update diagnosis status
    const updatedDiagnosis = { ...diagnosis, status: 'confirmed' as const };
    setDiagnoses(diagnoses.map(d => d.id === diagnosis.id ? updatedDiagnosis : d));

    // Generate treatment recommendations
    await generateTreatments(updatedDiagnosis);
  };

  const generateTreatments = async (diagnosis: Diagnosis) => {
    // Mock treatment recommendations
    const mockTreatments: Treatment[] = [
      {
        id: 't1',
        encounterId: encounter.id,
        diagnosisId: diagnosis.id,
        type: 'medication',
        riskScore: 45,
        requiresApproval: false,
        status: 'suggested',
        medication: {
          name: 'Aspirin',
          genericName: 'acetylsalicylic acid',
          rxNormCode: '1191',
          dose: '325 mg',
          route: 'PO',
          frequency: 'Once',
          duration: 'Immediate',
          controlledSubstance: false
        },
        contraindications: [],
        interactions: [],
        adverseEffects: ['Bleeding', 'Gastric irritation'],
        monitoringRequired: ['Bleeding signs', 'Platelet count'],
        suggestedBy: 'ai',
        suggestedAt: new Date(),
        reasoning: 'Evidence-based treatment for Acute Coronary Syndrome. Expected efficacy: Reduces mortality in ACS by 23%. Guideline source: ACC/AHA Guidelines for ACS Management.',
        clinicalGuidelines: ['https://www.acc.org/guidelines']
      },
      {
        id: 't2',
        encounterId: encounter.id,
        diagnosisId: diagnosis.id,
        type: 'medication',
        riskScore: 75,
        requiresApproval: true,
        status: 'suggested',
        medication: {
          name: 'Morphine',
          genericName: 'morphine sulfate',
          rxNormCode: '7052',
          dose: '2-4 mg',
          route: 'IV',
          frequency: 'PRN pain',
          duration: 'As needed',
          controlledSubstance: true,
          schedule: 'II'
        },
        contraindications: [],
        interactions: [],
        adverseEffects: ['Respiratory depression', 'Hypotension', 'Nausea'],
        monitoringRequired: ['Respiratory rate', 'Blood pressure', 'Pain level'],
        suggestedBy: 'ai',
        suggestedAt: new Date(),
        reasoning: 'Pain control for suspected ACS. Requires physician approval due to controlled substance classification.',
        clinicalGuidelines: ['https://www.acc.org/guidelines']
      }
    ];

    setTreatments(mockTreatments);

    // Check if any treatments require approval
    mockTreatments.forEach(treatment => {
      if (treatment.requiresApproval) {
        const approvalRequest: ApprovalRequest = {
          id: `ar-${treatment.id}`,
          encounterId: encounter.id,
          treatmentId: treatment.id,
          requestedBy: 'current-user-id',
          requestedAt: new Date(),
          priority: 'urgent',
          status: 'pending',
          approvalType: 'multi-level',
          requiredApprovers: ['attending_physician', 'pharmacist'],
          approvers: [],
          patientRiskFactors: ['Suspected ACS', 'Chest pain severity 8/10'],
          clinicalJustification: `Pain control for suspected Acute Coronary Syndrome. Risk Score: ${treatment.riskScore}/100`,
          alternativesConsidered: ['Non-opioid analgesics', 'Nitroglycerin'],
          oversightReason: ['controlled_substance', 'high_risk'],
          urgencyReason: 'Emergency department presentation with red flag symptoms'
        };

        onApprovalRequired(approvalRequest);
      }
    });
  };

  if (loading) {
    return (
      <div className="diagnosis-panel">
        <div className="loading-state">
          <div className="spinner"></div>
          <p>Analyzing symptoms and generating differential diagnoses...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="diagnosis-panel">
      <h3>AI-Generated Differential Diagnoses</h3>

      <div className="diagnoses-list">
        {diagnoses.map(diagnosis => (
          <div key={diagnosis.id} className="diagnosis-card">
            <div className="diagnosis-header">
              <h4>{diagnosis.description}</h4>
              <div className="diagnosis-scores">
                <span className="probability">Probability: {diagnosis.probability}%</span>
                <span className="confidence">Confidence: {diagnosis.confidenceScore}%</span>
              </div>
            </div>

            <div className="diagnosis-details">
              <p className="icd-code">ICD-10: {diagnosis.icd10Code}</p>
              <p className="reasoning">{diagnosis.reasoning}</p>

              <div className="evidence">
                <h5>Supporting Evidence:</h5>
                <ul>
                  {diagnosis.evidenceBasis.map((evidence, idx) => (
                    <li key={idx}>{evidence}</li>
                  ))}
                </ul>
              </div>

              <div className="recommended-tests">
                <h5>Recommended Diagnostic Tests:</h5>
                <ul>
                  {diagnosis.recommendedTests?.map((test, idx) => (
                    <li key={idx}>{test}</li>
                  ))}
                </ul>
              </div>
            </div>

            <div className="diagnosis-actions">
              {diagnosis.status === 'suggested' && (
                <>
                  <button
                    onClick={() => handleConfirmDiagnosis(diagnosis)}
                    className="btn-confirm"
                  >
                    Confirm Diagnosis & Get Treatment Recommendations
                  </button>
                  <button className="btn-reject">
                    Rule Out
                  </button>
                </>
              )}
              {diagnosis.status === 'confirmed' && (
                <span className="confirmed-badge">✓ Confirmed</span>
              )}
            </div>
          </div>
        ))}
      </div>

      {treatments.length > 0 && (
        <div className="treatments-section">
          <h3>Treatment Recommendations</h3>
          <div className="treatments-list">
            {treatments.map(treatment => (
              <div key={treatment.id} className="treatment-card">
                <div className="treatment-header">
                  <h4>{treatment.medication?.name || treatment.type}</h4>
                  <div className={`risk-score risk-${treatment.riskScore >= 70 ? 'high' : treatment.riskScore >= 40 ? 'medium' : 'low'}`}>
                    Risk Score: {treatment.riskScore}/100
                  </div>
                </div>

                {treatment.medication && (
                  <div className="medication-details">
                    <p><strong>Dose:</strong> {treatment.medication.dose}</p>
                    <p><strong>Route:</strong> {treatment.medication.route}</p>
                    <p><strong>Frequency:</strong> {treatment.medication.frequency}</p>
                    {treatment.medication.controlledSubstance && (
                      <span className="controlled-badge">⚠️ Schedule {treatment.medication.schedule} Controlled Substance</span>
                    )}
                  </div>
                )}

                <div className="treatment-reasoning">
                  <p>{treatment.reasoning}</p>
                </div>

                {treatment.requiresApproval && (
                  <div className="approval-required">
                    <span className="approval-badge">🔒 Physician Approval Required</span>
                    <p>This treatment requires oversight due to: {treatment.medication?.controlledSubstance ? 'Controlled Substance' : 'High Risk'}</p>
                    <p className="approval-status">Status: Pending Approval</p>
                  </div>
                )}

                {!treatment.requiresApproval && (
                  <button className="btn-prescribe">
                    Prescribe
                  </button>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default DiagnosisPanel;
