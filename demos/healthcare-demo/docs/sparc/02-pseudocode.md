# Phase 2: Pseudocode - Clinical Decision Support Algorithms

## 1. Symptom Analysis Algorithm

```pseudocode
FUNCTION analyzeSymptoms(encounter, symptoms[])
  // Step 1: Normalize and structure symptom data
  normalizedSymptoms = []
  FOR EACH symptom IN symptoms
    normalized = {
      description: symptom.description.toLowerCase().trim(),
      snomedCode: mapToSNOMED(symptom.description),
      severity: symptom.severity,
      duration: parseDuration(symptom.duration),
      redFlag: checkRedFlags(symptom)
    }
    normalizedSymptoms.push(normalized)
  END FOR

  // Step 2: Identify red flag symptoms requiring immediate attention
  redFlags = normalizedSymptoms.filter(s => s.redFlag == true)
  IF redFlags.length > 0 THEN
    createUrgentAlert(encounter, redFlags)
    priorityLevel = 'STAT'
  ELSE
    priorityLevel = calculateUrgency(normalizedSymptoms)
  END IF

  // Step 3: Correlate with patient medical history
  patientHistory = getPatientHistory(encounter.patientId)
  correlations = []
  FOR EACH symptom IN normalizedSymptoms
    FOR EACH condition IN patientHistory.conditions
      IF symptomRelatedToCondition(symptom, condition) THEN
        correlations.push({
          symptom: symptom,
          relatedCondition: condition,
          likelihood: calculateLikelihood(symptom, condition)
        })
      END IF
    END FOR
  END FOR

  // Step 4: Check medication side effects
  medicationEffects = []
  FOR EACH medication IN patientHistory.medications
    sideEffects = getMedicationSideEffects(medication)
    FOR EACH symptom IN normalizedSymptoms
      IF symptom.description IN sideEffects THEN
        medicationEffects.push({
          symptom: symptom,
          medication: medication,
          probability: getSideEffectProbability(medication, symptom)
        })
      END IF
    END FOR
  END FOR

  // Step 5: Generate symptom constellation patterns
  patterns = identifySymptomPatterns(normalizedSymptoms)

  RETURN {
    normalizedSymptoms: normalizedSymptoms,
    redFlags: redFlags,
    priorityLevel: priorityLevel,
    correlations: correlations,
    medicationEffects: medicationEffects,
    patterns: patterns
  }
END FUNCTION

FUNCTION checkRedFlags(symptom)
  redFlagKeywords = [
    'chest pain', 'crushing pain', 'shortness of breath',
    'sudden severe headache', 'worst headache of life',
    'altered mental status', 'confusion', 'seizure',
    'severe abdominal pain', 'severe bleeding',
    'loss of consciousness', 'stroke symptoms',
    'difficulty breathing', 'anaphylaxis'
  ]

  FOR EACH keyword IN redFlagKeywords
    IF symptom.description.contains(keyword) THEN
      RETURN true
    END IF
  END FOR

  // Check severity threshold
  IF symptom.severity >= 8 THEN
    RETURN true
  END IF

  RETURN false
END FUNCTION
```

## 2. Differential Diagnosis Ranking Algorithm

```pseudocode
FUNCTION generateDifferentialDiagnosis(encounter, symptomAnalysis)
  // Step 1: Retrieve candidate diagnoses from knowledge base
  candidateDiagnoses = []
  FOR EACH pattern IN symptomAnalysis.patterns
    matchingDiagnoses = searchDiagnosisDatabase(pattern)
    candidateDiagnoses.extend(matchingDiagnoses)
  END FOR

  // Remove duplicates
  candidateDiagnoses = removeDuplicates(candidateDiagnoses)

  // Step 2: Score each diagnosis using Bayesian inference
  scoredDiagnoses = []
  FOR EACH diagnosis IN candidateDiagnoses
    score = calculateDiagnosisScore(diagnosis, symptomAnalysis, encounter)
    scoredDiagnoses.push({
      diagnosis: diagnosis,
      score: score,
      confidence: calculateConfidence(score, symptomAnalysis),
      supportingEvidence: getSupportingEvidence(diagnosis, symptomAnalysis),
      refutingEvidence: getRefutingEvidence(diagnosis, symptomAnalysis)
    })
  END FOR

  // Step 3: Rank by score (highest first)
  rankedDiagnoses = sortDescending(scoredDiagnoses, 'score')

  // Step 4: Apply clinical filters
  filteredDiagnoses = []
  FOR EACH diagnosis IN rankedDiagnoses
    // Filter out diagnoses incompatible with patient demographics
    IF compatibleWithPatient(diagnosis, encounter.patient) THEN
      // Add prevalence weighting
      diagnosis.prevalenceAdjustedScore =
        diagnosis.score * getDiseasePrevalence(diagnosis, encounter.patient)
      filteredDiagnoses.push(diagnosis)
    END IF
  END FOR

  // Step 5: Re-rank after prevalence adjustment
  finalRanking = sortDescending(filteredDiagnoses, 'prevalenceAdjustedScore')

  // Step 6: Select top N diagnoses (typically 5-10)
  topDiagnoses = finalRanking.slice(0, 10)

  // Step 7: Generate recommended workup for each
  FOR EACH diagnosis IN topDiagnoses
    diagnosis.recommendedTests = generateDiagnosticWorkup(diagnosis, encounter)
    diagnosis.reasoning = generateExplanation(diagnosis, symptomAnalysis)
  END FOR

  RETURN {
    differentialDiagnoses: topDiagnoses,
    totalCandidatesEvaluated: candidateDiagnoses.length,
    analysisTimestamp: getCurrentTime(),
    confidence: calculateOverallConfidence(topDiagnoses)
  }
END FUNCTION

FUNCTION calculateDiagnosisScore(diagnosis, symptomAnalysis, encounter)
  score = 0.0
  weights = {
    symptomMatch: 0.4,
    patternMatch: 0.3,
    historyMatch: 0.2,
    demographicMatch: 0.1
  }

  // Symptom matching (0-100 points)
  requiredSymptoms = diagnosis.typicalSymptoms
  presentSymptoms = symptomAnalysis.normalizedSymptoms
  symptomMatchScore = 0
  FOR EACH required IN requiredSymptoms
    FOR EACH present IN presentSymptoms
      IF semanticSimilarity(required, present) > 0.8 THEN
        symptomMatchScore += (100 / requiredSymptoms.length)
        BREAK
      END IF
    END FOR
  END FOR
  score += symptomMatchScore * weights.symptomMatch

  // Pattern matching (0-100 points)
  patternMatchScore = 0
  FOR EACH pattern IN symptomAnalysis.patterns
    IF pattern IN diagnosis.classicPresentations THEN
      patternMatchScore = 100
      BREAK
    END IF
  END FOR
  score += patternMatchScore * weights.patternMatch

  // Medical history matching (0-100 points)
  historyMatchScore = 0
  patientHistory = getPatientHistory(encounter.patientId)
  FOR EACH riskFactor IN diagnosis.riskFactors
    IF riskFactor IN patientHistory.conditions OR
       riskFactor IN patientHistory.familyHistory THEN
      historyMatchScore += (100 / diagnosis.riskFactors.length)
    END IF
  END FOR
  score += historyMatchScore * weights.historyMatch

  // Demographic matching (0-100 points)
  demographicScore = calculateDemographicFit(diagnosis, encounter.patient)
  score += demographicScore * weights.demographicMatch

  // Apply modifiers
  IF symptomAnalysis.redFlags.length > 0 AND diagnosis.emergent THEN
    score *= 1.2  // Boost emergent diagnoses when red flags present
  END IF

  RETURN min(score, 100.0)  // Cap at 100
END FUNCTION
```

## 3. Treatment Recommendation Logic

```pseudocode
FUNCTION recommendTreatment(confirmedDiagnosis, encounter)
  patient = getPatient(encounter.patientId)

  // Step 1: Retrieve evidence-based treatment guidelines
  guidelines = getClinicalGuidelines(confirmedDiagnosis.icd10Code)
  treatmentOptions = guidelines.recommendedTreatments

  // Step 2: Filter treatments based on patient contraindications
  safeOptions = []
  FOR EACH treatment IN treatmentOptions
    contraindications = checkContraindications(treatment, patient)
    IF contraindications.length == 0 THEN
      safeOptions.push(treatment)
    ELSE
      // Log excluded treatment with reason
      logExcludedTreatment(treatment, contraindications)
    END IF
  END FOR

  // Step 3: Check drug-drug interactions
  FOR EACH treatment IN safeOptions
    IF treatment.type == 'medication' THEN
      interactions = checkDrugInteractions(
        treatment.medication,
        patient.medications
      )
      treatment.interactions = interactions
      treatment.interactionSeverity = max(interactions.map(i => i.severity))
    END IF
  END FOR

  // Step 4: Check drug-allergy interactions
  FOR EACH treatment IN safeOptions
    IF treatment.type == 'medication' THEN
      allergyCheck = checkAllergies(treatment.medication, patient.allergies)
      IF allergyCheck.hasAllergy THEN
        treatment.allergyWarning = allergyCheck
        treatment.requiresAllergyOverride = true
      END IF
    END IF
  END FOR

  // Step 5: Personalize dosing
  FOR EACH treatment IN safeOptions
    IF treatment.type == 'medication' THEN
      treatment.personalizedDose = calculateDose(
        treatment.medication,
        patient.weight,
        patient.age,
        patient.renalFunction,
        patient.hepaticFunction
      )
    END IF
  END FOR

  // Step 6: Calculate risk score for each treatment
  rankedTreatments = []
  FOR EACH treatment IN safeOptions
    riskScore = calculateTreatmentRiskScore(treatment, patient, encounter)
    treatment.riskScore = riskScore
    treatment.requiresApproval = determineOversightRequirement(
      treatment,
      riskScore,
      patient
    )
    rankedTreatments.push(treatment)
  END FOR

  // Step 7: Rank by efficacy and safety balance
  sortedTreatments = sortByEfficacySafety(rankedTreatments)

  // Step 8: Generate monitoring plan
  FOR EACH treatment IN sortedTreatments
    treatment.monitoringPlan = generateMonitoringPlan(treatment, patient)
  END FOR

  // Step 9: Add clinical context and reasoning
  FOR EACH treatment IN sortedTreatments
    treatment.reasoning = generateTreatmentReasoning(
      treatment,
      confirmedDiagnosis,
      guidelines
    )
    treatment.clinicalGuidelines = guidelines.references
  END FOR

  RETURN {
    recommendedTreatments: sortedTreatments,
    guidelineSource: guidelines.source,
    analysisDate: getCurrentTime()
  }
END FUNCTION

FUNCTION checkDrugInteractions(newMedication, currentMedications)
  interactions = []
  interactionDB = getDrugInteractionDatabase()

  FOR EACH currentMed IN currentMedications
    interaction = interactionDB.query(newMedication, currentMed)
    IF interaction EXISTS THEN
      interactions.push({
        medication1: newMedication,
        medication2: currentMed,
        severity: interaction.severity,  // major, moderate, minor
        mechanism: interaction.mechanism,
        clinicalEffects: interaction.effects,
        recommendations: interaction.management
      })
    END IF
  END FOR

  RETURN interactions
END FUNCTION
```

## 4. Risk Assessment Scoring

```pseudocode
FUNCTION calculateTreatmentRiskScore(treatment, patient, encounter)
  riskScore = 0  // 0-100 scale

  // Factor 1: Medication-specific risk (0-30 points)
  IF treatment.type == 'medication' THEN
    med = treatment.medication

    // Controlled substances
    IF med.controlledSubstance THEN
      riskScore += 15
    END IF

    // High-risk medication classes
    highRiskClasses = ['anticoagulant', 'chemotherapy', 'immunosuppressant', 'insulin']
    IF med.class IN highRiskClasses THEN
      riskScore += 10
    END IF

    // Narrow therapeutic index
    IF med.narrowTherapeuticIndex THEN
      riskScore += 5
    END IF
  END IF

  // Factor 2: Patient-specific risk (0-30 points)
  // Age extremes
  IF patient.age < 2 OR patient.age > 75 THEN
    riskScore += 8
  END IF

  // Organ dysfunction
  IF patient.renalFunction < 60 THEN  // GFR < 60 mL/min
    riskScore += 7
  END IF
  IF patient.hepaticFunction == 'impaired' THEN
    riskScore += 7
  END IF

  // Polypharmacy
  IF patient.medications.length > 10 THEN
    riskScore += 8
  END IF

  // Factor 3: Interaction risk (0-20 points)
  IF treatment.interactions.length > 0 THEN
    maxSeverity = max(treatment.interactions.map(i => i.severity))
    SWITCH maxSeverity
      CASE 'major':
        riskScore += 20
      CASE 'moderate':
        riskScore += 10
      CASE 'minor':
        riskScore += 3
    END SWITCH
  END IF

  // Factor 4: Allergy risk (0-20 points)
  IF treatment.allergyWarning EXISTS THEN
    SWITCH treatment.allergyWarning.severity
      CASE 'anaphylaxis':
        riskScore += 20
      CASE 'severe':
        riskScore += 15
      CASE 'moderate':
        riskScore += 8
    END SWITCH
  END IF

  // Apply contextual modifiers
  IF encounter.type == 'emergency' THEN
    riskScore *= 0.9  // Slightly reduce threshold for emergencies
  END IF

  IF patient.pregnancy == true THEN
    riskScore += 10  // Increase caution for pregnant patients
  END IF

  RETURN min(riskScore, 100)  // Cap at 100
END FUNCTION

FUNCTION determineOversightRequirement(treatment, riskScore, patient)
  requiresApproval = false
  approvalType = null
  oversightReason = []

  // Rule 1: Risk score threshold
  IF riskScore >= 70 THEN
    requiresApproval = true
    approvalType = 'physician'
    oversightReason.push('high_risk')
  END IF

  // Rule 2: Controlled substances
  IF treatment.medication.controlledSubstance THEN
    requiresApproval = true
    approvalType = 'multi-level'  // Physician + Pharmacist
    oversightReason.push('controlled_substance')
  END IF

  // Rule 3: Off-label use
  IF treatment.offLabel == true THEN
    requiresApproval = true
    approvalType = 'specialist'
    oversightReason.push('off_label')
  END IF

  // Rule 4: Cost threshold
  IF treatment.estimatedCost > 10000 THEN
    requiresApproval = true
    approvalType = 'multi-level'  // Clinical + Administrative
    oversightReason.push('cost_threshold')
  END IF

  // Rule 5: Allergy override
  IF treatment.requiresAllergyOverride THEN
    requiresApproval = true
    approvalType = 'specialist'
    oversightReason.push('allergy_override')
  END IF

  // Rule 6: Age-based policies
  IF patient.age < 2 THEN
    requiresApproval = true
    approvalType = 'specialist'
    oversightReason.push('pediatric_patient')
  END IF

  IF patient.age > 75 AND treatment.medication.class IN beers_criteria THEN
    requiresApproval = true
    approvalType = 'physician'
    oversightReason.push('geriatric_caution')
  END IF

  // Rule 7: REMS medications
  IF treatment.medication.rems == true THEN
    requiresApproval = true
    approvalType = 'multi-level'
    oversightReason.push('policy_requirement')
  END IF

  RETURN {
    requiresApproval: requiresApproval,
    approvalType: approvalType,
    oversightReason: oversightReason
  }
END FUNCTION
```

## 5. Approval Workflow State Machine

```pseudocode
FUNCTION initiateApprovalWorkflow(treatment, encounter)
  // Create approval request
  approvalRequest = {
    id: generateUUID(),
    encounterId: encounter.id,
    treatmentId: treatment.id,
    requestedBy: getCurrentUser().id,
    requestedAt: getCurrentTime(),
    status: 'pending',
    priority: determinePriority(treatment, encounter),
    approvalType: treatment.approvalType,
    oversightReason: treatment.oversightReason,
    clinicalJustification: generateJustification(treatment, encounter)
  }

  // Determine required approvers
  requiredApprovers = getRequiredApprovers(
    treatment.approvalType,
    encounter.facilityId
  )
  approvalRequest.requiredApprovers = requiredApprovers

  // Initialize approval chain
  approvalRequest.approvers = []
  approvalRequest.approvalChain = buildApprovalChain(
    requiredApprovers,
    treatment.approvalType
  )

  // Save to database
  saveApprovalRequest(approvalRequest)

  // Notify approvers
  FOR EACH approver IN requiredApprovers
    notifyApprover(approver, approvalRequest)
  END FOR

  // Create audit entry
  createAuditEntry({
    action: 'approval_requested',
    resourceType: 'approval',
    resourceId: approvalRequest.id,
    userId: getCurrentUser().id,
    details: approvalRequest
  })

  RETURN approvalRequest
END FUNCTION

FUNCTION processApprovalDecision(approvalRequestId, decision, justification)
  approvalRequest = getApprovalRequest(approvalRequestId)
  currentUser = getCurrentUser()

  // Validate user is authorized approver
  IF currentUser.id NOT IN approvalRequest.requiredApprovers THEN
    THROW Error('User not authorized to approve this request')
  END IF

  // Validate request is still pending
  IF approvalRequest.status != 'pending' THEN
    THROW Error('Approval request already resolved')
  END IF

  // Record approval decision
  approverRecord = {
    userId: currentUser.id,
    role: currentUser.role,
    decision: decision,  // 'approved' or 'rejected'
    justification: justification,
    timestamp: getCurrentTime()
  }
  approvalRequest.approvers.push(approverRecord)

  // Determine next state based on decision
  IF decision == 'rejected' THEN
    approvalRequest.status = 'rejected'
    approvalRequest.resolvedAt = getCurrentTime()

    // Update treatment status
    treatment = getTreatment(approvalRequest.treatmentId)
    treatment.status = 'rejected'
    saveTreatment(treatment)

    // Notify requester
    notifyRequester(approvalRequest, 'rejected')

  ELSE IF decision == 'approved' THEN
    // Check if all required approvers have approved
    allApproved = checkAllApproved(approvalRequest)

    IF allApproved THEN
      approvalRequest.status = 'approved'
      approvalRequest.resolvedAt = getCurrentTime()
      approvalRequest.timeToResolution =
        approvalRequest.resolvedAt - approvalRequest.requestedAt

      // Update treatment status
      treatment = getTreatment(approvalRequest.treatmentId)
      treatment.status = 'approved'
      treatment.approvedBy = currentUser.id
      treatment.approvedAt = getCurrentTime()
      saveTreatment(treatment)

      // Notify requester
      notifyRequester(approvalRequest, 'approved')

      // Generate administration instructions
      generateAdministrationInstructions(treatment)

    ELSE
      // Still waiting for other approvers
      approvalRequest.status = 'partially_approved'
      notifyRemainingApprovers(approvalRequest)
    END IF
  END IF

  // Save updated approval request
  saveApprovalRequest(approvalRequest)

  // Create audit entry
  createAuditEntry({
    action: decision == 'approved' ? 'approve' : 'reject',
    resourceType: 'approval',
    resourceId: approvalRequest.id,
    userId: currentUser.id,
    reasoning: justification,
    details: approverRecord
  })

  RETURN approvalRequest
END FUNCTION

FUNCTION determinePriority(treatment, encounter)
  // STAT priority
  IF encounter.type == 'emergency' AND treatment.riskScore < 30 THEN
    RETURN 'stat'
  END IF

  IF encounter.symptoms.any(s => s.redFlag == true) THEN
    RETURN 'stat'
  END IF

  // Urgent priority
  IF treatment.riskScore >= 70 AND encounter.type == 'emergency' THEN
    RETURN 'urgent'
  END IF

  IF treatment.medication.controlledSubstance AND
     treatment.medication.schedule IN ['II', 'III'] THEN
    RETURN 'urgent'
  END IF

  // Routine priority
  RETURN 'routine'
END FUNCTION
```

## 6. Audit Trail Generation

```pseudocode
FUNCTION createAuditEntry(auditData)
  entry = {
    id: generateUUID(),
    timestamp: getCurrentTime(),
    userId: auditData.userId,
    userRole: getUserRole(auditData.userId),
    action: auditData.action,
    resourceType: auditData.resourceType,
    resourceId: auditData.resourceId,
    changes: auditData.changes || null,
    ipAddress: getClientIPAddress(),
    userAgent: getClientUserAgent(),
    sessionId: getCurrentSessionId(),
    reasoning: auditData.reasoning || null,
    aiInvolved: auditData.aiInvolved || false,
    complianceFlags: []
  }

  // Tag with compliance frameworks
  IF entry.resourceType == 'patient' THEN
    entry.complianceFlags.push('HIPAA')
  END IF

  IF entry.action IN ['approve', 'reject'] THEN
    entry.complianceFlags.push('FDA_21CFR11')
  END IF

  IF entry.aiInvolved THEN
    entry.complianceFlags.push('AI_DECISION_AUDIT')
  END IF

  // Encrypt sensitive data before storage
  entry.encryptedData = encryptPHI({
    resourceId: entry.resourceId,
    changes: entry.changes
  })

  // Remove plaintext sensitive data
  DELETE entry.resourceId
  DELETE entry.changes

  // Write to immutable audit log
  writeToAuditLog(entry)

  // Trigger real-time compliance monitoring
  checkComplianceRules(entry)

  RETURN entry.id
END FUNCTION

FUNCTION generateAuditReport(filters)
  // Retrieve audit entries matching filters
  entries = queryAuditLog({
    startDate: filters.startDate,
    endDate: filters.endDate,
    userId: filters.userId,
    resourceType: filters.resourceType,
    action: filters.action
  })

  // Decrypt PHI for authorized users
  IF currentUserHasAuditAccess() THEN
    FOR EACH entry IN entries
      entry.decryptedData = decryptPHI(entry.encryptedData)
    END FOR
  END IF

  // Generate statistics
  report = {
    totalEntries: entries.length,
    entriesByAction: groupBy(entries, 'action'),
    entriesByUser: groupBy(entries, 'userId'),
    aiDecisions: entries.filter(e => e.aiInvolved == true).length,
    complianceFlags: aggregateComplianceFlags(entries),
    timeline: generateTimeline(entries),
    entries: entries
  }

  // Create tamper-proof hash of report
  report.integrityHash = generateSHA256Hash(report)
  report.generatedAt = getCurrentTime()
  report.generatedBy = getCurrentUser().id

  RETURN report
END FUNCTION
```

## 7. Supporting Utility Functions

```pseudocode
FUNCTION generateMonitoringPlan(treatment, patient)
  plan = {
    schedule: [],
    parameters: [],
    alerts: []
  }

  IF treatment.type == 'medication' THEN
    med = treatment.medication

    // Anticoagulation monitoring
    IF med.class == 'anticoagulant' THEN
      plan.schedule.push({
        test: 'INR/PT' OR 'Anti-Xa',
        frequency: 'Every 3 days initially, then weekly',
        target: med.therapeuticRange
      })
      plan.alerts.push('Notify if INR > 4.0 or < 1.5')
    END IF

    // Renal function monitoring
    IF med.renalExcretion > 50% THEN
      plan.schedule.push({
        test: 'Serum Creatinine, eGFR',
        frequency: 'Baseline, then every 3-6 months',
        target: 'Stable renal function'
      })
    END IF

    // Hepatic function monitoring
    IF med.hepatotoxicityRisk == 'high' THEN
      plan.schedule.push({
        test: 'AST, ALT, Bilirubin',
        frequency: 'Baseline, 2 weeks, 4 weeks, then quarterly',
        target: 'ALT < 2x upper limit normal'
      })
      plan.alerts.push('Hold medication if ALT > 3x ULN')
    END IF

    // Therapeutic drug monitoring
    IF med.narrowTherapeuticIndex THEN
      plan.schedule.push({
        test: med.name + ' serum level',
        frequency: 'Trough before 4th dose, then monthly',
        target: med.therapeuticRange
      })
    END IF
  END IF

  RETURN plan
END FUNCTION

FUNCTION generateJustification(treatment, encounter)
  justification = []

  // Clinical indication
  justification.push('Clinical Indication: ' + encounter.chiefComplaint)

  // Diagnosis support
  diagnosis = getDiagnosis(encounter, treatment.diagnosisId)
  justification.push('Diagnosis: ' + diagnosis.description +
                     ' (ICD-10: ' + diagnosis.icd10Code + ')')

  // Evidence basis
  justification.push('Evidence: ' + treatment.clinicalGuidelines.join(', '))

  // Risk-benefit analysis
  justification.push('Risk Score: ' + treatment.riskScore + '/100')
  justification.push('Expected Benefit: ' + treatment.expectedEfficacy)

  // Alternatives considered
  IF treatment.alternativesConsidered.length > 0 THEN
    justification.push('Alternatives Considered: ' +
                       treatment.alternativesConsidered.join('; '))
  END IF

  RETURN justification.join('\n')
END FUNCTION
```

## 8. Error Handling & Fallback Strategies

```pseudocode
FUNCTION handleClinicalDecisionError(error, context)
  // Log error with full context
  logError(error, context, 'CLINICAL_DECISION_ERROR')

  // Determine fallback strategy
  SWITCH error.type
    CASE 'DIAGNOSIS_SERVICE_UNAVAILABLE':
      // Fall back to symptom-based triage only
      RETURN {
        fallbackMode: 'symptom_triage',
        message: 'Diagnosis suggestions temporarily unavailable',
        recommendation: 'Proceed with clinical judgment'
      }

    CASE 'DRUG_INTERACTION_DATABASE_OFFLINE':
      // Require manual pharmacist review
      RETURN {
        fallbackMode: 'manual_review_required',
        message: 'Drug interaction checking unavailable',
        recommendation: 'Consult pharmacist before prescribing'
      }

    CASE 'APPROVAL_WORKFLOW_ERROR':
      // Emergency override protocol
      IF context.encounter.type == 'emergency' THEN
        RETURN {
          fallbackMode: 'emergency_override',
          message: 'Emergency override activated',
          requirement: 'Document verbal approval'
        }
      ELSE
        RETURN {
          fallbackMode: 'workflow_retry',
          message: 'Approval workflow error',
          recommendation: 'Retry in 60 seconds or contact IT'
        }
      END IF

    DEFAULT:
      // Generic clinical safety fallback
      RETURN {
        fallbackMode: 'clinical_judgment_only',
        message: 'Decision support temporarily unavailable',
        recommendation: 'Proceed with standard clinical protocols'
      }
  END SWITCH
END FUNCTION
```

---

**Next Phase**: Detailed system architecture and component integration design.
