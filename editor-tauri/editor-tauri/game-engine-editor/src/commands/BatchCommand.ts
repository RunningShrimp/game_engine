// Batch Command Implementation

import { Command, IBatchCommand } from '../types/commands';
import { BatchOptions } from '../types/history';

/**
 * Batch command for executing multiple commands in sequence or parallel
 */
export class BatchCommand implements IBatchCommand {
  id: string;
  timestamp: Date;
  description: string;
  commands: Command[];
  executeMode: 'sequential' | 'parallel';
  metadata?: Record<string, any>;

  private executedCommands: Command[] = [];
  private failedCommands: Array<{ command: Command; error: Error }> = [];

  constructor(commands: Command[], options: BatchOptions = { mode: 'sequential' }) {
    this.id = this.generateId();
    this.timestamp = new Date();
    this.commands = commands;
    this.executeMode = options.mode;
    this.description = `Batch (${commands.length} commands)`;
    this.metadata = {
      commandCount: commands.length,
      mode: options.mode,
      maxConcurrency: options.maxConcurrency,
    };
  }

  async execute(): Promise<void> {
    this.executedCommands = [];
    this.failedCommands = [];

    if (this.executeMode === 'sequential') {
      await this.executeSequential();
    } else {
      await this.executeParallel();
    }
  }

  private async executeSequential(): Promise<void> {
    for (const command of this.commands) {
      try {
        await command.execute();
        this.executedCommands.push(command);
      } catch (error) {
        this.failedCommands.push({ command, error: error as Error });
        throw new Error(
          `Batch command failed at command ${this.executedCommands.length}: ${(error as Error).message}`
        );
      }
    }
  }

  private async executeParallel(): Promise<void> {
    const promises = this.commands.map(async (command) => {
      try {
        await command.execute();
        return { success: true, command };
      } catch (error) {
        this.failedCommands.push({ command, error: error as Error });
        return { success: false, command, error };
      }
    });

    const results = await Promise.all(promises);

    for (const result of results) {
      if (result.success) {
        this.executedCommands.push(result.command);
      }
    }

    if (this.failedCommands.length > 0) {
      throw new Error(
        `Batch command failed: ${this.failedCommands.length} of ${this.commands.length} commands failed`
      );
    }
  }

  async undo(): Promise<void> {
    // Undo in reverse order of execution
    for (let i = this.executedCommands.length - 1; i >= 0; i--) {
      try {
        await this.executedCommands[i].undo();
      } catch (error) {
        console.error(`Error undoing command ${i}:`, error);
        throw error;
      }
    }
  }

  async redo(): Promise<void> {
    if (this.executeMode === 'sequential') {
      for (const command of this.executedCommands) {
        await command.redo();
      }
    } else {
      await Promise.all(this.executedCommands.map((cmd) => cmd.redo()));
    }
  }

  getExecutedCommands(): Command[] {
    return [...this.executedCommands];
  }

  getFailedCommands(): Array<{ command: Command; error: Error }> {
    return [...this.failedCommands];
  }

  private generateId(): string {
    return `batch_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}

/**
 * Utility function to create a batch command
 */
export function createBatchCommand(
  commands: Command[],
  mode: 'sequential' | 'parallel' = 'sequential'
): BatchCommand {
  return new BatchCommand(commands, { mode });
}
