// Selection System Type Definitions

import { Entity, Vector3 } from './engine';

/**
 * 3D bounding box for selection
 */
export interface Bounds3D {
  min: Vector3;
  max: Vector3;
  center: Vector3;
  size: Vector3;
}

/**
 * Selection state
 */
export interface SelectionState {
  selectedIds: string[];
  primarySelection: string | null;  // Primary selection for property display
  selectionOrder: string[];         // Selection order
  selectionBounds: Bounds3D;
}

/**
 * Selection filter
 */
export interface SelectionFilter {
  types?: string[];          // Entity type filter
  namePattern?: RegExp;      // Name pattern
  layer?: string;            // Layer filter
  components?: string[];     // Component filter
  custom?: (entity: Entity) => boolean;  // Custom filter
}

/**
 * Batch operation options
 */
export interface BatchOperationOptions {
  confirmThreshold?: number;  // Require confirmation above this count
  progressCallback?: (current: number, total: number) => void;
  undoName?: string;          // Custom undo name
}

/**
 * Bulk edit result
 */
export interface BulkEditResult {
  succeeded: string[];
  failed: Array<{ id: string; error: string }>;
  skipped: string[];
  totalAffected: number;
}

/**
 * Rename pattern for batch renaming
 */
export interface RenamePattern {
  mode: 'prefix' | 'suffix' | 'replace' | 'number';
  value: string;
  startNumber?: number;
  padding?: number;
}

/**
 * Selection history entry
 */
export interface SelectionHistoryEntry {
  ids: string[];
  primarySelection: string | null;
  timestamp: number;
}

/**
 * Rectangular selection box
 */
export interface Rect {
  start: { x: number; y: number };
  end: { x: number; y: number };
}

/**
 * Entity screen position for selection
 */
export interface EntityScreenPos {
  entityId: string;
  position: { x: number; y: number };
  bounds: Rect;
}

/**
 * Alignment options
 */
export interface AlignmentOptions {
  axis: 'x' | 'y' | 'z' | 'all';
  mode: 'min' | 'max' | 'center' | 'grid' | 'distribute';
  target?: string;  // Target entity ID for alignment
  spacing?: number; // Spacing for distribution
}

/**
 * Distribution options
 */
export interface DistributionOptions {
  axis: 'x' | 'y' | 'z';
  mode: 'equal' | 'custom';
  spacing?: number;
  bounds?: {
    start: number;
    end: number;
  };
}

/**
 * Material batch operation
 */
export interface MaterialBatchOperation {
  mode: 'apply' | 'replace' | 'modify';
  materialId?: string;
  oldMaterialId?: string;  // For replace mode
  properties?: Record<string, any>;
}

/**
 * Component batch operation
 */
export interface ComponentBatchOperation {
  mode: 'add' | 'remove' | 'modify' | 'toggle';
  componentType: string;
  properties?: Record<string, any>;
  enabled?: boolean;
}

/**
 * Selection mode
 */
export enum SelectionMode {
  Single = 'single',
  Multiple = 'multiple',
  Range = 'range',
  Box = 'box',
}

/**
 * Selection events
 */
export interface SelectionEvents {
  selectionChanged: (state: SelectionState) => void;
  primaryChanged: (id: string | null) => void;
  selectionAdded: (id: string) => void;
  selectionRemoved: (id: string) => void;
  selectionCleared: () => void;
}
