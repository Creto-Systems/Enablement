import React, { useState } from 'react';
import clsx from 'clsx';

export interface PendingApproval {
  id: string;
  agentId: string;
  symbol: string;
  side: 'buy' | 'sell';
  quantity: number;
  price: number;
  estimatedCost: number;
  timestamp: string;
  reasoning: string;
  riskAssessment: {
    score: number;
    level: 'low' | 'medium' | 'high';
    factors: string[];
  };
}

export interface ApprovalCardProps {
  approval: PendingApproval;
  onApprove: (approvalId: string, reason?: string) => void | Promise<void>;
  onReject: (approvalId: string, reason: string) => void | Promise<void>;
  loading?: boolean;
}

export function ApprovalCard({
  approval,
  onApprove,
  onReject,
  loading = false,
}: ApprovalCardProps) {
  const [showApproveDialog, setShowApproveDialog] = useState(false);
  const [showRejectDialog, setShowRejectDialog] = useState(false);
  const [approvalNotes, setApprovalNotes] = useState('');
  const [rejectionReason, setRejectionReason] = useState('');
  const [error, setError] = useState('');

  const handleApproveClick = () => {
    setShowApproveDialog(true);
    setError('');
  };

  const handleRejectClick = () => {
    setShowRejectDialog(true);
    setError('');
  };

  const handleConfirmApprove = async () => {
    await onApprove(approval.id, approvalNotes || undefined);
    setShowApproveDialog(false);
    setApprovalNotes('');
  };

  const handleConfirmReject = async () => {
    if (!rejectionReason.trim()) {
      setError('Reason is required for rejection');
      return;
    }

    await onReject(approval.id, rejectionReason);
    setShowRejectDialog(false);
    setRejectionReason('');
  };

  const handleCancel = () => {
    setShowApproveDialog(false);
    setShowRejectDialog(false);
    setApprovalNotes('');
    setRejectionReason('');
    setError('');
  };

  const getRiskColorClass = () => {
    switch (approval.riskAssessment.level) {
      case 'low':
        return 'bg-green-100 text-green-800';
      case 'medium':
        return 'bg-yellow-100 text-yellow-800';
      case 'high':
        return 'bg-red-100 text-red-800';
    }
  };

  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="bg-white border border-gray-200 rounded-lg p-4 shadow-sm">
      {/* Header */}
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <h3 className="text-lg font-semibold text-gray-900">{approval.symbol}</h3>
          <span
            className={clsx('px-2 py-1 rounded text-xs font-medium', {
              'bg-green-100 text-green-800': approval.side === 'buy',
              'bg-red-100 text-red-800': approval.side === 'sell',
            })}
          >
            {approval.side.toUpperCase()}
          </span>
        </div>
        <span className="text-xs text-gray-500">{formatTimestamp(approval.timestamp)}</span>
      </div>

      {/* Trade Details */}
      <div className="grid grid-cols-3 gap-3 mb-3">
        <div>
          <span className="text-xs text-gray-600 block">Quantity</span>
          <span className="text-sm font-medium">{approval.quantity}</span>
        </div>
        <div>
          <span className="text-xs text-gray-600 block">Price</span>
          <span className="text-sm font-medium">${approval.price.toFixed(2)}</span>
        </div>
        <div>
          <span className="text-xs text-gray-600 block">Total</span>
          <span className="text-sm font-medium">
            ${approval.estimatedCost.toLocaleString(undefined, { minimumFractionDigits: 2 })}
          </span>
        </div>
      </div>

      {/* Reasoning */}
      <div className="mb-3">
        <span className="text-xs text-gray-600 block mb-1">Reasoning</span>
        <p className="text-sm text-gray-700">{approval.reasoning}</p>
      </div>

      {/* Risk Assessment */}
      <div className={clsx('rounded-md p-3 mb-4', getRiskColorClass())}>
        <div className="flex items-center justify-between mb-2">
          <span className="text-xs font-medium">Risk Level: {approval.riskAssessment.level.toUpperCase()}</span>
          <span className="text-xs">Score: {(approval.riskAssessment.score * 100).toFixed(0)}%</span>
        </div>
        <ul className="space-y-1">
          {approval.riskAssessment.factors.map((factor, index) => (
            <li key={index} className="text-xs flex items-start gap-1">
              <span>•</span>
              <span>{factor}</span>
            </li>
          ))}
        </ul>
      </div>

      {/* Action Buttons */}
      <div className="flex gap-2">
        <button
          onClick={handleApproveClick}
          disabled={loading}
          className={clsx(
            'flex-1 px-4 py-2 rounded-md font-medium transition-colors',
            'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500',
            {
              'bg-green-600 text-white hover:bg-green-700': !loading,
              'bg-gray-400 text-gray-200 cursor-not-allowed': loading,
            }
          )}
        >
          Approve
        </button>
        <button
          onClick={handleRejectClick}
          disabled={loading}
          className={clsx(
            'flex-1 px-4 py-2 rounded-md font-medium transition-colors',
            'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500',
            {
              'bg-red-600 text-white hover:bg-red-700': !loading,
              'bg-gray-400 text-gray-200 cursor-not-allowed': loading,
            }
          )}
        >
          Reject
        </button>
      </div>

      {/* Approval Dialog */}
      {showApproveDialog && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 max-w-md w-full">
            <h3 className="text-lg font-semibold mb-4">Approve Trade</h3>
            <textarea
              value={approvalNotes}
              onChange={(e) => setApprovalNotes(e.target.value)}
              placeholder="Optional approval notes..."
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-green-500"
              rows={3}
            />
            <div className="flex gap-2 mt-4">
              <button
                onClick={handleConfirmApprove}
                className="flex-1 px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-green-500"
              >
                Confirm Approval
              </button>
              <button
                onClick={handleCancel}
                className="flex-1 px-4 py-2 bg-gray-200 text-gray-700 rounded-md hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-500"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Rejection Dialog */}
      {showRejectDialog && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 max-w-md w-full">
            <h3 className="text-lg font-semibold mb-4">Reject Trade</h3>
            <textarea
              value={rejectionReason}
              onChange={(e) => setRejectionReason(e.target.value)}
              placeholder="Reason for rejection (required)..."
              className={clsx(
                'w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2',
                {
                  'border-red-500 focus:ring-red-500': error,
                  'border-gray-300 focus:ring-red-500': !error,
                }
              )}
              rows={3}
            />
            {error && <p className="text-sm text-red-600 mt-1">{error}</p>}
            <div className="flex gap-2 mt-4">
              <button
                onClick={handleConfirmReject}
                className="flex-1 px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500"
              >
                Confirm Rejection
              </button>
              <button
                onClick={handleCancel}
                className="flex-1 px-4 py-2 bg-gray-200 text-gray-700 rounded-md hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-500"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
