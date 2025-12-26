import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { axe, toHaveNoViolations } from 'jest-axe';
import { TradeForm } from '../TradeForm';

expect.extend(toHaveNoViolations);

describe('TradeForm', () => {
  const mockOnSubmit = vi.fn();

  beforeEach(() => {
    mockOnSubmit.mockClear();
  });

  it('validates required fields', async () => {
    const user = userEvent.setup();
    render(<TradeForm onSubmit={mockOnSubmit} agentId="agent-1" />);

    const submitButton = screen.getByRole('button', { name: /submit trade/i });
    await user.click(submitButton);

    // Should show validation errors
    expect(await screen.findByText(/symbol is required/i)).toBeInTheDocument();
    expect(screen.getByText(/quantity must be greater than 0/i)).toBeInTheDocument();
    expect(mockOnSubmit).not.toHaveBeenCalled();
  });

  it('calculates total value on input', async () => {
    const user = userEvent.setup();
    render(<TradeForm onSubmit={mockOnSubmit} agentId="agent-1" />);

    const quantityInput = screen.getByLabelText(/quantity/i);
    const priceInput = screen.getByLabelText(/price/i);

    await user.type(quantityInput, '10');
    await user.type(priceInput, '150.50');

    await waitFor(() => {
      expect(screen.getByText(/total: \$1,505\.00/i)).toBeInTheDocument();
    });
  });

  it('shows warning for large trades', async () => {
    const user = userEvent.setup();
    render(
      <TradeForm
        onSubmit={mockOnSubmit}
        agentId="agent-1"
        maxTradeSize={1000}
      />
    );

    const quantityInput = screen.getByLabelText(/quantity/i);
    const priceInput = screen.getByLabelText(/price/i);

    await user.type(quantityInput, '20');
    await user.type(priceInput, '100');

    await waitFor(() => {
      expect(screen.getByText(/large trade warning/i)).toBeInTheDocument();
      expect(screen.getByText(/exceeds 50% of maximum/i)).toBeInTheDocument();
    });
  });

  it('submits trade on valid form', async () => {
    const user = userEvent.setup();
    render(<TradeForm onSubmit={mockOnSubmit} agentId="agent-1" />);

    await user.type(screen.getByLabelText(/symbol/i), 'AAPL');
    await user.selectOptions(screen.getByLabelText(/side/i), 'buy');
    await user.type(screen.getByLabelText(/quantity/i), '10');
    await user.type(screen.getByLabelText(/price/i), '150.00');
    await user.type(screen.getByLabelText(/reasoning/i), 'Good buy opportunity');

    await user.click(screen.getByRole('button', { name: /submit trade/i }));

    await waitFor(() => {
      expect(mockOnSubmit).toHaveBeenCalledWith({
        agentId: 'agent-1',
        symbol: 'AAPL',
        side: 'buy',
        quantity: 10,
        price: 150.00,
        reasoning: 'Good buy opportunity',
      });
    });
  });

  it('disables submit while loading', async () => {
    const user = userEvent.setup();
    render(<TradeForm onSubmit={mockOnSubmit} agentId="agent-1" loading />);

    const submitButton = screen.getByRole('button', { name: /submitting/i });
    expect(submitButton).toBeDisabled();

    await user.click(submitButton);
    expect(mockOnSubmit).not.toHaveBeenCalled();
  });

  it('resets form after successful submission', async () => {
    const user = userEvent.setup();
    mockOnSubmit.mockResolvedValue(undefined);

    const { rerender } = render(
      <TradeForm onSubmit={mockOnSubmit} agentId="agent-1" />
    );

    await user.type(screen.getByLabelText(/symbol/i), 'AAPL');
    await user.type(screen.getByLabelText(/quantity/i), '10');
    await user.type(screen.getByLabelText(/price/i), '150.00');

    await user.click(screen.getByRole('button', { name: /submit trade/i }));

    await waitFor(() => {
      expect(mockOnSubmit).toHaveBeenCalled();
    });

    // Re-render with loading=false
    rerender(<TradeForm onSubmit={mockOnSubmit} agentId="agent-1" loading={false} />);

    // Form should be reset
    expect(screen.getByLabelText(/symbol/i)).toHaveValue('');
  });

  it('validates symbol format', async () => {
    const user = userEvent.setup();
    render(<TradeForm onSubmit={mockOnSubmit} agentId="agent-1" />);

    const symbolInput = screen.getByLabelText(/symbol/i);
    await user.type(symbolInput, 'invalid123');

    await waitFor(() => {
      expect(screen.getByText(/invalid symbol format/i)).toBeInTheDocument();
    });
  });

  it('validates positive price', async () => {
    const user = userEvent.setup();
    render(<TradeForm onSubmit={mockOnSubmit} agentId="agent-1" />);

    const priceInput = screen.getByLabelText(/price/i);
    await user.type(priceInput, '-10');

    await waitFor(() => {
      expect(screen.getByText(/price must be positive/i)).toBeInTheDocument();
    });
  });

  it('is keyboard accessible', async () => {
    const { container } = render(<TradeForm onSubmit={mockOnSubmit} agentId="agent-1" />);

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('supports sell side', async () => {
    const user = userEvent.setup();
    render(<TradeForm onSubmit={mockOnSubmit} agentId="agent-1" />);

    await user.selectOptions(screen.getByLabelText(/side/i), 'sell');

    expect(screen.getByLabelText(/side/i)).toHaveValue('sell');
  });
});
