// Grid Snapping System for Gizmo

import { Vector3 } from '../utils/math3d';

export interface SnapConfig {
  enabled: boolean;
  translateValue: number;
  rotateValue: number;
  scaleValue: number;
}

export class SnapSystem {
  private config: SnapConfig;

  constructor(config: Partial<SnapConfig> = {}) {
    this.config = {
      enabled: true,
      translateValue: 1.0,
      rotateValue: 15.0,
      scaleValue: 0.1,
      ...config
    };
  }

  updateConfig(config: Partial<SnapConfig>) {
    this.config = { ...this.config, ...config };
  }

  isEnabled(): boolean {
    return this.config.enabled;
  }

  // Snap translation to grid
  snapTranslation(translation: Vector3): Vector3 {
    if (!this.config.enabled) {
      return translation;
    }

    return new Vector3(
      this.snapValue(translation.x, this.config.translateValue),
      this.snapValue(translation.y, this.config.translateValue),
      this.snapValue(translation.z, this.config.translateValue)
    );
  }

  // Snap rotation to angles
  snapRotation(rotation: Vector3): Vector3 {
    if (!this.config.enabled) {
      return rotation;
    }

    return new Vector3(
      this.snapAngle(rotation.x),
      this.snapAngle(rotation.y),
      this.snapAngle(rotation.z)
    );
  }

  // Snap scale to increments
  snapScale(scale: Vector3): Vector3 {
    if (!this.config.enabled) {
      return scale;
    }

    return new Vector3(
      this.snapValue(scale.x, this.config.scaleValue),
      this.snapValue(scale.y, this.config.scaleValue),
      this.snapValue(scale.z, this.config.scaleValue)
    );
  }

  // Snap single value to increment
  snapValue(value: number, increment: number): number {
    if (increment <= 0) return value;
    return Math.round(value / increment) * increment;
  }

  // Snap angle to rotation increments
  snapAngle(angle: number): number {
    return this.snapValue(angle, this.config.rotateValue);
  }

  // Get snapped delta for smooth dragging
  snapDelta(delta: Vector3, mode: 'translate' | 'rotate' | 'scale'): Vector3 {
    if (!this.config.enabled) {
      return delta;
    }

    switch (mode) {
      case 'translate':
        return this.snapTranslation(delta);
      case 'rotate':
        return this.snapRotation(delta);
      case 'scale':
        return this.snapScale(delta);
      default:
        return delta;
    }
  }

  // Get nearest snap value for visualization
  getNearestSnapValue(value: number, mode: 'translate' | 'rotate' | 'scale'): number {
    const increment = mode === 'translate'
      ? this.config.translateValue
      : mode === 'rotate'
      ? this.config.rotateValue
      : this.config.scaleValue;

    return this.snapValue(value, increment);
  }

  // Get distance to nearest snap point
  getSnapDistance(value: number, mode: 'translate' | 'rotate' | 'scale'): number {
    const snapped = this.getNearestSnapValue(value, mode);
    return Math.abs(value - snapped);
  }

  // Check if value is close to snap point
  isNearSnapPoint(value: number, mode: 'translate' | 'rotate' | 'scale', threshold: number = 0.1): boolean {
    return this.getSnapDistance(value, mode) < threshold;
  }

  // Calculate snap position for indicator
  calculateSnapIndicator(current: Vector3, mode: 'translate' | 'rotate' | 'scale'): Vector3 {
    switch (mode) {
      case 'translate':
        return this.snapTranslation(current);
      case 'rotate':
        return this.snapRotation(current);
      case 'scale':
        return this.snapScale(current);
      default:
        return current;
    }
  }

  // Get config values
  getConfig(): SnapConfig {
    return { ...this.config };
  }
}
