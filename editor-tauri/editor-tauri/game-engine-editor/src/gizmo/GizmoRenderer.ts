// Gizmo Renderer - Handles rendering of 3D gizmos

import { GizmoAxis, GizmoConfig, GizmoState, DEFAULT_GIZMO_CONFIG } from './types';
import { Vector3, Matrix4 } from '../utils/math3d';
import { Camera } from '../utils/raycast';
import { TransformMode } from '../types/engine';

export class GizmoRenderer {
  private config: GizmoConfig;
  private ctx: CanvasRenderingContext2D;

  constructor(ctx: CanvasRenderingContext2D, config: Partial<GizmoConfig> = {}) {
    this.ctx = ctx;
    this.config = { ...DEFAULT_GIZMO_CONFIG, ...config };
  }

  updateConfig(config: Partial<GizmoConfig>) {
    this.config = { ...this.config, ...config };
  }

  render(
    position: Vector3,
    state: GizmoState,
    camera: Camera,
    viewportWidth: number,
    viewportHeight: number
  ) {
    if (!state.activeAxis && !state.hoverAxis) return;

    // Get screen position
    const screenPos = camera.worldPointToScreen(position, viewportWidth, viewportHeight);

    // Set gizmo size based on state
    const size = state.hoverAxis ? this.config.hoverSize : this.config.size;

    // Render based on mode
    switch (state.mode) {
      case TransformMode.Translate:
        this.renderTranslateGizmo(screenPos, size, state, camera);
        break;
      case TransformMode.Rotate:
        this.renderRotateGizmo(screenPos, size, state, camera);
        break;
      case TransformMode.Scale:
        this.renderScaleGizmo(screenPos, size, state, camera);
        break;
    }
  }

  private renderTranslateGizmo(
    center: { x: number; y: number },
    size: number,
    state: GizmoState,
    camera: Camera
  ) {
    const arrowLength = size * 0.8;
    const arrowHeadSize = size * 0.2;

    // Calculate axis directions in screen space
    const origin = camera.position;
    const xAxisEnd = new Vector3(center.x + arrowLength, center.y, 0);
    const yAxisEnd = new Vector3(center.x, center.y - arrowLength, 0);
    const zAxisEnd = new Vector3(center.x - arrowLength * 0.5, center.y + arrowLength * 0.5, 0);

    // Draw X axis (Red)
    this.drawArrow(
      center.x,
      center.y,
      center.x + arrowLength,
      center.y,
      this.config.xAxisColor,
      state.hoverAxis === GizmoAxis.X || state.activeAxis === GizmoAxis.X
    );

    // Draw Y axis (Green)
    this.drawArrow(
      center.x,
      center.y,
      center.x,
      center.y - arrowLength,
      this.config.yAxisColor,
      state.hoverAxis === GizmoAxis.Y || state.activeAxis === GizmoAxis.Y
    );

    // Draw Z axis (Blue - diagonal)
    this.drawArrow(
      center.x,
      center.y,
      center.x - arrowLength * 0.5,
      center.y + arrowLength * 0.5,
      this.config.zAxisColor,
      state.hoverAxis === GizmoAxis.Z || state.activeAxis === GizmoAxis.Z
    );

    // Draw center sphere for screen-space translation
    this.drawCircle(
      center.x,
      center.y,
      8,
      state.hoverAxis === GizmoAxis.Screen || state.activeAxis === GizmoAxis.Screen
        ? this.config.selectionColor
        : '#FFFFFF',
      true
    );
  }

  private renderRotateGizmo(
    center: { x: number; y: number },
    size: number,
    state: GizmoState,
    camera: Camera
  ) {
    const radius = size * 0.6;
    const lineWidth = 3;

    // Draw X rotation circle (Red)
    this.drawArc(
      center.x,
      center.y,
      radius,
      -Math.PI * 0.2,
      Math.PI * 0.2,
      this.config.xAxisColor,
      lineWidth,
      state.hoverAxis === GizmoAxis.X || state.activeAxis === GizmoAxis.X
    );

    // Draw Y rotation circle (Green)
    this.drawArc(
      center.x,
      center.y,
      radius * 0.9,
      Math.PI * 0.3,
      Math.PI * 0.7,
      this.config.yAxisColor,
      lineWidth,
      state.hoverAxis === GizmoAxis.Y || state.activeAxis === GizmoAxis.Y
    );

    // Draw Z rotation circle (Blue - full circle)
    this.drawArc(
      center.x,
      center.y,
      radius,
      0,
      Math.PI * 2,
      this.config.zAxisColor,
      lineWidth,
      state.hoverAxis === GizmoAxis.Z || state.activeAxis === GizmoAxis.Z
    );

    // Draw screen-space rotation circle (White)
    this.drawCircle(
      center.x,
      center.y,
      radius * 1.1,
      this.config.selectionColor,
      state.hoverAxis === GizmoAxis.Screen || state.activeAxis === GizmoAxis.Screen
    );
  }

  private renderScaleGizmo(
    center: { x: number; y: number },
    size: number,
    state: GizmoState,
    camera: Camera
  ) {
    const axisLength = size * 0.7;
    const boxSize = size * 0.15;

    // Draw X axis (Red with box at end)
    this.drawAxisWithBox(
      center.x,
      center.y,
      center.x + axisLength,
      center.y,
      boxSize,
      this.config.xAxisColor,
      state.hoverAxis === GizmoAxis.X || state.activeAxis === GizmoAxis.X
    );

    // Draw Y axis (Green with box at end)
    this.drawAxisWithBox(
      center.x,
      center.y,
      center.x,
      center.y - axisLength,
      boxSize,
      this.config.yAxisColor,
      state.hoverAxis === GizmoAxis.Y || state.activeAxis === GizmoAxis.Y
    );

    // Draw Z axis (Blue with box at end - diagonal)
    this.drawAxisWithBox(
      center.x,
      center.y,
      center.x - axisLength * 0.5,
      center.y + axisLength * 0.5,
      boxSize,
      this.config.zAxisColor,
      state.hoverAxis === GizmoAxis.Z || state.activeAxis === GizmoAxis.Z
    );

    // Draw center box for uniform scale
    this.drawBox(
      center.x - boxSize / 2,
      center.y - boxSize / 2,
      boxSize,
      boxSize,
      state.hoverAxis === GizmoAxis.XYZ || state.activeAxis === GizmoAxis.XYZ
        ? this.config.selectionColor
        : '#FFFFFF',
      true
    );
  }

  private drawArrow(
    x1: number,
    y1: number,
    x2: number,
    y2: number,
    color: string,
    highlighted: boolean
  ) {
    const ctx = this.ctx;
    const headLength = 15;
    const headWidth = 10;
    const angle = Math.atan2(y2 - y1, x2 - x1);

    ctx.save();
    ctx.globalAlpha = highlighted ? this.config.hoverAlpha : this.config.alpha;
    ctx.strokeStyle = highlighted ? this.config.hoverColor : color;
    ctx.fillStyle = highlighted ? this.config.hoverColor : color;
    ctx.lineWidth = highlighted ? 3 : 2;

    // Draw line
    ctx.beginPath();
    ctx.moveTo(x1, y1);
    ctx.lineTo(x2, y2);
    ctx.stroke();

    // Draw arrowhead
    ctx.beginPath();
    ctx.moveTo(x2, y2);
    ctx.lineTo(
      x2 - headLength * Math.cos(angle - Math.PI / 6),
      y2 - headLength * Math.sin(angle - Math.PI / 6)
    );
    ctx.lineTo(
      x2 - headLength * Math.cos(angle + Math.PI / 6),
      y2 - headLength * Math.sin(angle + Math.PI / 6)
    );
    ctx.closePath();
    ctx.fill();

    ctx.restore();
  }

  private drawArc(
    x: number,
    y: number,
    radius: number,
    startAngle: number,
    endAngle: number,
    color: string,
    lineWidth: number,
    highlighted: boolean
  ) {
    const ctx = this.ctx;

    ctx.save();
    ctx.globalAlpha = highlighted ? this.config.hoverAlpha : this.config.alpha;
    ctx.strokeStyle = highlighted ? this.config.hoverColor : color;
    ctx.lineWidth = highlighted ? lineWidth + 2 : lineWidth;
    ctx.lineCap = 'round';

    ctx.beginPath();
    ctx.arc(x, y, radius, startAngle, endAngle);
    ctx.stroke();

    ctx.restore();
  }

  private drawCircle(
    x: number,
    y: number,
    radius: number,
    color: string,
    filled: boolean = false
  ) {
    const ctx = this.ctx;

    ctx.save();
    ctx.globalAlpha = this.config.alpha;
    ctx.strokeStyle = color;
    ctx.fillStyle = color;
    ctx.lineWidth = 2;

    ctx.beginPath();
    ctx.arc(x, y, radius, 0, Math.PI * 2);

    if (filled) {
      ctx.fill();
    } else {
      ctx.stroke();
    }

    ctx.restore();
  }

  private drawBox(
    x: number,
    y: number,
    width: number,
    height: number,
    color: string,
    filled: boolean = false
  ) {
    const ctx = this.ctx;

    ctx.save();
    ctx.globalAlpha = this.config.alpha;
    ctx.strokeStyle = color;
    ctx.fillStyle = color;
    ctx.lineWidth = 2;

    if (filled) {
      ctx.fillRect(x, y, width, height);
    } else {
      ctx.strokeRect(x, y, width, height);
    }

    ctx.restore();
  }

  private drawAxisWithBox(
    x1: number,
    y1: number,
    x2: number,
    y2: number,
    boxSize: number,
    color: string,
    highlighted: boolean
  ) {
    // Draw line
    this.ctx.save();
    this.ctx.globalAlpha = highlighted ? this.config.hoverAlpha : this.config.alpha;
    this.ctx.strokeStyle = highlighted ? this.config.hoverColor : color;
    this.ctx.lineWidth = highlighted ? 3 : 2;
    this.ctx.beginPath();
    this.ctx.moveTo(x1, y1);
    this.ctx.lineTo(x2, y2);
    this.ctx.stroke();
    this.ctx.restore();

    // Draw box at end
    this.drawBox(
      x2 - boxSize / 2,
      y2 - boxSize / 2,
      boxSize,
      boxSize,
      highlighted ? this.config.hoverColor : color,
      true
    );
  }
}
