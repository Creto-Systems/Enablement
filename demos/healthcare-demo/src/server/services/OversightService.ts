import { ApprovalRequest, Treatment, Encounter, ApproverDecision } from '@shared/types';
import { v4 as uuid } from 'uuid';

interface ApprovalChainLink {
  role: string;
  specialty?: string;
  required: boolean;
}

export class OversightService {
  async createApprovalRequest(
    treatment: Treatment,
    encounter: Encounter,
    requestedBy: string
  ): Promise<ApprovalRequest> {
    const approvalChain = this.buildApprovalChain(treatment);
    const priority = this.determinePriority(treatment, encounter);

    const request: ApprovalRequest = {
      id: uuid(),
      encounterId: encounter.id,
      treatmentId: treatment.id,
      requestedBy,
      requestedAt: new Date(),
      priority,
      status: 'pending',
      approvalType: this.determineApprovalType(treatment),
      requiredApprovers: approvalChain.map(link => link.role),
      approvers: [],
      patientRiskFactors: this.extractRiskFactors(encounter),
      clinicalJustification: this.generateJustification(treatment, encounter),
      alternativesConsidered: [],
      oversightReason: this.extractOversightReasons(treatment),
      urgencyReason: priority === 'stat' ? 'Emergency department presentation with red flag symptoms' : undefined
    };

    return request;
  }

  async processApproval(
    request: ApprovalRequest,
    userId: string,
    userRole: string,
    decision: 'approved' | 'rejected',
    justification: string
  ): Promise<ApprovalRequest> {
    // Validate user is authorized approver
    if (!this.isAuthorizedApprover(userId, userRole, request)) {
      throw new Error('User not authorized to approve this request');
    }

    // Validate request is still pending
    if (request.status !== 'pending') {
      throw new Error('Approval request already resolved');
    }

    // Record decision
    const approverDecision: ApproverDecision = {
      userId,
      role: userRole,
      decision,
      justification,
      timestamp: new Date()
    };
    request.approvers.push(approverDecision);

    // Determine next state
    if (decision === 'rejected') {
      request.status = 'rejected';
      request.resolvedAt = new Date();
      request.timeToResolution = this.calculateTimeToResolution(request);
    } else if (this.isFullyApproved(request)) {
      request.status = 'approved';
      request.resolvedAt = new Date();
      request.timeToResolution = this.calculateTimeToResolution(request);
    }

    return request;
  }

  async getApprovalQueue(userId: string, userRole: string): Promise<ApprovalRequest[]> {
    // In production, this would query the database
    // For demo, return empty array
    return [];
  }

  private buildApprovalChain(treatment: Treatment): ApprovalChainLink[] {
    const chain: ApprovalChainLink[] = [];

    // Rule-based approval routing
    if (treatment.medication?.controlledSubstance) {
      chain.push({ role: 'attending_physician', required: true });
      chain.push({ role: 'pharmacist', required: true });
    }

    if (treatment.riskScore >= 70) {
      chain.push({ role: 'specialist', required: true });
    }

    // Default to physician approval if no specific rules matched
    if (chain.length === 0) {
      chain.push({ role: 'physician', required: true });
    }

    return chain;
  }

  private determineApprovalType(treatment: Treatment): ApprovalRequest['approvalType'] {
    if (treatment.medication?.controlledSubstance) {
      return 'multi-level';
    }
    if (treatment.riskScore >= 70) {
      return 'specialist';
    }
    return 'physician';
  }

  private determinePriority(
    treatment: Treatment,
    encounter: Encounter
  ): ApprovalRequest['priority'] {
    // STAT priority for emergencies with low risk
    if (encounter.type === 'emergency' && treatment.riskScore < 30) {
      return 'stat';
    }

    // Check for red flag symptoms
    const hasRedFlags = encounter.symptoms.some(s => s.redFlag);
    if (hasRedFlags) {
      return 'stat';
    }

    // Urgent priority for high-risk emergencies
    if (treatment.riskScore >= 70 && encounter.type === 'emergency') {
      return 'urgent';
    }

    // Controlled substances
    if (treatment.medication?.controlledSubstance &&
        ['II', 'III'].includes(treatment.medication.schedule || '')) {
      return 'urgent';
    }

    return 'routine';
  }

  private extractRiskFactors(encounter: Encounter): string[] {
    const riskFactors: string[] = [];

    // Extract from red flag symptoms
    encounter.symptoms
      .filter(s => s.redFlag)
      .forEach(s => {
        riskFactors.push(`Red flag symptom: ${s.description}`);
      });

    return riskFactors;
  }

  private generateJustification(treatment: Treatment, encounter: Encounter): string {
    const parts: string[] = [];

    parts.push(`Clinical Indication: ${encounter.chiefComplaint}`);
    parts.push(`Risk Score: ${treatment.riskScore}/100`);

    if (treatment.reasoning) {
      parts.push(`Rationale: ${treatment.reasoning}`);
    }

    return parts.join('\n');
  }

  private extractOversightReasons(treatment: Treatment): ApprovalRequest['oversightReason'] {
    const reasons: ApprovalRequest['oversightReason'] = [];

    if (treatment.medication?.controlledSubstance) {
      reasons.push('controlled_substance');
    }

    if (treatment.riskScore >= 70) {
      reasons.push('high_risk');
    }

    return reasons;
  }

  private isAuthorizedApprover(
    userId: string,
    userRole: string,
    request: ApprovalRequest
  ): boolean {
    // Check if user's role is in required approvers
    return request.requiredApprovers.includes(userRole);
  }

  private isFullyApproved(request: ApprovalRequest): boolean {
    // Check if all required approvers have approved
    const approvedRoles = request.approvers
      .filter(a => a.decision === 'approved')
      .map(a => a.role);

    return request.requiredApprovers.every(required =>
      approvedRoles.includes(required)
    );
  }

  private calculateTimeToResolution(request: ApprovalRequest): number {
    if (!request.resolvedAt) return 0;

    const diff = request.resolvedAt.getTime() - request.requestedAt.getTime();
    return Math.floor(diff / 1000); // Convert to seconds
  }
}
