import { BaseAgent } from './BaseAgent.js';
import type {
  AgentMessage,
  TripConstraints,
  BudgetAnalysis,
  CostBreakdown,
  OptimizationSuggestion,
} from '@shared/types';

/**
 * Budget Agent - Monitors costs and provides optimizations
 */
export class BudgetAgent extends BaseAgent {
  private costs: CostBreakdown = {
    flights: 0,
    hotels: 0,
    activities: 0,
    buffer: 0.1, // 10% buffer
  };

  private constraints: TripConstraints | null = null;

  constructor(tripId: string) {
    super('budget', tripId);
  }

  protected async processMessage(message: AgentMessage): Promise<void> {
    const { action, data } = message.payload;

    switch (action) {
      case 'startPlanning':
        await this.handleStartPlanning(data as { constraints: TripConstraints });
        break;

      case 'costUpdate':
        await this.handleCostUpdate(
          data as { category: string; amount: number }
        );
        break;

      case 'analyzeComplete':
        await this.handleAnalyzeComplete();
        break;

      default:
        console.log(`BudgetAgent: Unknown action ${action}`);
    }
  }

  private async handleStartPlanning(data: {
    constraints: TripConstraints;
  }): Promise<void> {
    this.updateStatus('working');
    this.constraints = data.constraints;
    this.resetCosts();
  }

  private async handleCostUpdate(data: {
    category: string;
    amount: number;
  }): Promise<void> {
    const { category, amount } = data;

    // Update cost breakdown
    if (category === 'flights') {
      this.costs.flights = amount;
    } else if (category === 'hotels') {
      this.costs.hotels = amount;
    } else if (category === 'activities') {
      this.costs.activities = amount;
    }

    // Check if over budget and send alert
    const totalCost = this.calculateTotalCost();
    if (
      this.constraints &&
      totalCost > this.constraints.budget.max
    ) {
      await this.sendMessage(
        'coordinator',
        'notification',
        'budgetAlert',
        {
          severity: 'high',
          message: 'Budget exceeded',
          currentTotal: totalCost,
          budgetMax: this.constraints.budget.max,
        },
        'critical'
      );
    }
  }

  private async handleAnalyzeComplete(): Promise<void> {
    if (!this.constraints) {
      throw new Error('No constraints set');
    }

    const analysis = await this.search(this.constraints);

    this.updateStatus('completed');
  }

  async search(constraints: TripConstraints): Promise<BudgetAnalysis> {
    this.constraints = constraints;

    const totalCost = this.calculateTotalCost();
    const totalWithBuffer = totalCost * (1 + this.costs.buffer);

    const analysis: BudgetAnalysis = {
      totalCost,
      totalWithBuffer,
      budget: constraints.budget,
      breakdown: { ...this.costs },
      overBudget: totalWithBuffer > constraints.budget.max,
      underBudget: totalWithBuffer < constraints.budget.min,
      utilizationRate: totalWithBuffer / constraints.budget.max,
      suggestions: [],
    };

    // Generate optimization suggestions
    if (analysis.overBudget) {
      analysis.suggestions = this.generateReductionSuggestions(
        totalWithBuffer - constraints.budget.max
      );
    } else if (analysis.underBudget) {
      analysis.suggestions = this.generateUpgradeSuggestions(
        constraints.budget.max - totalWithBuffer
      );
    }

    // Send analysis to coordinator
    await this.sendResults('coordinator', {
      analysis,
      category: 'budget',
    });

    return analysis;
  }

  /**
   * Calculate total cost
   */
  private calculateTotalCost(): number {
    return this.costs.flights + this.costs.hotels + this.costs.activities;
  }

  /**
   * Generate cost reduction suggestions
   */
  private generateReductionSuggestions(
    overageAmount: number
  ): OptimizationSuggestion[] {
    const suggestions: OptimizationSuggestion[] = [];
    const totalCost = this.calculateTotalCost();

    // Suggest flight reductions if significant portion
    if (this.costs.flights > totalCost * 0.5) {
      suggestions.push({
        category: 'flights',
        action: 'Consider flights with connections instead of direct flights',
        potentialSavings: this.costs.flights * 0.2,
      });
    }

    // Suggest hotel reductions
    if (this.costs.hotels > totalCost * 0.4) {
      suggestions.push({
        category: 'hotels',
        action: 'Switch to a 3-star hotel or different neighborhood',
        potentialSavings: this.costs.hotels * 0.25,
      });
    }

    // Suggest activity reductions
    if (this.costs.activities > totalCost * 0.3) {
      suggestions.push({
        category: 'activities',
        action: 'Replace 2 paid activities with free alternatives',
        potentialSavings: this.costs.activities * 0.3,
      });
    }

    return suggestions;
  }

  /**
   * Generate upgrade suggestions
   */
  private generateUpgradeSuggestions(
    remainingBudget: number
  ): OptimizationSuggestion[] {
    const suggestions: OptimizationSuggestion[] = [];

    if (remainingBudget > 200) {
      suggestions.push({
        category: 'flights',
        action: 'Upgrade to premium economy for added comfort',
        additionalCost: remainingBudget * 0.4,
      });

      suggestions.push({
        category: 'hotels',
        action: 'Upgrade to 5-star hotel with spa and premium amenities',
        additionalCost: remainingBudget * 0.4,
      });

      suggestions.push({
        category: 'activities',
        action: 'Add premium experiences (helicopter tour, Michelin dining)',
        additionalCost: remainingBudget * 0.2,
      });
    }

    return suggestions;
  }

  /**
   * Reset cost tracking
   */
  private resetCosts(): void {
    this.costs = {
      flights: 0,
      hotels: 0,
      activities: 0,
      buffer: 0.1,
    };
  }
}
