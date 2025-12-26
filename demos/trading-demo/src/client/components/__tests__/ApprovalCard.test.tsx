import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { axe, toHaveNoViolations } from 'jest-axe';
import { ApprovalCard } from '../ApprovalCard';
import { mockApproval } from '../../test-utils/test-helpers';

expect.extend(toHaveNoViolations);

describe('ApprovalCard', () => {
  const mockOnApprove = vi.fn();
  const mockOnReject = vi.fn();

  beforeEach(() => {
    mockOnApprove.mockClear();
    mockOnReject.mockClear();
  });

  it('displays trade details', () => {
    render(
      <ApprovalCard
        approval={mockApproval}
        onApprove={mockOnApprove}
        onReject={mockOnReject}
      />
    );

    expect(screen.getByText('AAPL')).toBeInTheDocument();
    expect(screen.getByText(/buy/i)).toBeInTheDocument();
    expect(screen.getByText('10')).toBeInTheDocument();
    expect(screen.getByText('$155.00')).toBeInTheDocument();
    expect(screen.getByText('$1,550.00')).toBeInTheDocument();
  });

  it('shows risk assessment', () => {
    render(
      <ApprovalCard
        approval={mockApproval}
        onApprove={mockOnApprove}
        onReject={mockOnReject}
      />
    );

    expect(screen.getByText(/risk level: low/i)).toBeInTheDocument();
    expect(screen.getByText(/position size within limits/i)).toBeInTheDocument();
    expect(screen.getByText(/strong technical indicators/i)).toBeInTheDocument();
    expect(screen.getByText(/diversification maintained/i)).toBeInTheDocument();
  });

  it('calls onApprove with reason', async () => {
    const user = userEvent.setup();
    render(
      <ApprovalCard
        approval={mockApproval}
        onApprove={mockOnApprove}
        onReject={mockOnReject}
      />
    );

    const approveButton = screen.getByRole('button', { name: /approve/i });
    await user.click(approveButton);

    // Should show reason dialog
    const reasonInput = screen.getByPlaceholderText(/optional approval notes/i);
    await user.type(reasonInput, 'Looks good');

    const confirmButton = screen.getByRole('button', { name: /confirm approval/i });
    await user.click(confirmButton);

    await waitFor(() => {
      expect(mockOnApprove).toHaveBeenCalledWith(mockApproval.id, 'Looks good');
    });
  });

  it('calls onReject with reason', async () => {
    const user = userEvent.setup();
    render(
      <ApprovalCard
        approval={mockApproval}
        onApprove={mockOnApprove}
        onReject={mockOnReject}
      />
    );

    const rejectButton = screen.getByRole('button', { name: /reject/i });
    await user.click(rejectButton);

    // Should show reason dialog
    const reasonInput = screen.getByPlaceholderText(/reason for rejection/i);
    await user.type(reasonInput, 'Too risky');

    const confirmButton = screen.getByRole('button', { name: /confirm rejection/i });
    await user.click(confirmButton);

    await waitFor(() => {
      expect(mockOnReject).toHaveBeenCalledWith(mockApproval.id, 'Too risky');
    });
  });

  it('shows loading state during action', async () => {
    const user = userEvent.setup();
    render(
      <ApprovalCard
        approval={mockApproval}
        onApprove={mockOnApprove}
        onReject={mockOnReject}
        loading
      />
    );

    const approveButton = screen.getByRole('button', { name: /approve/i });
    const rejectButton = screen.getByRole('button', { name: /reject/i });

    expect(approveButton).toBeDisabled();
    expect(rejectButton).toBeDisabled();

    await user.click(approveButton);
    expect(mockOnApprove).not.toHaveBeenCalled();
  });

  it('requires rejection reason', async () => {
    const user = userEvent.setup();
    render(
      <ApprovalCard
        approval={mockApproval}
        onApprove={mockOnApprove}
        onReject={mockOnReject}
      />
    );

    const rejectButton = screen.getByRole('button', { name: /reject/i });
    await user.click(rejectButton);

    const confirmButton = screen.getByRole('button', { name: /confirm rejection/i });
    await user.click(confirmButton);

    // Should show error
    expect(await screen.findByText(/reason is required/i)).toBeInTheDocument();
    expect(mockOnReject).not.toHaveBeenCalled();
  });

  it('displays trade reasoning', () => {
    render(
      <ApprovalCard
        approval={mockApproval}
        onApprove={mockOnApprove}
        onReject={mockOnReject}
      />
    );

    expect(screen.getByText(mockApproval.reasoning)).toBeInTheDocument();
  });

  it('shows high risk indicator', () => {
    const highRiskApproval = {
      ...mockApproval,
      riskAssessment: {
        score: 0.8,
        level: 'high' as const,
        factors: ['Large position size', 'High volatility'],
      },
    };

    render(
      <ApprovalCard
        approval={highRiskApproval}
        onApprove={mockOnApprove}
        onReject={mockOnReject}
      />
    );

    expect(screen.getByText(/risk level: high/i)).toBeInTheDocument();
    const riskIndicator = screen.getByText(/risk level/i).closest('div');
    expect(riskIndicator).toHaveClass('bg-red-100');
  });

  it('is keyboard accessible', async () => {
    const { container } = render(
      <ApprovalCard
        approval={mockApproval}
        onApprove={mockOnApprove}
        onReject={mockOnReject}
      />
    );

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('can cancel approval dialog', async () => {
    const user = userEvent.setup();
    render(
      <ApprovalCard
        approval={mockApproval}
        onApprove={mockOnApprove}
        onReject={mockOnReject}
      />
    );

    const approveButton = screen.getByRole('button', { name: /approve/i });
    await user.click(approveButton);

    const cancelButton = screen.getByRole('button', { name: /cancel/i });
    await user.click(cancelButton);

    // Dialog should close
    expect(screen.queryByPlaceholderText(/optional approval notes/i)).not.toBeInTheDocument();
    expect(mockOnApprove).not.toHaveBeenCalled();
  });

  it('formats timestamp correctly', () => {
    render(
      <ApprovalCard
        approval={mockApproval}
        onApprove={mockOnApprove}
        onReject={mockOnReject}
      />
    );

    // Should display formatted timestamp
    expect(screen.getByText(/jan 3, 2024/i)).toBeInTheDocument();
  });
});
