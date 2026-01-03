// Gizmo Interaction Controller

import { GizmoAxis, GizmoState, GizmoInteractionResult, GizmoConfig, DEFAULT_GIZMO_CONFIG } from './types';
import { Vector3 } from '../utils/math3d';
import { Camera, Ray, Plane, LineSegment, Circle, LineHelper, CircleHelper } from '../utils/raycast';
import { SnapSystem } from './SnapSystem';
import { TransformMode, Space } from '../types/engine';

export class GizmoController {
  private state: GizmoState;
  private config: GizmoConfig;
  private snapSystem: SnapSystem;
  private dragStartMouse: { x: number; y: number };

  constructor(config: Partial<GizmoConfig> = {}) {
    this.config = { ...DEFAULT_GIZMO_CONFIG, ...config };
    this.snapSystem = new SnapSystem({
      enabled: this.config.snapEnabled,
      translateValue: this.config.snapValue,
      rotateValue: 15,
      scaleValue: 0.1
    });

    this.state = {
      mode: TransformMode.Translate,
      space: Space.World,
      activeAxis: GizmoAxis.None,
      hoverAxis: GizmoAxis.None,
      isDragging: false,
      startPosition: Vector3.zero,
      currentPosition: Vector3.zero,
      delta: Vector3.zero
    };

    this.dragStartMouse = { x: 0, y: 0 };
  }

  updateConfig(config: Partial<GizmoConfig>) {
    this.config = { ...this.config, ...config };
    this.snapSystem.updateConfig({
      enabled: this.config.snapEnabled,
      translateValue: this.config.snapValue,
      rotateValue: 15,
      scaleValue: 0.1
    });
  }

  setMode(mode: TransformMode) {
    this.state.mode = mode;
    this.state.activeAxis = GizmoAxis.None;
    this.state.hoverAxis = GizmoAxis.None;
  }

  setSpace(space: Space) {
    this.state.space = space;
  }

  getState(): GizmoState {
    return { ...this.state };
  }

  // Handle mouse move for hover detection
  handleMouseMove(
    mouseX: number,
    mouseY: number,
    targetPosition: Vector3,
    camera: Camera,
    viewportWidth: number,
    viewportHeight: number
  ): GizmoAxis | null {
    if (this.state.isDragging) {
      return this.handleDrag(mouseX, mouseY, targetPosition, camera, viewportWidth, viewportHeight);
    }

    // Check hover for each axis
    const hoveredAxis = this.hitTest(
      mouseX,
      mouseY,
      targetPosition,
      camera,
      viewportWidth,
      viewportHeight
    );

    this.state.hoverAxis = hoveredAxis || GizmoAxis.None;
    return this.state.hoverAxis;
  }

  // Handle mouse down to start drag
  handleMouseDown(
    mouseX: number,
    mouseY: number,
    targetPosition: Vector3,
    camera: Camera,
    viewportWidth: number,
    viewportHeight: number
  ): boolean {
    const hitAxis = this.hitTest(
      mouseX,
      mouseY,
      targetPosition,
      camera,
      viewportWidth,
      viewportHeight
    );

    if (hitAxis && hitAxis !== GizmoAxis.None) {
      this.state.isDragging = true;
      this.state.activeAxis = hitAxis;
      this.state.startPosition = targetPosition.clone();
      this.state.currentPosition = targetPosition.clone();
      this.state.delta = Vector3.zero;
      this.dragStartMouse = { x: mouseX, y: mouseY };
      return true;
    }

    return false;
  }

  // Handle mouse up to end drag
  handleMouseUp(): GizmoInteractionResult | null {
    if (!this.state.isDragging) {
      return null;
    }

    const result = this.calculateTransform();

    this.state.isDragging = false;
    this.state.activeAxis = GizmoAxis.None;
    this.state.delta = Vector3.zero;

    return result;
  }

  // Hit test to find which gizmo element is under mouse
  private hitTest(
    mouseX: number,
    mouseY: number,
    targetPosition: Vector3,
    camera: Camera,
    viewportWidth: number,
    viewportHeight: number
  ): GizmoAxis | null {
    const ray = camera.screenPointToRay(mouseX, mouseY, viewportWidth, viewportHeight);
    const size = this.config.size;

    // Get screen position of target
    const screenPos = camera.worldPointToScreen(targetPosition, viewportWidth, viewportHeight);

    // Check in 2D screen space
    const hitThreshold = 15; // pixels

    if (this.state.mode === TransformMode.Translate) {
      return this.hitTestTranslate(mouseX, mouseY, screenPos, size, hitThreshold);
    } else if (this.state.mode === TransformMode.Rotate) {
      return this.hitTestRotate(mouseX, mouseY, screenPos, size, hitThreshold);
    } else if (this.state.mode === TransformMode.Scale) {
      return this.hitTestScale(mouseX, mouseY, screenPos, size, hitThreshold);
    }

    return null;
  }

  private hitTestTranslate(
    mouseX: number,
    mouseY: number,
    screenPos: { x: number; y: number },
    size: number,
    threshold: number
  ): GizmoAxis | null {
    const arrowLength = size * 0.8;

    // Check X axis (horizontal right)
    const xDist = this.pointToLineDistance(
      mouseX,
      mouseY,
      screenPos.x,
      screenPos.y,
      screenPos.x + arrowLength,
      screenPos.y
    );
    if (xDist < threshold && mouseX > screenPos.x) {
      return GizmoAxis.X;
    }

    // Check Y axis (vertical up)
    const yDist = this.pointToLineDistance(
      mouseX,
      mouseY,
      screenPos.x,
      screenPos.y,
      screenPos.x,
      screenPos.y - arrowLength
    );
    if (yDist < threshold && mouseY < screenPos.y) {
      return GizmoAxis.Y;
    }

    // Check Z axis (diagonal down-left)
    const zDist = this.pointToLineDistance(
      mouseX,
      mouseY,
      screenPos.x,
      screenPos.y,
      screenPos.x - arrowLength * 0.5,
      screenPos.y + arrowLength * 0.5
    );
    if (zDist < threshold) {
      return GizmoAxis.Z;
    }

    // Check center sphere for screen space
    const centerDist = Math.sqrt(
      Math.pow(mouseX - screenPos.x, 2) + Math.pow(mouseY - screenPos.y, 2)
    );
    if (centerDist < 10) {
      return GizmoAxis.Screen;
    }

    return null;
  }

  private hitTestRotate(
    mouseX: number,
    mouseY: number,
    screenPos: { x: number; y: number },
    size: number,
    threshold: number
  ): GizmoAxis | null {
    const radius = size * 0.6;
    const distFromCenter = Math.sqrt(
      Math.pow(mouseX - screenPos.x, 2) + Math.pow(mouseY - screenPos.y, 2)
    );

    // Check Z axis (full circle)
    if (Math.abs(distFromCenter - radius) < threshold) {
      return GizmoAxis.Z;
    }

    // Check screen-space rotation (outer circle)
    if (Math.abs(distFromCenter - radius * 1.1) < threshold) {
      return GizmoAxis.Screen;
    }

    // Check X and Y axis arcs (simplified)
    // In a real implementation, these would be more precise arc hit tests
    const angle = Math.atan2(mouseY - screenPos.y, mouseX - screenPos.x);
    if (distFromCenter < radius + threshold && distFromCenter > radius - threshold) {
      // X axis is roughly horizontal, Y axis is roughly vertical
      if (Math.abs(Math.cos(angle)) > 0.7) {
        return GizmoAxis.X;
      } else if (Math.abs(Math.sin(angle)) > 0.7) {
        return GizmoAxis.Y;
      }
    }

    return null;
  }

  private hitTestScale(
    mouseX: number,
    mouseY: number,
    screenPos: { x: number; y: number },
    size: number,
    threshold: number
  ): GizmoAxis | null {
    const axisLength = size * 0.7;
    const boxSize = size * 0.15;

    // Check X axis
    const xDist = this.pointToLineDistance(
      mouseX,
      mouseY,
      screenPos.x,
      screenPos.y,
      screenPos.x + axisLength,
      screenPos.y
    );
    if (xDist < threshold && mouseX > screenPos.x) {
      return GizmoAxis.X;
    }

    // Check Y axis
    const yDist = this.pointToLineDistance(
      mouseX,
      mouseY,
      screenPos.x,
      screenPos.y,
      screenPos.x,
      screenPos.y - axisLength
    );
    if (yDist < threshold && mouseY < screenPos.y) {
      return GizmoAxis.Y;
    }

    // Check Z axis
    const zDist = this.pointToLineDistance(
      mouseX,
      mouseY,
      screenPos.x,
      screenPos.y,
      screenPos.x - axisLength * 0.5,
      screenPos.y + axisLength * 0.5
    );
    if (zDist < threshold) {
      return GizmoAxis.Z;
    }

    // Check center box for uniform scale
    const centerDist = Math.sqrt(
      Math.pow(mouseX - screenPos.x, 2) + Math.pow(mouseY - screenPos.y, 2)
    );
    if (centerDist < boxSize) {
      return GizmoAxis.XYZ;
    }

    return null;
  }

  private pointToLineDistance(
    px: number,
    py: number,
    x1: number,
    y1: number,
    x2: number,
    y2: number
  ): number {
    const A = px - x1;
    const B = py - y1;
    const C = x2 - x1;
    const D = y2 - y1;

    const dot = A * C + B * D;
    const lenSq = C * C + D * D;
    let param = -1;

    if (lenSq !== 0) {
      param = dot / lenSq;
    }

    let xx, yy;

    if (param < 0) {
      xx = x1;
      yy = y1;
    } else if (param > 1) {
      xx = x2;
      yy = y2;
    } else {
      xx = x1 + param * C;
      yy = y1 + param * D;
    }

    const dx = px - xx;
    const dy = py - yy;

    return Math.sqrt(dx * dx + dy * dy);
  }

  // Handle dragging operation
  private handleDrag(
    mouseX: number,
    mouseY: number,
    targetPosition: Vector3,
    camera: Camera,
    viewportWidth: number,
    viewportHeight: number
  ): GizmoAxis | null {
    const mouseDelta = {
      x: mouseX - this.dragStartMouse.x,
      y: mouseY - this.dragStartMouse.y
    };

    // Calculate delta based on active axis
    let delta = Vector3.zero;

    switch (this.state.activeAxis) {
      case GizmoAxis.X:
        delta = new Vector3(mouseDelta.x * 0.01, 0, 0);
        break;
      case GizmoAxis.Y:
        delta = new Vector3(0, -mouseDelta.y * 0.01, 0);
        break;
      case GizmoAxis.Z:
        delta = new Vector3(mouseDelta.x * 0.007, mouseDelta.y * 0.007, 0);
        break;
      case GizmoAxis.Screen:
        delta = new Vector3(mouseDelta.x * 0.01, -mouseDelta.y * 0.01, 0);
        break;
      case GizmoAxis.XYZ:
        const uniformDelta = (Math.abs(mouseDelta.x) + Math.abs(mouseDelta.y)) * 0.01;
        delta = new Vector3(uniformDelta, uniformDelta, uniformDelta);
        break;
    }

    // Apply snapping
    if (this.state.mode === TransformMode.Translate) {
      delta = this.snapSystem.snapDelta(delta, 'translate');
    } else if (this.state.mode === TransformMode.Rotate) {
      delta = this.snapSystem.snapDelta(delta, 'rotate');
    } else if (this.state.mode === TransformMode.Scale) {
      delta = this.snapSystem.snapDelta(delta, 'scale');
    }

    this.state.delta = delta;
    this.state.currentPosition = this.state.startPosition.add(delta);

    return this.state.activeAxis;
  }

  // Calculate final transform result
  private calculateTransform(): GizmoInteractionResult {
    const result: GizmoInteractionResult = {
      axis: this.state.activeAxis,
      delta: this.state.delta,
      snapped: this.snapSystem.isEnabled()
    };

    switch (this.state.mode) {
      case TransformMode.Translate:
        result.newPosition = this.state.currentPosition;
        break;
      case TransformMode.Rotate:
        result.newRotation = this.state.currentPosition;
        break;
      case TransformMode.Scale:
        result.newScale = this.state.currentPosition;
        break;
    }

    return result;
  }
}
