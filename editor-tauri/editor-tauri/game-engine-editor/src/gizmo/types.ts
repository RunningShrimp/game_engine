// Gizmo system type definitions

import { Vector3 } from '../utils/math3d';
import { TransformMode, Space } from '../types/engine';

export enum GizmoAxis {
  None = 'none',
  X = 'x',
  Y = 'y',
  Z = 'z',
  XY = 'xy',
  XZ = 'xz',
  YZ = 'yz',
  XYZ = 'xyz',
  Screen = 'screen'
}

export interface GizmoState {
  mode: TransformMode;
  space: Space;
  activeAxis: GizmoAxis;
  hoverAxis: GizmoAxis;
  isDragging: boolean;
  startPosition: Vector3;
  currentPosition: Vector3;
  delta: Vector3;
}

export interface GizmoConfig {
  size: number;
  hoverSize: number;
  snapEnabled: boolean;
  snapValue: number;
  alpha: number;
  hoverAlpha: number;
  xAxisColor: string;
  yAxisColor: string;
  zAxisColor: string;
  selectionColor: string;
  hoverColor: string;
}

export const DEFAULT_GIZMO_CONFIG: GizmoConfig = {
  size: 100,
  hoverSize: 120,
  snapEnabled: false,
  snapValue: 1.0,
  alpha: 0.8,
  hoverAlpha: 1.0,
  xAxisColor: '#FF4444',
  yAxisColor: '#44FF44',
  zAxisColor: '#4444FF',
  selectionColor: '#FFFF44',
  hoverColor: '#FFFFFF'
};

export interface GizmoTransform {
  position: Vector3;
  rotation: Vector3; // Euler angles in degrees
  scale: Vector3;
}

export interface GizmoInteractionResult {
  axis: GizmoAxis;
  delta: Vector3;
  newPosition?: Vector3;
  newRotation?: Vector3;
  newScale?: Vector3;
  snapped?: boolean;
}
