import { Diagnosis, Symptom, SymptomAnalysis, Patient } from '@shared/types';
import { v4 as uuid } from 'uuid';

interface DiagnosisInput {
  symptomAnalysis: SymptomAnalysis;
  patientContext: Patient;
  clinicalGuidelines: any;
}

interface DiagnosisCandidate {
  icd10Code: string;
  description: string;
  typicalSymptoms: string[];
  riskFactors: string[];
  classicPresentations: string[];
  emergent: boolean;
  prevalence: number;
}

export class DiagnosisService {
  private knowledgeBase: Map<string, DiagnosisCandidate>;

  constructor() {
    this.knowledgeBase = this.initializeKnowledgeBase();
  }

  async generateDifferential(input: DiagnosisInput): Promise<{
    diagnoses: Diagnosis[];
    totalCandidatesEvaluated: number;
    analysisTimestamp: Date;
    confidence: number;
  }> {
    const { symptomAnalysis, patientContext } = input;

    // Step 1: Retrieve candidate diagnoses
    const candidateDiagnoses = this.retrieveCandidates(symptomAnalysis);

    // Step 2: Score each diagnosis
    const scoredDiagnoses = candidateDiagnoses.map(candidate => {
      const score = this.calculateDiagnosisScore(
        candidate,
        symptomAnalysis,
        patientContext
      );

      return {
        candidate,
        score,
        confidence: this.calculateConfidence(score, symptomAnalysis),
        supportingEvidence: this.getSupportingEvidence(candidate, symptomAnalysis),
        refutingEvidence: this.getRefutingEvidence(candidate, symptomAnalysis)
      };
    });

    // Step 3: Filter and rank
    const filteredDiagnoses = scoredDiagnoses
      .filter(d => this.isCompatibleWithPatient(d.candidate, patientContext))
      .map(d => ({
        ...d,
        prevalenceAdjustedScore: d.score * this.getDiseasePrevalence(d.candidate, patientContext)
      }))
      .sort((a, b) => b.prevalenceAdjustedScore - a.prevalenceAdjustedScore)
      .slice(0, 10);

    // Step 4: Convert to Diagnosis objects
    const diagnoses: Diagnosis[] = filteredDiagnoses.map(d => ({
      id: uuid(),
      encounterId: '', // Will be set by caller
      icd10Code: d.candidate.icd10Code,
      description: d.candidate.description,
      type: 'differential',
      probability: Math.round(d.prevalenceAdjustedScore),
      confidenceScore: Math.round(d.confidence * 100),
      evidenceBasis: d.supportingEvidence,
      suggestedBy: 'ai',
      suggestedAt: new Date(),
      reasoning: this.generateExplanation(d.candidate, symptomAnalysis),
      clinicalGuidelines: this.getClinicalGuidelines(d.candidate),
      recommendedTests: this.generateDiagnosticWorkup(d.candidate),
      status: 'suggested'
    }));

    return {
      diagnoses,
      totalCandidatesEvaluated: candidateDiagnoses.length,
      analysisTimestamp: new Date(),
      confidence: this.calculateOverallConfidence(diagnoses)
    };
  }

  private retrieveCandidates(symptomAnalysis: SymptomAnalysis): DiagnosisCandidate[] {
    const candidates: DiagnosisCandidate[] = [];

    // Pattern matching against knowledge base
    for (const pattern of symptomAnalysis.patterns) {
      for (const [code, candidate] of this.knowledgeBase.entries()) {
        if (candidate.classicPresentations.includes(pattern)) {
          candidates.push(candidate);
        }
      }
    }

    // Symptom keyword matching
    for (const symptom of symptomAnalysis.normalizedSymptoms) {
      for (const [code, candidate] of this.knowledgeBase.entries()) {
        const hasMatch = candidate.typicalSymptoms.some(typical =>
          symptom.description.toLowerCase().includes(typical.toLowerCase())
        );
        if (hasMatch && !candidates.includes(candidate)) {
          candidates.push(candidate);
        }
      }
    }

    return candidates;
  }

  private calculateDiagnosisScore(
    diagnosis: DiagnosisCandidate,
    symptomAnalysis: SymptomAnalysis,
    patient: Patient
  ): number {
    let score = 0;
    const weights = {
      symptomMatch: 0.4,
      patternMatch: 0.3,
      historyMatch: 0.2,
      demographicMatch: 0.1
    };

    // Symptom matching (0-100 points)
    const symptomMatchScore = this.calculateSymptomMatch(
      diagnosis.typicalSymptoms,
      symptomAnalysis.normalizedSymptoms
    );
    score += symptomMatchScore * weights.symptomMatch;

    // Pattern matching (0-100 points)
    const patternMatchScore = this.calculatePatternMatch(
      diagnosis.classicPresentations,
      symptomAnalysis.patterns
    );
    score += patternMatchScore * weights.patternMatch;

    // Medical history matching (0-100 points)
    const historyMatchScore = this.calculateHistoryMatch(
      diagnosis.riskFactors,
      patient.medicalHistory
    );
    score += historyMatchScore * weights.historyMatch;

    // Demographic matching (0-100 points)
    const demographicScore = this.calculateDemographicFit(diagnosis, patient);
    score += demographicScore * weights.demographicMatch;

    // Apply modifiers
    if (symptomAnalysis.redFlags.length > 0 && diagnosis.emergent) {
      score *= 1.2; // Boost emergent diagnoses when red flags present
    }

    return Math.min(score, 100);
  }

  private calculateSymptomMatch(
    typicalSymptoms: string[],
    presentSymptoms: Symptom[]
  ): number {
    if (typicalSymptoms.length === 0) return 0;

    let matches = 0;
    for (const typical of typicalSymptoms) {
      const hasMatch = presentSymptoms.some(present =>
        present.description.toLowerCase().includes(typical.toLowerCase())
      );
      if (hasMatch) matches++;
    }

    return (matches / typicalSymptoms.length) * 100;
  }

  private calculatePatternMatch(
    classicPresentations: string[],
    patterns: string[]
  ): number {
    for (const pattern of patterns) {
      if (classicPresentations.includes(pattern)) {
        return 100;
      }
    }
    return 0;
  }

  private calculateHistoryMatch(
    riskFactors: string[],
    medicalHistory: Patient['medicalHistory']
  ): number {
    if (riskFactors.length === 0) return 50; // Neutral score

    let matches = 0;
    for (const riskFactor of riskFactors) {
      const hasCondition = medicalHistory.conditions.some(c =>
        c.description.toLowerCase().includes(riskFactor.toLowerCase())
      );
      if (hasCondition) matches++;
    }

    return (matches / riskFactors.length) * 100;
  }

  private calculateDemographicFit(
    diagnosis: DiagnosisCandidate,
    patient: Patient
  ): number {
    // Simplified demographic scoring
    // In production, this would use epidemiological data
    return 50; // Neutral score
  }

  private getDiseasePrevalence(
    diagnosis: DiagnosisCandidate,
    patient: Patient
  ): number {
    return diagnosis.prevalence;
  }

  private isCompatibleWithPatient(
    diagnosis: DiagnosisCandidate,
    patient: Patient
  ): boolean {
    // Check for basic compatibility
    // In production, would check age/gender restrictions
    return true;
  }

  private calculateConfidence(
    score: number,
    symptomAnalysis: SymptomAnalysis
  ): number {
    let confidence = score / 100;

    // Reduce confidence if symptoms are vague
    if (symptomAnalysis.normalizedSymptoms.length < 2) {
      confidence *= 0.7;
    }

    // Increase confidence if red flags match emergent diagnosis
    if (symptomAnalysis.redFlags.length > 0) {
      confidence *= 1.1;
    }

    return Math.min(confidence, 1.0);
  }

  private getSupportingEvidence(
    diagnosis: DiagnosisCandidate,
    symptomAnalysis: SymptomAnalysis
  ): string[] {
    const evidence: string[] = [];

    for (const symptom of symptomAnalysis.normalizedSymptoms) {
      for (const typical of diagnosis.typicalSymptoms) {
        if (symptom.description.toLowerCase().includes(typical.toLowerCase())) {
          evidence.push(`Patient presents with ${symptom.description}`);
        }
      }
    }

    return evidence;
  }

  private getRefutingEvidence(
    diagnosis: DiagnosisCandidate,
    symptomAnalysis: SymptomAnalysis
  ): string[] {
    // Simplified - would check for contradictory symptoms
    return [];
  }

  private generateExplanation(
    diagnosis: DiagnosisCandidate,
    symptomAnalysis: SymptomAnalysis
  ): string {
    const supportingSymptoms = diagnosis.typicalSymptoms
      .filter(typical =>
        symptomAnalysis.normalizedSymptoms.some(s =>
          s.description.toLowerCase().includes(typical.toLowerCase())
        )
      )
      .slice(0, 3);

    return `Diagnosis suggested based on presenting symptoms: ${supportingSymptoms.join(', ')}. ${
      diagnosis.emergent ? 'This is a potentially emergent condition requiring immediate evaluation.' : ''
    }`;
  }

  private getClinicalGuidelines(diagnosis: DiagnosisCandidate): string[] {
    return [
      'https://www.uptodate.com',
      'https://www.dynamed.com'
    ];
  }

  private generateDiagnosticWorkup(diagnosis: DiagnosisCandidate): string[] {
    // Simplified test recommendations
    const tests: string[] = ['Complete Blood Count', 'Comprehensive Metabolic Panel'];

    if (diagnosis.emergent) {
      tests.push('Troponin', 'ECG', 'Chest X-Ray');
    }

    return tests;
  }

  private calculateOverallConfidence(diagnoses: Diagnosis[]): number {
    if (diagnoses.length === 0) return 0;

    const topConfidence = diagnoses[0].confidenceScore || 0;
    return topConfidence / 100;
  }

  private initializeKnowledgeBase(): Map<string, DiagnosisCandidate> {
    const kb = new Map<string, DiagnosisCandidate>();

    // Acute Coronary Syndrome
    kb.set('I24.9', {
      icd10Code: 'I24.9',
      description: 'Acute Coronary Syndrome',
      typicalSymptoms: ['chest pain', 'crushing pain', 'shortness of breath', 'diaphoresis'],
      riskFactors: ['hypertension', 'diabetes', 'hyperlipidemia', 'smoking'],
      classicPresentations: ['cardiac_chest_pain'],
      emergent: true,
      prevalence: 0.15
    });

    // Pneumonia
    kb.set('J18.9', {
      icd10Code: 'J18.9',
      description: 'Pneumonia, unspecified organism',
      typicalSymptoms: ['cough', 'fever', 'shortness of breath', 'chest pain'],
      riskFactors: ['copd', 'immunosuppression', 'age > 65'],
      classicPresentations: ['respiratory_infection'],
      emergent: false,
      prevalence: 0.25
    });

    // Migraine
    kb.set('G43.909', {
      icd10Code: 'G43.909',
      description: 'Migraine, unspecified, not intractable',
      typicalSymptoms: ['headache', 'photophobia', 'nausea', 'vomiting'],
      riskFactors: ['family history', 'female gender'],
      classicPresentations: ['migraine_headache'],
      emergent: false,
      prevalence: 0.35
    });

    // Sepsis
    kb.set('A41.9', {
      icd10Code: 'A41.9',
      description: 'Sepsis, unspecified organism',
      typicalSymptoms: ['fever', 'altered mental status', 'hypotension', 'tachycardia'],
      riskFactors: ['immunosuppression', 'recent surgery', 'indwelling catheter'],
      classicPresentations: ['septic_presentation'],
      emergent: true,
      prevalence: 0.10
    });

    // Type 2 Diabetes
    kb.set('E11.9', {
      icd10Code: 'E11.9',
      description: 'Type 2 Diabetes Mellitus',
      typicalSymptoms: ['polyuria', 'polydipsia', 'fatigue', 'blurred vision'],
      riskFactors: ['obesity', 'family history', 'sedentary lifestyle'],
      classicPresentations: ['hyperglycemic_symptoms'],
      emergent: false,
      prevalence: 0.30
    });

    return kb;
  }
}
