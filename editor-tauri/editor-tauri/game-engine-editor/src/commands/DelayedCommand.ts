// Delayed Command Implementation

import { Command, IDelayedCommand } from '../types/commands';

/**
 * Delayed command for deferred execution after a specified delay
 */
export class DelayedCommand implements IDelayedCommand {
  id: string;
  timestamp: Date;
  description: string;
  command: Command;
  delay: number;
  executeAt: Date;
  scheduled: boolean;
  metadata?: Record<string, any>;

  private timerId?: NodeJS.Timeout;
  private executor?: () => void;
  private rejector?: (error: Error) => void;

  constructor(command: Command, delay: number) {
    this.id = this.generateId();
    this.timestamp = new Date();
    this.command = command;
    this.delay = delay;
    this.executeAt = new Date(Date.now() + delay);
    this.scheduled = false;
    this.description = `Delayed: ${command.description}`;
    this.metadata = {
      originalCommand: command.description,
      delay,
      executeAt: this.executeAt,
    };
  }

  async execute(): Promise<void> {
    if (this.scheduled) {
      throw new Error('Command is already scheduled');
    }

    return new Promise<void>((resolve, reject) => {
      this.scheduled = true;
      this.executor = resolve;
      this.rejector = reject;

      this.timerId = setTimeout(async () => {
        try {
          await this.command.execute();
          resolve();
        } catch (error) {
          reject(error);
        }
      }, this.delay);
    });
  }

  async undo(): Promise<void> {
    if (this.timerId) {
      clearTimeout(this.timerId);
      this.timerId = undefined;
    }

    if (this.scheduled && this.command) {
      await this.command.undo();
    }

    this.scheduled = false;
  }

  async redo(): Promise<void> {
    await this.command.redo();
  }

  /**
   * Cancel the delayed execution
   */
  cancel(): void {
    if (this.timerId) {
      clearTimeout(this.timerId);
      this.timerId = undefined;
    }

    this.scheduled = false;

    if (this.rejector) {
      this.rejector(new Error('Command was cancelled'));
    }
  }

  /**
   * Execute immediately without waiting for the delay
   */
  async executeNow(): Promise<void> {
    this.cancel();

    try {
      await this.command.execute();
      if (this.executor) {
        this.executor();
      }
    } catch (error) {
      if (this.rejector) {
        this.rejector(error as Error);
      }
      throw error;
    }
  }

  /**
   * Check if the command is ready to execute
   */
  isReady(): boolean {
    return new Date() >= this.executeAt;
  }

  /**
   * Get remaining time until execution
   */
  getRemainingTime(): number {
    const remaining = this.executeAt.getTime() - Date.now();
    return Math.max(0, remaining);
  }

  private generateId(): string {
    return `delayed_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}

/**
 * Utility function to create a delayed command
 */
export function createDelayedCommand(command: Command, delay: number): DelayedCommand {
  return new DelayedCommand(command, delay);
}

/**
 * Schedule a command to execute at a specific time
 */
export function scheduleCommandAt(command: Command, executeAt: Date): DelayedCommand {
  const delay = executeAt.getTime() - Date.now();
  if (delay < 0) {
    throw new Error('Execution time must be in the future');
  }
  return new DelayedCommand(command, delay);
}

/**
 * Create a debounced command that only executes after a delay
 * from the last call
 */
export class DebouncedCommand {
  private command: Command;
  private delay: number;
  private delayedCommand?: DelayedCommand;

  constructor(command: Command, delay: number) {
    this.command = command;
    this.delay = delay;
  }

  async execute(): Promise<void> {
    if (this.delayedCommand) {
      this.delayedCommand.cancel();
    }

    this.delayedCommand = new DelayedCommand(this.command, this.delay);
    await this.delayedCommand.execute();
  }

  cancel(): void {
    if (this.delayedCommand) {
      this.delayedCommand.cancel();
      this.delayedCommand = undefined;
    }
  }
}

/**
 * Create a throttled command that executes at most once per delay
 */
export class ThrottledCommand {
  private command: Command;
  private delay: number;
  private lastExecution: number = 0;
  private pendingExecution?: Promise<void>;

  constructor(command: Command, delay: number) {
    this.command = command;
    this.delay = delay;
  }

  async execute(): Promise<void> {
    const now = Date.now();
    const timeSinceLastExecution = now - this.lastExecution;

    if (timeSinceLastExecution >= this.delay) {
      this.lastExecution = now;
      return this.command.execute();
    } else {
      // Wait until throttle period is over
      if (!this.pendingExecution) {
        this.pendingExecution = new Promise<void>((resolve) => {
          setTimeout(async () => {
            this.lastExecution = Date.now();
            try {
              await this.command.execute();
            } finally {
              this.pendingExecution = undefined;
              resolve();
            }
          }, this.delay - timeSinceLastExecution);
        });
      }
      return this.pendingExecution;
    }
  }
}
