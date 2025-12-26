import {
  Treatment,
  Diagnosis,
  Patient,
  DrugInteraction,
  MedicationDetails
} from '@shared/types';
import { v4 as uuid } from 'uuid';

interface TreatmentGuideline {
  icd10Code: string;
  recommendedTreatments: TreatmentOption[];
  source: string;
  references: string[];
}

interface TreatmentOption {
  type: 'medication' | 'procedure' | 'therapy' | 'lifestyle';
  medication?: MedicationDetails;
  expectedEfficacy: string;
  contraindications: string[];
  monitoring: string[];
}

export class TreatmentService {
  private guidelines: Map<string, TreatmentGuideline>;
  private drugInteractionDB: Map<string, DrugInteraction[]>;

  constructor() {
    this.guidelines = this.initializeGuidelines();
    this.drugInteractionDB = this.initializeDrugInteractions();
  }

  async recommend(
    diagnosis: Diagnosis,
    patient: Patient
  ): Promise<Treatment[]> {
    // Step 1: Retrieve evidence-based guidelines
    const guideline = this.guidelines.get(diagnosis.icd10Code);
    if (!guideline) {
      return [];
    }

    const treatmentOptions = guideline.recommendedTreatments;

    // Step 2: Filter by contraindications
    const safeOptions = treatmentOptions.filter(option => {
      const contraindications = this.checkContraindications(option, patient);
      return contraindications.length === 0;
    });

    // Step 3: Check drug interactions
    const optionsWithInteractions = safeOptions.map(option => {
      if (option.type === 'medication' && option.medication) {
        const interactions = this.checkDrugInteractions(
          option.medication,
          patient.medicalHistory.medications
        );
        return { option, interactions };
      }
      return { option, interactions: [] };
    });

    // Step 4: Check allergies
    const optionsWithAllergyCheck = optionsWithInteractions.map(({ option, interactions }) => {
      let allergyWarning = null;
      if (option.type === 'medication' && option.medication) {
        allergyWarning = this.checkAllergies(option.medication, patient.medicalHistory.allergies);
      }
      return { option, interactions, allergyWarning };
    });

    // Step 5: Calculate risk scores
    const treatments: Treatment[] = optionsWithAllergyCheck.map(
      ({ option, interactions, allergyWarning }) => {
        const treatment: Treatment = {
          id: uuid(),
          encounterId: '', // Will be set by caller
          diagnosisId: diagnosis.id,
          type: option.type,
          riskScore: 0,
          requiresApproval: false,
          status: 'suggested',
          medication: option.medication,
          contraindications: option.contraindications,
          interactions,
          adverseEffects: this.getAdverseEffects(option),
          monitoringRequired: option.monitoring,
          suggestedBy: 'ai',
          suggestedAt: new Date(),
          reasoning: this.generateTreatmentReasoning(option, diagnosis, guideline),
          clinicalGuidelines: guideline.references
        };

        // Calculate risk score
        treatment.riskScore = this.calculateTreatmentRiskScore(treatment, patient);

        // Determine if approval required
        const oversight = this.determineOversightRequirement(treatment, patient);
        treatment.requiresApproval = oversight.requiresApproval;

        return treatment;
      }
    );

    // Step 6: Rank by efficacy and safety
    return this.sortByEfficacySafety(treatments);
  }

  private checkContraindications(option: TreatmentOption, patient: Patient): string[] {
    const contraindications: string[] = [];

    // Check patient conditions against treatment contraindications
    for (const contraindication of option.contraindications) {
      const hasCondition = patient.medicalHistory.conditions.some(c =>
        c.description.toLowerCase().includes(contraindication.toLowerCase())
      );
      if (hasCondition) {
        contraindications.push(contraindication);
      }
    }

    return contraindications;
  }

  private checkDrugInteractions(
    newMedication: MedicationDetails,
    currentMedications: Patient['medicalHistory']['medications']
  ): DrugInteraction[] {
    const interactions: DrugInteraction[] = [];

    for (const currentMed of currentMedications) {
      const key = `${newMedication.name}:${currentMed.name}`;
      const interaction = this.drugInteractionDB.get(key);
      if (interaction) {
        interactions.push(...interaction);
      }
    }

    return interactions;
  }

  private checkAllergies(
    medication: MedicationDetails,
    allergies: Patient['medicalHistory']['allergies']
  ): any {
    for (const allergy of allergies) {
      if (
        allergy.allergen.toLowerCase() === medication.name.toLowerCase() ||
        allergy.allergen.toLowerCase() === medication.genericName.toLowerCase()
      ) {
        return {
          hasAllergy: true,
          allergen: allergy.allergen,
          reaction: allergy.reaction,
          severity: allergy.severity
        };
      }
    }
    return null;
  }

  private calculateTreatmentRiskScore(treatment: Treatment, patient: Patient): number {
    let riskScore = 0;

    // Factor 1: Medication-specific risk (0-30 points)
    if (treatment.type === 'medication' && treatment.medication) {
      const med = treatment.medication;

      if (med.controlledSubstance) {
        riskScore += 15;
      }

      // High-risk medication classes
      const highRiskClasses = ['anticoagulant', 'chemotherapy', 'immunosuppressant', 'insulin'];
      if (highRiskClasses.some(cls => med.name.toLowerCase().includes(cls))) {
        riskScore += 10;
      }
    }

    // Factor 2: Patient age (0-30 points)
    const age = this.calculateAge(patient.demographics.dateOfBirth);
    if (age < 2 || age > 75) {
      riskScore += 15;
    }

    // Factor 3: Interaction risk (0-20 points)
    if (treatment.interactions.length > 0) {
      const maxSeverity = treatment.interactions.reduce((max, interaction) => {
        const severityScore = interaction.severity === 'major' ? 20 : interaction.severity === 'moderate' ? 10 : 3;
        return Math.max(max, severityScore);
      }, 0);
      riskScore += maxSeverity;
    }

    // Factor 4: Polypharmacy (0-20 points)
    if (patient.medicalHistory.medications.length > 10) {
      riskScore += 10;
    }

    return Math.min(riskScore, 100);
  }

  private determineOversightRequirement(
    treatment: Treatment,
    patient: Patient
  ): { requiresApproval: boolean; approvalType: string; oversightReason: string[] } {
    let requiresApproval = false;
    let approvalType = 'physician';
    const oversightReason: string[] = [];

    // Rule 1: Risk score threshold
    if (treatment.riskScore >= 70) {
      requiresApproval = true;
      oversightReason.push('high_risk');
    }

    // Rule 2: Controlled substances
    if (treatment.medication?.controlledSubstance) {
      requiresApproval = true;
      approvalType = 'multi-level';
      oversightReason.push('controlled_substance');
    }

    // Rule 3: Age-based policies
    const age = this.calculateAge(patient.demographics.dateOfBirth);
    if (age < 2) {
      requiresApproval = true;
      approvalType = 'specialist';
      oversightReason.push('pediatric_patient');
    }

    if (age > 75) {
      requiresApproval = true;
      oversightReason.push('geriatric_caution');
    }

    return { requiresApproval, approvalType, oversightReason };
  }

  private getAdverseEffects(option: TreatmentOption): string[] {
    // Simplified - would query drug database
    return ['Nausea', 'Dizziness', 'Headache'];
  }

  private generateTreatmentReasoning(
    option: TreatmentOption,
    diagnosis: Diagnosis,
    guideline: TreatmentGuideline
  ): string {
    return `Evidence-based treatment for ${diagnosis.description}. Expected efficacy: ${option.expectedEfficacy}. Guideline source: ${guideline.source}.`;
  }

  private sortByEfficacySafety(treatments: Treatment[]): Treatment[] {
    return treatments.sort((a, b) => {
      // Prefer lower risk scores (safer treatments)
      return a.riskScore - b.riskScore;
    });
  }

  private calculateAge(dateOfBirth: Date): number {
    const today = new Date();
    const birthDate = new Date(dateOfBirth);
    let age = today.getFullYear() - birthDate.getFullYear();
    const monthDiff = today.getMonth() - birthDate.getMonth();

    if (monthDiff < 0 || (monthDiff === 0 && today.getDate() < birthDate.getDate())) {
      age--;
    }

    return age;
  }

  private initializeGuidelines(): Map<string, TreatmentGuideline> {
    const guidelines = new Map<string, TreatmentGuideline>();

    // Acute Coronary Syndrome
    guidelines.set('I24.9', {
      icd10Code: 'I24.9',
      recommendedTreatments: [
        {
          type: 'medication',
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
          expectedEfficacy: 'Reduces mortality in ACS by 23%',
          contraindications: ['active bleeding', 'severe thrombocytopenia'],
          monitoring: ['Bleeding signs', 'Platelet count']
        },
        {
          type: 'medication',
          medication: {
            name: 'Nitroglycerin',
            genericName: 'nitroglycerin',
            rxNormCode: '7448',
            dose: '0.4 mg',
            route: 'SL',
            frequency: 'PRN chest pain',
            duration: 'As needed',
            controlledSubstance: false
          },
          expectedEfficacy: 'Reduces chest pain and myocardial oxygen demand',
          contraindications: ['hypotension', 'use of PDE5 inhibitors'],
          monitoring: ['Blood pressure', 'Heart rate']
        }
      ],
      source: 'ACC/AHA Guidelines for ACS Management',
      references: ['https://www.acc.org/guidelines']
    });

    // Pneumonia
    guidelines.set('J18.9', {
      icd10Code: 'J18.9',
      recommendedTreatments: [
        {
          type: 'medication',
          medication: {
            name: 'Azithromycin',
            genericName: 'azithromycin',
            rxNormCode: '18631',
            dose: '500 mg',
            route: 'PO',
            frequency: 'Daily',
            duration: '5 days',
            controlledSubstance: false
          },
          expectedEfficacy: '85% clinical cure rate for CAP',
          contraindications: ['QT prolongation', 'macrolide allergy'],
          monitoring: ['Clinical response', 'QTc interval if risk factors']
        }
      ],
      source: 'IDSA/ATS Community-Acquired Pneumonia Guidelines',
      references: ['https://www.idsociety.org/guidelines']
    });

    // Type 2 Diabetes
    guidelines.set('E11.9', {
      icd10Code: 'E11.9',
      recommendedTreatments: [
        {
          type: 'medication',
          medication: {
            name: 'Metformin',
            genericName: 'metformin',
            rxNormCode: '6809',
            dose: '500 mg',
            route: 'PO',
            frequency: 'Twice daily',
            duration: 'Chronic',
            controlledSubstance: false
          },
          expectedEfficacy: 'Reduces HbA1c by 1-2%',
          contraindications: ['eGFR < 30', 'lactic acidosis risk'],
          monitoring: ['Renal function', 'HbA1c', 'Vitamin B12']
        },
        {
          type: 'lifestyle',
          expectedEfficacy: 'Reduces HbA1c by 0.5-2%',
          contraindications: [],
          monitoring: ['Weight', 'Blood glucose', 'Physical activity']
        }
      ],
      source: 'ADA Standards of Medical Care in Diabetes',
      references: ['https://diabetesjournals.org/care/issue/46/Supplement_1']
    });

    return guidelines;
  }

  private initializeDrugInteractions(): Map<string, DrugInteraction[]> {
    const interactions = new Map<string, DrugInteraction[]>();

    // Warfarin + Aspirin
    interactions.set('Warfarin:Aspirin', [{
      medication1: 'Warfarin',
      medication2: 'Aspirin',
      severity: 'major',
      mechanism: 'Additive antiplatelet/anticoagulant effects',
      clinicalEffects: 'Increased bleeding risk',
      recommendations: 'Monitor INR closely, assess bleeding risk, consider gastro-protection'
    }]);

    // Metformin + Contrast
    interactions.set('Metformin:Iodinated Contrast', [{
      medication1: 'Metformin',
      medication2: 'Iodinated Contrast',
      severity: 'major',
      mechanism: 'Risk of contrast-induced nephropathy',
      clinicalEffects: 'Lactic acidosis',
      recommendations: 'Hold metformin 48h before and after contrast, check renal function'
    }]);

    return interactions;
  }
}
