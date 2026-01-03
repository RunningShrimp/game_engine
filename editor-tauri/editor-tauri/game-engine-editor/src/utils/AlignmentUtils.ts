// Alignment and Distribution Utilities

import { Entity, Vector3 } from '../types/engine';
import { AlignmentOptions, DistributionOptions } from '../types/selection';

export class AlignmentUtils {
  /**
   * Align entities to specified target
   */
  static alignEntities(
    entities: Entity[],
    options: AlignmentOptions
  ): Record<string, Partial<Entity>> {
    if (entities.length === 0) return {};

    const updates: Record<string, Partial<Entity>> = {};
    const targetEntity = options.target
      ? entities.find((e) => e.id === options.target)
      : entities[0];

    if (!targetEntity) return {};

    const targetValue = this.getTargetValue(
      targetEntity.transform.position,
      options
    );

    entities.forEach((entity) => {
      if (entity.id === targetEntity?.id) return;
      if (entity.locked) return;

      const newPosition = { ...entity.transform.position };

      switch (options.axis) {
        case 'x':
          newPosition.x = targetValue;
          break;
        case 'y':
          newPosition.y = targetValue;
          break;
        case 'z':
          newPosition.z = targetValue;
          break;
        case 'all':
          newPosition.x = targetValue;
          newPosition.y = targetValue;
          newPosition.z = targetValue;
          break;
      }

      updates[entity.id] = {
        transform: {
          ...entity.transform,
          position: newPosition,
        },
      };
    });

    return updates;
  }

  /**
   * Align entities to grid
   */
  static alignToGrid(
    entities: Entity[],
    gridSize: number
  ): Record<string, Partial<Entity>> {
    const updates: Record<string, Partial<Entity>> = {};

    entities.forEach((entity) => {
      if (entity.locked) return;

      const newPosition = {
        x: Math.round(entity.transform.position.x / gridSize) * gridSize,
        y: Math.round(entity.transform.position.y / gridSize) * gridSize,
        z: Math.round(entity.transform.position.z / gridSize) * gridSize,
      };

      updates[entity.id] = {
        transform: {
          ...entity.transform,
          position: newPosition,
        },
      };
    });

    return updates;
  }

  /**
   * Distribute entities evenly
   */
  static distributeEntities(
    entities: Entity[],
    options: DistributionOptions
  ): Record<string, Partial<Entity>> {
    if (entities.length < 2) return {};

    const updates: Record<string, Partial<Entity>> = {};

    // Sort entities by axis
    const sortedEntities = [...entities].sort((a, b) => {
      const aVal = a.transform.position[options.axis];
      const bVal = b.transform.position[options.axis];
      return aVal - bVal;
    });

    const firstEntity = sortedEntities[0];
    const lastEntity = sortedEntities[sortedEntities.length - 1];

    let start: number;
    let end: number;
    let spacing: number;

    if (options.mode === 'equal') {
      start = firstEntity.transform.position[options.axis];
      end = lastEntity.transform.position[options.axis];
      spacing = (end - start) / (sortedEntities.length - 1);
    } else if (options.mode === 'custom' && options.spacing !== undefined) {
      spacing = options.spacing;
      start = firstEntity.transform.position[options.axis];
      end = start + spacing * (sortedEntities.length - 1);
    } else if (options.bounds) {
      start = options.bounds.start;
      end = options.bounds.end;
      spacing = (end - start) / (sortedEntities.length - 1);
    } else {
      return {};
    }

    sortedEntities.forEach((entity, index) => {
      if (entity.locked) return;

      const newPosition = { ...entity.transform.position };
      newPosition[options.axis] = start + spacing * index;

      updates[entity.id] = {
        transform: {
          ...entity.transform,
          position: newPosition,
        },
      };
    });

    return updates;
  }

  /**
   * Arrange entities in a grid
   */
  static arrangeInGrid(
    entities: Entity[],
    columns: number,
    spacing: { x: number; y: number; z: number }
  ): Record<string, Partial<Entity>> {
    const updates: Record<string, Partial<Entity>> = {};

    entities.forEach((entity, index) => {
      if (entity.locked) return;

      const row = Math.floor(index / columns);
      const col = index % columns;

      const newPosition = {
        x: col * spacing.x,
        y: 0,
        z: row * spacing.z,
      };

      updates[entity.id] = {
        transform: {
          ...entity.transform,
          position: newPosition,
        },
      };
    });

    return updates;
  }

  /**
   * Arrange entities in a circle
   */
  static arrangeInCircle(
    entities: Entity[],
    radius: number,
    axis: 'x' | 'y' | 'z' = 'y'
  ): Record<string, Partial<Entity>> {
    const updates: Record<string, Partial<Entity>> = {};

    const angleStep = (2 * Math.PI) / entities.length;

    entities.forEach((entity, index) => {
      if (entity.locked) return;

      const angle = index * angleStep;
      const newPosition = { ...entity.transform.position };

      switch (axis) {
        case 'y':
          newPosition.x = Math.cos(angle) * radius;
          newPosition.z = Math.sin(angle) * radius;
          break;
        case 'x':
          newPosition.y = Math.cos(angle) * radius;
          newPosition.z = Math.sin(angle) * radius;
          break;
        case 'z':
          newPosition.x = Math.cos(angle) * radius;
          newPosition.y = Math.sin(angle) * radius;
          break;
      }

      updates[entity.id] = {
        transform: {
          ...entity.transform,
          position: newPosition,
        },
      };
    });

    return updates;
  }

  /**
   * Arrange entities in a line
   */
  static arrangeInLine(
    entities: Entity[],
    direction: Vector3,
    spacing: number
  ): Record<string, Partial<Entity>> {
    const updates: Record<string, Partial<Entity>> = {};

    // Normalize direction
    const length = Math.sqrt(
      direction.x * direction.x +
      direction.y * direction.y +
      direction.z * direction.z
    );

    if (length === 0) return {};

    const normalizedDir = {
      x: direction.x / length,
      y: direction.y / length,
      z: direction.z / length,
    };

    entities.forEach((entity, index) => {
      if (entity.locked) return;

      const newPosition = {
        x: normalizedDir.x * spacing * index,
        y: normalizedDir.y * spacing * index,
        z: normalizedDir.z * spacing * index,
      };

      updates[entity.id] = {
        transform: {
          ...entity.transform,
          position: newPosition,
        },
      };
    });

    return updates;
  }

  /**
   * Match rotation of entities to target
   */
  static matchRotation(
    entities: Entity[],
    targetEntityId: string
  ): Record<string, Partial<Entity>> {
    const updates: Record<string, Partial<Entity>> = {};
    const targetEntity = entities.find((e) => e.id === targetEntityId);

    if (!targetEntity) return {};

    entities.forEach((entity) => {
      if (entity.id === targetEntityId) return;
      if (entity.locked) return;

      updates[entity.id] = {
        transform: {
          ...entity.transform,
          rotation: { ...targetEntity.transform.rotation },
        },
      };
    });

    return updates;
  }

  /**
   * Match scale of entities to target
   */
  static matchScale(
    entities: Entity[],
    targetEntityId: string
  ): Record<string, Partial<Entity>> {
    const updates: Record<string, Partial<Entity>> = {};
    const targetEntity = entities.find((e) => e.id === targetEntityId);

    if (!targetEntity) return {};

    entities.forEach((entity) => {
      if (entity.id === targetEntityId) return;
      if (entity.locked) return;

      updates[entity.id] = {
        transform: {
          ...entity.transform,
          scale: { ...targetEntity.transform.scale },
        },
      };
    });

    return updates;
  }

  /**
   * Randomize positions
   */
  static randomizePositions(
    entities: Entity[],
    area: { min: Vector3; max: Vector3 }
  ): Record<string, Partial<Entity>> {
    const updates: Record<string, Partial<Entity>> = {};

    entities.forEach((entity) => {
      if (entity.locked) return;

      const newPosition = {
        x:
          area.min.x +
          Math.random() * (area.max.x - area.min.x),
        y:
          area.min.y +
          Math.random() * (area.max.y - area.min.y),
        z:
          area.min.z +
          Math.random() * (area.max.z - area.min.z),
      };

      updates[entity.id] = {
        transform: {
          ...entity.transform,
          position: newPosition,
        },
      };
    });

    return updates;
  }

  /**
   * Randomize rotations
   */
  static randomizeRotations(
    entities: Entity[],
    axis: 'x' | 'y' | 'z' | 'all' = 'all'
  ): Record<string, Partial<Entity>> {
    const updates: Record<string, Partial<Entity>> = {};

    entities.forEach((entity) => {
      if (entity.locked) return;

      const currentRotation = entity.transform.rotation;

      // Convert to Euler angles (simplified)
      let newRotation = { ...currentRotation };

      if (axis === 'all' || axis === 'x') {
        const angleX = Math.random() * Math.PI * 2;
        // Apply rotation to quaternion (simplified)
        newRotation = this.applyAxisRotation(newRotation, angleX, 'x');
      }

      if (axis === 'all' || axis === 'y') {
        const angleY = Math.random() * Math.PI * 2;
        newRotation = this.applyAxisRotation(newRotation, angleY, 'y');
      }

      if (axis === 'all' || axis === 'z') {
        const angleZ = Math.random() * Math.PI * 2;
        newRotation = this.applyAxisRotation(newRotation, angleZ, 'z');
      }

      updates[entity.id] = {
        transform: {
          ...entity.transform,
          rotation: newRotation,
        },
      };
    });

    return updates;
  }

  /**
   * Randomize scales
   */
  static randomizeScales(
    entities: Entity[],
    minScale: number,
    maxScale: number
  ): Record<string, Partial<Entity>> {
    const updates: Record<string, Partial<Entity>> = {};

    entities.forEach((entity) => {
      if (entity.locked) return;

      const newScale = {
        x: minScale + Math.random() * (maxScale - minScale),
        y: minScale + Math.random() * (maxScale - minScale),
        z: minScale + Math.random() * (maxScale - minScale),
      };

      updates[entity.id] = {
        transform: {
          ...entity.transform,
          scale: newScale,
        },
      };
    });

    return updates;
  }

  /**
   * Get target value for alignment
   */
  private static getTargetValue(
    position: Vector3,
    options: AlignmentOptions
  ): number {
    switch (options.mode) {
      case 'min':
        return position[options.axis];
      case 'max':
        return position[options.axis];
      case 'center':
        return position[options.axis];
      case 'grid':
        // Round to nearest grid
        const gridSize = options.spacing || 1;
        return Math.round(position[options.axis] / gridSize) * gridSize;
      default:
        return position[options.axis];
    }
  }

  /**
   * Apply axis rotation to quaternion
   */
  private static applyAxisRotation(
    rotation: { x: number; y: number; z: number; w: number },
    angle: number,
    axis: 'x' | 'y' | 'z'
  ): { x: number; y: number; z: number; w: number } {
    const cos = Math.cos(angle / 2);
    const sin = Math.sin(angle / 2);

    let rotationQuat;
    switch (axis) {
      case 'x':
        rotationQuat = { x: sin, y: 0, z: 0, w: cos };
        break;
      case 'y':
        rotationQuat = { x: 0, y: sin, z: 0, w: cos };
        break;
      case 'z':
        rotationQuat = { x: 0, y: 0, z: sin, w: cos };
        break;
    }

    // Multiply quaternions
    return this.multiplyQuaternions(rotation, rotationQuat!);
  }

  /**
   * Multiply two quaternions
   */
  private static multiplyQuaternions(
    a: { x: number; y: number; z: number; w: number },
    b: { x: number; y: number; z: number; w: number }
  ): { x: number; y: number; z: number; w: number } {
    return {
      x:
        a.w * b.x +
        a.x * b.w +
        a.y * b.z -
        a.z * b.y,
      y:
        a.w * b.y -
        a.x * b.z +
        a.y * b.w +
        a.z * b.x,
      z:
        a.w * b.z +
        a.x * b.y -
        a.y * b.x +
        a.z * b.w,
      w:
        a.w * b.w -
        a.x * b.x -
        a.y * b.y -
        a.z * b.z,
    };
  }
}
