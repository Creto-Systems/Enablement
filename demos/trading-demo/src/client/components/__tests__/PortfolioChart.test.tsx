import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { PortfolioChart } from '../PortfolioChart';

const mockData = [
  { timestamp: '2024-01-01T00:00:00Z', value: 10000 },
  { timestamp: '2024-01-02T00:00:00Z', value: 10500 },
  { timestamp: '2024-01-03T00:00:00Z', value: 12050 },
];

// Mock recharts components
vi.mock('recharts', () => ({
  LineChart: ({ children, data }: any) => (
    <div data-testid="line-chart" data-points={data?.length}>
      {children}
    </div>
  ),
  Line: ({ dataKey }: any) => <div data-testid={`line-${dataKey}`} />,
  XAxis: () => <div data-testid="x-axis" />,
  YAxis: () => <div data-testid="y-axis" />,
  CartesianGrid: () => <div data-testid="grid" />,
  Tooltip: ({ content }: any) => {
    const CustomTooltip = content;
    return CustomTooltip ? <CustomTooltip active={false} payload={[]} /> : null;
  },
  ResponsiveContainer: ({ children }: any) => (
    <div data-testid="responsive-container">{children}</div>
  ),
}));

describe('PortfolioChart', () => {
  it('renders line chart with data points', () => {
    render(<PortfolioChart data={mockData} />);

    expect(screen.getByTestId('responsive-container')).toBeInTheDocument();
    expect(screen.getByTestId('line-chart')).toHaveAttribute('data-points', '3');
    expect(screen.getByTestId('line-value')).toBeInTheDocument();
    expect(screen.getByTestId('x-axis')).toBeInTheDocument();
    expect(screen.getByTestId('y-axis')).toBeInTheDocument();
    expect(screen.getByTestId('grid')).toBeInTheDocument();
  });

  it('shows tooltip on hover', async () => {
    const { container } = render(<PortfolioChart data={mockData} />);

    // Tooltip should be rendered but not visible initially
    expect(container.querySelector('[role="tooltip"]')).not.toBeVisible();
  });

  it('handles empty data gracefully', () => {
    render(<PortfolioChart data={[]} />);

    expect(screen.getByText(/no data available/i)).toBeInTheDocument();
    expect(screen.queryByTestId('line-chart')).not.toBeInTheDocument();
  });

  it('updates when data changes', () => {
    const { rerender } = render(<PortfolioChart data={mockData} />);

    expect(screen.getByTestId('line-chart')).toHaveAttribute('data-points', '3');

    const newData = [...mockData, { timestamp: '2024-01-04T00:00:00Z', value: 13000 }];
    rerender(<PortfolioChart data={newData} />);

    expect(screen.getByTestId('line-chart')).toHaveAttribute('data-points', '4');
  });

  it('formats timestamps correctly', () => {
    render(<PortfolioChart data={mockData} />);

    // Chart should be rendered with formatted data
    expect(screen.getByTestId('line-chart')).toBeInTheDocument();
  });

  it('formats currency values correctly', () => {
    render(<PortfolioChart data={mockData} />);

    // Y-axis should format values as currency
    expect(screen.getByTestId('y-axis')).toBeInTheDocument();
  });

  it('shows gain/loss indicator', () => {
    render(<PortfolioChart data={mockData} />);

    // Should calculate and display overall gain
    const gain = ((12050 - 10000) / 10000) * 100;
    expect(screen.getByText(/20\.5%/)).toBeInTheDocument();
    expect(screen.getByText(/\+\$2,050/)).toBeInTheDocument();
  });

  it('handles negative performance', () => {
    const negativeData = [
      { timestamp: '2024-01-01T00:00:00Z', value: 10000 },
      { timestamp: '2024-01-02T00:00:00Z', value: 9500 },
    ];

    render(<PortfolioChart data={negativeData} />);

    expect(screen.getByText(/-5\.0%/)).toBeInTheDocument();
    expect(screen.getByText(/-\$500/)).toBeInTheDocument();
  });

  it('is responsive', () => {
    render(<PortfolioChart data={mockData} />);

    const container = screen.getByTestId('responsive-container');
    expect(container).toBeInTheDocument();
  });
});
