// Conditional Command Implementation

import { Command, IConditionalCommand } from '../types/commands';

/**
 * Conditional command for conditional execution based on predicates
 */
export class ConditionalCommand implements IConditionalCommand {
  id: string;
  timestamp: Date;
  description: string;
  condition: () => boolean | Promise<boolean>;
  thenCommand: Command;
  elseCommand?: Command;
  conditionResult?: boolean;
  metadata?: Record<string, any>;

  private executedCommand?: Command;

  constructor(
    condition: () => boolean | Promise<boolean>,
    thenCommand: Command,
    elseCommand?: Command
  ) {
    this.id = this.generateId();
    this.timestamp = new Date();
    this.condition = condition;
    this.thenCommand = thenCommand;
    this.elseCommand = elseCommand;
    this.description = 'Conditional Command';
    this.metadata = {
      hasElse: !!elseCommand,
    };
  }

  async execute(): Promise<void> {
    this.conditionResult = await this.evaluateCondition();

    if (this.conditionResult) {
      this.executedCommand = this.thenCommand;
      await this.thenCommand.execute();
    } else if (this.elseCommand) {
      this.executedCommand = this.elseCommand;
      await this.elseCommand.execute();
    }
  }

  async undo(): Promise<void> {
    if (this.executedCommand) {
      await this.executedCommand.undo();
    }
  }

  async redo(): Promise<void> {
    if (this.executedCommand) {
      await this.executedCommand.redo();
    }
  }

  private async evaluateCondition(): Promise<boolean> {
    try {
      return await this.condition();
    } catch (error) {
      console.error('Error evaluating condition:', error);
      return false;
    }
  }

  getExecutedBranch(): Command | undefined {
    return this.executedCommand;
  }

  private generateId(): string {
    return `cond_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}

/**
 * Utility function to create a conditional command
 */
export function createConditionalCommand(
  condition: () => boolean | Promise<boolean>,
  thenCommand: Command,
  elseCommand?: Command
): ConditionalCommand {
  return new ConditionalCommand(condition, thenCommand, elseCommand);
}

/**
 * Create a conditional command based on entity state
 */
export function createEntityStateConditionalCommand(
  entityCheck: () => boolean,
  thenCommand: Command,
  elseCommand?: Command
): ConditionalCommand {
  return createConditionalCommand(entityCheck, thenCommand, elseCommand);
}

/**
 * Create a conditional command based on user confirmation
 */
export function createConfirmationConditionalCommand(
  message: string,
  thenCommand: Command,
  elseCommand?: Command
): ConditionalCommand {
  const condition = () => {
    return Promise.resolve(confirm(message));
  };

  return createConditionalCommand(condition, thenCommand, elseCommand);
}
