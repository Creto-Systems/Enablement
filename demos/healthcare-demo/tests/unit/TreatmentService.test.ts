import { TreatmentService } from '@server/services/TreatmentService';
import { Diagnosis, Patient } from '@shared/types';

describe('TreatmentService', () => {
  let treatmentService: TreatmentService;
  let mockPatient: Patient;
  let mockDiagnosis: Diagnosis;

  beforeEach(() => {
    treatmentService = new TreatmentService();

    mockPatient = {
      id: 'patient-1',
      mrn: 'MRN001',
      demographics: {
        firstName: 'Jane',
        lastName: 'Smith',
        dateOfBirth: new Date('1980-06-15'),
        gender: 'female',
        contactInfo: {
          phone: '555-0456',
          email: 'jane@example.com',
          address: {
            street: '456 Oak Ave',
            city: 'Cambridge',
            state: 'MA',
            zipCode: '02138',
            country: 'USA'
          }
        }
      },
      medicalHistory: {
        conditions: [],
        medications: [],
        allergies: [
          {
            id: 'a1',
            allergen: 'Penicillin',
            reaction: 'Rash',
            severity: 'moderate',
            verifiedDate: new Date()
          }
        ]
      },
      createdAt: new Date(),
      updatedAt: new Date()
    };

    mockDiagnosis = {
      id: 'd1',
      encounterId: 'e1',
      icd10Code: 'I24.9',
      description: 'Acute Coronary Syndrome',
      type: 'confirmed',
      suggestedBy: 'ai',
      suggestedAt: new Date(),
      reasoning: 'Test diagnosis',
      evidenceBasis: [],
      status: 'confirmed'
    };
  });

  describe('recommend', () => {
    it('should generate treatment recommendations', async () => {
      const treatments = await treatmentService.recommend(mockDiagnosis, mockPatient);

      expect(treatments).toBeDefined();
      expect(Array.isArray(treatments)).toBe(true);
      expect(treatments.length).toBeGreaterThan(0);
    });

    it('should calculate risk scores for all treatments', async () => {
      const treatments = await treatmentService.recommend(mockDiagnosis, mockPatient);

      treatments.forEach(treatment => {
        expect(treatment.riskScore).toBeDefined();
        expect(treatment.riskScore).toBeGreaterThanOrEqual(0);
        expect(treatment.riskScore).toBeLessThanOrEqual(100);
      });
    });

    it('should identify controlled substances requiring approval', async () => {
      const treatments = await treatmentService.recommend(mockDiagnosis, mockPatient);

      const controlledSubstances = treatments.filter(
        t => t.medication?.controlledSubstance
      );

      if (controlledSubstances.length > 0) {
        controlledSubstances.forEach(treatment => {
          expect(treatment.requiresApproval).toBe(true);
        });
      }
    });

    it('should flag high-risk treatments for approval', async () => {
      const treatments = await treatmentService.recommend(mockDiagnosis, mockPatient);

      const highRiskTreatments = treatments.filter(t => t.riskScore >= 70);

      highRiskTreatments.forEach(treatment => {
        expect(treatment.requiresApproval).toBe(true);
      });
    });

    it('should include medication details when applicable', async () => {
      const treatments = await treatmentService.recommend(mockDiagnosis, mockPatient);

      const medicationTreatments = treatments.filter(t => t.type === 'medication');

      medicationTreatments.forEach(treatment => {
        expect(treatment.medication).toBeDefined();
        expect(treatment.medication?.name).toBeTruthy();
        expect(treatment.medication?.dose).toBeTruthy();
        expect(treatment.medication?.route).toBeTruthy();
        expect(treatment.medication?.frequency).toBeTruthy();
      });
    });

    it('should check drug interactions', async () => {
      mockPatient.medicalHistory.medications = [
        {
          id: 'm1',
          name: 'Warfarin',
          genericName: 'warfarin',
          rxNormCode: '11289',
          dose: '5 mg',
          route: 'PO',
          frequency: 'Daily',
          startDate: new Date(),
          prescribedBy: 'Dr. Test',
          controlledSubstance: false
        }
      ];

      const treatments = await treatmentService.recommend(mockDiagnosis, mockPatient);

      expect(treatments).toBeDefined();
      // Interactions array should exist even if empty
      treatments.forEach(treatment => {
        expect(treatment.interactions).toBeDefined();
        expect(Array.isArray(treatment.interactions)).toBe(true);
      });
    });

    it('should include monitoring requirements', async () => {
      const treatments = await treatmentService.recommend(mockDiagnosis, mockPatient);

      treatments.forEach(treatment => {
        expect(treatment.monitoringRequired).toBeDefined();
        expect(Array.isArray(treatment.monitoringRequired)).toBe(true);
      });
    });

    it('should provide clinical reasoning', async () => {
      const treatments = await treatmentService.recommend(mockDiagnosis, mockPatient);

      treatments.forEach(treatment => {
        expect(treatment.reasoning).toBeTruthy();
        expect(typeof treatment.reasoning).toBe('string');
      });
    });

    it('should rank treatments by safety', async () => {
      const treatments = await treatmentService.recommend(mockDiagnosis, mockPatient);

      if (treatments.length >= 2) {
        // Treatments should be sorted by risk score (ascending)
        for (let i = 0; i < treatments.length - 1; i++) {
          expect(treatments[i].riskScore).toBeLessThanOrEqual(
            treatments[i + 1].riskScore
          );
        }
      }
    });
  });
});
