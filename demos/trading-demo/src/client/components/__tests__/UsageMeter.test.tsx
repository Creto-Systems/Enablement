import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { axe, toHaveNoViolations } from 'jest-axe';
import { UsageMeter } from '../UsageMeter';

expect.extend(toHaveNoViolations);

describe('UsageMeter', () => {
  it('shows usage percentage', () => {
    render(<UsageMeter used={350} total={1000} label="Budget" />);

    expect(screen.getByText('35%')).toBeInTheDocument();
    expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '35');
  });

  it('changes color at thresholds', () => {
    const { rerender } = render(<UsageMeter used={300} total={1000} label="Budget" />);

    // Green at < 60%
    let progressBar = screen.getByRole('progressbar').querySelector('[class*="bg-"]');
    expect(progressBar).toHaveClass('bg-green-500');

    // Yellow at 60-79%
    rerender(<UsageMeter used={700} total={1000} label="Budget" />);
    progressBar = screen.getByRole('progressbar').querySelector('[class*="bg-"]');
    expect(progressBar).toHaveClass('bg-yellow-500');

    // Red at >= 80%
    rerender(<UsageMeter used={850} total={1000} label="Budget" />);
    progressBar = screen.getByRole('progressbar').querySelector('[class*="bg-"]');
    expect(progressBar).toHaveClass('bg-red-500');
  });

  it('displays remaining budget', () => {
    render(<UsageMeter used={350} total={1000} label="Budget" />);

    expect(screen.getByText('$350')).toBeInTheDocument();
    expect(screen.getByText('/ $1,000')).toBeInTheDocument();
    expect(screen.getByText('$650 remaining')).toBeInTheDocument();
  });

  it('shows warning icon at 80%', () => {
    const { rerender } = render(<UsageMeter used={750} total={1000} label="Budget" />);

    // No warning below 80%
    expect(screen.queryByTitle(/warning/i)).not.toBeInTheDocument();

    // Warning at 80%+
    rerender(<UsageMeter used={850} total={1000} label="Budget" />);
    expect(screen.getByTitle(/high usage warning/i)).toBeInTheDocument();
  });

  it('handles 100% usage', () => {
    render(<UsageMeter used={1000} total={1000} label="Budget" />);

    expect(screen.getByText('100%')).toBeInTheDocument();
    expect(screen.getByText('$0 remaining')).toBeInTheDocument();
    expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '100');
  });

  it('handles 0% usage', () => {
    render(<UsageMeter used={0} total={1000} label="Budget" />);

    expect(screen.getByText('0%')).toBeInTheDocument();
    expect(screen.getByText('$1,000 remaining')).toBeInTheDocument();
  });

  it('is accessible', async () => {
    const { container } = render(<UsageMeter used={500} total={1000} label="Budget" />);

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('displays custom label', () => {
    render(<UsageMeter used={500} total={1000} label="API Calls" />);

    expect(screen.getByText('API Calls')).toBeInTheDocument();
  });

  it('handles very small percentages', () => {
    render(<UsageMeter used={5} total={10000} label="Budget" />);

    expect(screen.getByText('0%')).toBeInTheDocument(); // Rounds to 0
  });

  it('shows critical warning at 90%', () => {
    render(<UsageMeter used={950} total={1000} label="Budget" />);

    expect(screen.getByText(/critical/i)).toBeInTheDocument();
  });
});
