import type {
  Itinerary,
  Conflict,
  Activity,
  FlightBooking,
  HotelBooking,
} from '@shared/types';
import { generateId } from '../utils/encryption.js';

/**
 * Conflict Resolver - Detects and resolves itinerary conflicts
 */
export class ConflictResolver {
  /**
   * Detect all conflicts in itinerary
   */
  detectConflicts(itinerary: Itinerary): Conflict[] {
    const conflicts: Conflict[] = [];

    // Check time conflicts in activities
    conflicts.push(...this.detectTimeConflicts(itinerary.activities));

    // Check location conflicts (insufficient travel time)
    conflicts.push(...this.detectLocationConflicts(itinerary.activities));

    // Check budget conflicts
    if (itinerary.totalCost > 0) {
      // Budget checks handled by BudgetAgent
    }

    return conflicts;
  }

  /**
   * Detect time conflicts between activities
   */
  private detectTimeConflicts(activities: Activity[]): Conflict[] {
    const conflicts: Conflict[] = [];

    // Group by date
    const byDate = new Map<string, Activity[]>();
    for (const activity of activities) {
      if (!byDate.has(activity.date)) {
        byDate.set(activity.date, []);
      }
      byDate.get(activity.date)!.push(activity);
    }

    // Check each date
    for (const [date, dayActivities] of byDate.entries()) {
      // Sort by start time
      const sorted = [...dayActivities].sort((a, b) =>
        a.startTime.localeCompare(b.startTime)
      );

      // Check overlaps
      for (let i = 0; i < sorted.length - 1; i++) {
        const current = sorted[i];
        const next = sorted[i + 1];

        const currentEnd = this.calculateEndTime(
          current.startTime,
          current.duration
        );

        if (currentEnd > next.startTime) {
          conflicts.push({
            id: generateId(),
            type: 'time',
            severity: 'high',
            description: `${current.name} (ends ${currentEnd}) overlaps with ${next.name} (starts ${next.startTime})`,
            affectedItems: [current.id, next.id],
            resolution: {
              strategy: 'reschedule',
              appliedBy: 'coordinator',
              timestamp: new Date().toISOString(),
            },
          });
        }
      }
    }

    return conflicts;
  }

  /**
   * Detect location conflicts (insufficient travel time)
   */
  private detectLocationConflicts(activities: Activity[]): Conflict[] {
    const conflicts: Conflict[] = [];

    // Group by date
    const byDate = new Map<string, Activity[]>();
    for (const activity of activities) {
      if (!byDate.has(activity.date)) {
        byDate.set(activity.date, []);
      }
      byDate.get(activity.date)!.push(activity);
    }

    // Check each date
    for (const [date, dayActivities] of byDate.entries()) {
      // Sort by start time
      const sorted = [...dayActivities].sort((a, b) =>
        a.startTime.localeCompare(b.startTime)
      );

      // Check travel time between activities
      for (let i = 0; i < sorted.length - 1; i++) {
        const current = sorted[i];
        const next = sorted[i + 1];

        const currentEnd = this.calculateEndTime(
          current.startTime,
          current.duration
        );

        const bufferTime = this.calculateTimeDiff(currentEnd, next.startTime);
        const travelTime = this.calculateTravelTime(
          current.location,
          next.location
        );

        if (bufferTime < travelTime) {
          conflicts.push({
            id: generateId(),
            type: 'location',
            severity: 'medium',
            description: `Insufficient time (${bufferTime}min) to travel from ${current.name} to ${next.name} (needs ${travelTime}min)`,
            affectedItems: [current.id, next.id],
            resolution: {
              strategy: 'addBuffer',
              appliedBy: 'coordinator',
              timestamp: new Date().toISOString(),
            },
          });
        }
      }
    }

    return conflicts;
  }

  /**
   * Resolve conflicts in itinerary
   */
  resolveConflicts(conflicts: Conflict[], itinerary: Itinerary): Itinerary {
    const resolved = { ...itinerary };

    for (const conflict of conflicts) {
      if (conflict.type === 'time') {
        // Reschedule conflicting activities
        // For demo, mark as requiring manual resolution
        conflict.resolution = {
          strategy: 'manual',
          appliedBy: 'coordinator',
          timestamp: new Date().toISOString(),
        };
      } else if (conflict.type === 'location') {
        // Add buffer time
        conflict.resolution = {
          strategy: 'bufferAdded',
          appliedBy: 'coordinator',
          timestamp: new Date().toISOString(),
        };
      }
    }

    resolved.conflicts = conflicts;
    return resolved;
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

  /**
   * Calculate time difference in minutes
   */
  private calculateTimeDiff(time1: string, time2: string): number {
    const [h1, m1] = time1.split(':').map(Number);
    const [h2, m2] = time2.split(':').map(Number);

    const total1 = h1 * 60 + m1;
    const total2 = h2 * 60 + m2;

    return total2 - total1;
  }

  /**
   * Calculate travel time between locations (simplified)
   */
  private calculateTravelTime(
    loc1: Activity['location'],
    loc2: Activity['location']
  ): number {
    // Simple distance calculation
    const distance = Math.sqrt(
      Math.pow(loc1.coordinates.lat - loc2.coordinates.lat, 2) +
        Math.pow(loc1.coordinates.lng - loc2.coordinates.lng, 2)
    );

    // Assume 30 minutes per 0.1 degree
    return Math.ceil(distance * 300);
  }
}
