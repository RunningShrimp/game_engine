// Transaction Command Implementation

import { Command, ITransactionCommand } from '../types/commands';
import { TransactionOptions } from '../types/history';

/**
 * Transaction command with atomic execution and rollback capability
 */
export class TransactionCommand implements ITransactionCommand {
  id: string;
  timestamp: Date;
  description: string;
  commands: Command[];
  rollbackOnFailure: boolean;
  executedCommands: Command[];
  metadata?: Record<string, any>;

  private currentState: 'pending' | 'executing' | 'completed' | 'failed' | 'rolled-back' = 'pending';
  private executionError?: Error;
  private options: TransactionOptions;

  constructor(commands: Command[], options: TransactionOptions = { rollbackOnFailure: true, stopOnFirstError: true }) {
    this.id = this.generateId();
    this.timestamp = new Date();
    this.commands = commands;
    this.rollbackOnFailure = options.rollbackOnFailure;
    this.executedCommands = [];
    this.description = `Transaction (${commands.length} commands)`;
    this.options = options;
    this.metadata = {
      ...options.metadata,
      commandCount: commands.length,
      rollbackOnFailure: options.rollbackOnFailure,
    };
  }

  async execute(): Promise<void> {
    if (this.currentState === 'executing') {
      throw new Error('Transaction is already executing');
    }

    this.currentState = 'executing';
    this.executedCommands = [];

    try {
      for (const command of this.commands) {
        await command.execute();
        this.executedCommands.push(command);

        if (this.options.timeout) {
          await this.checkTimeout(this.options.timeout);
        }
      }

      this.currentState = 'completed';
    } catch (error) {
      this.executionError = error as Error;
      this.currentState = 'failed';

      if (this.rollbackOnFailure) {
        await this.rollback();
        throw new Error(
          `Transaction failed and rolled back: ${(error as Error).message}`
        );
      }

      throw error;
    }
  }

  async undo(): Promise<void> {
    if (this.currentState === 'rolled-back') {
      return; // Already rolled back
    }

    // Undo in reverse order
    for (let i = this.executedCommands.length - 1; i >= 0; i--) {
      try {
        await this.executedCommands[i].undo();
      } catch (error) {
        console.error(`Error during transaction undo at step ${i}:`, error);
        if (this.options.stopOnFirstError) {
          throw error;
        }
      }
    }

    this.currentState = 'rolled-back';
  }

  async redo(): Promise<void> {
    // Re-execute all commands
    for (const command of this.executedCommands) {
      await command.redo();
    }
    this.currentState = 'completed';
  }

  private async rollback(): Promise<void> {
    console.log('Rolling back transaction...');
    for (let i = this.executedCommands.length - 1; i >= 0; i--) {
      try {
        await this.executedCommands[i].undo();
      } catch (error) {
        console.error(`Error during rollback at step ${i}:`, error);
        // Continue rolling back despite errors
      }
    }
    this.currentState = 'rolled-back';
  }

  private async checkTimeout(timeout: number): Promise<void> {
    return new Promise((resolve) => {
      setTimeout(() => {
        if (this.currentState === 'executing') {
          throw new Error(`Transaction timeout after ${timeout}ms`);
        }
        resolve();
      }, timeout);
    });
  }

  getStatus(): string {
    return this.currentState;
  }

  getExecutionError(): Error | undefined {
    return this.executionError;
  }

  private generateId(): string {
    return `txn_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}

/**
 * Utility function to create a transaction command
 */
export function createTransactionCommand(
  commands: Command[],
  options?: TransactionOptions
): TransactionCommand {
  return new TransactionCommand(commands, options);
}
