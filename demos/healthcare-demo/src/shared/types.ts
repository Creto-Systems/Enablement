// Shared type definitions for client and server

export interface Patient {
  id: string;
  mrn: string;
  demographics: {
    firstName: string;
    lastName: string;
    dateOfBirth: Date;
    gender: 'male' | 'female' | 'other' | 'unknown';
    contactInfo: {
      phone: string;
      email?: string;
      address: Address;
    };
  };
  insurance?: {
    provider: string;
    policyNumber: string;
    groupNumber: string;
  };
  medicalHistory: {
    conditions: Condition[];
    medications: Medication[];
    allergies: Allergy[];
  };
  createdAt: Date;
  updatedAt: Date;
}

export interface Address {
  street: string;
  city: string;
  state: string;
  zipCode: string;
  country: string;
}

export interface Condition {
  id: string;
  icd10Code: string;
  description: string;
  diagnosedDate: Date;
  status: 'active' | 'resolved' | 'chronic';
}

export interface Medication {
  id: string;
  name: string;
  genericName: string;
  rxNormCode: string;
  dose: string;
  route: string;
  frequency: string;
  startDate: Date;
  endDate?: Date;
  prescribedBy: string;
  controlledSubstance: boolean;
}

export interface Allergy {
  id: string;
  allergen: string;
  reaction: string;
  severity: 'mild' | 'moderate' | 'severe' | 'anaphylaxis';
  verifiedDate: Date;
}

export interface Encounter {
  id: string;
  patientId: string;
  type: 'outpatient' | 'emergency' | 'inpatient' | 'telehealth';
  status: 'active' | 'completed' | 'cancelled';
  chiefComplaint: string;
  symptoms: Symptom[];
  vitalSigns?: VitalSigns;
  diagnoses: Diagnosis[];
  treatments: Treatment[];
  approvalRequests: ApprovalRequest[];
  providerId: string;
  facilityId: string;
  startTime: Date;
  endTime?: Date;
}

export interface Symptom {
  id: string;
  encounterId: string;
  description: string;
  snomedCode?: string;
  onset: Date;
  duration: string;
  severity: number; // 1-10
  characteristics: {
    location?: string;
    quality?: string;
    radiation?: string;
    timing?: string;
    exacerbatingFactors?: string[];
    relievingFactors?: string[];
  };
  redFlag: boolean;
  recordedAt: Date;
  recordedBy: string;
}

export interface VitalSigns {
  bloodPressureSystolic: number;
  bloodPressureDiastolic: number;
  heartRate: number;
  temperature: number;
  respiratoryRate: number;
  oxygenSaturation: number;
  weight?: number;
  height?: number;
  recordedAt: Date;
}

export interface Diagnosis {
  id: string;
  encounterId: string;
  icd10Code: string;
  description: string;
  type: 'differential' | 'confirmed' | 'ruled-out';
  probability?: number;
  confidenceScore?: number;
  evidenceBasis: string[];
  suggestedBy: 'ai' | 'physician';
  suggestedAt: Date;
  confirmedBy?: string;
  confirmedAt?: Date;
  reasoning: string;
  clinicalGuidelines?: string[];
  recommendedTests?: string[];
  status: 'suggested' | 'confirmed' | 'rejected';
}

export interface Treatment {
  id: string;
  encounterId: string;
  diagnosisId: string;
  type: 'medication' | 'procedure' | 'therapy' | 'lifestyle';
  riskScore: number; // 0-100
  requiresApproval: boolean;
  approvalRequestId?: string;
  status: 'suggested' | 'pending_approval' | 'approved' | 'rejected' | 'administered';
  medication?: MedicationDetails;
  contraindications: string[];
  interactions: DrugInteraction[];
  adverseEffects: string[];
  monitoringRequired: string[];
  suggestedBy: 'ai' | 'physician';
  suggestedAt: Date;
  approvedBy?: string;
  approvedAt?: Date;
  administeredBy?: string;
  administeredAt?: Date;
  reasoning: string;
  clinicalGuidelines?: string[];
}

export interface MedicationDetails {
  name: string;
  genericName: string;
  rxNormCode: string;
  dose: string;
  route: string;
  frequency: string;
  duration: string;
  controlledSubstance: boolean;
  schedule?: 'II' | 'III' | 'IV' | 'V';
}

export interface DrugInteraction {
  medication1: string;
  medication2: string;
  severity: 'major' | 'moderate' | 'minor';
  mechanism: string;
  clinicalEffects: string;
  recommendations: string;
}

export interface ApprovalRequest {
  id: string;
  encounterId: string;
  treatmentId: string;
  requestedBy: string;
  requestedAt: Date;
  priority: 'routine' | 'urgent' | 'stat';
  status: 'pending' | 'approved' | 'rejected' | 'cancelled';
  approvalType: 'physician' | 'specialist' | 'pharmacist' | 'multi-level';
  requiredApprovers: string[];
  approvers: ApproverDecision[];
  patientRiskFactors: string[];
  clinicalJustification: string;
  alternativesConsidered: string[];
  urgencyReason?: string;
  oversightReason: OversightReason[];
  policyReference?: string;
  resolvedAt?: Date;
  timeToResolution?: number;
}

export type OversightReason =
  | 'controlled_substance'
  | 'high_risk'
  | 'off_label'
  | 'cost_threshold'
  | 'policy_requirement'
  | 'allergy_override'
  | 'pediatric_patient'
  | 'geriatric_caution';

export interface ApproverDecision {
  userId: string;
  role: string;
  decision: 'approved' | 'rejected';
  justification: string;
  timestamp: Date;
}

export interface AuditEntry {
  id: string;
  timestamp: Date;
  userId: string;
  userRole: string;
  action: 'create' | 'read' | 'update' | 'delete' | 'approve' | 'reject';
  resourceType: 'patient' | 'encounter' | 'diagnosis' | 'treatment' | 'approval';
  resourceId: string;
  changes?: FieldChange[];
  ipAddress: string;
  userAgent: string;
  sessionId: string;
  reasoning?: string;
  aiInvolved: boolean;
  complianceFlags: string[];
}

export interface FieldChange {
  field: string;
  oldValue: any;
  newValue: any;
}

export interface User {
  id: string;
  email: string;
  role: 'physician' | 'nurse' | 'pharmacist' | 'admin' | 'auditor';
  firstName: string;
  lastName: string;
  credentials?: string;
  specialty?: string;
  npi?: string;
  licenseNumber?: string;
  facilityId: string;
  createdAt: Date;
  lastLoginAt?: Date;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: any;
  };
  metadata?: {
    timestamp: Date;
    requestId: string;
  };
}

export interface PaginatedResponse<T> extends ApiResponse<T[]> {
  pagination: {
    page: number;
    pageSize: number;
    totalItems: number;
    totalPages: number;
  };
}

export interface ClinicalRecommendation {
  diagnoses: Diagnosis[];
  treatmentPlans: Treatment[];
  timestamp: Date;
  confidence: number;
}

export interface SymptomAnalysis {
  normalizedSymptoms: Symptom[];
  redFlags: Symptom[];
  priorityLevel: 'stat' | 'urgent' | 'routine';
  correlations: any[];
  medicationEffects: any[];
  patterns: string[];
}
