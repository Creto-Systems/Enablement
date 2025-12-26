import type { TripState } from '@shared/types';

/**
 * State Manager - Maintains consistent trip state across agents
 */
export class StateManager {
  private state: TripState;
  private locks: Map<string, string> = new Map();

  constructor(tripId: string) {
    this.state = {
      tripId,
      data: {},
      version: 0,
      lastUpdated: Date.now(),
    };
  }

  /**
   * Update state with lock mechanism
   */
  update(path: string, value: unknown, agentId = 'coordinator'): number {
    // Check lock
    const currentLock = this.locks.get(path);
    if (currentLock && currentLock !== agentId) {
      throw new Error(`Path ${path} is locked by ${currentLock}`);
    }

    // Acquire lock
    this.locks.set(path, agentId);

    // Update data
    this.setPath(this.state.data, path, value);
    this.state.version++;
    this.state.lastUpdated = Date.now();

    // Release lock
    this.locks.delete(path);

    return this.state.version;
  }

  /**
   * Get value at path
   */
  get(path: string): unknown {
    return this.getPath(this.state.data, path);
  }

  /**
   * Get full state snapshot
   */
  snapshot(): TripState {
    return {
      ...this.state,
      data: JSON.parse(JSON.stringify(this.state.data)),
    };
  }

  /**
   * Restore from snapshot
   */
  restore(snapshot: TripState): void {
    if (snapshot.tripId !== this.state.tripId) {
      throw new Error('Invalid snapshot for trip');
    }

    this.state = {
      ...snapshot,
      data: JSON.parse(JSON.stringify(snapshot.data)),
      lastUpdated: Date.now(),
    };
  }

  /**
   * Set value at nested path
   */
  private setPath(obj: Record<string, any>, path: string, value: unknown): void {
    const parts = path.split('.');
    const last = parts.pop()!;

    let current = obj;
    for (const part of parts) {
      if (!(part in current)) {
        current[part] = {};
      }
      current = current[part];
    }

    current[last] = value;
  }

  /**
   * Get value at nested path
   */
  private getPath(obj: Record<string, any>, path: string): unknown {
    const parts = path.split('.');
    let current: any = obj;

    for (const part of parts) {
      if (current === null || current === undefined) {
        return undefined;
      }
      current = current[part];
    }

    return current;
  }
}
