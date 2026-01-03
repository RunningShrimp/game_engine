// Batch Operation Manager

import { invoke } from '@tauri-apps/api/tauri';
import { Entity, Vector3, Quaternion } from '../types/engine';
import { Command, CompositeCommand } from '../types/commands';
import {
  BatchOperationOptions,
  BulkEditResult,
  RenamePattern,
  MaterialBatchOperation,
  ComponentBatchOperation,
} from '../types/selection';
import { SelectionManager } from './SelectionManager';
import { HistoryManager } from './HistoryManager';

export class BatchOperationManager {
  constructor(
    private selectionManager: SelectionManager,
    private historyManager: HistoryManager,
    private getEntity: (id: string) => Entity | undefined,
    private updateEntity: (id: string, updates: Partial<Entity>) => void,
    private addEntity: (entity: Entity) => void,
    private removeEntity: (id: string) => void
  ) {}

  // ==================== Batch Delete ====================

  /**
   * Delete all selected entities
   */
  async batchDelete(options?: BatchOperationOptions): Promise<BulkEditResult> {
    const selectedIds = this.selectionManager.getSelectedIds();

    if (selectedIds.length === 0) {
      return {
        succeeded: [],
        failed: [],
        skipped: [],
        totalAffected: 0,
      };
    }

    // Check confirmation threshold
    if (
      options?.confirmThreshold &&
      selectedIds.length >= options.confirmThreshold
    ) {
      const confirmed = await this.requestConfirmation(
        `Delete ${selectedIds.length} entities?`,
        'This action cannot be undone.'
      );
      if (!confirmed) {
        return {
          succeeded: [],
          failed: [],
          skipped: selectedIds,
          totalAffected: 0,
        };
      }
    }

    const result: BulkEditResult = {
      succeeded: [],
      failed: [],
      skipped: [],
      totalAffected: 0,
    };

    const commands: Command[] = [];

    for (const id of selectedIds) {
      try {
        const entity = this.getEntity(id);
        if (!entity) {
          result.skipped.push(id);
          continue;
        }

        if (entity.locked) {
          result.skipped.push(id);
          continue;
        }

        // Create delete command
        const command = await this.createDeleteCommand(entity);
        commands.push(command);

        result.succeeded.push(id);
        this.reportProgress(options, result.succeeded.length, selectedIds.length);
      } catch (error) {
        result.failed.push({
          id,
          error: error instanceof Error ? error.message : 'Unknown error',
        });
      }
    }

    // Execute composite command
    if (commands.length > 0) {
      const compositeCommand = new CompositeCommand(commands);
      compositeCommand.description = options?.undoName || `Delete ${commands.length} entities`;
      await this.historyManager.executeCommand(compositeCommand);
      this.selectionManager.deselectAll();
    }

    result.totalAffected = result.succeeded.length;
    return result;
  }

  // ==================== Batch Rename ====================

  /**
   * Rename all selected entities
   */
  async batchRename(
    pattern: RenamePattern,
    options?: BatchOperationOptions
  ): Promise<BulkEditResult> {
    const selectedIds = this.selectionManager.getSelectedIds();
    const primarySelection = this.selectionManager.getPrimarySelection();

    if (selectedIds.length === 0) {
      return {
        succeeded: [],
        failed: [],
        skipped: [],
        totalAffected: 0,
      };
    }

    const result: BulkEditResult = {
      succeeded: [],
      failed: [],
      skipped: [],
      totalAffected: 0,
    };

    const commands: Command[] = [];
    const startNumber = pattern.startNumber || 1;
    const padding = pattern.padding || 3;

    for (let i = 0; i < selectedIds.length; i++) {
      const id = selectedIds[i];
      try {
        const entity = this.getEntity(id);
        if (!entity) {
          result.skipped.push(id);
          continue;
        }

        const oldName = entity.name;
        let newName: string;

        switch (pattern.mode) {
          case 'prefix':
            newName = `${pattern.value}_${oldName}`;
            break;
          case 'suffix':
            newName = `${oldName}_${pattern.value}`;
            break;
          case 'replace':
            newName = pattern.value;
            break;
          case 'number':
            const number = startNumber + i;
            const paddedNumber = String(number).padStart(padding, '0');
            newName = `${pattern.value}_${paddedNumber}`;
            break;
          default:
            newName = oldName;
        }

        const command = await this.createRenameCommand(id, oldName, newName);
        commands.push(command);

        result.succeeded.push(id);
        this.reportProgress(options, i + 1, selectedIds.length);
      } catch (error) {
        result.failed.push({
          id,
          error: error instanceof Error ? error.message : 'Unknown error',
        });
      }
    }

    // Execute composite command
    if (commands.length > 0) {
      const compositeCommand = new CompositeCommand(commands);
      compositeCommand.description = options?.undoName || `Rename ${commands.length} entities`;
      await this.historyManager.executeCommand(compositeCommand);
    }

    result.totalAffected = result.succeeded.length;
    return result;
  }

  // ==================== Batch Move ====================

  /**
   * Move all selected entities
   */
  async batchMove(
    offset: Vector3,
    options?: BatchOperationOptions
  ): Promise<BulkEditResult> {
    const selectedIds = this.selectionManager.getSelectedIds();

    if (selectedIds.length === 0) {
      return {
        succeeded: [],
        failed: [],
        skipped: [],
        totalAffected: 0,
      };
    }

    const result: BulkEditResult = {
      succeeded: [],
      failed: [],
      skipped: [],
      totalAffected: 0,
    };

    const commands: Command[] = [];

    for (let i = 0; i < selectedIds.length; i++) {
      const id = selectedIds[i];
      try {
        const entity = this.getEntity(id);
        if (!entity) {
          result.skipped.push(id);
          continue;
        }

        if (entity.locked) {
          result.skipped.push(id);
          continue;
        }

        const oldTransform = { ...entity.transform };
        const newTransform = {
          ...entity.transform,
          position: {
            x: entity.transform.position.x + offset.x,
            y: entity.transform.position.y + offset.y,
            z: entity.transform.position.z + offset.z,
          },
        };

        const command = await this.createTransformCommand(id, oldTransform, newTransform);
        commands.push(command);

        result.succeeded.push(id);
        this.reportProgress(options, i + 1, selectedIds.length);
      } catch (error) {
        result.failed.push({
          id,
          error: error instanceof Error ? error.message : 'Unknown error',
        });
      }
    }

    // Execute composite command
    if (commands.length > 0) {
      const compositeCommand = new CompositeCommand(commands);
      compositeCommand.description = options?.undoName || `Move ${commands.length} entities`;
      await this.historyManager.executeCommand(compositeCommand);
    }

    result.totalAffected = result.succeeded.length;
    return result;
  }

  // ==================== Batch Rotate ====================

  /**
   * Rotate all selected entities
   */
  async batchRotate(
    rotation: Vector3, // Euler angles in radians
    space: 'world' | 'local' = 'local',
    options?: BatchOperationOptions
  ): Promise<BulkEditResult> {
    const selectedIds = this.selectionManager.getSelectedIds();

    if (selectedIds.length === 0) {
      return {
        succeeded: [],
        failed: [],
        skipped: [],
        totalAffected: 0,
      };
    }

    const result: BulkEditResult = {
      succeeded: [],
      failed: [],
      skipped: [],
      totalAffected: 0,
    };

    const commands: Command[] = [];

    for (let i = 0; i < selectedIds.length; i++) {
      const id = selectedIds[i];
      try {
        const entity = this.getEntity(id);
        if (!entity) {
          result.skipped.push(id);
          continue;
        }

        if (entity.locked) {
          result.skipped.push(id);
          continue;
        }

        const oldTransform = { ...entity.transform };
        const newRotation = this.applyRotation(
          entity.transform.rotation,
          rotation,
          space
        );

        const newTransform = {
          ...entity.transform,
          rotation: newRotation,
        };

        const command = await this.createTransformCommand(id, oldTransform, newTransform);
        commands.push(command);

        result.succeeded.push(id);
        this.reportProgress(options, i + 1, selectedIds.length);
      } catch (error) {
        result.failed.push({
          id,
          error: error instanceof Error ? error.message : 'Unknown error',
        });
      }
    }

    // Execute composite command
    if (commands.length > 0) {
      const compositeCommand = new CompositeCommand(commands);
      compositeCommand.description = options?.undoName || `Rotate ${commands.length} entities`;
      await this.historyManager.executeCommand(compositeCommand);
    }

    result.totalAffected = result.succeeded.length;
    return result;
  }

  // ==================== Batch Scale ====================

  /**
   * Scale all selected entities
   */
  async batchScale(
    scale: Vector3,
    options?: BatchOperationOptions
  ): Promise<BulkEditResult> {
    const selectedIds = this.selectionManager.getSelectedIds();

    if (selectedIds.length === 0) {
      return {
        succeeded: [],
        failed: [],
        skipped: [],
        totalAffected: 0,
      };
    }

    const result: BulkEditResult = {
      succeeded: [],
      failed: [],
      skipped: [],
      totalAffected: 0,
    };

    const commands: Command[] = [];

    for (let i = 0; i < selectedIds.length; i++) {
      const id = selectedIds[i];
      try {
        const entity = this.getEntity(id);
        if (!entity) {
          result.skipped.push(id);
          continue;
        }

        if (entity.locked) {
          result.skipped.push(id);
          continue;
        }

        const oldTransform = { ...entity.transform };
        const newTransform = {
          ...entity.transform,
          scale: {
            x: entity.transform.scale.x * scale.x,
            y: entity.transform.scale.y * scale.y,
            z: entity.transform.scale.z * scale.z,
          },
        };

        const command = await this.createTransformCommand(id, oldTransform, newTransform);
        commands.push(command);

        result.succeeded.push(id);
        this.reportProgress(options, i + 1, selectedIds.length);
      } catch (error) {
        result.failed.push({
          id,
          error: error instanceof Error ? error.message : 'Unknown error',
        });
      }
    }

    // Execute composite command
    if (commands.length > 0) {
      const compositeCommand = new CompositeCommand(commands);
      compositeCommand.description = options?.undoName || `Scale ${commands.length} entities`;
      await this.historyManager.executeCommand(compositeCommand);
    }

    result.totalAffected = result.succeeded.length;
    return result;
  }

  // ==================== Batch Toggle Enabled ====================

  /**
   * Enable or disable all selected entities
   */
  async batchToggleEnabled(
    enabled: boolean,
    options?: BatchOperationOptions
  ): Promise<BulkEditResult> {
    const selectedIds = this.selectionManager.getSelectedIds();

    if (selectedIds.length === 0) {
      return {
        succeeded: [],
        failed: [],
        skipped: [],
        totalAffected: 0,
      };
    }

    const result: BulkEditResult = {
      succeeded: [],
      failed: [],
      skipped: [],
      totalAffected: 0,
    };

    const commands: Command[] = [];

    for (let i = 0; i < selectedIds.length; i++) {
      const id = selectedIds[i];
      try {
        const entity = this.getEntity(id);
        if (!entity) {
          result.skipped.push(id);
          continue;
        }

        // Assuming entities have an 'enabled' property
        // Adjust if your Entity type doesn't have this
        const oldEnabled = (entity as any).enabled ?? true;

        if (oldEnabled === enabled) {
          result.skipped.push(id);
          continue;
        }

        const command = await this.createToggleEnabledCommand(id, oldEnabled, enabled);
        commands.push(command);

        result.succeeded.push(id);
        this.reportProgress(options, i + 1, selectedIds.length);
      } catch (error) {
        result.failed.push({
          id,
          error: error instanceof Error ? error.message : 'Unknown error',
        });
      }
    }

    // Execute composite command
    if (commands.length > 0) {
      const compositeCommand = new CompositeCommand(commands);
      compositeCommand.description = options?.undoName || `${enabled ? 'Enable' : 'Disable'} ${commands.length} entities`;
      await this.historyManager.executeCommand(compositeCommand);
    }

    result.totalAffected = result.succeeded.length;
    return result;
  }

  // ==================== Batch Toggle Visibility ====================

  /**
   * Show or hide all selected entities
   */
  async batchToggleVisibility(
    visible: boolean,
    options?: BatchOperationOptions
  ): Promise<BulkEditResult> {
    const selectedIds = this.selectionManager.getSelectedIds();

    if (selectedIds.length === 0) {
      return {
        succeeded: [],
        failed: [],
        skipped: [],
        totalAffected: 0,
      };
    }

    const result: BulkEditResult = {
      succeeded: [],
      failed: [],
      skipped: [],
      totalAffected: 0,
    };

    const commands: Command[] = [];

    for (let i = 0; i < selectedIds.length; i++) {
      const id = selectedIds[i];
      try {
        const entity = this.getEntity(id);
        if (!entity) {
          result.skipped.push(id);
          continue;
        }

        const oldVisibility = entity.visible;

        if (oldVisibility === visible) {
          result.skipped.push(id);
          continue;
        }

        const command = await this.createToggleVisibilityCommand(id, oldVisibility, visible);
        commands.push(command);

        result.succeeded.push(id);
        this.reportProgress(options, i + 1, selectedIds.length);
      } catch (error) {
        result.failed.push({
          id,
          error: error instanceof Error ? error.message : 'Unknown error',
        });
      }
    }

    // Execute composite command
    if (commands.length > 0) {
      const compositeCommand = new CompositeCommand(commands);
      compositeCommand.description = options?.undoName || `${visible ? 'Show' : 'Hide'} ${commands.length} entities`;
      await this.historyManager.executeCommand(compositeCommand);
    }

    result.totalAffected = result.succeeded.length;
    return result;
  }

  // ==================== Batch Toggle Locked ====================

  /**
   * Lock or unlock all selected entities
   */
  async batchToggleLocked(
    locked: boolean,
    options?: BatchOperationOptions
  ): Promise<BulkEditResult> {
    const selectedIds = this.selectionManager.getSelectedIds();

    if (selectedIds.length === 0) {
      return {
        succeeded: [],
        failed: [],
        skipped: [],
        totalAffected: 0,
      };
    }

    const result: BulkEditResult = {
      succeeded: [],
      failed: [],
      skipped: [],
      totalAffected: 0,
    };

    const commands: Command[] = [];

    for (let i = 0; i < selectedIds.length; i++) {
      const id = selectedIds[i];
      try {
        const entity = this.getEntity(id);
        if (!entity) {
          result.skipped.push(id);
          continue;
        }

        const oldLock = entity.locked;

        if (oldLock === locked) {
          result.skipped.push(id);
          continue;
        }

        const command = await this.createToggleLockCommand(id, oldLock, locked);
        commands.push(command);

        result.succeeded.push(id);
        this.reportProgress(options, i + 1, selectedIds.length);
      } catch (error) {
        result.failed.push({
          id,
          error: error instanceof Error ? error.message : 'Unknown error',
        });
      }
    }

    // Execute composite command
    if (commands.length > 0) {
      const compositeCommand = new CompositeCommand(commands);
      compositeCommand.description = options?.undoName || `${locked ? 'Lock' : 'Unlock'} ${commands.length} entities`;
      await this.historyManager.executeCommand(compositeCommand);
    }

    result.totalAffected = result.succeeded.length;
    return result;
  }

  // ==================== Batch Apply Material ====================

  /**
   * Apply material to all selected entities
   */
  async batchApplyMaterial(
    operation: MaterialBatchOperation,
    options?: BatchOperationOptions
  ): Promise<BulkEditResult> {
    const selectedIds = this.selectionManager.getSelectedIds();

    if (selectedIds.length === 0) {
      return {
        succeeded: [],
        failed: [],
        skipped: [],
        totalAffected: 0,
      };
    }

    // Call Rust backend for material operation
    try {
      const result = await invoke<BulkEditResult>('batch_apply_material', {
        ids: selectedIds,
        operation,
      });

      // Create undo command
      const command = await this.createMaterialCommand(selectedIds, operation);
      await this.historyManager.executeCommand(command);

      return result;
    } catch (error) {
      console.error('Batch apply material error:', error);
      return {
        succeeded: [],
        failed: selectedIds.map((id) => ({
          id,
          error: error instanceof Error ? error.message : 'Unknown error',
        })),
        skipped: [],
        totalAffected: 0,
      };
    }
  }

  // ==================== Batch Component Operations ====================

  /**
   * Perform batch component operation
   */
  async batchComponentOperation(
    operation: ComponentBatchOperation,
    options?: BatchOperationOptions
  ): Promise<BulkEditResult> {
    const selectedIds = this.selectionManager.getSelectedIds();

    if (selectedIds.length === 0) {
      return {
        succeeded: [],
        failed: [],
        skipped: [],
        totalAffected: 0,
      };
    }

    // Call Rust backend for component operation
    try {
      const result = await invoke<BulkEditResult>('batch_component_operation', {
        ids: selectedIds,
        operation,
      });

      // Create undo command
      const command = await this.createComponentCommand(selectedIds, operation);
      await this.historyManager.executeCommand(command);

      return result;
    } catch (error) {
      console.error('Batch component operation error:', error);
      return {
        succeeded: [],
        failed: selectedIds.map((id) => ({
          id,
          error: error instanceof Error ? error.message : 'Unknown error',
        })),
        skipped: [],
        totalAffected: 0,
      };
    }
  }

  // ==================== Private Helpers ====================

  /**
   * Request user confirmation
   */
  private async requestConfirmation(title: string, message: string): Promise<boolean> {
    // Implement confirmation dialog
    // For now, return true (auto-confirm)
    return true;
  }

  /**
   * Report progress
   */
  private reportProgress(
    options: BatchOperationOptions | undefined,
    current: number,
    total: number
  ): void {
    if (options?.progressCallback) {
      options.progressCallback(current, total);
    }
  }

  /**
   * Apply rotation to quaternion
   */
  private applyRotation(
    rotation: Quaternion,
    euler: Vector3,
    space: 'world' | 'local'
  ): Quaternion {
    // Simplified rotation application
    // In a real implementation, you'd use proper quaternion math
    const cosX = Math.cos(euler.x / 2);
    const sinX = Math.sin(euler.x / 2);
    const cosY = Math.cos(euler.y / 2);
    const sinY = Math.sin(euler.y / 2);
    const cosZ = Math.cos(euler.z / 2);
    const sinZ = Math.sin(euler.z / 2);

    if (space === 'local') {
      // Local rotation: multiply quaternions
      return {
        x: rotation.x * cosX * cosY * cosZ + rotation.y * sinX * cosY * cosZ,
        y: rotation.y * cosX * cosY * cosZ - rotation.x * sinX * cosY * cosZ,
        z: rotation.z * cosX * cosY * cosZ + rotation.w * sinZ,
        w: rotation.w * cosX * cosY * cosZ - rotation.z * sinZ,
      };
    } else {
      // World rotation
      return {
        x: sinX * cosY * cosZ + cosX * sinY * sinZ,
        y: cosX * sinY * cosZ - sinX * cosY * sinZ,
        z: cosX * cosY * sinZ - sinX * sinY * cosZ,
        w: cosX * cosY * cosZ + sinX * sinY * sinZ,
      };
    }
  }

  /**
   * Create delete command
   */
  private async createDeleteCommand(entity: Entity): Promise<Command> {
    // Import command classes dynamically to avoid circular dependency
    const { DeleteEntityCommand } = await import('../types/commands');
    return new DeleteEntityCommand(
      entity,
      this.removeEntity,
      (e: Entity) => this.addEntity(e)
    );
  }

  /**
   * Create rename command
   */
  private async createRenameCommand(
    entityId: string,
    oldName: string,
    newName: string
  ): Promise<Command> {
    const { RenameEntityCommand } = await import('../types/commands');
    return new RenameEntityCommand(entityId, oldName, newName, (id, name) => {
      this.updateEntity(id, { name });
    });
  }

  /**
   * Create transform command
   */
  private async createTransformCommand(
    entityId: string,
    oldTransform: Entity['transform'],
    newTransform: Entity['transform']
  ): Promise<Command> {
    const { TransformEntityCommand } = await import('../types/commands');
    return new TransformEntityCommand(
      entityId,
      oldTransform,
      newTransform,
      (id, transform) => {
        this.updateEntity(id, { transform });
      }
    );
  }

  /**
   * Create toggle visibility command
   */
  private async createToggleVisibilityCommand(
    entityId: string,
    oldVisibility: boolean,
    newVisibility: boolean
  ): Promise<Command> {
    const { ToggleVisibilityCommand } = await import('../types/commands');
    return new ToggleVisibilityCommand(
      entityId,
      oldVisibility,
      newVisibility,
      (id) => {
        const entity = this.getEntity(id);
        if (entity) {
          this.updateEntity(id, { visible: !entity.visible });
        }
      }
    );
  }

  /**
   * Create toggle lock command
   */
  private async createToggleLockCommand(
    entityId: string,
    oldLock: boolean,
    newLock: boolean
  ): Promise<Command> {
    const { ToggleLockCommand } = await import('../types/commands');
    return new ToggleLockCommand(
      entityId,
      oldLock,
      newLock,
      (id) => {
        const entity = this.getEntity(id);
        if (entity) {
          this.updateEntity(id, { locked: !entity.locked });
        }
      }
    );
  }

  /**
   * Create toggle enabled command
   */
  private async createToggleEnabledCommand(
    entityId: string,
    oldEnabled: boolean,
    newEnabled: boolean
  ): Promise<Command> {
    // Implement if you have an enabled property
    const command: Command = {
      description: 'Toggle Enabled',
      execute: async () => {
        this.updateEntity(entityId, { enabled: newEnabled } as any);
      },
      undo: async () => {
        this.updateEntity(entityId, { enabled: oldEnabled } as any);
      },
      redo: async () => {
        this.updateEntity(entityId, { enabled: newEnabled } as any);
      },
    };
    return command;
  }

  /**
   * Create material command
   */
  private async createMaterialCommand(
    entityIds: string[],
    operation: MaterialBatchOperation
  ): Promise<Command> {
    const command: Command = {
      description: 'Material Operation',
      execute: async () => {
        await invoke('batch_apply_material', { ids: entityIds, operation });
      },
      undo: async () => {
        // Implement undo for material operations
        await invoke('batch_apply_material_undo', { ids: entityIds, operation });
      },
      redo: async () => {
        await invoke('batch_apply_material', { ids: entityIds, operation });
      },
    };
    return command;
  }

  /**
   * Create component command
   */
  private async createComponentCommand(
    entityIds: string[],
    operation: ComponentBatchOperation
  ): Promise<Command> {
    const command: Command = {
      description: 'Component Operation',
      execute: async () => {
        await invoke('batch_component_operation', { ids: entityIds, operation });
      },
      undo: async () => {
        // Implement undo for component operations
        await invoke('batch_component_operation_undo', { ids: entityIds, operation });
      },
      redo: async () => {
        await invoke('batch_component_operation', { ids: entityIds, operation });
      },
    };
    return command;
  }
}
