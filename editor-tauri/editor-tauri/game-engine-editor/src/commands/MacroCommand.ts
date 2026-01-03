// Macro Command Implementation

import { Command, IMacroCommand } from '../types/commands';

/**
 * Macro command for reusable command sequences with parameters
 */
export class MacroCommand implements IMacroCommand {
  id: string;
  timestamp: Date;
  description: string;
  commands: Command[];
  name: string;
  parameters: Record<string, any>;
  template?: boolean;
  metadata?: Record<string, any>;

  private boundCommands: Command[] = [];

  constructor(
    name: string,
    commands: Command[],
    parameters: Record<string, any> = {},
    template: boolean = false
  ) {
    this.id = this.generateId();
    this.timestamp = new Date();
    this.name = name;
    this.commands = commands;
    this.parameters = parameters;
    this.template = template;
    this.description = `Macro: ${name}`;
    this.metadata = {
      name,
      parameterCount: Object.keys(parameters).length,
      commandCount: commands.length,
      template,
    };

    if (!template) {
      this.bindCommands();
    }
  }

  async execute(params?: Record<string, any>): Promise<void> {
    const effectiveParams = { ...this.parameters, ...params };

    for (const command of this.boundCommands.length > 0 ? this.boundCommands : this.commands) {
      if (this.isParameterizedCommand(command)) {
        await this.executeParameterizedCommand(command, effectiveParams);
      } else {
        await command.execute();
      }
    }
  }

  async undo(): Promise<void> {
    for (let i = this.boundCommands.length - 1; i >= 0; i--) {
      const command = this.boundCommands[i];
      if (this.isParameterizedCommand(command)) {
        await command.undo();
      } else {
        await command.undo();
      }
    }
  }

  async redo(): Promise<void> {
    for (const command of this.boundCommands.length > 0 ? this.boundCommands : this.commands) {
      await command.redo();
    }
  }

  /**
   * Bind parameters to commands for template macros
   */
  private bindCommands(): void {
    this.boundCommands = this.commands.map(cmd => {
      if (this.isParameterizedCommand(cmd)) {
        return this.bindCommandParameters(cmd, this.parameters);
      }
      return cmd;
    });
  }

  private isParameterizedCommand(command: any): command is ParameterizedCommand {
    return command && typeof command.bind === 'function';
  }

  private executeParameterizedCommand(
    command: ParameterizedCommand,
    params: Record<string, any>
  ): Promise<void> {
    const bound = command.bind(params);
    return bound.execute();
  }

  private bindCommandParameters(
    command: ParameterizedCommand,
    params: Record<string, any>
  ): Command {
    return command.bind(params);
  }

  /**
   * Create a new instance with different parameters
   */
  withParameters(parameters: Record<string, any>): MacroCommand {
    return new MacroCommand(this.name, this.commands, { ...this.parameters, ...parameters });
  }

  /**
   * Export macro definition for serialization
   */
  toJSON(): any {
    return {
      name: this.name,
      parameters: this.parameters,
      commandCount: this.commands.length,
      template: this.template,
      id: this.id,
    };
  }

  private generateId(): string {
    return `macro_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}

/**
 * Parameterized command interface for template binding
 */
export interface ParameterizedCommand extends Command {
  bind(parameters: Record<string, any>): Command;
}

/**
 * Utility function to create a macro command
 */
export function createMacroCommand(
  name: string,
  commands: Command[],
  parameters?: Record<string, any>
): MacroCommand {
  return new MacroCommand(name, commands, parameters);
}

/**
 * Utility to create a parameterized command
 */
export function createParameterizedCommand<T extends Command>(
  commandFactory: (params: Record<string, any>) => T
): ParameterizedCommand {
  return {
    ...commandFactory({}),
    bind(params: Record<string, any>): Command {
      return commandFactory(params);
    },
    execute() {
      return this.execute();
    },
    undo() {
      return this.undo();
    },
    redo() {
      return this.redo();
    },
  } as ParameterizedCommand;
}
