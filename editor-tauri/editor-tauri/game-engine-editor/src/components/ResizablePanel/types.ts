/**
 * Type definitions for ResizablePanel component system
 */

import type { CSSProperties, ReactNode, MouseEvent } from 'react';

/**
 * Props for the PanelHandle component
 */
export interface PanelHandleProps {
  /** Position of the handle relative to the panel */
  position: 'left' | 'right';
  /** Whether the handle is currently being dragged */
  isDragging: boolean;
  /** Callback when drag starts */
  onDragStart: (e: MouseEvent<HTMLDivElement>) => void;
}

/**
 * Props for the ResizablePanel component
 */
export interface ResizablePanelProps {
  /** Content to display in the panel */
  children: ReactNode;
  /** Position of the resize handle */
  position?: 'left' | 'right';
  /** Initial width of the panel in pixels */
  initialWidth?: number;
  /** Minimum width in pixels */
  minWidth?: number;
  /** Maximum width in pixels */
  maxWidth?: number;
  /** Additional CSS classes */
  className?: string;
  /** Callback when panel is resized */
  onResize?: (width: number) => void;
  /** Callback when resize starts */
  onResizeStart?: () => void;
  /** Callback when resize ends */
  onResizeEnd?: () => void;
  /** Debounce delay in milliseconds (0 = no debounce) */
  resizeDebounce?: number;
  /** Additional inline styles */
  style?: CSSProperties;
}

/**
 * Panel state interface for external state management
 */
export interface PanelState {
  width: number;
  isDragging: boolean;
}

/**
 * Panel configuration interface
 */
export interface PanelConfig {
  position: 'left' | 'right';
  width: number;
  minWidth: number;
  maxWidth: number;
}
