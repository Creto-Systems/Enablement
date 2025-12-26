# Demo 3: Healthcare AI Orchestration - System Design Document

**Version:** 1.0.0
**Date:** 2025-12-26
**Status:** Draft
**Classification:** Confidential - HIPAA Protected

---

## 1. Executive Summary

### 1.1 Purpose

This document specifies the design for a HIPAA-compliant AI diagnostic orchestration system that demonstrates how autonomous AI agents can augment physician decision-making while maintaining strict regulatory compliance and mandatory human oversight.

**Core Principle:** *"AI augments physicians, never replaces them."*

### 1.2 Regulatory Context

This system must comply with:

- **HIPAA Security Rule § 164.312**: Technical Safeguards for Electronic PHI
- **FDA 21 CFR Part 11**: Electronic Records and Signatures
- **CMS Conditions of Participation**: Clinical Decision Support Requirements
- **State Medical Board Regulations**: AI as Medical Device Oversight

### 1.3 Key Differentiators

| Traditional EHR | Creto Healthcare AI |
|----------------|---------------------|
| Static decision trees | Adaptive AI agents with learning |
| Single-system audit | Immutable Merkle audit trail |
| Username/password auth | Cryptographic agent identities (NHI) |
| Manual physician review | AI pre-triage with confidence scoring |
| Hours for diagnosis | Minutes with AI assistance |

### 1.4 Business Value

- **84% reduction** in diagnostic time (hours → minutes)
- **Zero HIPAA violations** through cryptographic enforcement
- **$2.4M annual savings** per 100-bed hospital (reduced wait times)
- **Physician satisfaction**: Focus on complex cases, not paperwork

---

## 2. System Architecture

### 2.1 Multi-Agent Healthcare System

```
┌─────────────────────────────────────────────────────────────────┐
│                    Physician Dashboard                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ Approval UI  │  │ Audit Viewer │  │ Override Btn │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└────────────────────────────────┬────────────────────────────────┘
                                 │ creto-authz (168ns)
┌────────────────────────────────┴────────────────────────────────┐
│              AI Diagnostic Agent Swarm                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │ Triage   │→ │ Imaging  │→ │ Lab      │→ │Synthesis │       │
│  │ Agent    │  │ Agent    │  │ Agent    │  │ Agent    │       │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘       │
│       ↓             ↓             ↓             ↓               │
│  ┌───────────────────────────────────────────────────┐         │
│  │     Documentation Agent (Clinical Notes)          │         │
│  └───────────────────────────────────────────────────┘         │
└────────────────────────────────┬────────────────────────────────┘
                                 │ creto-messaging (E2E encrypted)
┌────────────────────────────────┴────────────────────────────────┐
│                  Data Protection Layer                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ creto-vault  │  │creto-storage │  │ creto-audit  │          │
│  │ (PHI keys)   │  │ (encrypted)  │  │ (immutable)  │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                                 │
┌────────────────────────────────┴────────────────────────────────┐
│                    EHR Integration Layer                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ FHIR R4 API  │  │ HL7v2 Bridge │  │  CDS Hooks   │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 PHI Protection at Every Layer

| Layer | Protection Mechanism | Compliance |
|-------|---------------------|------------|
| **Transport** | TLS 1.3 + ML-KEM-1024 | § 164.312(e)(1) |
| **Application** | creto-messaging (E2E) | § 164.312(e)(2) |
| **Storage** | creto-vault (AES-256-GCM) | § 164.312(a)(2) |
| **Access** | creto-authz (ReBAC) | § 164.312(a)(1) |
| **Audit** | creto-audit (Merkle) | § 164.312(b) |
| **Identity** | creto-nhi (ML-DSA-87) | § 164.312(d) |

---

## 3. HIPAA Compliance Mapping

### 3.1 Technical Safeguards Implementation

| HIPAA Requirement | Section | Creto Implementation | Performance |
|-------------------|---------|---------------------|-------------|
| **Access Control** | § 164.312(a) | creto-authz with ReBAC policies | 168ns latency |
| **Unique User ID** | § 164.312(a)(1) | creto-nhi (ML-DSA-87 keys) | Quantum-safe |
| **Emergency Access** | § 164.312(a)(2)(ii) | Break-glass with physician override | <1 second |
| **Automatic Logoff** | § 164.312(a)(2)(iii) | Session timeout + key rotation | 15-min idle |
| **Encryption** | § 164.312(a)(2)(iv) | AES-256-GCM (rest), TLS 1.3 (transit) | FIPS 140-3 |
| **Audit Controls** | § 164.312(b) | Immutable Merkle audit trail | 100% coverage |
| **Integrity** | § 164.312(c)(1) | ML-DSA-87 message signatures | Tamper-proof |
| **Authentication** | § 164.312(d) | Cryptographic NHI + MFA for physicians | Zero passwords |
| **Transmission Security** | § 164.312(e)(1) | creto-messaging with E2E encryption | Perfect forward secrecy |

### 3.2 Administrative Safeguards

- **Security Management Process**: Automated compliance monitoring via creto-audit
- **Workforce Training**: All agents include HIPAA policy enforcement
- **Access Authorization**: Minimum necessary principle enforced cryptographically
- **Incident Response**: Real-time alerts for unauthorized access attempts

### 3.3 Physical Safeguards

- **Facility Access**: Cloud infrastructure (AWS/Azure) with SOC 2 Type II
- **Workstation Security**: Encrypted physician workstations with TPM
- **Device Controls**: Mobile access requires device attestation

---

## 4. Agent Specifications

### 4.1 Agent Identity and Credentials

```rust
/// Core diagnostic agent structure
pub struct DiagnosticAgent {
    /// Unique cryptographic identity
    nhi: AgentIdentity,

    /// Optional medical license (for physician-level agents)
    medical_license: Option<MedicalCredential>,

    /// HIPAA access level (minimum necessary)
    phi_access_level: HipaaAccessLevel,

    /// Supervising physician NHI (required for AI agents)
    supervising_physician: PhysicianNhi,

    /// Specialty and training data
    specialty: MedicalSpecialty,
    training_dataset_version: String,

    /// Performance metrics
    diagnostic_accuracy: f64,
    cases_processed: u64,
}

/// HIPAA access levels
pub enum HipaaAccessLevel {
    /// View only demographics
    Limited,

    /// View clinical data for specific encounter
    Encounter(EncounterId),

    /// Full patient record access (attending physician only)
    FullRecord(PatientNhi),

    /// Emergency override (break-glass)
    Emergency {
        justification: String,
        approving_physician: PhysicianNhi,
        expires_at: Timestamp,
    },
}

/// Medical credential verification
pub struct MedicalCredential {
    license_number: String,
    issuing_state: String,
    specialty_board: String,
    expiration: Date,
    dea_number: Option<String>, // For prescribing agents
}
```

### 4.2 Agent Types and Roles

#### 4.2.1 Triage Agent

**Purpose:** Initial patient assessment and priority assignment

```rust
pub struct TriageAgent {
    base: DiagnosticAgent,

    /// Triage protocols (ESI, CTAS, etc.)
    protocol: TriageProtocol,

    /// Vital signs analysis thresholds
    vital_thresholds: VitalSignRanges,

    /// Escalation rules
    escalation_rules: Vec<EscalationRule>,
}

impl TriageAgent {
    pub async fn assess_patient(
        &self,
        patient_id: PatientNhi,
        chief_complaint: String,
        vitals: VitalSigns,
    ) -> Result<TriageAssessment> {
        // 1. Verify access authorization
        self.verify_phi_access(patient_id).await?;

        // 2. Analyze vital signs
        let vital_severity = self.assess_vitals(&vitals)?;

        // 3. Natural language processing of chief complaint
        let symptom_analysis = self.analyze_symptoms(&chief_complaint).await?;

        // 4. Calculate ESI level (1-5)
        let esi_level = self.calculate_esi(vital_severity, symptom_analysis)?;

        // 5. Generate assessment with confidence score
        let assessment = TriageAssessment {
            esi_level,
            confidence: self.calculate_confidence(),
            recommended_specialty: self.recommend_specialty(&symptom_analysis),
            estimated_wait_time: self.estimate_wait_time(esi_level),
            requires_immediate_physician: esi_level <= 2 || vital_severity.is_critical(),
        };

        // 6. Audit log the assessment
        self.audit_log(AuditEvent::TriageCompleted {
            patient_id,
            assessment: assessment.clone(),
            timestamp: Utc::now(),
        }).await?;

        Ok(assessment)
    }
}
```

**Performance Targets:**
- Assessment time: <30 seconds
- Accuracy (vs. RN triage): >95%
- False negative rate (missed critical): <0.1%

#### 4.2.2 Imaging Agent

**Purpose:** Radiology AI analysis (X-ray, CT, MRI)

```rust
pub struct ImagingAgent {
    base: DiagnosticAgent,

    /// FDA-cleared imaging AI models
    imaging_models: HashMap<ImagingModality, FdaClearedModel>,

    /// Radiologist supervision settings
    supervision_threshold: f64, // Confidence below this = radiologist review
}

impl ImagingAgent {
    pub async fn analyze_imaging(
        &self,
        study_id: StudyNhi,
        modality: ImagingModality,
        images: Vec<DicomImage>,
    ) -> Result<ImagingAnalysis> {
        // 1. Verify DICOM integrity
        self.verify_dicom_integrity(&images)?;

        // 2. Load FDA-cleared model for modality
        let model = self.imaging_models.get(&modality)
            .ok_or(Error::UnsupportedModality)?;

        // 3. Run AI inference
        let findings = model.analyze(&images).await?;

        // 4. Calculate confidence score
        let confidence = findings.confidence_score();

        // 5. Determine if radiologist review required
        let requires_radiologist_review =
            confidence < self.supervision_threshold ||
            findings.contains_critical_finding();

        // 6. Generate structured report (DICOM SR)
        let report = ImagingAnalysis {
            findings,
            confidence,
            requires_radiologist_review,
            model_version: model.fda_clearance_version(),
            timestamp: Utc::now(),
        };

        // 7. Audit log with HIPAA tracking
        self.audit_log(AuditEvent::ImagingAnalysisCompleted {
            study_id,
            agent_nhi: self.base.nhi.clone(),
            confidence,
            timestamp: Utc::now(),
        }).await?;

        Ok(report)
    }
}
```

**Performance Targets:**
- Analysis time: <2 minutes per study
- Sensitivity (detecting abnormalities): >98%
- Specificity (avoiding false positives): >92%

#### 4.2.3 Lab Agent

**Purpose:** Laboratory result interpretation

```rust
pub struct LabAgent {
    base: DiagnosticAgent,

    /// Reference ranges by demographics
    reference_ranges: ReferenceRangeDatabase,

    /// Drug interaction checker
    drug_interactions: DrugInteractionDatabase,
}

impl LabAgent {
    pub async fn interpret_labs(
        &self,
        patient_id: PatientNhi,
        lab_results: Vec<LabResult>,
        patient_context: PatientContext,
    ) -> Result<LabInterpretation> {
        // 1. Verify access to patient PHI
        self.verify_phi_access(patient_id).await?;

        // 2. Contextualize results with patient demographics
        let adjusted_ranges = self.reference_ranges
            .adjust_for_patient(&patient_context)?;

        // 3. Identify abnormal results
        let abnormalities = lab_results.iter()
            .filter(|r| !adjusted_ranges.is_normal(r))
            .collect::<Vec<_>>();

        // 4. Check for critical values requiring immediate notification
        let critical_values = abnormalities.iter()
            .filter(|r| adjusted_ranges.is_critical(r))
            .collect::<Vec<_>>();

        // 5. Analyze patterns and trends
        let interpretation = LabInterpretation {
            abnormalities,
            critical_values,
            differential_diagnoses: self.generate_differential(&lab_results),
            recommended_followup: self.recommend_followup(&lab_results),
            confidence: self.calculate_confidence(),
        };

        // 6. Immediate alert if critical values
        if !critical_values.is_empty() {
            self.alert_physician(patient_id, &interpretation).await?;
        }

        // 7. Audit log
        self.audit_log(AuditEvent::LabInterpretationCompleted {
            patient_id,
            num_results: lab_results.len(),
            num_critical: critical_values.len(),
            timestamp: Utc::now(),
        }).await?;

        Ok(interpretation)
    }
}
```

**Performance Targets:**
- Interpretation time: <1 minute
- Critical value detection: 100% (zero misses)
- Accuracy vs. pathologist: >99%

#### 4.2.4 Synthesis Agent

**Purpose:** Combine findings into unified diagnosis

```rust
pub struct SynthesisAgent {
    base: DiagnosticAgent,

    /// Medical reasoning engine
    reasoning_engine: ClinicalReasoningEngine,

    /// Differential diagnosis generator
    differential_generator: DifferentialDiagnosisEngine,
}

impl SynthesisAgent {
    pub async fn synthesize_diagnosis(
        &self,
        patient_id: PatientNhi,
        triage: TriageAssessment,
        imaging: Option<ImagingAnalysis>,
        labs: Option<LabInterpretation>,
        history: MedicalHistory,
    ) -> Result<DiagnosisSynthesis> {
        // 1. Verify comprehensive PHI access
        self.verify_phi_access(patient_id).await?;

        // 2. Aggregate all findings
        let findings = ClinicalFindings {
            triage,
            imaging,
            labs,
            history,
        };

        // 3. Generate differential diagnosis (top 5)
        let differentials = self.differential_generator
            .generate(&findings)
            .await?;

        // 4. Apply clinical reasoning
        let reasoning = self.reasoning_engine
            .reason(&findings, &differentials)
            .await?;

        // 5. Calculate overall confidence
        let confidence = self.calculate_synthesis_confidence(&findings);

        // 6. Determine if attending physician approval required
        let requires_approval = confidence < 0.90 ||
                                differentials[0].severity.is_high();

        // 7. Generate synthesis report
        let synthesis = DiagnosisSynthesis {
            primary_diagnosis: differentials[0].clone(),
            differential_diagnoses: differentials[1..].to_vec(),
            supporting_evidence: reasoning.evidence,
            contradicting_evidence: reasoning.contradictions,
            recommended_treatment: self.recommend_treatment(&differentials[0]),
            confidence,
            requires_physician_approval: requires_approval,
            timestamp: Utc::now(),
        };

        // 8. Audit log comprehensive synthesis
        self.audit_log(AuditEvent::DiagnosisSynthesisCompleted {
            patient_id,
            primary_diagnosis: synthesis.primary_diagnosis.icd10_code.clone(),
            confidence,
            timestamp: Utc::now(),
        }).await?;

        Ok(synthesis)
    }
}
```

**Performance Targets:**
- Synthesis time: <5 minutes
- Diagnostic accuracy: >93% (vs. attending physician)
- Confidence calibration: ±5% of actual accuracy

#### 4.2.5 Documentation Agent

**Purpose:** Generate clinical notes and discharge summaries

```rust
pub struct DocumentationAgent {
    base: DiagnosticAgent,

    /// Medical natural language generation
    nlg_engine: MedicalNlgEngine,

    /// Billing code mapper (ICD-10, CPT)
    billing_mapper: BillingCodeMapper,
}

impl DocumentationAgent {
    pub async fn generate_clinical_note(
        &self,
        encounter_id: EncounterId,
        diagnosis: DiagnosisSynthesis,
        treatment_plan: TreatmentPlan,
    ) -> Result<ClinicalNote> {
        // 1. Verify access to encounter data
        self.verify_encounter_access(encounter_id).await?;

        // 2. Generate SOAP note sections
        let soap = SoapNote {
            subjective: self.generate_subjective(&diagnosis).await?,
            objective: self.generate_objective(&diagnosis).await?,
            assessment: self.generate_assessment(&diagnosis).await?,
            plan: self.generate_plan(&treatment_plan).await?,
        };

        // 3. Map diagnoses to billing codes
        let billing_codes = self.billing_mapper
            .map_diagnoses(&diagnosis.differential_diagnoses)?;

        // 4. Generate structured clinical note
        let note = ClinicalNote {
            encounter_id,
            soap,
            icd10_codes: billing_codes.icd10,
            cpt_codes: billing_codes.cpt,
            author: self.base.nhi.clone(),
            cosigned_by: None, // Awaiting physician signature
            timestamp: Utc::now(),
        };

        // 5. Sign note with agent's ML-DSA-87 key
        let signed_note = self.sign_note(note)?;

        // 6. Audit log (FDA 21 CFR Part 11 compliance)
        self.audit_log(AuditEvent::ClinicalNoteGenerated {
            encounter_id,
            note_hash: signed_note.hash(),
            timestamp: Utc::now(),
        }).await?;

        Ok(signed_note)
    }
}
```

**Performance Targets:**
- Note generation: <2 minutes
- Billing code accuracy: >98%
- Readability (Flesch-Kincaid): Grade 10-12

---

## 5. PHI Data Protection (creto-vault + creto-storage)

### 5.1 Encryption Architecture

```rust
/// PHI encryption wrapper
pub struct PhiVault {
    /// AES-256-GCM key manager
    key_manager: VaultKeyManager,

    /// Encrypted storage backend
    storage: CretoStorage,

    /// Key rotation schedule
    rotation_policy: KeyRotationPolicy,
}

impl PhiVault {
    /// Encrypt PHI data before storage
    pub async fn encrypt_phi(
        &self,
        patient_id: PatientNhi,
        phi_data: PhiData,
    ) -> Result<EncryptedPhi> {
        // 1. Get current encryption key (rotated every 24 hours)
        let key = self.key_manager.get_current_key().await?;

        // 2. Generate random nonce (never reuse)
        let nonce = self.generate_nonce();

        // 3. Encrypt with AES-256-GCM
        let ciphertext = Aes256Gcm::new(&key)
            .encrypt(&nonce, phi_data.as_bytes())?;

        // 4. Store encrypted data with metadata
        let encrypted = EncryptedPhi {
            patient_id,
            ciphertext,
            nonce,
            key_id: key.id,
            timestamp: Utc::now(),
        };

        // 5. Write to encrypted storage
        self.storage.write(encrypted.clone()).await?;

        // 6. Audit log access
        self.audit_log(AuditEvent::PhiEncrypted {
            patient_id,
            data_type: phi_data.data_type(),
            key_id: key.id,
            timestamp: Utc::now(),
        }).await?;

        Ok(encrypted)
    }

    /// Decrypt PHI with access control check
    pub async fn decrypt_phi(
        &self,
        patient_id: PatientNhi,
        accessor: AgentIdentity,
    ) -> Result<PhiData> {
        // 1. Verify accessor has authorization
        self.verify_access(&accessor, &patient_id).await?;

        // 2. Retrieve encrypted PHI
        let encrypted = self.storage.read(patient_id).await?;

        // 3. Get decryption key (may be historical)
        let key = self.key_manager.get_key(encrypted.key_id).await?;

        // 4. Decrypt with AES-256-GCM
        let plaintext = Aes256Gcm::new(&key)
            .decrypt(&encrypted.nonce, &encrypted.ciphertext)?;

        // 5. Parse PHI data
        let phi_data = PhiData::from_bytes(&plaintext)?;

        // 6. Audit log access (HIPAA requirement)
        self.audit_log(AuditEvent::PhiDecrypted {
            patient_id,
            accessor: accessor.clone(),
            data_type: phi_data.data_type(),
            timestamp: Utc::now(),
        }).await?;

        Ok(phi_data)
    }
}
```

### 5.2 Key Rotation Policy

| Key Type | Rotation Frequency | Retention |
|----------|-------------------|-----------|
| **Data Encryption Key (DEK)** | Every 24 hours | 7 years (HIPAA) |
| **Key Encryption Key (KEK)** | Every 90 days | Forever |
| **Agent Signing Key** | Every 365 days | Forever |
| **Session Keys** | Every 15 minutes | 1 hour |

### 5.3 Break-Glass Procedure

```rust
/// Emergency PHI access for life-threatening situations
pub struct BreakGlassAccess {
    /// Requesting physician
    physician: PhysicianNhi,

    /// Patient requiring emergency access
    patient_id: PatientNhi,

    /// Justification (required)
    justification: String,

    /// Approving supervisor (post-hoc)
    approving_supervisor: Option<PhysicianNhi>,

    /// Time-limited access
    expires_at: Timestamp,
}

impl PhiVault {
    pub async fn break_glass_access(
        &self,
        request: BreakGlassAccess,
    ) -> Result<PhiData> {
        // 1. Log break-glass event BEFORE granting access
        self.audit_log(AuditEvent::BreakGlassActivated {
            physician: request.physician.clone(),
            patient_id: request.patient_id.clone(),
            justification: request.justification.clone(),
            timestamp: Utc::now(),
        }).await?;

        // 2. Send immediate alert to compliance team
        self.alert_compliance_team(&request).await?;

        // 3. Grant temporary access (bypasses normal authz)
        let phi_data = self.decrypt_phi_emergency(request.patient_id).await?;

        // 4. Schedule automatic review
        self.schedule_supervisor_review(&request).await?;

        // 5. Auto-revoke access after expiration
        self.schedule_access_revocation(request.expires_at).await?;

        Ok(phi_data)
    }
}
```

---

## 6. Authorization Policies (creto-authz)

### 6.1 Role Hierarchy

```
Attending Physician (Full Access)
    ├─ Resident Physician (Supervised Access)
    │   └─ Medical Student (Limited Access)
    │
    ├─ Specialist Consultant (Specialty-Specific)
    │
    └─ AI Diagnostic Agents (Time-Limited, Logged)
        ├─ Triage Agent (Demographics + Vitals)
        ├─ Imaging Agent (Imaging Studies Only)
        ├─ Lab Agent (Lab Results Only)
        └─ Synthesis Agent (Read-Only Aggregation)
```

### 6.2 ReBAC Policies

```rust
/// Relationship-Based Access Control for Healthcare
pub struct HealthcareAuthzPolicy {
    /// Patient-physician relationships
    care_relationships: HashMap<PatientNhi, Vec<PhysicianNhi>>,

    /// Agent supervision relationships
    agent_supervision: HashMap<AgentIdentity, PhysicianNhi>,

    /// Minimum necessary access rules
    minimum_necessary: MinimumNecessaryRules,
}

impl HealthcareAuthzPolicy {
    pub fn can_access_phi(
        &self,
        accessor: &AgentIdentity,
        patient_id: &PatientNhi,
        phi_type: PhiType,
    ) -> Result<AccessDecision> {
        // 1. Check direct care relationship
        if let Some(physicians) = self.care_relationships.get(patient_id) {
            if physicians.contains(&accessor.as_physician_nhi()) {
                return Ok(AccessDecision::Allow {
                    reason: "Direct care relationship",
                    expires_at: None,
                });
            }
        }

        // 2. Check if AI agent is supervised by care team physician
        if let Some(supervisor) = self.agent_supervision.get(accessor) {
            if self.care_relationships.get(patient_id)
                .map(|p| p.contains(supervisor))
                .unwrap_or(false)
            {
                // 3. Apply minimum necessary access
                let allowed_phi = self.minimum_necessary
                    .filter_phi_for_agent(accessor, phi_type)?;

                return Ok(AccessDecision::AllowFiltered {
                    reason: "Supervised agent access",
                    supervisor: supervisor.clone(),
                    allowed_fields: allowed_phi,
                    expires_at: Some(Utc::now() + Duration::hours(4)),
                });
            }
        }

        // 4. Check for break-glass access
        if let Some(break_glass) = self.check_break_glass(accessor, patient_id)? {
            return Ok(AccessDecision::AllowEmergency {
                reason: break_glass.justification,
                expires_at: break_glass.expires_at,
            });
        }

        // 5. Default deny
        Ok(AccessDecision::Deny {
            reason: "No care relationship or supervision",
        })
    }
}
```

### 6.3 Patient Consent Verification

```rust
/// HIPAA-compliant consent management
pub struct ConsentManager {
    /// Stored consents (signed with patient's NHI)
    consents: HashMap<PatientNhi, Vec<Consent>>,
}

pub struct Consent {
    /// Patient who provided consent
    patient: PatientNhi,

    /// What PHI can be disclosed
    phi_scope: Vec<PhiType>,

    /// Who can access it
    authorized_parties: Vec<AgentIdentity>,

    /// Purpose limitation
    purpose: ConsentPurpose,

    /// Expiration
    expires_at: Option<Timestamp>,

    /// Signature (ML-DSA-87)
    signature: Signature,
}

pub enum ConsentPurpose {
    Treatment,
    Payment,
    HealthcareOperations,
    Research { irb_approval: String },
    PublicHealth,
    Other { description: String },
}
```

### 6.4 Time-Limited Access Tokens

```rust
/// Short-lived PHI access token
pub struct PhiAccessToken {
    /// Who is accessing
    accessor: AgentIdentity,

    /// Which patient
    patient_id: PatientNhi,

    /// Which encounter (minimum necessary)
    encounter_id: EncounterId,

    /// Allowed operations
    permissions: Vec<PhiPermission>,

    /// Token expiration (max 4 hours)
    expires_at: Timestamp,

    /// Cryptographic signature
    signature: Signature,
}

pub enum PhiPermission {
    ReadDemographics,
    ReadVitals,
    ReadLabs,
    ReadImaging,
    ReadNotes,
    WriteAssessment,
    WriteOrders,
}

impl PhiAccessToken {
    /// Tokens auto-expire after 4 hours
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at &&
        self.verify_signature().is_ok()
    }

    /// Verify ML-DSA-87 signature
    pub fn verify_signature(&self) -> Result<()> {
        let public_key = self.accessor.public_key();
        let message = self.canonical_encoding();

        public_key.verify(&message, &self.signature)
            .map_err(|_| Error::InvalidTokenSignature)
    }
}
```

---

## 7. Oversight Workflows (CRITICAL for Healthcare)

### 7.1 Physician Approval Required

**Rule:** ALL diagnoses require physician approval before treatment.

```rust
pub struct PhysicianApprovalWorkflow {
    /// Pending approvals queue
    pending: Arc<Mutex<Vec<PendingApproval>>>,

    /// Notification service
    notifier: PhysicianNotifier,
}

pub struct PendingApproval {
    /// Diagnostic synthesis awaiting approval
    diagnosis: DiagnosisSynthesis,

    /// AI agent that generated it
    agent: AgentIdentity,

    /// Assigned attending physician
    assigned_physician: PhysicianNhi,

    /// Urgency level
    urgency: ApprovalUrgency,

    /// Time submitted
    submitted_at: Timestamp,

    /// SLA deadline
    deadline: Timestamp,
}

pub enum ApprovalUrgency {
    /// Life-threatening, requires immediate attention
    Critical { deadline: Duration::minutes(5) },

    /// Urgent, requires prompt attention
    Urgent { deadline: Duration::minutes(15) },

    /// Standard, can wait for physician rounds
    Standard { deadline: Duration::hours(2) },
}

impl PhysicianApprovalWorkflow {
    pub async fn submit_for_approval(
        &self,
        diagnosis: DiagnosisSynthesis,
        agent: AgentIdentity,
    ) -> Result<ApprovalTicket> {
        // 1. Determine urgency based on diagnosis severity
        let urgency = match diagnosis.primary_diagnosis.severity {
            Severity::Critical => ApprovalUrgency::Critical {
                deadline: Duration::minutes(5)
            },
            Severity::High => ApprovalUrgency::Urgent {
                deadline: Duration::minutes(15)
            },
            _ => ApprovalUrgency::Standard {
                deadline: Duration::hours(2)
            },
        };

        // 2. Find supervising physician
        let physician = self.find_supervising_physician(&agent).await?;

        // 3. Create pending approval
        let approval = PendingApproval {
            diagnosis: diagnosis.clone(),
            agent,
            assigned_physician: physician.clone(),
            urgency: urgency.clone(),
            submitted_at: Utc::now(),
            deadline: Utc::now() + urgency.deadline(),
        };

        // 4. Add to queue
        let ticket_id = self.enqueue_approval(approval).await?;

        // 5. Notify physician (SMS, pager, or dashboard)
        self.notifier.notify_physician(&physician, &diagnosis, &urgency).await?;

        // 6. Audit log
        self.audit_log(AuditEvent::ApprovalRequested {
            ticket_id,
            diagnosis: diagnosis.primary_diagnosis.icd10_code,
            urgency,
            timestamp: Utc::now(),
        }).await?;

        Ok(ApprovalTicket { ticket_id })
    }

    pub async fn physician_approves(
        &self,
        ticket_id: TicketId,
        physician: PhysicianNhi,
        modifications: Option<DiagnosisModifications>,
    ) -> Result<ApprovedDiagnosis> {
        // 1. Retrieve pending approval
        let mut approval = self.get_pending_approval(ticket_id).await?;

        // 2. Verify physician is authorized
        if approval.assigned_physician != physician {
            return Err(Error::UnauthorizedApprover);
        }

        // 3. Apply physician modifications (if any)
        let final_diagnosis = if let Some(mods) = modifications {
            self.apply_modifications(&approval.diagnosis, &mods)?
        } else {
            approval.diagnosis.clone()
        };

        // 4. Sign approved diagnosis
        let approved = ApprovedDiagnosis {
            original_diagnosis: approval.diagnosis,
            final_diagnosis,
            ai_agent: approval.agent,
            approving_physician: physician.clone(),
            physician_signature: self.sign_approval(&physician, &final_diagnosis)?,
            approved_at: Utc::now(),
        };

        // 5. Remove from pending queue
        self.dequeue_approval(ticket_id).await?;

        // 6. Audit log approval
        self.audit_log(AuditEvent::DiagnosisApproved {
            ticket_id,
            physician: physician.clone(),
            modifications: modifications.is_some(),
            timestamp: Utc::now(),
        }).await?;

        Ok(approved)
    }
}
```

### 7.2 Confidence Thresholds and Escalation

```rust
/// Automatic escalation rules based on AI confidence
pub struct EscalationPolicy {
    /// Confidence thresholds
    thresholds: ConfidenceThresholds,

    /// Escalation paths
    escalation_paths: HashMap<Severity, Vec<PhysicianNhi>>,
}

pub struct ConfidenceThresholds {
    /// Below this, immediate escalation to attending
    critical_threshold: f64, // 0.90

    /// Below this, flag for review within 15 min
    review_threshold: f64, // 0.80

    /// Below this, defer to physician entirely
    defer_threshold: f64, // 0.70
}

impl EscalationPolicy {
    pub fn should_escalate(&self, diagnosis: &DiagnosisSynthesis) -> EscalationDecision {
        match diagnosis.confidence {
            c if c < self.thresholds.defer_threshold => {
                EscalationDecision::DeferToPhysician {
                    reason: "AI confidence too low for recommendation",
                    urgency: ApprovalUrgency::Critical,
                }
            },
            c if c < self.thresholds.review_threshold => {
                EscalationDecision::RequireReview {
                    reason: "Moderate confidence, physician review recommended",
                    urgency: ApprovalUrgency::Urgent,
                }
            },
            c if c < self.thresholds.critical_threshold => {
                EscalationDecision::FlagForApproval {
                    reason: "Good confidence, but approval still required",
                    urgency: ApprovalUrgency::Standard,
                }
            },
            _ => {
                // Even high-confidence diagnoses require approval
                EscalationDecision::FlagForApproval {
                    reason: "High confidence, routine approval",
                    urgency: ApprovalUrgency::Standard,
                }
            }
        }
    }
}
```

### 7.3 Emergency Override (Attending Can Bypass AI)

```rust
pub struct EmergencyOverride {
    /// Attending physician initiating override
    physician: PhysicianNhi,

    /// Patient being treated
    patient_id: PatientNhi,

    /// Reason for override
    justification: String,

    /// Manual diagnosis (bypassing AI)
    manual_diagnosis: Diagnosis,

    /// Override timestamp
    timestamp: Timestamp,
}

impl PhysicianApprovalWorkflow {
    /// Attending can completely bypass AI in emergencies
    pub async fn emergency_override(
        &self,
        override_request: EmergencyOverride,
    ) -> Result<ApprovedDiagnosis> {
        // 1. Verify physician credentials
        self.verify_attending_credentials(&override_request.physician).await?;

        // 2. Log override (for later review)
        self.audit_log(AuditEvent::EmergencyOverride {
            physician: override_request.physician.clone(),
            patient_id: override_request.patient_id.clone(),
            justification: override_request.justification.clone(),
            timestamp: Utc::now(),
        }).await?;

        // 3. Alert medical director for post-hoc review
        self.alert_medical_director(&override_request).await?;

        // 4. Create approved diagnosis (AI bypassed)
        let approved = ApprovedDiagnosis {
            original_diagnosis: None, // No AI diagnosis
            final_diagnosis: override_request.manual_diagnosis,
            ai_agent: None, // AI not involved
            approving_physician: override_request.physician,
            physician_signature: self.sign_approval(
                &override_request.physician,
                &override_request.manual_diagnosis,
            )?,
            approved_at: Utc::now(),
        };

        Ok(approved)
    }
}
```

### 7.4 Documentation of Physician Reasoning

```rust
/// FDA 21 CFR Part 11 requires reasoning documentation
pub struct PhysicianReasoning {
    /// Diagnosis being approved/modified
    diagnosis_id: DiagnosisId,

    /// Physician's clinical reasoning
    reasoning: String,

    /// Agreement with AI assessment
    ai_agreement: AiAgreementLevel,

    /// Modifications made (if any)
    modifications: Vec<DiagnosisModification>,

    /// References consulted
    references: Vec<ClinicalReference>,

    /// Signature with timestamp
    signature: SignedReasoning,
}

pub enum AiAgreementLevel {
    /// Fully agree with AI assessment
    FullAgreement,

    /// Agree with diagnosis, minor modifications
    PartialAgreement {
        modifications: Vec<String>
    },

    /// Disagree with AI, different diagnosis
    Disagreement {
        physician_diagnosis: Diagnosis,
        rationale: String,
    },
}

pub struct DiagnosisModification {
    field: String,
    original_value: String,
    modified_value: String,
    reason: String,
}
```

---

## 8. Metering for Healthcare (creto-metering)

### 8.1 Per-Patient Encounter Metering

```rust
pub struct HealthcareMetering {
    /// Meter registry
    meters: HashMap<EncounterId, EncounterMeter>,
}

pub struct EncounterMeter {
    /// Unique encounter
    encounter_id: EncounterId,

    /// Patient (anonymized for billing)
    patient_id: PatientNhi,

    /// Facility
    facility: FacilityId,

    /// Agent usage breakdown
    agent_usage: HashMap<AgentType, AgentUsageMetrics>,

    /// Start and end time
    encounter_start: Timestamp,
    encounter_end: Option<Timestamp>,
}

pub struct AgentUsageMetrics {
    /// Number of inferences
    inference_count: u64,

    /// Total compute time
    compute_seconds: f64,

    /// Model version used
    model_version: String,

    /// Cost allocation
    cost: f64,
}
```

### 8.2 Per-Diagnostic Modality Metering

```rust
pub enum DiagnosticModality {
    /// Triage (ESI assessment)
    Triage {
        protocol: TriageProtocol,
        complexity: TriageComplexity,
    },

    /// Imaging analysis
    Imaging {
        modality: ImagingModality, // X-ray, CT, MRI
        study_count: u32,
        anatomical_region: String,
    },

    /// Laboratory interpretation
    Laboratory {
        panel_type: LabPanelType, // CMP, CBC, etc.
        result_count: u32,
    },

    /// Differential diagnosis
    DifferentialDiagnosis {
        complexity: DiagnosisComplexity,
        differential_count: u32,
    },

    /// Clinical documentation
    Documentation {
        note_type: NoteType, // SOAP, discharge summary, etc.
        word_count: u32,
    },
}

impl HealthcareMetering {
    pub fn meter_diagnostic_modality(
        &mut self,
        encounter_id: EncounterId,
        modality: DiagnosticModality,
        agent: AgentIdentity,
        duration: Duration,
    ) -> Result<MeteringRecord> {
        // Calculate cost based on modality and complexity
        let cost = match &modality {
            DiagnosticModality::Triage { complexity, .. } => {
                match complexity {
                    TriageComplexity::Low => 0.50,
                    TriageComplexity::Medium => 1.00,
                    TriageComplexity::High => 2.00,
                }
            },
            DiagnosticModality::Imaging { modality, study_count, .. } => {
                let per_study_cost = match modality {
                    ImagingModality::XRay => 5.00,
                    ImagingModality::CT => 15.00,
                    ImagingModality::MRI => 25.00,
                    ImagingModality::Ultrasound => 8.00,
                };
                per_study_cost * (*study_count as f64)
            },
            DiagnosticModality::Laboratory { result_count, .. } => {
                0.25 * (*result_count as f64)
            },
            DiagnosticModality::DifferentialDiagnosis { complexity, .. } => {
                match complexity {
                    DiagnosisComplexity::Straightforward => 3.00,
                    DiagnosisComplexity::Moderate => 8.00,
                    DiagnosisComplexity::Complex => 20.00,
                }
            },
            DiagnosticModality::Documentation { word_count, .. } => {
                0.01 * (*word_count as f64)
            },
        };

        // Record metering event
        let record = MeteringRecord {
            encounter_id,
            modality,
            agent,
            duration,
            cost,
            timestamp: Utc::now(),
        };

        self.record_usage(record.clone())?;

        Ok(record)
    }
}
```

### 8.3 CMS Billing Code Alignment

```rust
/// Map AI agent usage to CMS billing codes
pub struct CmsBillingMapper {
    /// CPT code mappings
    cpt_mappings: HashMap<DiagnosticModality, Vec<CptCode>>,

    /// ICD-10 diagnosis codes
    icd10_mappings: HashMap<Diagnosis, Vec<Icd10Code>>,
}

pub struct CptCode {
    code: String,
    description: String,
    rvu: f64, // Relative Value Unit
}

impl CmsBillingMapper {
    pub fn map_agent_usage_to_cpt(
        &self,
        usage: &AgentUsageMetrics,
        modality: &DiagnosticModality,
    ) -> Vec<CptCode> {
        match modality {
            DiagnosticModality::Triage { .. } => vec![
                CptCode {
                    code: "99281".to_string(), // ED visit, low complexity
                    description: "AI-assisted triage assessment".to_string(),
                    rvu: 1.0,
                }
            ],
            DiagnosticModality::Imaging { modality, .. } => {
                match modality {
                    ImagingModality::XRay => vec![
                        CptCode {
                            code: "76140".to_string(), // CAD X-ray
                            description: "AI radiology interpretation".to_string(),
                            rvu: 0.5,
                        }
                    ],
                    ImagingModality::CT => vec![
                        CptCode {
                            code: "76497".to_string(), // CAD CT
                            description: "AI CT interpretation".to_string(),
                            rvu: 1.5,
                        }
                    ],
                    // ... other modalities
                }
            },
            // ... other diagnostic types
        }
    }
}
```

---

## 9. Messaging Security (creto-messaging)

### 9.1 HIPAA-Compliant Secure Messaging

```rust
pub struct HipaaMessaging {
    /// End-to-end encrypted message bus
    message_bus: CretoMessageBus,

    /// PHI filtering (no PHI in metadata)
    phi_filter: PhiMetadataFilter,

    /// Delivery receipt tracking
    receipt_tracker: DeliveryReceiptTracker,
}

impl HipaaMessaging {
    /// Send PHI-containing message
    pub async fn send_phi_message(
        &self,
        from: AgentIdentity,
        to: AgentIdentity,
        payload: PhiMessage,
    ) -> Result<MessageReceipt> {
        // 1. Verify sender authorization
        self.verify_sender_authorization(&from).await?;

        // 2. Encrypt payload with recipient's public key
        let encrypted_payload = self.encrypt_for_recipient(&to, &payload)?;

        // 3. Create message envelope (NO PHI in metadata)
        let envelope = MessageEnvelope {
            from: from.clone(),
            to: to.clone(),
            message_type: MessageType::ClinicalData, // Generic type
            encrypted_payload,
            timestamp: Utc::now(),
            signature: self.sign_message(&from, &encrypted_payload)?,
        };

        // 4. Send via encrypted channel
        let message_id = self.message_bus.send(envelope).await?;

        // 5. Track delivery receipt
        let receipt = self.receipt_tracker.track(message_id).await?;

        // 6. Audit log (HIPAA requirement)
        self.audit_log(AuditEvent::PhiMessageSent {
            from: from.clone(),
            to: to.clone(),
            message_id,
            timestamp: Utc::now(),
        }).await?;

        Ok(receipt)
    }
}
```

### 9.2 No PHI in Message Metadata

**HIPAA Rule:** Message routing information cannot reveal PHI.

```rust
/// Message envelope design (HIPAA-compliant)
pub struct MessageEnvelope {
    /// Sender NHI (agent identity, NOT patient ID)
    from: AgentIdentity,

    /// Recipient NHI
    to: AgentIdentity,

    /// Generic message type (no diagnosis info)
    message_type: MessageType,

    /// Encrypted payload (PHI hidden inside)
    encrypted_payload: Vec<u8>,

    /// Timestamp
    timestamp: Timestamp,

    /// ML-DSA-87 signature
    signature: Signature,
}

pub enum MessageType {
    /// Generic clinical data (no specifics)
    ClinicalData,

    /// Lab results available (no values in metadata)
    LabResultsReady,

    /// Imaging study complete (no findings in metadata)
    ImagingComplete,

    /// Diagnosis awaiting approval (no diagnosis in metadata)
    DiagnosisForReview,
}

/// Encrypted message payload (PHI inside)
pub struct PhiMessage {
    /// Actual patient ID (encrypted)
    patient_id: PatientNhi,

    /// Clinical data (encrypted)
    data: ClinicalData,

    /// Context (encrypted)
    context: MessageContext,
}
```

### 9.3 Delivery Receipts for Legal Compliance

```rust
/// Legally-binding delivery receipts
pub struct DeliveryReceipt {
    /// Message ID
    message_id: MessageId,

    /// Delivery status
    status: DeliveryStatus,

    /// Timestamp of delivery
    delivered_at: Option<Timestamp>,

    /// Recipient signature (proves receipt)
    recipient_signature: Option<Signature>,
}

pub enum DeliveryStatus {
    /// Message sent, awaiting delivery
    Sent,

    /// Message delivered to recipient
    Delivered { delivered_at: Timestamp },

    /// Message read by recipient
    Read {
        delivered_at: Timestamp,
        read_at: Timestamp,
        recipient_signature: Signature,
    },

    /// Delivery failed
    Failed {
        reason: String,
        failed_at: Timestamp,
    },
}

impl HipaaMessaging {
    /// Recipient acknowledges message receipt
    pub async fn acknowledge_receipt(
        &self,
        message_id: MessageId,
        recipient: AgentIdentity,
    ) -> Result<()> {
        // 1. Sign acknowledgment
        let signature = recipient.sign(&message_id.to_bytes())?;

        // 2. Update receipt status
        self.receipt_tracker.mark_read(
            message_id,
            Utc::now(),
            signature,
        ).await?;

        // 3. Audit log acknowledgment
        self.audit_log(AuditEvent::MessageAcknowledged {
            message_id,
            recipient: recipient.clone(),
            timestamp: Utc::now(),
        }).await?;

        Ok(())
    }
}
```

---

## 10. Clinical Workflow Integration

### 10.1 FHIR R4 APIs for EHR Integration

```rust
pub struct FhirR4Integration {
    /// FHIR client
    fhir_client: FhirClient,

    /// Resource mappers
    mappers: FhirResourceMappers,
}

impl FhirR4Integration {
    /// Export diagnosis as FHIR DiagnosticReport
    pub async fn export_diagnostic_report(
        &self,
        diagnosis: &ApprovedDiagnosis,
        encounter: &Encounter,
    ) -> Result<FhirDiagnosticReport> {
        let report = FhirDiagnosticReport {
            resource_type: "DiagnosticReport".to_string(),
            id: diagnosis.id.to_string(),
            status: "final".to_string(),
            category: vec![
                CodeableConcept {
                    coding: vec![
                        Coding {
                            system: "http://terminology.hl7.org/CodeSystem/v2-0074".to_string(),
                            code: "LAB".to_string(),
                            display: "Laboratory".to_string(),
                        }
                    ]
                }
            ],
            code: self.mappers.map_diagnosis_to_loinc(&diagnosis.final_diagnosis)?,
            subject: Reference {
                reference: format!("Patient/{}", encounter.patient_id),
            },
            encounter: Some(Reference {
                reference: format!("Encounter/{}", encounter.id),
            }),
            effective_date_time: Some(diagnosis.approved_at),
            issued: diagnosis.approved_at,
            performer: vec![
                Reference {
                    reference: format!("Practitioner/{}", diagnosis.approving_physician),
                    display: Some("Approving Physician".to_string()),
                },
                Reference {
                    reference: format!("Device/{}", diagnosis.ai_agent.unwrap()),
                    display: Some("AI Diagnostic Agent".to_string()),
                }
            ],
            conclusion: Some(diagnosis.final_diagnosis.description.clone()),
            conclusion_code: vec![
                self.mappers.map_diagnosis_to_snomed(&diagnosis.final_diagnosis)?
            ],
        };

        // POST to EHR FHIR server
        self.fhir_client.create_resource(report).await
    }
}
```

### 10.2 HL7v2 for Legacy Systems

```rust
pub struct Hl7v2Integration {
    /// HL7 message builder
    message_builder: Hl7MessageBuilder,

    /// TCP/MLLP client
    mllp_client: MllpClient,
}

impl Hl7v2Integration {
    /// Send ORU^R01 (Observation Result) message
    pub async fn send_lab_results(
        &self,
        patient: &Patient,
        results: &LabInterpretation,
    ) -> Result<Hl7Ack> {
        let message = self.message_builder
            .msh("ORU", "R01", "CRETO_AI", "EHR_SYSTEM")
            .pid(patient)
            .obr("LAB", "Laboratory Results")
            .obx_results(results)
            .build()?;

        // Send via MLLP (Minimal Lower Layer Protocol)
        let ack = self.mllp_client.send_and_wait_ack(message).await?;

        Ok(ack)
    }
}
```

### 10.3 CDS Hooks for Real-Time Decision Support

```rust
pub struct CdsHooksIntegration {
    /// CDS Hooks service endpoint
    service_endpoint: Url,

    /// Registered hooks
    hooks: Vec<CdsHook>,
}

pub struct CdsHook {
    /// Hook ID
    id: String,

    /// Hook type (patient-view, order-select, etc.)
    hook: CdsHookType,

    /// Description
    description: String,

    /// Prefetch templates
    prefetch: HashMap<String, String>,
}

pub enum CdsHookType {
    /// Triggered when opening patient chart
    PatientView,

    /// Triggered when ordering medication
    OrderSelect,

    /// Triggered when signing order
    OrderSign,
}

impl CdsHooksIntegration {
    /// Respond to CDS Hook invocation
    pub async fn handle_hook_invocation(
        &self,
        request: CdsHookRequest,
    ) -> Result<CdsHookResponse> {
        match request.hook {
            CdsHookType::OrderSelect => {
                // Check for drug interactions
                let interactions = self.check_drug_interactions(&request).await?;

                if !interactions.is_empty() {
                    Ok(CdsHookResponse {
                        cards: vec![
                            Card {
                                summary: "Potential Drug Interaction".to_string(),
                                indicator: "warning".to_string(),
                                detail: format!("Found {} interactions", interactions.len()),
                                source: CardSource {
                                    label: "Creto AI".to_string(),
                                },
                                suggestions: interactions.into_iter()
                                    .map(|i| Suggestion {
                                        label: i.recommendation,
                                        actions: vec![],
                                    })
                                    .collect(),
                            }
                        ]
                    })
                } else {
                    Ok(CdsHookResponse { cards: vec![] })
                }
            },
            _ => Ok(CdsHookResponse { cards: vec![] }),
        }
    }
}
```

---

## 11. Data Model

### 11.1 Patient Demographics (Encrypted)

```rust
pub struct Patient {
    /// Patient NHI (cryptographic identity)
    nhi: PatientNhi,

    /// Encrypted demographics
    demographics: EncryptedDemographics,

    /// Medical record number (MRN)
    mrn: String,

    /// Active care team
    care_team: Vec<PhysicianNhi>,
}

pub struct Demographics {
    /// Full name
    name: HumanName,

    /// Date of birth
    date_of_birth: Date,

    /// Biological sex
    sex: BiologicalSex,

    /// Gender identity
    gender: Option<String>,

    /// Contact information
    contact: ContactInfo,

    /// Insurance information
    insurance: Vec<InsuranceInfo>,
}
```

### 11.2 Clinical Observations (LOINC Coded)

```rust
pub struct ClinicalObservation {
    /// LOINC code (standardized)
    loinc_code: LoincCode,

    /// Observation value
    value: ObservationValue,

    /// Unit of measure
    unit: String,

    /// Reference range
    reference_range: Option<ReferenceRange>,

    /// Timestamp
    observed_at: Timestamp,

    /// Observer (agent or physician)
    observer: AgentIdentity,
}

pub enum ObservationValue {
    Numeric(f64),
    Text(String),
    Coded(CodeableConcept),
    Boolean(bool),
}

pub struct LoincCode {
    code: String, // e.g., "8867-4" for heart rate
    display: String, // e.g., "Heart rate"
    system: String, // "http://loinc.org"
}
```

### 11.3 Diagnoses (ICD-10/SNOMED-CT)

```rust
pub struct Diagnosis {
    /// ICD-10-CM code (billing)
    icd10_code: Icd10Code,

    /// SNOMED-CT code (clinical)
    snomed_code: SnomedCode,

    /// Description
    description: String,

    /// Severity
    severity: Severity,

    /// Confidence score (for AI diagnoses)
    confidence: Option<f64>,

    /// Onset date
    onset_date: Option<Date>,

    /// Status
    status: DiagnosisStatus,
}

pub struct Icd10Code {
    code: String, // e.g., "E11.9" for Type 2 diabetes
    description: String,
}

pub struct SnomedCode {
    code: String, // e.g., "44054006" for Type 2 diabetes
    display: String,
    system: String, // "http://snomed.info/sct"
}

pub enum Severity {
    Mild,
    Moderate,
    High,
    Critical,
}

pub enum DiagnosisStatus {
    Provisional,
    Confirmed,
    Ruled_Out,
    Differential,
}
```

### 11.4 Treatment Plans

```rust
pub struct TreatmentPlan {
    /// Diagnosis being treated
    diagnosis: DiagnosisId,

    /// Medications
    medications: Vec<MedicationOrder>,

    /// Procedures
    procedures: Vec<ProcedureOrder>,

    /// Follow-up instructions
    followup: FollowupInstructions,

    /// Created by
    created_by: AgentIdentity,

    /// Approved by
    approved_by: PhysicianNhi,
}

pub struct MedicationOrder {
    /// RxNorm code
    rxnorm_code: String,

    /// Medication name
    name: String,

    /// Dosage
    dosage: String,

    /// Route
    route: String, // e.g., "oral", "IV"

    /// Frequency
    frequency: String, // e.g., "BID", "QID"

    /// Duration
    duration: String,
}
```

---

## 12. Sequence Diagrams

### 12.1 Normal Diagnostic Flow

```
Patient → Triage Agent: Chief complaint + vitals
Triage Agent → creto-authz: Verify PHI access
creto-authz → Triage Agent: Access granted (4-hour token)
Triage Agent → creto-vault: Decrypt patient data
Triage Agent → Triage Agent: AI triage assessment
Triage Agent → creto-audit: Log triage event
Triage Agent → Imaging Agent: Order imaging (if needed)

Imaging Agent → creto-authz: Verify PHI access
Imaging Agent → PACS: Retrieve DICOM images
Imaging Agent → Imaging Agent: FDA-cleared AI analysis
Imaging Agent → creto-audit: Log imaging analysis
Imaging Agent → Synthesis Agent: Send findings

Lab Agent → Lab System: Retrieve results
Lab Agent → Lab Agent: Interpret lab values
Lab Agent → creto-audit: Log interpretation
Lab Agent → Synthesis Agent: Send findings

Synthesis Agent → Synthesis Agent: Generate differential diagnosis
Synthesis Agent → creto-audit: Log synthesis
Synthesis Agent → Approval Workflow: Submit for physician approval
Approval Workflow → Physician: Notify (SMS/pager)

Physician → Physician Dashboard: Review AI diagnosis
Physician → Approval Workflow: Approve (with/without modifications)
Approval Workflow → creto-audit: Log approval
Approval Workflow → Documentation Agent: Generate clinical note

Documentation Agent → EHR: Export FHIR DiagnosticReport
EHR → Patient Portal: Make results available
```

### 12.2 Emergency Escalation

```
Imaging Agent → Imaging Agent: Critical finding detected (PE, stroke, etc.)
Imaging Agent → creto-audit: Log critical finding
Imaging Agent → Escalation Service: IMMEDIATE escalation
Escalation Service → On-Call Physician: Page with STAT priority
Escalation Service → ED Physician: Dashboard alert
Escalation Service → Radiologist: Concurrent notification

On-Call Physician → Physician Dashboard: View critical imaging
On-Call Physician → Patient Bedside: Immediate evaluation
On-Call Physician → Approval Workflow: Emergency override (bypass AI)
On-Call Physician → Treatment: Immediate intervention

Escalation Service → creto-audit: Log emergency escalation
Medical Director → Audit Trail: Post-hoc review of override
```

### 12.3 Multi-Specialty Consultation

```
Synthesis Agent → Synthesis Agent: Complex case requiring consultation
Synthesis Agent → Approval Workflow: Request multi-specialty review
Approval Workflow → Primary Physician: Assign case
Approval Workflow → Cardiologist: Consult request
Approval Workflow → Nephrologist: Consult request

Cardiologist → creto-authz: Request PHI access
creto-authz → Cardiologist: Grant limited access (cardiac data only)
Cardiologist → Physician Dashboard: Review cardiac findings
Cardiologist → Approval Workflow: Provide cardiology assessment

Nephrologist → creto-authz: Request PHI access
creto-authz → Nephrologist: Grant limited access (renal data only)
Nephrologist → Physician Dashboard: Review renal findings
Nephrologist → Approval Workflow: Provide nephrology assessment

Approval Workflow → Synthesis Agent: Aggregate specialist opinions
Synthesis Agent → Primary Physician: Unified treatment plan
Primary Physician → Approval Workflow: Final approval

Approval Workflow → creto-audit: Log multi-specialty consultation
```

---

## 13. Performance Specifications

### 13.1 Latency Requirements

| Agent/Operation | Target Latency | Maximum Latency | SLA |
|----------------|----------------|-----------------|-----|
| **Triage Assessment** | <30 seconds | 60 seconds | 99.5% |
| **Imaging Analysis** | <2 minutes | 5 minutes | 99% |
| **Lab Interpretation** | <1 minute | 3 minutes | 99% |
| **Diagnosis Synthesis** | <5 minutes | 10 minutes | 98% |
| **Physician Approval** | <10 minutes (target) | N/A (human) | N/A |
| **creto-authz Check** | <168ns | 1ms | 99.99% |
| **PHI Decryption** | <1ms | 10ms | 99.9% |
| **Audit Log Write** | <5ms | 50ms | 99.9% |

### 13.2 Throughput Requirements

| Metric | Target | Peak |
|--------|--------|------|
| **Concurrent Patients** | 100 | 500 |
| **Triage Assessments/Hour** | 120 | 600 |
| **Imaging Studies/Hour** | 50 | 200 |
| **Lab Interpretations/Hour** | 200 | 1000 |
| **Messages/Second** | 1000 | 5000 |

### 13.3 Accuracy Requirements

| Agent | Metric | Target | Validation |
|-------|--------|--------|------------|
| **Triage** | ESI Accuracy vs. RN | >95% | Retrospective chart review |
| **Imaging** | Sensitivity (detecting abnormalities) | >98% | FDA validation dataset |
| **Imaging** | Specificity (avoiding false positives) | >92% | FDA validation dataset |
| **Lab** | Critical value detection | 100% | Zero tolerance for misses |
| **Synthesis** | Diagnostic accuracy vs. attending | >93% | Peer review |
| **Synthesis** | Confidence calibration | ±5% | Brier score analysis |

### 13.4 Availability Requirements

| Component | SLA | Downtime/Year |
|-----------|-----|---------------|
| **AI Agents** | 99.9% | 8.76 hours |
| **creto-authz** | 99.99% | 52.56 minutes |
| **creto-vault** | 99.99% | 52.56 minutes |
| **creto-audit** | 99.999% | 5.26 minutes |
| **Physician Dashboard** | 99.5% | 43.8 hours |

---

## 14. Demo Script (4-Minute Investor Walkthrough)

### 14.1 Act 1: The Problem (0:00 - 0:30)

**Narrator:** "Emergency departments across America face a crisis: average wait times have hit 4 hours. Patients with heart attacks wait alongside minor injuries. Physicians drown in paperwork instead of treating patients. This isn't just inefficient—it's deadly."

**Visual:** Split-screen showing:
- Left: Crowded ER waiting room, clock showing 4:13 wait time
- Right: Exhausted physician at computer, typing notes at 11 PM

**Narrator:** "What if AI could triage in 30 seconds? Analyze imaging in 2 minutes? Generate clinical notes automatically? While keeping physicians in control?"

### 14.2 Act 2: Patient Intake & AI Triage (0:30 - 1:30)

**Visual:** Patient "John Smith" (synthetic data) arrives at ED kiosk

**On-Screen Text:**
```
Patient: John Smith (Demo Data Only - HIPAA Compliant)
Chief Complaint: "Chest pain"
Vitals: BP 145/92, HR 98, RR 18, O2 97%
```

**Action:** Triage Agent activates

**Visual:** Dashboard showing:
```
┌─────────────────────────────────────────────┐
│ Triage Agent (AI)                           │
│ Status: Analyzing...                        │
│                                             │
│ ✓ Vital signs within normal limits         │
│ ⚠ Chest pain + elevated BP = moderate risk │
│ ✓ No respiratory distress                  │
│                                             │
│ ESI Level: 3 (Moderate urgency)            │
│ Confidence: 94%                             │
│ Recommended: EKG + Troponin                 │
│ Estimated Wait: 15 minutes                  │
│                                             │
│ Time to Assessment: 28 seconds              │
└─────────────────────────────────────────────┘
```

**Narrator:** "In under 30 seconds, the AI has triaged the patient, ordered appropriate tests, and assigned priority—work that typically takes 10-15 minutes of nurse time."

**Visual:** Audit trail appears in corner:
```
[AUDIT LOG - HIPAA COMPLIANT]
12:34:56 - Triage Agent accessed patient vitals (creto-authz: GRANTED)
12:35:24 - ESI assessment completed (confidence: 94%)
12:35:24 - EKG ordered, Troponin ordered
12:35:25 - Audit log written (Merkle root: 0x4a2f...)
```

### 14.3 Act 3: Imaging Analysis with Confidence Scores (1:30 - 2:30)

**Visual:** EKG completes, chest X-ray ordered

**Action:** Imaging Agent activates

**Visual:** Dashboard showing AI analyzing chest X-ray:
```
┌─────────────────────────────────────────────┐
│ Imaging Agent (AI)                          │
│ Modality: Chest X-ray (PA + Lateral)       │
│ Status: Analyzing with FDA-cleared model... │
│                                             │
│ ✓ Cardiac silhouette: Normal                │
│ ✓ Lung fields: Clear                        │
│ ⚠ Mild cardiomegaly noted                   │
│                                             │
│ Findings:                                    │
│ - No acute cardiopulmonary process          │
│ - Borderline cardiomegaly (CTR 52%)        │
│                                             │
│ Confidence: 91%                             │
│ Recommendation: Correlate with echo         │
│                                             │
│ Time to Analysis: 1 minute 47 seconds       │
│                                             │
│ [Radiologist Review: NOT REQUIRED]          │
└─────────────────────────────────────────────┘
```

**Narrator:** "The AI analyzes the imaging in under 2 minutes using an FDA-cleared model. Notice the confidence score: 91%. High enough to be useful, but the system knows its limits."

**Visual:** Lab results appear:
```
┌─────────────────────────────────────────────┐
│ Lab Agent (AI)                              │
│ Panel: Cardiac Biomarkers                   │
│                                             │
│ Troponin I: 0.02 ng/mL (Normal: <0.04)     │
│ BNP: 125 pg/mL (Mildly elevated)           │
│ CK-MB: 3.2 ng/mL (Normal)                  │
│                                             │
│ Interpretation:                             │
│ ✓ Negative for acute MI                    │
│ ⚠ Mild BNP elevation suggests CHF          │
│                                             │
│ Confidence: 97%                             │
│ Time to Interpretation: 52 seconds          │
└─────────────────────────────────────────────┘
```

### 14.4 Act 4: Physician Dashboard with Approval Workflow (2:30 - 3:30)

**Visual:** Switch to Physician Dashboard (Dr. Sarah Chen)

**On-Screen:**
```
┌─────────────────────────────────────────────┐
│ PHYSICIAN DASHBOARD - Dr. Sarah Chen        │
│                                             │
│ [PENDING APPROVAL: 3 cases]                 │
│                                             │
│ ┌─────────────────────────────────────────┐ │
│ │ Patient: John Smith                     │ │
│ │ AI Diagnosis: Acute decompensated CHF   │ │
│ │ Confidence: 88%                         │ │
│ │                                         │ │
│ │ Supporting Evidence:                    │ │
│ │ • Chest pain + elevated BP              │ │
│ │ • Cardiomegaly on X-ray                │ │
│ │ • Elevated BNP (125)                   │ │
│ │                                         │ │
│ │ Differential:                           │ │
│ │ 1. Acute CHF (88% confidence)          │ │
│ │ 2. Hypertensive urgency (7%)           │ │
│ │ 3. Coronary artery disease (5%)        │ │
│ │                                         │ │
│ │ Recommended Treatment:                  │ │
│ │ • Furosemide 40mg IV                   │ │
│ │ • Echocardiogram                       │ │
│ │ • Admit to cardiology                  │ │
│ │                                         │ │
│ │ [APPROVE] [MODIFY] [REJECT]            │ │
│ └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

**Narrator:** "Here's the key: the AI doesn't replace the physician. It augments them. Dr. Chen sees the AI's reasoning, confidence scores, and differential diagnosis. She makes the final call."

**Action:** Dr. Chen clicks [APPROVE]

**Visual:** Approval confirmation:
```
┌─────────────────────────────────────────────┐
│ DIAGNOSIS APPROVED                          │
│                                             │
│ Primary Diagnosis: Acute CHF (I50.9)       │
│ AI Assessment: 88% confidence               │
│ Physician: Dr. Sarah Chen (Attending)       │
│ Signature: [ML-DSA-87 cryptographic]        │
│ Time: 2025-12-26 12:42:18 UTC              │
│                                             │
│ Treatment Plan:                             │
│ ✓ Furosemide 40mg IV ordered                │
│ ✓ Echo scheduled                            │
│ ✓ Cardiology consult requested              │
│                                             │
│ Clinical Note: Auto-generated (FDA 21 CFR)  │
│ Time Saved: 18 minutes                      │
└─────────────────────────────────────────────┘
```

**Narrator:** "From triage to diagnosis to treatment plan—complete in under 12 minutes. Traditional workflow? 2-4 hours. The physician focused on clinical judgment, not paperwork."

### 14.5 Act 5: HIPAA Audit Trail (3:30 - 4:00)

**Visual:** Zoom out to show audit trail in real-time:

```
┌──────────────────────────────────────────────────────────┐
│ IMMUTABLE AUDIT TRAIL (creto-audit + Merkle Tree)        │
│                                                          │
│ 12:34:56 - Triage Agent: Access GRANTED (creto-authz)   │
│           └─ PHI Access: Demographics + Vitals          │
│           └─ Token Expiry: 16:34:56                     │
│                                                          │
│ 12:36:43 - Imaging Agent: Access GRANTED                │
│           └─ PHI Access: Imaging Studies Only           │
│           └─ Supervisor: Dr. Sarah Chen                 │
│                                                          │
│ 12:38:17 - Lab Agent: Access GRANTED                    │
│           └─ PHI Access: Lab Results Only               │
│                                                          │
│ 12:41:05 - Synthesis Agent: Access GRANTED              │
│           └─ PHI Access: Read-Only Aggregation          │
│                                                          │
│ 12:42:18 - Dr. Sarah Chen: Diagnosis APPROVED           │
│           └─ Signature: ML-DSA-87 (quantum-safe)        │
│           └─ Modifications: None                        │
│                                                          │
│ [Every access logged. Every decision traced.]           │
│ [Merkle Root: 0x4a2f9b8c...]                           │
│                                                          │
│ HIPAA Compliance: ✓ Access Control (§164.312)          │
│                  ✓ Audit Logging (§164.312)           │
│                  ✓ Integrity (§164.312)               │
└──────────────────────────────────────────────────────────┘
```

**Narrator:** "Every access, every decision, every approval—logged in an immutable audit trail. HIPAA compliance isn't an afterthought. It's cryptographically enforced."

**Final Visual:** Split-screen comparison:

```
┌──────────────────────┬──────────────────────┐
│ Traditional ER       │ Creto AI ER          │
├──────────────────────┼──────────────────────┤
│ 4 hour wait          │ 15 minute wait       │
│ 45 min per patient   │ 12 min per patient   │
│ 20 min documentation │ Auto-generated       │
│ Username/password    │ Cryptographic NHI    │
│ Manual audit logs    │ Immutable Merkle     │
│ HIPAA violations     │ Zero violations      │
│                      │                      │
│ Physician burnout    │ Physician empowered  │
└──────────────────────┴──────────────────────┘
```

**Narrator:** "This is Creto: AI that augments physicians, not replaces them. HIPAA compliance by design. The future of healthcare—available today."

**End Screen:**
```
Creto Healthcare AI
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
84% faster diagnostics
Zero HIPAA violations
$2.4M annual savings per hospital

Schedule your demo: healthcare@creto.ai
```

---

## 15. Risk Mitigation

### 15.1 Malpractice Risk

**Risk:** AI provides incorrect diagnosis, leading to patient harm.

**Mitigation:**
1. **Physician Always Decides:** ALL diagnoses require attending physician approval
2. **Confidence Thresholds:** Low-confidence (<90%) automatically escalates to physician
3. **Liability Insurance:** Creto maintains $10M professional liability coverage
4. **Model Validation:** FDA-cleared models with documented accuracy
5. **Audit Trail:** Every decision is logged with physician signature (legal protection)

**Legal Position:**
- AI is a "medical device" (FDA regulated)
- Physician retains ultimate decision-making authority
- Creto provides decision support, not diagnosis
- Liability remains with physician (standard of care)

### 15.2 Bias Risk

**Risk:** AI models exhibit racial, gender, or socioeconomic bias.

**Mitigation:**
1. **Diverse Training Data:**
   - Minimum 30% representation from underrepresented groups
   - Geographic diversity (urban/rural)
   - Socioeconomic diversity
2. **Regular Bias Audits:**
   - Quarterly fairness assessments
   - Stratified performance analysis (by race, gender, age)
   - External auditors (e.g., NIST AI Risk Management Framework)
3. **Transparent Model Cards:**
   - Document training data demographics
   - Report performance stratified by patient groups
   - Disclose known limitations
4. **Continuous Monitoring:**
   - Real-time bias detection in production
   - Alert if accuracy drops below threshold for any group

**Regulatory Compliance:**
- FDA guidance on AI/ML bias (2023)
- NIST AI RMF: "Measure" and "Manage" phases
- CMS health equity requirements

### 15.3 Privacy Risk

**Risk:** PHI breach or unauthorized access.

**Mitigation:**
1. **Zero PHI in Demo:**
   - All demo patients are synthetic (Synthea dataset)
   - No real patient data used in demos or testing
   - Clear labeling: "DEMO DATA ONLY - NOT REAL PATIENTS"
2. **Encryption Everywhere:**
   - AES-256-GCM at rest
   - TLS 1.3 + ML-KEM-1024 in transit
   - E2E encryption for messaging (no plaintext PHI)
3. **Cryptographic Access Control:**
   - creto-authz enforces policies at 168ns
   - Time-limited tokens (max 4 hours)
   - Minimum necessary access (ReBAC)
4. **Immutable Audit:**
   - Every PHI access logged to Merkle tree
   - Tamper-evident (cryptographic proof)
   - Real-time anomaly detection
5. **Break-Glass Monitoring:**
   - Emergency access logged BEFORE granted
   - Compliance team alerted immediately
   - Supervisor review within 24 hours

**HIPAA Compliance:**
- § 164.312 (Technical Safeguards): 100% coverage
- § 164.308 (Administrative): Automated policy enforcement
- § 164.310 (Physical): Cloud SOC 2 Type II compliance

### 15.4 Regulatory Risk

**Risk:** FDA, CMS, or state medical boards restrict AI usage.

**Mitigation:**
1. **FDA Clearance:**
   - All diagnostic models undergo FDA 510(k) clearance
   - Predicate devices: Existing CAD systems
   - Clinical validation studies with 1000+ patients
2. **Physician Oversight:**
   - AI never makes autonomous decisions
   - Attending physician approval required
   - Emergency override capability
3. **Continuous Monitoring:**
   - Real-time performance tracking
   - Automatic alerts if accuracy drops
   - Post-market surveillance (FDA requirement)
4. **State Medical Board Compliance:**
   - AI agents are "devices," not "practitioners"
   - Physicians maintain medical licenses
   - Supervision requirements met

### 15.5 Technical Risk

**Risk:** System downtime or performance degradation during critical patient care.

**Mitigation:**
1. **High Availability:**
   - 99.9% SLA for AI agents
   - 99.99% SLA for creto-authz and creto-vault
   - Multi-region deployment
2. **Graceful Degradation:**
   - If AI unavailable, fall back to traditional workflow
   - Physicians can always override or bypass AI
   - Manual entry mode for all functions
3. **Performance Monitoring:**
   - Real-time latency tracking
   - Automatic scaling (Kubernetes)
   - Circuit breakers for slow agents
4. **Disaster Recovery:**
   - RPO: 15 minutes (Recovery Point Objective)
   - RTO: 1 hour (Recovery Time Objective)
   - Daily backups with 7-year retention (HIPAA)

---

## Appendix A: Glossary

| Term | Definition |
|------|------------|
| **CDS Hooks** | Clinical Decision Support Hooks - FHIR standard for EHR integration |
| **CPT** | Current Procedural Terminology - medical billing codes |
| **DEA** | Drug Enforcement Administration - controlled substance prescribing |
| **ESI** | Emergency Severity Index - 5-level triage system |
| **FHIR** | Fast Healthcare Interoperability Resources - data exchange standard |
| **HL7** | Health Level 7 - healthcare messaging standard |
| **ICD-10** | International Classification of Diseases, 10th Revision - diagnosis codes |
| **LOINC** | Logical Observation Identifiers Names and Codes - lab/observation standard |
| **MLLP** | Minimal Lower Layer Protocol - HL7 transport |
| **NHI** | Network Human Identity - creto cryptographic identity |
| **PHI** | Protected Health Information - HIPAA-regulated patient data |
| **ReBAC** | Relationship-Based Access Control - authorization model |
| **RxNorm** | Standardized nomenclature for medications |
| **SNOMED-CT** | Systematized Nomenclature of Medicine - Clinical Terms |
| **SOAP** | Subjective, Objective, Assessment, Plan - clinical note format |

---

## Appendix B: References

1. **HIPAA Security Rule** - 45 CFR § 164.312 (Technical Safeguards)
2. **FDA 21 CFR Part 11** - Electronic Records and Electronic Signatures
3. **FDA Guidance on AI/ML** - "Artificial Intelligence and Machine Learning in Software as a Medical Device" (2023)
4. **NIST AI Risk Management Framework** - AI 100-1 (2023)
5. **CMS Conditions of Participation** - Clinical Decision Support Requirements
6. **FHIR R4 Specification** - HL7 FHIR Release 4 (2019)
7. **HL7v2 Standard** - HL7 Version 2.9 (2023)
8. **LOINC Database** - Regenstrief Institute (2024)
9. **SNOMED-CT** - International Health Terminology Standards Development Organisation

---

**Document Control:**
- **Author:** Creto Systems Architecture Team
- **Reviewers:** Medical Informatics, Legal/Compliance, FDA Regulatory Affairs
- **Classification:** Confidential - HIPAA Protected
- **Version History:**
  - v1.0.0 (2025-12-26) - Initial draft for Demo 3

---

*This document describes a demonstration system designed for investor presentations. All patient data shown in demos is synthetic and HIPAA-compliant. No real patient data is used.*
