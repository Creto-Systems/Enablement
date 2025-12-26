import React, { useState, useEffect } from 'react';
import { Patient } from '@shared/types';

interface PatientCardProps {
  patient: Patient | null;
  onSelect: (patient: Patient) => void;
}

const PatientCard: React.FC<PatientCardProps> = ({ patient, onSelect }) => {
  const [patients, setPatients] = useState<Patient[]>([]);
  const [searchTerm, setSearchTerm] = useState('');

  useEffect(() => {
    // In production, fetch from API
    // For demo, load from demo data
    loadDemoPatients();
  }, []);

  const loadDemoPatients = async () => {
    // Simulate API call
    const demoPatients: Patient[] = [
      {
        id: '1',
        mrn: 'MRN001234',
        demographics: {
          firstName: 'John',
          lastName: 'Smith',
          dateOfBirth: new Date('1970-05-15'),
          gender: 'male',
          contactInfo: {
            phone: '555-0123',
            email: 'john.smith@example.com',
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
              description: 'Essential Hypertension',
              diagnosedDate: new Date('2015-03-20'),
              status: 'chronic'
            },
            {
              id: 'c2',
              icd10Code: 'E11.9',
              description: 'Type 2 Diabetes Mellitus',
              diagnosedDate: new Date('2018-07-10'),
              status: 'chronic'
            }
          ],
          medications: [
            {
              id: 'm1',
              name: 'Lisinopril',
              genericName: 'lisinopril',
              rxNormCode: '29046',
              dose: '10 mg',
              route: 'PO',
              frequency: 'Once daily',
              startDate: new Date('2015-03-20'),
              prescribedBy: 'Dr. Johnson',
              controlledSubstance: false
            },
            {
              id: 'm2',
              name: 'Metformin',
              genericName: 'metformin',
              rxNormCode: '6809',
              dose: '500 mg',
              route: 'PO',
              frequency: 'Twice daily',
              startDate: new Date('2018-07-10'),
              prescribedBy: 'Dr. Johnson',
              controlledSubstance: false
            }
          ],
          allergies: [
            {
              id: 'a1',
              allergen: 'Penicillin',
              reaction: 'Rash, itching',
              severity: 'moderate',
              verifiedDate: new Date('1975-01-01')
            }
          ]
        },
        createdAt: new Date('2015-01-01'),
        updatedAt: new Date()
      }
    ];

    setPatients(demoPatients);
    if (demoPatients.length > 0) {
      onSelect(demoPatients[0]);
    }
  };

  const filteredPatients = patients.filter(p =>
    `${p.demographics.firstName} ${p.demographics.lastName} ${p.mrn}`
      .toLowerCase()
      .includes(searchTerm.toLowerCase())
  );

  const calculateAge = (dob: Date): number => {
    const today = new Date();
    const birthDate = new Date(dob);
    let age = today.getFullYear() - birthDate.getFullYear();
    const monthDiff = today.getMonth() - birthDate.getMonth();
    if (monthDiff < 0 || (monthDiff === 0 && today.getDate() < birthDate.getDate())) {
      age--;
    }
    return age;
  };

  return (
    <div className="patient-card">
      <div className="patient-search">
        <input
          type="text"
          placeholder="Search patients..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
        />
      </div>

      <div className="patient-list">
        {filteredPatients.map(p => (
          <div
            key={p.id}
            className={`patient-item ${patient?.id === p.id ? 'selected' : ''}`}
            onClick={() => onSelect(p)}
          >
            <div className="patient-name">
              {p.demographics.firstName} {p.demographics.lastName}
            </div>
            <div className="patient-details">
              <span>MRN: {p.mrn}</span>
              <span>Age: {calculateAge(p.demographics.dateOfBirth)}</span>
              <span>{p.demographics.gender}</span>
            </div>
          </div>
        ))}
      </div>

      {patient && (
        <div className="patient-info">
          <h3>Patient Information</h3>
          <div className="info-grid">
            <div className="info-item">
              <label>Name:</label>
              <span>{patient.demographics.firstName} {patient.demographics.lastName}</span>
            </div>
            <div className="info-item">
              <label>MRN:</label>
              <span>{patient.mrn}</span>
            </div>
            <div className="info-item">
              <label>DOB:</label>
              <span>{new Date(patient.demographics.dateOfBirth).toLocaleDateString()}</span>
            </div>
            <div className="info-item">
              <label>Gender:</label>
              <span>{patient.demographics.gender}</span>
            </div>
          </div>

          <h4>Active Conditions</h4>
          <ul className="conditions-list">
            {patient.medicalHistory.conditions.map(condition => (
              <li key={condition.id}>
                {condition.description} ({condition.icd10Code})
              </li>
            ))}
          </ul>

          <h4>Current Medications</h4>
          <ul className="medications-list">
            {patient.medicalHistory.medications.map(med => (
              <li key={med.id}>
                {med.name} {med.dose} {med.frequency}
              </li>
            ))}
          </ul>

          <h4 className="allergies-header">⚠️ Allergies</h4>
          <ul className="allergies-list">
            {patient.medicalHistory.allergies.map(allergy => (
              <li key={allergy.id} className={`allergy-${allergy.severity}`}>
                <strong>{allergy.allergen}</strong>: {allergy.reaction}
                <span className="severity-badge">{allergy.severity}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
};

export default PatientCard;
