import { DiagnosisService } from '@server/services/DiagnosisService';
import { SymptomAnalysis, Patient, Symptom } from '@shared/types';

describe('DiagnosisService', () => {
  let diagnosisService: DiagnosisService;
  let mockPatient: Patient;
  let mockSymptomAnalysis: SymptomAnalysis;

  beforeEach(() => {
    diagnosisService = new DiagnosisService();

    mockPatient = {
      id: 'patient-1',
      mrn: 'MRN001',
      demographics: {
        firstName: 'John',
        lastName: 'Doe',
        dateOfBirth: new Date('1970-01-01'),
        gender: 'male',
        contactInfo: {
          phone: '555-0123',
          email: 'john@example.com',
          address: {
            street: '123 Main St',
            city: 'Boston',
            state: 'MA',
            zipCode: '02101',
            country: 'USA'
          }
        }
      },
      medicalHistory: {
        conditions: [
          {
            id: 'c1',
            icd10Code: 'I10',
            description: 'Hypertension',
            diagnosedDate: new Date(),
            status: 'chronic'
          }
        ],
        medications: [],
        allergies: []
      },
      createdAt: new Date(),
      updatedAt: new Date()
    };

    mockSymptomAnalysis = {
      normalizedSymptoms: [
        {
          id: 's1',
          encounterId: 'e1',
          description: 'chest pain',
          onset: new Date(),
          duration: '2 hours',
          severity: 8,
          characteristics: {
            quality: 'crushing'
          },
          redFlag: true,
          recordedAt: new Date(),
          recordedBy: 'user-1'
        } as Symptom
      ],
      redFlags: [],
      priorityLevel: 'stat',
      correlations: [],
      medicationEffects: [],
      patterns: ['cardiac_chest_pain']
    };
  });

  describe('generateDifferential', () => {
    it('should generate differential diagnoses', async () => {
      const result = await diagnosisService.generateDifferential({
        symptomAnalysis: mockSymptomAnalysis,
        patientContext: mockPatient,
        clinicalGuidelines: {}
      });

      expect(result.diagnoses).toBeDefined();
      expect(result.diagnoses.length).toBeGreaterThan(0);
      expect(result.totalCandidatesEvaluated).toBeGreaterThan(0);
      expect(result.analysisTimestamp).toBeInstanceOf(Date);
    });

    it('should rank diagnoses by probability', async () => {
      const result = await diagnosisService.generateDifferential({
        symptomAnalysis: mockSymptomAnalysis,
        patientContext: mockPatient,
        clinicalGuidelines: {}
      });

      expect(result.diagnoses.length).toBeGreaterThanOrEqual(2);
      expect(result.diagnoses[0].probability).toBeGreaterThanOrEqual(
        result.diagnoses[1].probability || 0
      );
    });

    it('should include ICD-10 codes for all diagnoses', async () => {
      const result = await diagnosisService.generateDifferential({
        symptomAnalysis: mockSymptomAnalysis,
        patientContext: mockPatient,
        clinicalGuidelines: {}
      });

      result.diagnoses.forEach(diagnosis => {
        expect(diagnosis.icd10Code).toBeTruthy();
        expect(typeof diagnosis.icd10Code).toBe('string');
      });
    });

    it('should provide evidence basis for each diagnosis', async () => {
      const result = await diagnosisService.generateDifferential({
        symptomAnalysis: mockSymptomAnalysis,
        patientContext: mockPatient,
        clinicalGuidelines: {}
      });

      result.diagnoses.forEach(diagnosis => {
        expect(diagnosis.evidenceBasis).toBeDefined();
        expect(Array.isArray(diagnosis.evidenceBasis)).toBe(true);
      });
    });

    it('should include recommended tests', async () => {
      const result = await diagnosisService.generateDifferential({
        symptomAnalysis: mockSymptomAnalysis,
        patientContext: mockPatient,
        clinicalGuidelines: {}
      });

      result.diagnoses.forEach(diagnosis => {
        expect(diagnosis.recommendedTests).toBeDefined();
        expect(Array.isArray(diagnosis.recommendedTests)).toBe(true);
      });
    });

    it('should calculate confidence scores', async () => {
      const result = await diagnosisService.generateDifferential({
        symptomAnalysis: mockSymptomAnalysis,
        patientContext: mockPatient,
        clinicalGuidelines: {}
      });

      result.diagnoses.forEach(diagnosis => {
        expect(diagnosis.confidenceScore).toBeDefined();
        expect(diagnosis.confidenceScore).toBeGreaterThanOrEqual(0);
        expect(diagnosis.confidenceScore).toBeLessThanOrEqual(100);
      });
    });

    it('should mark diagnoses as AI-suggested', async () => {
      const result = await diagnosisService.generateDifferential({
        symptomAnalysis: mockSymptomAnalysis,
        patientContext: mockPatient,
        clinicalGuidelines: {}
      });

      result.diagnoses.forEach(diagnosis => {
        expect(diagnosis.suggestedBy).toBe('ai');
        expect(diagnosis.status).toBe('suggested');
      });
    });

    it('should provide clinical reasoning', async () => {
      const result = await diagnosisService.generateDifferential({
        symptomAnalysis: mockSymptomAnalysis,
        patientContext: mockPatient,
        clinicalGuidelines: {}
      });

      result.diagnoses.forEach(diagnosis => {
        expect(diagnosis.reasoning).toBeTruthy();
        expect(typeof diagnosis.reasoning).toBe('string');
        expect(diagnosis.reasoning.length).toBeGreaterThan(10);
      });
    });
  });
});
