import { Project, ProjectSchedule, BudgetAnalysis, BudgetRecommendation } from '@shared/types';
import { ProjectModel } from '../models/Project';

export class ProjectService {
  /**
   * Calculate budget burn rate and forecast depletion
   */
  static calculateBudgetBurnRate(project: Project, spentAmount: number): BudgetAnalysis {
    const totalDays = ProjectModel.getDurationInDays(project);
    const elapsedDays = ProjectModel.getElapsedDays(project);
    const remainingDays = ProjectModel.getRemainingDays(project);

    const totalBudget = project.budget.totalAmount || 0;
    const remainingBudget = totalBudget - spentAmount;

    // Calculate burn rates
    const actualBurnRate = elapsedDays > 0 ? spentAmount / elapsedDays : 0;
    const plannedBurnRate = totalDays > 0 ? totalBudget / totalDays : 0;

    // Forecast
    const daysUntilDepletion =
      actualBurnRate > 0 ? remainingBudget / actualBurnRate : Infinity;
    const projectedDepletionDate =
      daysUntilDepletion !== Infinity
        ? new Date(Date.now() + daysUntilDepletion * 24 * 60 * 60 * 1000)
        : undefined;

    // Variance
    const plannedSpendToDate = plannedBurnRate * elapsedDays;
    const spendVariance = spentAmount - plannedSpendToDate;
    const spendVariancePercent =
      plannedSpendToDate > 0 ? (spendVariance / plannedSpendToDate) * 100 : 0;

    // Status
    let status: 'OnTrack' | 'AtRisk' | 'OverBudget' = 'OnTrack';
    if (spendVariancePercent > 10) {
      status = 'OverBudget';
    } else if (spendVariancePercent > 5) {
      status = 'AtRisk';
    }

    // Recommendations
    const recommendations: BudgetRecommendation[] = [];

    if (
      projectedDepletionDate &&
      projectedDepletionDate < new Date(project.timeline.endDate)
    ) {
      const daysShort = Math.ceil(
        (new Date(project.timeline.endDate).getTime() - projectedDepletionDate.getTime()) /
          (24 * 60 * 60 * 1000)
      );

      recommendations.push({
        priority: 'HIGH',
        action: `Budget will be depleted ${daysShort} days before project end`,
        suggestion: 'Reduce burn rate or request budget increase',
      });
    }

    if (actualBurnRate > plannedBurnRate * 1.2) {
      recommendations.push({
        priority: 'MEDIUM',
        action: 'Burn rate 20% above plan',
        suggestion: 'Review resource allocation and scope',
      });
    }

    return {
      budget: {
        total: totalBudget,
        spent: spentAmount,
        remaining: remainingBudget,
        percentUsed: totalBudget > 0 ? (spentAmount / totalBudget) * 100 : 0,
      },
      timeline: {
        totalDays,
        elapsedDays,
        remainingDays,
        percentComplete: totalDays > 0 ? (elapsedDays / totalDays) * 100 : 0,
      },
      burnRate: {
        actual: actualBurnRate,
        planned: plannedBurnRate,
        variance: actualBurnRate - plannedBurnRate,
        variancePercent:
          plannedBurnRate > 0 ? ((actualBurnRate - plannedBurnRate) / plannedBurnRate) * 100 : 0,
      },
      forecast: {
        projectedDepletionDate,
        daysUntilDepletion,
        projectedTotalSpend: actualBurnRate * totalDays,
      },
      status,
      recommendations,
    };
  }

  /**
   * Calculate project health score based on multiple factors
   */
  static calculateHealthScore(
    project: Project,
    budgetAnalysis: BudgetAnalysis,
    completionRate: number
  ): number {
    let score = 100;

    // Budget health (40% weight)
    if (budgetAnalysis.status === 'OverBudget') {
      score -= 40;
    } else if (budgetAnalysis.status === 'AtRisk') {
      score -= 20;
    }

    // Schedule health (30% weight)
    const scheduleVariance = completionRate - budgetAnalysis.timeline.percentComplete;
    if (scheduleVariance < -10) {
      // More than 10% behind schedule
      score -= 30;
    } else if (scheduleVariance < -5) {
      score -= 15;
    }

    // Milestone health (30% weight)
    const overdueMilestones = ProjectModel.getOverdueMilestones(project);
    const totalMilestones = project.timeline.milestones.length;
    if (totalMilestones > 0) {
      const overdueRate = (overdueMilestones.length / totalMilestones) * 100;
      if (overdueRate > 20) {
        score -= 30;
      } else if (overdueRate > 10) {
        score -= 15;
      }
    }

    return Math.max(0, Math.min(100, score));
  }

  /**
   * Get project status summary with all key metrics
   */
  static getProjectSummary(project: Project, spentAmount: number, completionRate: number) {
    const budgetAnalysis = this.calculateBudgetBurnRate(project, spentAmount);
    const healthScore = this.calculateHealthScore(project, budgetAnalysis, completionRate);

    const upcomingMilestones = ProjectModel.getUpcomingMilestones(project);
    const overdueMilestones = ProjectModel.getOverdueMilestones(project);
    const completedMilestones = ProjectModel.getCompletedMilestones(project);

    return {
      project,
      budgetAnalysis,
      healthScore,
      milestones: {
        total: project.timeline.milestones.length,
        completed: completedMilestones.length,
        upcoming: upcomingMilestones.length,
        overdue: overdueMilestones.length,
      },
      team: {
        size: project.team.resources.length,
        projectManager: project.team.projectManagerId,
      },
      status: {
        overall: project.status,
        budget: budgetAnalysis.status,
        schedule: project.health.scheduleStatus,
      },
    };
  }

  /**
   * Validate project can be started
   */
  static canStartProject(project: Project): { canStart: boolean; reasons: string[] } {
    const reasons: string[] = [];

    if (!project.team.projectManagerId) {
      reasons.push('Project manager not assigned');
    }

    if (project.team.resources.length === 0) {
      reasons.push('No resources assigned to project');
    }

    if (!project.budget.totalAmount && !project.budget.totalHours) {
      reasons.push('No budget defined');
    }

    if (project.timeline.milestones.length === 0) {
      reasons.push('No milestones defined');
    }

    return {
      canStart: reasons.length === 0,
      reasons,
    };
  }

  /**
   * Check if project needs attention (alerts)
   */
  static needsAttention(project: Project, budgetAnalysis: BudgetAnalysis): boolean {
    // Budget issues
    if (budgetAnalysis.status === 'OverBudget' || budgetAnalysis.status === 'AtRisk') {
      return true;
    }

    // Schedule issues
    if (project.health.scheduleStatus === 'Delayed') {
      return true;
    }

    // Overdue milestones
    const overdueMilestones = ProjectModel.getOverdueMilestones(project);
    if (overdueMilestones.length > 0) {
      return true;
    }

    // Low health score
    if (project.health.overallHealth < 70) {
      return true;
    }

    return false;
  }
}
