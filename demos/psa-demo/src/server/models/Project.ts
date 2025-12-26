import {
  Project,
  ProjectTimeline,
  ProjectBudget,
  ProjectTeam,
  ProjectBilling,
  ProjectHealth,
  Milestone,
  ResourceAssignment,
} from '@shared/types';
import { v4 as uuidv4 } from 'uuid';

export class ProjectModel {
  static create(data: Partial<Project>): Project {
    const now = new Date();

    return {
      id: uuidv4(),
      clientId: data.clientId || '',
      name: data.name || '',
      description: data.description || '',
      type: data.type || 'TimeAndMaterial',
      status: data.status || 'Planning',
      timeline: data.timeline || this.getDefaultTimeline(),
      budget: data.budget || this.getDefaultBudget(),
      team: data.team || this.getDefaultTeam(),
      billing: data.billing || this.getDefaultBilling(),
      health: data.health || this.getDefaultHealth(),
      metadata: {
        createdAt: now,
        updatedAt: now,
        tags: data.metadata?.tags || [],
      },
    };
  }

  static update(project: Project, updates: Partial<Project>): Project {
    return {
      ...project,
      ...updates,
      metadata: {
        ...project.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static addMilestone(project: Project, milestone: Omit<Milestone, 'id'>): Project {
    const newMilestone: Milestone = {
      id: uuidv4(),
      ...milestone,
    };

    return {
      ...project,
      timeline: {
        ...project.timeline,
        milestones: [...project.timeline.milestones, newMilestone],
      },
      metadata: {
        ...project.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static updateMilestone(project: Project, milestoneId: string, updates: Partial<Milestone>): Project {
    return {
      ...project,
      timeline: {
        ...project.timeline,
        milestones: project.timeline.milestones.map((m) =>
          m.id === milestoneId ? { ...m, ...updates } : m
        ),
      },
      metadata: {
        ...project.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static assignResource(project: Project, assignment: Omit<ResourceAssignment, 'userId'> & { userId: string }): Project {
    const existingIndex = project.team.resources.findIndex((r) => r.userId === assignment.userId);

    if (existingIndex >= 0) {
      // Update existing assignment
      const updatedResources = [...project.team.resources];
      updatedResources[existingIndex] = { ...updatedResources[existingIndex], ...assignment };

      return {
        ...project,
        team: {
          ...project.team,
          resources: updatedResources,
        },
        metadata: {
          ...project.metadata,
          updatedAt: new Date(),
        },
      };
    } else {
      // Add new assignment
      return {
        ...project,
        team: {
          ...project.team,
          resources: [...project.team.resources, assignment],
        },
        metadata: {
          ...project.metadata,
          updatedAt: new Date(),
        },
      };
    }
  }

  static removeResource(project: Project, userId: string): Project {
    return {
      ...project,
      team: {
        ...project.team,
        resources: project.team.resources.filter((r) => r.userId !== userId),
      },
      metadata: {
        ...project.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static updateBudget(project: Project, spent: number): Project {
    const totalBudget = project.budget.totalAmount || 0;
    const remainingBudget = totalBudget - spent;
    const burnRate = totalBudget > 0 ? (spent / totalBudget) * 100 : 0;

    return {
      ...project,
      budget: {
        ...project.budget,
        burnRate,
        remainingBudget,
      },
      metadata: {
        ...project.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static updateHealth(project: Project, health: Partial<ProjectHealth>): Project {
    return {
      ...project,
      health: {
        ...project.health,
        ...health,
      },
      metadata: {
        ...project.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static changeStatus(
    project: Project,
    status: 'Planning' | 'Active' | 'OnHold' | 'Completed' | 'Cancelled'
  ): Project {
    return {
      ...project,
      status,
      metadata: {
        ...project.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static addTag(project: Project, tag: string): Project {
    if (project.metadata.tags.includes(tag)) return project;

    return {
      ...project,
      metadata: {
        ...project.metadata,
        tags: [...project.metadata.tags, tag],
        updatedAt: new Date(),
      },
    };
  }

  static removeTag(project: Project, tag: string): Project {
    return {
      ...project,
      metadata: {
        ...project.metadata,
        tags: project.metadata.tags.filter((t) => t !== tag),
        updatedAt: new Date(),
      },
    };
  }

  static isActive(project: Project): boolean {
    return project.status === 'Active';
  }

  static isOverBudget(project: Project): boolean {
    return project.health.budgetStatus === 'OverBudget';
  }

  static isDelayed(project: Project): boolean {
    return project.health.scheduleStatus === 'Delayed';
  }

  static getCompletedMilestones(project: Project): Milestone[] {
    return project.timeline.milestones.filter((m) => m.status === 'Completed');
  }

  static getUpcomingMilestones(project: Project): Milestone[] {
    const now = new Date();
    return project.timeline.milestones.filter(
      (m) => m.status !== 'Completed' && new Date(m.dueDate) >= now
    );
  }

  static getOverdueMilestones(project: Project): Milestone[] {
    const now = new Date();
    return project.timeline.milestones.filter(
      (m) => m.status !== 'Completed' && new Date(m.dueDate) < now
    );
  }

  static getDurationInDays(project: Project): number {
    const start = new Date(project.timeline.startDate);
    const end = new Date(project.timeline.endDate);
    return Math.ceil((end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24));
  }

  static getElapsedDays(project: Project): number {
    const start = new Date(project.timeline.startDate);
    const now = new Date();
    return Math.max(0, Math.ceil((now.getTime() - start.getTime()) / (1000 * 60 * 60 * 24)));
  }

  static getRemainingDays(project: Project): number {
    const now = new Date();
    const end = new Date(project.timeline.endDate);
    return Math.max(0, Math.ceil((end.getTime() - now.getTime()) / (1000 * 60 * 60 * 24)));
  }

  private static getDefaultTimeline(): ProjectTimeline {
    const now = new Date();
    const endDate = new Date();
    endDate.setMonth(endDate.getMonth() + 3); // 3 months default

    return {
      startDate: now,
      endDate: endDate,
      milestones: [],
    };
  }

  private static getDefaultBudget(): ProjectBudget {
    return {
      currency: 'USD',
      burnRate: 0,
      remainingBudget: 0,
    };
  }

  private static getDefaultTeam(): ProjectTeam {
    return {
      projectManagerId: '',
      resources: [],
    };
  }

  private static getDefaultBilling(): ProjectBilling {
    return {
      billingCycle: 'Monthly',
      rateCard: {
        id: '',
        name: 'Standard Rate Card',
        baseRates: {},
      },
      invoiceSchedule: [],
    };
  }

  private static getDefaultHealth(): ProjectHealth {
    return {
      budgetStatus: 'OnTrack',
      scheduleStatus: 'OnTrack',
      overallHealth: 100,
    };
  }

  static validate(project: Partial<Project>): { isValid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (!project.clientId) {
      errors.push('Client ID is required');
    }

    if (!project.name || project.name.trim() === '') {
      errors.push('Project name is required');
    }

    if (project.timeline) {
      const start = new Date(project.timeline.startDate);
      const end = new Date(project.timeline.endDate);

      if (end <= start) {
        errors.push('End date must be after start date');
      }
    }

    if (project.budget) {
      if (project.budget.totalAmount !== undefined && project.budget.totalAmount < 0) {
        errors.push('Budget amount must be positive');
      }

      if (project.budget.totalHours !== undefined && project.budget.totalHours < 0) {
        errors.push('Budget hours must be positive');
      }
    }

    if (project.team && !project.team.projectManagerId) {
      errors.push('Project manager is required');
    }

    return {
      isValid: errors.length === 0,
      errors,
    };
  }
}
