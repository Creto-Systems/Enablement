import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { axe, toHaveNoViolations } from 'jest-axe';
import { AgentCard } from '../AgentCard';
import { mockAgent } from '../../test-utils/test-helpers';

expect.extend(toHaveNoViolations);

describe('AgentCard', () => {
  it('renders agent name and status', () => {
    render(<AgentCard agent={mockAgent} onSelect={vi.fn()} />);

    expect(screen.getByText('Conservative Trader')).toBeInTheDocument();
    expect(screen.getByText('active')).toBeInTheDocument();
  });

  it('shows budget utilization bar', () => {
    render(<AgentCard agent={mockAgent} onSelect={vi.fn()} />);

    const progressBar = screen.getByRole('progressbar');
    expect(progressBar).toHaveAttribute('aria-valuenow', '35');
    expect(progressBar).toHaveAttribute('aria-valuemin', '0');
    expect(progressBar).toHaveAttribute('aria-valuemax', '100');

    // Check budget text
    expect(screen.getByText('$3,500')).toBeInTheDocument();
    expect(screen.getByText('/ $10,000')).toBeInTheDocument();
  });

  it('displays daily P&L with correct color', () => {
    // Positive P&L
    render(<AgentCard agent={mockAgent} onSelect={vi.fn()} />);
    const positivePnL = screen.getByText(/\$234\.56/);
    expect(positivePnL).toHaveClass('text-green-600');

    // Negative P&L
    const negativeAgent = {
      ...mockAgent,
      performance: { ...mockAgent.performance, dailyPnL: -150.25 },
    };
    render(<AgentCard agent={negativeAgent} onSelect={vi.fn()} />);
    const negativePnL = screen.getByText(/\$150\.25/);
    expect(negativePnL).toHaveClass('text-red-600');
  });

  it('calls onSelect when clicked', () => {
    const onSelect = vi.fn();
    render(<AgentCard agent={mockAgent} onSelect={onSelect} />);

    const card = screen.getByRole('button', { name: /conservative trader/i });
    fireEvent.click(card);

    expect(onSelect).toHaveBeenCalledWith(mockAgent.id);
    expect(onSelect).toHaveBeenCalledTimes(1);
  });

  it('is keyboard accessible', async () => {
    const onSelect = vi.fn();
    const { container } = render(<AgentCard agent={mockAgent} onSelect={onSelect} />);

    const card = screen.getByRole('button');

    // Should be focusable
    card.focus();
    expect(card).toHaveFocus();

    // Should activate on Enter
    fireEvent.keyDown(card, { key: 'Enter', code: 'Enter' });
    expect(onSelect).toHaveBeenCalledWith(mockAgent.id);

    // Should activate on Space
    fireEvent.keyDown(card, { key: ' ', code: 'Space' });
    expect(onSelect).toHaveBeenCalledTimes(2);

    // Check accessibility
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('shows selected state', () => {
    render(<AgentCard agent={mockAgent} onSelect={vi.fn()} selected />);

    const card = screen.getByRole('button');
    expect(card).toHaveClass('ring-2', 'ring-blue-500');
    expect(card).toHaveAttribute('aria-pressed', 'true');
  });

  it('displays win rate', () => {
    render(<AgentCard agent={mockAgent} onSelect={vi.fn()} />);

    expect(screen.getByText('68%')).toBeInTheDocument();
    expect(screen.getByText(/win rate/i)).toBeInTheDocument();
  });

  it('handles inactive status', () => {
    const inactiveAgent = { ...mockAgent, status: 'inactive' as const };
    render(<AgentCard agent={inactiveAgent} onSelect={vi.fn()} />);

    expect(screen.getByText('inactive')).toBeInTheDocument();
    expect(screen.getByRole('button')).toHaveClass('opacity-60');
  });
});
