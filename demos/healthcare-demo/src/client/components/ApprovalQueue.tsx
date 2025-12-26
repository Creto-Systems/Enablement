import React, { useState } from 'react';
import { ApprovalRequest } from '@shared/types';

interface ApprovalQueueProps {
  approvalRequests: ApprovalRequest[];
  onApprovalProcessed: (requestId: string, decision: 'approved' | 'rejected') => void;
}

const ApprovalQueue: React.FC<ApprovalQueueProps> = ({ approvalRequests, onApprovalProcessed }) => {
  const [selectedRequest, setSelectedRequest] = useState<ApprovalRequest | null>(null);
  const [justification, setJustification] = useState('');

  const handleApprove = () => {
    if (!selectedRequest) return;
    if (!justification) {
      alert('Please provide justification for approval');
      return;
    }

    onApprovalProcessed(selectedRequest.id, 'approved');
    setSelectedRequest(null);
    setJustification('');
  };

  const handleReject = () => {
    if (!selectedRequest) return;
    if (!justification) {
      alert('Please provide justification for rejection');
      return;
    }

    onApprovalProcessed(selectedRequest.id, 'rejected');
    setSelectedRequest(null);
    setJustification('');
  };

  const getPriorityClass = (priority: ApprovalRequest['priority']): string => {
    switch (priority) {
      case 'stat': return 'priority-stat';
      case 'urgent': return 'priority-urgent';
      default: return 'priority-routine';
    }
  };

  const getOversightReasonLabel = (reason: string): string => {
    const labels: Record<string, string> = {
      'controlled_substance': 'Controlled Substance',
      'high_risk': 'High Risk Treatment',
      'off_label': 'Off-Label Use',
      'cost_threshold': 'Cost Threshold Exceeded',
      'policy_requirement': 'Policy Requirement',
      'allergy_override': 'Allergy Override',
      'pediatric_patient': 'Pediatric Patient',
      'geriatric_caution': 'Geriatric Caution'
    };
    return labels[reason] || reason;
  };

  return (
    <div className="approval-queue">
      <h2>Approval Queue</h2>

      {approvalRequests.length === 0 ? (
        <div className="empty-queue">
          <p>No pending approvals</p>
        </div>
      ) : (
        <div className="queue-layout">
          <div className="queue-list">
            <h3>Pending Requests ({approvalRequests.length})</h3>
            {approvalRequests.map(request => (
              <div
                key={request.id}
                className={`queue-item ${selectedRequest?.id === request.id ? 'selected' : ''} ${getPriorityClass(request.priority)}`}
                onClick={() => setSelectedRequest(request)}
              >
                <div className="queue-item-header">
                  <span className="priority-badge">{request.priority.toUpperCase()}</span>
                  <span className="time-waiting">
                    {Math.floor((new Date().getTime() - new Date(request.requestedAt).getTime()) / 60000)} min waiting
                  </span>
                </div>
                <div className="queue-item-title">
                  Treatment Approval Request
                </div>
                <div className="queue-item-reasons">
                  {request.oversightReason.map((reason, idx) => (
                    <span key={idx} className="reason-tag">
                      {getOversightReasonLabel(reason)}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>

          <div className="queue-details">
            {selectedRequest ? (
              <>
                <h3>Approval Request Details</h3>

                <div className="detail-section">
                  <h4>Priority</h4>
                  <span className={`priority-badge ${getPriorityClass(selectedRequest.priority)}`}>
                    {selectedRequest.priority.toUpperCase()}
                  </span>
                  {selectedRequest.urgencyReason && (
                    <p className="urgency-reason">{selectedRequest.urgencyReason}</p>
                  )}
                </div>

                <div className="detail-section">
                  <h4>Oversight Reasons</h4>
                  <div className="oversight-reasons">
                    {selectedRequest.oversightReason.map((reason, idx) => (
                      <div key={idx} className="oversight-reason-item">
                        <span className="reason-badge">{getOversightReasonLabel(reason)}</span>
                      </div>
                    ))}
                  </div>
                </div>

                <div className="detail-section">
                  <h4>Clinical Justification</h4>
                  <div className="justification-box">
                    {selectedRequest.clinicalJustification}
                  </div>
                </div>

                <div className="detail-section">
                  <h4>Patient Risk Factors</h4>
                  <ul className="risk-factors">
                    {selectedRequest.patientRiskFactors.map((factor, idx) => (
                      <li key={idx}>⚠️ {factor}</li>
                    ))}
                  </ul>
                </div>

                <div className="detail-section">
                  <h4>Required Approvers</h4>
                  <div className="approvers-list">
                    {selectedRequest.requiredApprovers.map((approver, idx) => (
                      <span key={idx} className="approver-badge">
                        {approver.replace('_', ' ').toUpperCase()}
                      </span>
                    ))}
                  </div>
                </div>

                {selectedRequest.approvers.length > 0 && (
                  <div className="detail-section">
                    <h4>Approval History</h4>
                    <div className="approval-history">
                      {selectedRequest.approvers.map((approver, idx) => (
                        <div key={idx} className="approval-entry">
                          <span className={`decision-badge ${approver.decision}`}>
                            {approver.decision.toUpperCase()}
                          </span>
                          <span>{approver.role}</span>
                          <span>{new Date(approver.timestamp).toLocaleString()}</span>
                          <p className="approver-justification">{approver.justification}</p>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                <div className="detail-section">
                  <h4>Your Decision</h4>
                  <textarea
                    className="justification-input"
                    placeholder="Enter justification for your decision (required)"
                    value={justification}
                    onChange={(e) => setJustification(e.target.value)}
                    rows={4}
                  />
                </div>

                <div className="approval-actions">
                  <button
                    onClick={handleApprove}
                    className="btn-approve"
                    disabled={!justification}
                  >
                    ✓ Approve Treatment
                  </button>
                  <button
                    onClick={handleReject}
                    className="btn-reject"
                    disabled={!justification}
                  >
                    ✗ Reject Treatment
                  </button>
                </div>
              </>
            ) : (
              <div className="no-selection">
                <p>Select a request from the queue to review</p>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};

export default ApprovalQueue;
