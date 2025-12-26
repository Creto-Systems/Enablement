import { BaseAgent } from './BaseAgent.js';
import type {
  AgentMessage,
  TripConstraints,
  Activity,
} from '@shared/types';
import { generateActivities } from '../utils/mockData.js';

/**
 * Activity Agent - Searches and schedules activities
 */
export class ActivityAgent extends BaseAgent {
  constructor(tripId: string) {
    super('activity', tripId);
  }

  protected async processMessage(message: AgentMessage): Promise<void> {
    const { action, data } = message.payload;

    switch (action) {
      case 'startPlanning':
        await this.handleStartPlanning(data as { constraints: TripConstraints });
        break;

      case 'requestAlternative':
        await this.handleAlternativeRequest(message.from);
        break;

      default:
        console.log(`ActivityAgent: Unknown action ${action}`);
    }
  }

  private async handleStartPlanning(data: {
    constraints: TripConstraints;
  }): Promise<void> {
    this.updateStatus('working');

    // Simulate search delay
    await new Promise((resolve) => setTimeout(resolve, 1200));

    const results = await this.search(data.constraints);

    this.updateStatus('completed');
  }

  private async handleAlternativeRequest(requesterId: string): Promise<void> {
    this.updateStatus('working');

    // Generate alternative options
    await new Promise((resolve) => setTimeout(resolve, 700));

    this.updateStatus('completed');
  }

  async search(constraints: TripConstraints): Promise<Activity[]> {
    // Generate mock activity data
    const activities = generateActivities(constraints, this.id);

    // Validate schedule (no conflicts)
    const validatedActivities = this.validateSchedule(activities);

    // Send results to coordinator
    await this.sendResults('coordinator', {
      activities: validatedActivities,
      category: 'activities',
    });

    // Calculate and notify budget
    const totalCost = validatedActivities.reduce((sum, a) => sum + a.price, 0);
    await this.notifyBudget('activities', totalCost);

    return validatedActivities;
  }

  /**
   * Validate activity schedule for conflicts
   */
  private validateSchedule(activities: Activity[]): Activity[] {
    const validated: Activity[] = [];
    const dateMap = new Map<string, Activity[]>();

    // Group by date
    for (const activity of activities) {
      const date = activity.date;
      if (!dateMap.has(date)) {
        dateMap.set(date, []);
      }
      dateMap.get(date)!.push(activity);
    }

    // Check each day for time conflicts
    for (const [date, dayActivities] of dateMap.entries()) {
      // Sort by start time
      dayActivities.sort((a, b) =>
        a.startTime.localeCompare(b.startTime)
      );

      for (let i = 0; i < dayActivities.length; i++) {
        const current = dayActivities[i];

        // Check if overlaps with previous
        if (i > 0) {
          const previous = validated[validated.length - 1];
          if (previous.date === date) {
            const prevEnd = this.calculateEndTime(
              previous.startTime,
              previous.duration
            );
            if (prevEnd > current.startTime) {
              // Skip conflicting activity
              continue;
            }
          }
        }

        validated.push(current);
      }
    }

    return validated;
  }

  /**
   * Calculate end time of activity
   */
  private calculateEndTime(startTime: string, duration: number): string {
    const [hours, minutes] = startTime.split(':').map(Number);
    const totalMinutes = hours * 60 + minutes + duration;
    const endHours = Math.floor(totalMinutes / 60) % 24;
    const endMinutes = totalMinutes % 60;
    return `${endHours.toString().padStart(2, '0')}:${endMinutes.toString().padStart(2, '0')}`;
  }
}
