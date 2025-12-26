import { OversightService } from '@server/services/OversightService';
import { Treatment, Encounter, ApprovalRequest } from '@shared/types';

describe('OversightService', () => {
  let oversightService: OversightService;
  let mockTreatment: Treatment;
  let mockEncounter: Encounter;

  beforeEach(() => {
    oversightService = new OversightService();

    mockEncounter = {
      id: 'e1',
      patientId: 'p1',
      type: 'emergency',
      status: 'active',
      chiefComplaint: 'Chest pain',
      symptoms: [
        {
          id: 's1',
          encounterId: 'e1',
          description: 'Severe chest pain',
          onset: new Date(),
          duration: '1 hour',
          severity: 9,
          characteristics: {},
          redFlag: true,
          recordedAt: new Date(),
          recordedBy: 'user-1'
        }
      ],
      diagnoses: [],
      treatments: [],
      approvalRequests: [],
      providerId: 'provider-1',
      facilityId: 'facility-1',
      startTime: new Date()
    };

    mockTreatment = {
      id: 't1',
      encounterId: 'e1',
      diagnosisId: 'd1',
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
        frequency: 'PRN',
        duration: 'As needed',
        controlledSubstance: true,
        schedule: 'II'
      },
      contraindications: [],
      interactions: [],
      adverseEffects: [],
      monitoringRequired: [],
      suggestedBy: 'ai',
      suggestedAt: new Date(),
      reasoning: 'Pain control'
    };
  });

  describe('createApprovalRequest', () => {
    it('should create approval request for controlled substances', async () => {
      const request = await oversightService.createApprovalRequest(
        mockTreatment,
        mockEncounter,
        'physician-1'
      );

      expect(request).toBeDefined();
      expect(request.id).toBeTruthy();
      expect(request.treatmentId).toBe(mockTreatment.id);
      expect(request.status).toBe('pending');
      expect(request.oversightReason).toContain('controlled_substance');
    });

    it('should set correct priority for emergency encounters', async () => {
      const request = await oversightService.createApprovalRequest(
        mockTreatment,
        mockEncounter,
        'physician-1'
      );

      expect(request.priority).toBe('urgent');
    });

    it('should set STAT priority for low-risk emergency treatments', async () => {
      mockTreatment.riskScore = 25;
      mockTreatment.medication!.controlledSubstance = false;

      const request = await oversightService.createApprovalRequest(
        mockTreatment,
        mockEncounter,
        'physician-1'
      );

      expect(request.priority).toBe('stat');
    });

    it('should require multi-level approval for controlled substances', async () => {
      const request = await oversightService.createApprovalRequest(
        mockTreatment,
        mockEncounter,
        'physician-1'
      );

      expect(request.approvalType).toBe('multi-level');
      expect(request.requiredApprovers).toContain('attending_physician');
      expect(request.requiredApprovers).toContain('pharmacist');
    });

    it('should include clinical justification', async () => {
      const request = await oversightService.createApprovalRequest(
        mockTreatment,
        mockEncounter,
        'physician-1'
      );

      expect(request.clinicalJustification).toBeTruthy();
      expect(request.clinicalJustification).toContain(mockEncounter.chiefComplaint);
      expect(request.clinicalJustification).toContain(mockTreatment.riskScore.toString());
    });

    it('should extract patient risk factors from symptoms', async () => {
      const request = await oversightService.createApprovalRequest(
        mockTreatment,
        mockEncounter,
        'physician-1'
      );

      expect(request.patientRiskFactors).toBeDefined();
      expect(Array.isArray(request.patientRiskFactors)).toBe(true);
    });
  });

  describe('processApproval', () => {
    let approvalRequest: ApprovalRequest;

    beforeEach(async () => {
      approvalRequest = await oversightService.createApprovalRequest(
        mockTreatment,
        mockEncounter,
        'physician-1'
      );
    });

    it('should record approval decision', async () => {
      const result = await oversightService.processApproval(
        approvalRequest,
        'approver-1',
        'attending_physician',
        'approved',
        'Appropriate for pain control'
      );

      expect(result.approvers.length).toBe(1);
      expect(result.approvers[0].decision).toBe('approved');
      expect(result.approvers[0].justification).toBe('Appropriate for pain control');
    });

    it('should reject request immediately on rejection', async () => {
      const result = await oversightService.processApproval(
        approvalRequest,
        'approver-1',
        'attending_physician',
        'rejected',
        'Alternative treatment preferred'
      );

      expect(result.status).toBe('rejected');
      expect(result.resolvedAt).toBeInstanceOf(Date);
    });

    it('should require all approvers for multi-level approval', async () => {
      // First approval
      let result = await oversightService.processApproval(
        approvalRequest,
        'physician-1',
        'attending_physician',
        'approved',
        'Medically necessary'
      );

      expect(result.status).toBe('pending');

      // Second approval
      result = await oversightService.processApproval(
        result,
        'pharmacist-1',
        'pharmacist',
        'approved',
        'No contraindications'
      );

      expect(result.status).toBe('approved');
      expect(result.resolvedAt).toBeInstanceOf(Date);
    });

    it('should calculate time to resolution', async () => {
      const result = await oversightService.processApproval(
        approvalRequest,
        'approver-1',
        'attending_physician',
        'rejected',
        'Test rejection'
      );

      expect(result.timeToResolution).toBeDefined();
      expect(result.timeToResolution).toBeGreaterThanOrEqual(0);
    });

    it('should throw error for unauthorized approver', async () => {
      await expect(
        oversightService.processApproval(
          approvalRequest,
          'unauthorized-user',
          'nurse',
          'approved',
          'Test'
        )
      ).rejects.toThrow('not authorized');
    });

    it('should throw error for already resolved requests', async () => {
      approvalRequest.status = 'approved';

      await expect(
        oversightService.processApproval(
          approvalRequest,
          'approver-1',
          'attending_physician',
          'approved',
          'Test'
        )
      ).rejects.toThrow('already resolved');
    });
  });
});
