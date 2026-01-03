// Command Pattern for Undo/Redo functionality

import { Entity } from './engine';

export interface Command {
  execute(): void | Promise<void>;
  undo(): void | Promise<void>;
  redo(): void | Promise<void>;
  description: string;
  id?: string;
  timestamp?: Date;
  metadata?: Record<string, any>;
}

/**
 * Batch command for executing multiple commands together
 */
export interface IBatchCommand extends Command {
  commands: Command[];
  executeMode: 'sequential' | 'parallel';
}

/**
 * Transaction command with rollback capability
 */
export interface ITransactionCommand extends Command {
  commands: Command[];
  rollbackOnFailure: boolean;
  executedCommands: Command[];
}

/**
 * Macro command for reusable command sequences
 */
export interface IMacroCommand extends Command {
  commands: Command[];
  name: string;
  parameters: Record<string, any>;
  template?: boolean;
}

/**
 * Conditional command for conditional execution
 */
export interface IConditionalCommand extends Command {
  condition: () => boolean | Promise<boolean>;
  thenCommand: Command;
  elseCommand?: Command;
  conditionResult?: boolean;
}

/**
 * Delayed command for deferred execution
 */
export interface IDelayedCommand extends Command {
  command: Command;
  delay: number; // milliseconds
  executeAt: Date;
  scheduled: boolean;
}

/**
 * Periodic save command for auto-save
 */
export interface IPeriodicSaveCommand extends Command {
  interval: number; // milliseconds
  lastSave: Date;
  saveCallback: () => Promise<void>;
}

// Command for creating an entity
export class CreateEntityCommand implements Command {
  description = 'Create Entity';

  constructor(
    private entityId: string,
    private entity: Entity,
    private addEntity: (entity: Entity) => void,
    private removeEntity: (entityId: string) => void
  ) {}

  execute(): void {
    this.addEntity(this.entity);
  }

  undo(): void {
    this.removeEntity(this.entityId);
  }

  redo(): void {
    this.execute();
  }
}

// Command for deleting an entity
export class DeleteEntityCommand implements Command {
  description = 'Delete Entity';

  constructor(
    private entity: Entity,
    private removeEntity: (entityId: string) => void,
    private addEntity: (entity: Entity) => void
  ) {}

  execute(): void {
    this.removeEntity(this.entity.id);
  }

  undo(): void {
    this.addEntity(this.entity);
  }

  redo(): void {
    this.execute();
  }
}

// Command for renaming an entity
export class RenameEntityCommand implements Command {
  description = 'Rename Entity';

  constructor(
    private entityId: string,
    private oldName: string,
    private newName: string,
    private updateEntityName: (entityId: string, name: string) => void
  ) {}

  execute(): void {
    this.updateEntityName(this.entityId, this.newName);
  }

  undo(): void {
    this.updateEntityName(this.entityId, this.oldName);
  }

  redo(): void {
    this.execute();
  }
}

// Command for transforming an entity
export class TransformEntityCommand implements Command {
  description = 'Transform Entity';

  constructor(
    private entityId: string,
    private oldTransform: Entity['transform'],
    private newTransform: Entity['transform'],
    private updateTransform: (entityId: string, transform: Entity['transform']) => void
  ) {}

  execute(): void {
    this.updateTransform(this.entityId, this.newTransform);
  }

  undo(): void {
    this.updateTransform(this.entityId, this.oldTransform);
  }

  redo(): void {
    this.execute();
  }
}

// Command for duplicating an entity
export class DuplicateEntityCommand implements Command {
  description = 'Duplicate Entity';

  constructor(
    private newEntity: Entity,
    private addEntity: (entity: Entity) => void,
    private removeEntity: (entityId: string) => void
  ) {}

  execute(): void {
    this.addEntity(this.newEntity);
  }

  undo(): void {
    this.removeEntity(this.newEntity.id);
  }

  redo(): void {
    this.execute();
  }
}

// Command for toggling entity visibility
export class ToggleVisibilityCommand implements Command {
  description = 'Toggle Visibility';

  constructor(
    private entityId: string,
    private oldVisibility: boolean,
    private newVisibility: boolean,
    private toggleVisibility: (entityId: string) => void
  ) {}

  execute(): void {
    if (this.newVisibility !== this.oldVisibility) {
      this.toggleVisibility(this.entityId);
    }
  }

  undo(): void {
    if (this.newVisibility !== this.oldVisibility) {
      this.toggleVisibility(this.entityId);
    }
  }

  redo(): void {
    this.execute();
  }
}

// Command for toggling entity lock
export class ToggleLockCommand implements Command {
  description = 'Toggle Lock';

  constructor(
    private entityId: string,
    private oldLock: boolean,
    private newLock: boolean,
    private toggleLock: (entityId: string) => void
  ) {}

  execute(): void {
    if (this.newLock !== this.oldLock) {
      this.toggleLock(this.entityId);
    }
  }

  undo(): void {
    if (this.newLock !== this.oldLock) {
      this.toggleLock(this.entityId);
    }
  }

  redo(): void {
    this.execute();
  }
}

// Composite command for batching multiple commands
export class CompositeCommand implements Command {
  description = 'Composite Command';

  constructor(private commands: Command[]) {
    if (commands.length > 0) {
      this.description = `Batch (${commands.length} commands)`;
    }
  }

  async execute(): Promise<void> {
    for (const command of this.commands) {
      await command.execute();
    }
  }

  async undo(): Promise<void> {
    for (let i = this.commands.length - 1; i >= 0; i--) {
      await this.commands[i].undo();
    }
  }

  async redo(): Promise<void> {
    for (const command of this.commands) {
      await command.redo();
    }
  }
}
