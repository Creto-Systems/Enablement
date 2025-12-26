# Phase 3: Architecture - Clinical Decision Support System

## 1. System Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Client Layer                                │
│  ┌────────────────┐  ┌──────────────┐  ┌─────────────────────┐    │
│  │ Clinical       │  │  Approval    │  │  Audit Dashboard    │    │
│  │ Dashboard      │  │  Queue       │  │                     │    │
│  └────────────────┘  └──────────────┘  └─────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
                              ↕ HTTPS/TLS 1.3
┌─────────────────────────────────────────────────────────────────────┐
│                      API Gateway Layer                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐     │
│  │ Auth/AuthZ   │  │ Rate Limiter │  │  Request Validator   │     │
│  │ (OAuth 2.0)  │  │              │  │                      │     │
│  └──────────────┘  └──────────────┘  └──────────────────────┘     │
└─────────────────────────────────────────────────────────────────────┘
                              ↕ Internal mTLS
┌─────────────────────────────────────────────────────────────────────┐
│                     Application Services Layer                       │
│  ┌──────────────────┐  ┌─────────────────┐  ┌──────────────────┐  │
│  │ Clinical         │  │  Diagnosis      │  │  Treatment       │  │
│  │ Encounter API    │  │  Engine         │  │  Recommender     │  │
│  └──────────────────┘  └─────────────────┘  └──────────────────┘  │
│                                                                       │
│  ┌──────────────────┐  ┌─────────────────┐  ┌──────────────────┐  │
│  │ Creto-Oversight  │  │  Risk           │  │  Audit           │  │
│  │ Integration      │  │  Assessor       │  │  Service         │  │
│  └──────────────────┘  └─────────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                              ↕
┌─────────────────────────────────────────────────────────────────────┐
│                     Data & Integration Layer                         │
│  ┌──────────────────┐  ┌─────────────────┐  ┌──────────────────┐  │
│  │ PostgreSQL       │  │  Redis Cache    │  │  Event Bus       │  │
│  │ (Patient Data)   │  │                 │  │  (RabbitMQ)      │  │
│  └──────────────────┘  └─────────────────┘  └──────────────────┘  │
│                                                                       │
│  ┌──────────────────┐  ┌─────────────────┐  ┌──────────────────┐  │
│  │ Medical          │  │  Drug           │  │  HL7 FHIR        │  │
│  │ Knowledge Base   │  │  Interaction DB │  │  Gateway         │  │
│  └──────────────────┘  └─────────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                              ↕
┌─────────────────────────────────────────────────────────────────────┐
│                    External Integrations                             │
│  ┌──────────────────┐  ┌─────────────────┐  ┌──────────────────┐  │
│  │ EHR Systems      │  │  Pharmacy       │  │  Lab Systems     │  │
│  │ (Epic, Cerner)   │  │  Systems        │  │                  │  │
│  └──────────────────┘  └─────────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

## 2. Component Architecture

### 2.1 Clinical Decision Engine

```typescript
// Core decision engine architecture
class ClinicalDecisionEngine {
  private diagnosisService: DiagnosisService;
  private treatmentService: TreatmentService;
  private riskAssessor: RiskAssessor;
  private oversightRouter: OversightRouter;

  async analyzeEncounter(encounter: Encounter): Promise<ClinicalRecommendation> {
    // Parallel analysis pipeline
    const [
      symptomAnalysis,
      patientContext,
      clinicalGuidelines
    ] = await Promise.all([
      this.analyzeSymptoms(encounter.symptoms),
      this.loadPatientContext(encounter.patientId),
      this.loadClinicalGuidelines(encounter.chiefComplaint)
    ]);

    // Generate differential diagnoses
    const diagnoses = await this.diagnosisService.generateDifferential({
      symptomAnalysis,
      patientContext,
      clinicalGuidelines
    });

    // For each diagnosis, recommend treatments
    const treatmentPlans = await Promise.all(
      diagnoses.top5.map(diagnosis =>
        this.treatmentService.recommend(diagnosis, patientContext)
      )
    );

    // Risk assessment and oversight routing
    const assessedPlans = await Promise.all(
      treatmentPlans.map(async plan => {
        const riskScore = await this.riskAssessor.assess(plan, patientContext);
        const oversight = this.oversightRouter.determineRequirement(riskScore, plan);
        return { ...plan, riskScore, oversight };
      })
    );

    return {
      diagnoses,
      treatmentPlans: assessedPlans,
      timestamp: new Date(),
      confidence: this.calculateConfidence(diagnoses, assessedPlans)
    };
  }
}
```

### 2.2 Creto-Oversight Integration

```typescript
// Oversight integration service
class OversightIntegrationService {
  private approvalQueue: ApprovalQueueManager;
  private notificationService: NotificationService;
  private workflowEngine: WorkflowEngine;

  async createApprovalRequest(
    treatment: Treatment,
    encounter: Encounter
  ): Promise<ApprovalRequest> {
    // Determine approval chain based on oversight requirements
    const approvalChain = this.buildApprovalChain(treatment);

    const request: ApprovalRequest = {
      id: uuid(),
      treatmentId: treatment.id,
      encounterId: encounter.id,
      requestedBy: getCurrentUser().id,
      requestedAt: new Date(),
      priority: this.determinePriority(treatment, encounter),
      status: 'pending',
      approvalChain,
      clinicalContext: this.extractClinicalContext(encounter, treatment)
    };

    // Persist to database with transaction
    await this.approvalQueue.enqueue(request);

    // Trigger notifications
    await this.notifyApprovers(request);

    // Create audit trail
    await this.auditService.log({
      action: 'approval_requested',
      resourceType: 'approval',
      resourceId: request.id,
      context: request
    });

    return request;
  }

  async processApproval(
    requestId: string,
    decision: 'approved' | 'rejected',
    justification: string
  ): Promise<ApprovalRequest> {
    const request = await this.approvalQueue.get(requestId);

    // Validate user authorization
    await this.validateApprover(getCurrentUser(), request);

    // Record decision
    request.approvers.push({
      userId: getCurrentUser().id,
      decision,
      justification,
      timestamp: new Date()
    });

    // Check if all required approvals obtained
    if (this.isFullyApproved(request)) {
      request.status = 'approved';
      await this.activateTreatment(request.treatmentId);
      await this.notifyRequester(request, 'approved');
    } else if (decision === 'rejected') {
      request.status = 'rejected';
      await this.notifyRequester(request, 'rejected');
    }

    await this.approvalQueue.update(request);
    return request;
  }

  private buildApprovalChain(treatment: Treatment): ApprovalChain {
    const chain: ApprovalChain = [];

    // Rule-based approval routing
    if (treatment.oversightReason.includes('controlled_substance')) {
      chain.push({ role: 'attending_physician', required: true });
      chain.push({ role: 'pharmacist', required: true });
    }

    if (treatment.oversightReason.includes('high_risk')) {
      chain.push({ role: 'specialist', specialty: treatment.specialty, required: true });
    }

    if (treatment.oversightReason.includes('cost_threshold')) {
      chain.push({ role: 'utilization_reviewer', required: true });
    }

    return chain;
  }
}
```

### 2.3 Audit & Compliance Service

```typescript
// HIPAA-compliant audit logging
class AuditService {
  private auditLog: AuditLogRepository;
  private encryptionService: EncryptionService;

  async log(entry: AuditEntryInput): Promise<void> {
    // Encrypt PHI before logging
    const encryptedData = await this.encryptionService.encryptPHI({
      resourceId: entry.resourceId,
      changes: entry.changes
    });

    const auditEntry: AuditEntry = {
      id: uuid(),
      timestamp: new Date(),
      userId: entry.userId || getCurrentUser().id,
      userRole: getCurrentUser().role,
      action: entry.action,
      resourceType: entry.resourceType,
      encryptedData,
      ipAddress: getClientIP(),
      userAgent: getUserAgent(),
      sessionId: getSessionId(),
      reasoning: entry.reasoning,
      aiInvolved: entry.aiInvolved || false,
      complianceFlags: this.determineComplianceFlags(entry)
    };

    // Write to immutable append-only log
    await this.auditLog.append(auditEntry);

    // Real-time compliance monitoring
    await this.checkComplianceRules(auditEntry);
  }

  async generateComplianceReport(
    startDate: Date,
    endDate: Date,
    filters?: AuditFilters
  ): Promise<ComplianceReport> {
    const entries = await this.auditLog.query({
      startDate,
      endDate,
      ...filters
    });

    return {
      period: { startDate, endDate },
      totalEntries: entries.length,
      breakdown: {
        byAction: this.groupBy(entries, 'action'),
        byUser: this.groupBy(entries, 'userId'),
        byResourceType: this.groupBy(entries, 'resourceType')
      },
      aiDecisions: entries.filter(e => e.aiInvolved).length,
      oversightCompliance: this.assessOversightCompliance(entries),
      hipaaCompliance: this.assessHIPAACompliance(entries),
      generatedAt: new Date(),
      integrityHash: this.generateHash(entries)
    };
  }

  private determineComplianceFlags(entry: AuditEntryInput): string[] {
    const flags: string[] = [];

    if (entry.resourceType === 'patient') {
      flags.push('HIPAA');
    }

    if (entry.action === 'approve' || entry.action === 'reject') {
      flags.push('FDA_21CFR11');
    }

    if (entry.aiInvolved) {
      flags.push('AI_DECISION_AUDIT');
    }

    return flags;
  }
}
```

## 3. Data Architecture

### 3.1 Database Schema

```sql
-- Patient table with encryption
CREATE TABLE patients (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  mrn VARCHAR(50) UNIQUE NOT NULL,
  encrypted_demographics JSONB NOT NULL,
  encrypted_ssn BYTEA,
  insurance_info JSONB,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW(),
  created_by UUID REFERENCES users(id)
);

-- Encounters
CREATE TABLE encounters (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  patient_id UUID REFERENCES patients(id),
  encounter_type VARCHAR(20) CHECK (encounter_type IN ('outpatient', 'emergency', 'inpatient', 'telehealth')),
  status VARCHAR(20) DEFAULT 'active',
  chief_complaint TEXT,
  provider_id UUID REFERENCES users(id),
  facility_id UUID REFERENCES facilities(id),
  start_time TIMESTAMP DEFAULT NOW(),
  end_time TIMESTAMP,
  created_at TIMESTAMP DEFAULT NOW()
);

-- Symptoms
CREATE TABLE symptoms (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  encounter_id UUID REFERENCES encounters(id),
  description TEXT NOT NULL,
  snomed_code VARCHAR(50),
  onset TIMESTAMP,
  duration VARCHAR(50),
  severity INTEGER CHECK (severity BETWEEN 1 AND 10),
  characteristics JSONB,
  red_flag BOOLEAN DEFAULT FALSE,
  recorded_at TIMESTAMP DEFAULT NOW(),
  recorded_by UUID REFERENCES users(id)
);

-- Diagnoses
CREATE TABLE diagnoses (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  encounter_id UUID REFERENCES encounters(id),
  icd10_code VARCHAR(10) NOT NULL,
  description TEXT NOT NULL,
  diagnosis_type VARCHAR(20) CHECK (diagnosis_type IN ('differential', 'confirmed', 'ruled-out')),
  probability DECIMAL(5,2),
  confidence_score DECIMAL(5,2),
  evidence_basis JSONB,
  suggested_by VARCHAR(20) CHECK (suggested_by IN ('ai', 'physician')),
  suggested_at TIMESTAMP DEFAULT NOW(),
  confirmed_by UUID REFERENCES users(id),
  confirmed_at TIMESTAMP,
  reasoning TEXT,
  clinical_guidelines JSONB,
  status VARCHAR(20) DEFAULT 'suggested'
);

-- Treatments
CREATE TABLE treatments (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  encounter_id UUID REFERENCES encounters(id),
  diagnosis_id UUID REFERENCES diagnoses(id),
  treatment_type VARCHAR(20) CHECK (treatment_type IN ('medication', 'procedure', 'therapy', 'lifestyle')),
  risk_score INTEGER CHECK (risk_score BETWEEN 0 AND 100),
  requires_approval BOOLEAN DEFAULT FALSE,
  approval_request_id UUID REFERENCES approval_requests(id),
  status VARCHAR(30) DEFAULT 'suggested',
  medication_details JSONB,
  contraindications JSONB,
  interactions JSONB,
  monitoring_plan JSONB,
  suggested_by VARCHAR(20),
  suggested_at TIMESTAMP DEFAULT NOW(),
  approved_by UUID REFERENCES users(id),
  approved_at TIMESTAMP,
  reasoning TEXT
);

-- Approval Requests (Creto-Oversight)
CREATE TABLE approval_requests (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  encounter_id UUID REFERENCES encounters(id),
  treatment_id UUID REFERENCES treatments(id),
  requested_by UUID REFERENCES users(id),
  requested_at TIMESTAMP DEFAULT NOW(),
  priority VARCHAR(20) CHECK (priority IN ('routine', 'urgent', 'stat')),
  status VARCHAR(30) DEFAULT 'pending',
  approval_type VARCHAR(30),
  required_approvers JSONB NOT NULL,
  approvers JSONB DEFAULT '[]',
  oversight_reason JSONB NOT NULL,
  clinical_justification TEXT,
  resolved_at TIMESTAMP,
  time_to_resolution INTEGER
);

-- Audit Log (Immutable)
CREATE TABLE audit_log (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  timestamp TIMESTAMP DEFAULT NOW() NOT NULL,
  user_id UUID REFERENCES users(id),
  user_role VARCHAR(50),
  action VARCHAR(50) NOT NULL,
  resource_type VARCHAR(50) NOT NULL,
  encrypted_data BYTEA,
  ip_address INET,
  user_agent TEXT,
  session_id UUID,
  reasoning TEXT,
  ai_involved BOOLEAN DEFAULT FALSE,
  compliance_flags JSONB
);

-- Prevent updates/deletes on audit log
CREATE RULE audit_log_immutable AS ON UPDATE TO audit_log DO INSTEAD NOTHING;
CREATE RULE audit_log_no_delete AS ON DELETE TO audit_log DO INSTEAD NOTHING;

-- Indexes for performance
CREATE INDEX idx_encounters_patient ON encounters(patient_id);
CREATE INDEX idx_encounters_provider ON encounters(provider_id);
CREATE INDEX idx_symptoms_encounter ON symptoms(encounter_id);
CREATE INDEX idx_diagnoses_encounter ON diagnoses(encounter_id);
CREATE INDEX idx_treatments_encounter ON treatments(encounter_id);
CREATE INDEX idx_approvals_status ON approval_requests(status) WHERE status = 'pending';
CREATE INDEX idx_audit_timestamp ON audit_log(timestamp DESC);
CREATE INDEX idx_audit_user ON audit_log(user_id);
CREATE INDEX idx_audit_resource ON audit_log(resource_type, encrypted_data);
```

### 3.2 Caching Strategy

```typescript
// Redis caching for clinical data
class ClinicalCacheService {
  private redis: RedisClient;

  // Cache patient context (15 min TTL)
  async cachePatientContext(patientId: string, context: PatientContext): Promise<void> {
    await this.redis.setex(
      `patient:${patientId}:context`,
      900, // 15 minutes
      JSON.stringify(context)
    );
  }

  // Cache drug interaction results (1 hour TTL)
  async cacheDrugInteractions(medicationIds: string[], interactions: Interaction[]): Promise<void> {
    const key = `interactions:${medicationIds.sort().join(':')}`;
    await this.redis.setex(key, 3600, JSON.stringify(interactions));
  }

  // Cache diagnosis suggestions (5 min TTL for rapid iterations)
  async cacheDiagnosisSuggestions(symptomHash: string, diagnoses: Diagnosis[]): Promise<void> {
    await this.redis.setex(
      `diagnosis:${symptomHash}`,
      300,
      JSON.stringify(diagnoses)
    );
  }

  // Invalidation on data updates
  async invalidatePatientCache(patientId: string): Promise<void> {
    await this.redis.del(`patient:${patientId}:context`);
  }
}
```

## 4. Security Architecture

### 4.1 Authentication & Authorization

```typescript
// Role-Based Access Control (RBAC)
enum Role {
  PHYSICIAN = 'physician',
  NURSE = 'nurse',
  PHARMACIST = 'pharmacist',
  ADMIN = 'admin',
  AUDITOR = 'auditor'
}

interface Permission {
  resource: string;
  actions: ('create' | 'read' | 'update' | 'delete' | 'approve')[];
}

const rolePermissions: Record<Role, Permission[]> = {
  [Role.PHYSICIAN]: [
    { resource: 'patient', actions: ['create', 'read', 'update'] },
    { resource: 'encounter', actions: ['create', 'read', 'update'] },
    { resource: 'diagnosis', actions: ['create', 'read', 'update'] },
    { resource: 'treatment', actions: ['create', 'read', 'update', 'approve'] },
    { resource: 'approval', actions: ['create', 'read', 'approve'] }
  ],
  [Role.NURSE]: [
    { resource: 'patient', actions: ['read'] },
    { resource: 'encounter', actions: ['create', 'read', 'update'] },
    { resource: 'symptom', actions: ['create', 'read'] },
    { resource: 'vitals', actions: ['create', 'read'] }
  ],
  [Role.PHARMACIST]: [
    { resource: 'treatment', actions: ['read', 'approve'] },
    { resource: 'approval', actions: ['read', 'approve'] }
  ],
  [Role.AUDITOR]: [
    { resource: 'audit', actions: ['read'] }
  ]
};

class AuthorizationService {
  hasPermission(user: User, resource: string, action: string): boolean {
    const permissions = rolePermissions[user.role];
    return permissions.some(p =>
      p.resource === resource && p.actions.includes(action)
    );
  }

  async enforceAuthorization(user: User, resource: string, action: string): Promise<void> {
    if (!this.hasPermission(user, resource, action)) {
      await this.auditService.log({
        action: 'unauthorized_access_attempt',
        resourceType: resource,
        userId: user.id,
        reasoning: `Attempted ${action} on ${resource} without permission`
      });
      throw new ForbiddenError(`Insufficient permissions for ${action} on ${resource}`);
    }
  }
}
```

### 4.2 Encryption

```typescript
// PHI Encryption Service
class EncryptionService {
  private algorithm = 'aes-256-gcm';
  private keyDerivation = 'pbkdf2';

  async encryptPHI(data: any): Promise<EncryptedData> {
    const key = await this.deriveKey();
    const iv = crypto.randomBytes(16);
    const cipher = crypto.createCipheriv(this.algorithm, key, iv);

    const encrypted = Buffer.concat([
      cipher.update(JSON.stringify(data), 'utf8'),
      cipher.final()
    ]);

    const authTag = cipher.getAuthTag();

    return {
      encryptedData: encrypted.toString('base64'),
      iv: iv.toString('base64'),
      authTag: authTag.toString('base64'),
      algorithm: this.algorithm
    };
  }

  async decryptPHI(encrypted: EncryptedData): Promise<any> {
    const key = await this.deriveKey();
    const decipher = crypto.createDecipheriv(
      this.algorithm,
      key,
      Buffer.from(encrypted.iv, 'base64')
    );

    decipher.setAuthTag(Buffer.from(encrypted.authTag, 'base64'));

    const decrypted = Buffer.concat([
      decipher.update(Buffer.from(encrypted.encryptedData, 'base64')),
      decipher.final()
    ]);

    return JSON.parse(decrypted.toString('utf8'));
  }

  private async deriveKey(): Promise<Buffer> {
    const masterKey = process.env.ENCRYPTION_MASTER_KEY;
    const salt = Buffer.from(process.env.ENCRYPTION_SALT, 'base64');

    return crypto.pbkdf2Sync(masterKey, salt, 100000, 32, 'sha512');
  }
}
```

## 5. Integration Architecture

### 5.1 HL7 FHIR Integration

```typescript
// FHIR R4 integration for EHR interoperability
class FHIRIntegrationService {
  async exportPatientToFHIR(patient: Patient): Promise<fhir.Patient> {
    return {
      resourceType: 'Patient',
      id: patient.id,
      identifier: [{
        system: 'urn:oid:2.16.840.1.113883.4.1',
        value: patient.mrn
      }],
      name: [{
        family: patient.demographics.lastName,
        given: [patient.demographics.firstName]
      }],
      gender: patient.demographics.gender as fhir.Gender,
      birthDate: patient.demographics.dateOfBirth.toISOString().split('T')[0]
    };
  }

  async exportEncounterToFHIR(encounter: Encounter): Promise<fhir.Encounter> {
    return {
      resourceType: 'Encounter',
      id: encounter.id,
      status: encounter.status as fhir.EncounterStatus,
      class: this.mapEncounterType(encounter.type),
      subject: {
        reference: `Patient/${encounter.patientId}`
      },
      period: {
        start: encounter.startTime.toISOString(),
        end: encounter.endTime?.toISOString()
      }
    };
  }

  async importFHIRBundle(bundle: fhir.Bundle): Promise<void> {
    for (const entry of bundle.entry || []) {
      const resource = entry.resource;

      switch (resource.resourceType) {
        case 'Patient':
          await this.importPatient(resource as fhir.Patient);
          break;
        case 'Observation':
          await this.importObservation(resource as fhir.Observation);
          break;
        case 'MedicationRequest':
          await this.importMedication(resource as fhir.MedicationRequest);
          break;
      }
    }
  }
}
```

## 6. Deployment Architecture

```yaml
# Kubernetes deployment for healthcare demo
apiVersion: apps/v1
kind: Deployment
metadata:
  name: clinical-decision-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: clinical-decision-api
  template:
    metadata:
      labels:
        app: clinical-decision-api
    spec:
      containers:
      - name: api
        image: healthcare-demo/api:latest
        ports:
        - containerPort: 3000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: url
        - name: ENCRYPTION_MASTER_KEY
          valueFrom:
            secretKeyRef:
              name: encryption-keys
              key: master
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 3000
          initialDelaySeconds: 10
          periodSeconds: 5
```

## 7. Performance & Scalability

- **Target Metrics**:
  - API Response Time: p95 < 500ms
  - Diagnosis Generation: < 3 seconds
  - Concurrent Users: 500+
  - Database Connections: Pool of 20-50

- **Horizontal Scaling**: Stateless API servers behind load balancer
- **Database**: Read replicas for query distribution
- **Caching**: Redis cluster for distributed caching
- **CDN**: Static assets served via CDN

## 8. Disaster Recovery

- **Backup Strategy**: Automated daily backups, 7-year retention for audit logs
- **RTO**: 4 hours
- **RPO**: 4 hours (last backup)
- **Failover**: Multi-AZ deployment with automatic failover
- **Testing**: Quarterly disaster recovery drills

---

**Next Phase**: Refinement through Test-Driven Development implementation.
