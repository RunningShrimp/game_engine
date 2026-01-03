// Selection Gizmo for Multiple Entities

import React, { useRef, useEffect } from 'react';
import { SelectionManager } from '../../utils/SelectionManager';
import { Vector3, Bounds3D } from '../../types/engine';

interface SelectionGizmoProps {
  selectionManager: SelectionManager;
  onTransformStart?: () => void;
  onTransformChange?: (delta: { position: Vector3; rotation: Vector3; scale: Vector3 }) => void;
  onTransformEnd?: () => void;
}

export function SelectionGizmo({
  selectionManager,
  onTransformStart,
  onTransformChange,
  onTransformEnd,
}: SelectionGizmoProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [bounds, setBounds] = React.useState<Bounds3D | null>(null);
  const [isVisible, setIsVisible] = React.useState(false);

  useEffect(() => {
    const unsubscribe = selectionManager.onSelectionChange((state) => {
      if (state.selectedIds.length > 1) {
        setBounds(state.selectionBounds);
        setIsVisible(true);
      } else {
        setIsVisible(false);
      }
    });

    return () => {
      unsubscribe();
    };
  }, [selectionManager]);

  useEffect(() => {
    if (!isVisible || !bounds) return;

    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Draw selection bounds
    drawSelectionBounds(ctx, bounds);

    // Draw center point
    drawCenterPoint(ctx, bounds);

    // Draw axes at center
    drawAxes(ctx, bounds);
  }, [isVisible, bounds]);

  if (!isVisible || !bounds) return null;

  return (
    <canvas
      ref={canvasRef}
      className="selection-gizmo"
      style={{
        position: 'absolute',
        top: 0,
        left: 0,
        pointerEvents: 'none',
        zIndex: 999,
      }}
      width={800}
      height={600}
    />
  );
}

/**
 * Draw selection bounds
 */
function drawSelectionBounds(ctx: CanvasRenderingContext2D, bounds: Bounds3D): void {
  ctx.strokeStyle = '#00ff00';
  ctx.lineWidth = 2;
  ctx.setLineDash([5, 5]);

  // Draw a simplified bounding box representation
  // In a real 3D editor, this would project the 3D bounds to 2D screen space
  const { min, max } = bounds;

  // For now, draw a simple rectangle representing the bounds
  // In reality, you'd need to project the 3D box to 2D screen space
  const x = (min.x + max.x) / 2;
  const y = (min.y + max.y) / 2;
  const width = Math.abs(max.x - min.x);
  const height = Math.abs(max.y - min.y);

  ctx.strokeRect(x - width / 2, y - height / 2, width, height);

  ctx.setLineDash([]);
}

/**
 * Draw center point
 */
function drawCenterPoint(ctx: CanvasRenderingContext2D, bounds: Bounds3D): void {
  const { center } = bounds;

  ctx.fillStyle = '#ff0000';
  ctx.beginPath();
  ctx.arc(center.x, center.y, 5, 0, 2 * Math.PI);
  ctx.fill();
}

/**
 * Draw axes at selection center
 */
function drawAxes(ctx: CanvasRenderingContext2D, bounds: Bounds3D): void {
  const { center } = bounds;
  const axisLength = 50;

  // X axis (red)
  ctx.strokeStyle = '#ff0000';
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(center.x, center.y);
  ctx.lineTo(center.x + axisLength, center.y);
  ctx.stroke();

  // Y axis (green)
  ctx.strokeStyle = '#00ff00';
  ctx.beginPath();
  ctx.moveTo(center.x, center.y);
  ctx.lineTo(center.x, center.y + axisLength);
  ctx.stroke();

  // Z axis (blue) - represented as diagonal in 2D
  ctx.strokeStyle = '#0000ff';
  ctx.beginPath();
  ctx.moveTo(center.x, center.y);
  ctx.lineTo(center.x - axisLength * 0.7, center.y - axisLength * 0.7);
  ctx.stroke();

  // Draw axis labels
  ctx.font = '12px Arial';
  ctx.fillStyle = '#ff0000';
  ctx.fillText('X', center.x + axisLength + 5, center.y);
  ctx.fillStyle = '#00ff00';
  ctx.fillText('Y', center.x, center.y + axisLength + 15);
  ctx.fillStyle = '#0000ff';
  ctx.fillText('Z', center.x - axisLength - 10, center.y - axisLength * 0.7);
}

/**
 * Multi-Selection Transform Handler Component
 */
interface TransformHandlerProps {
  selectionManager: SelectionManager;
  onTransform: (type: 'translate' | 'rotate' | 'scale', values: Vector3) => void;
}

export function MultiSelectTransformHandler({
  selectionManager,
  onTransform,
}: TransformHandlerProps) {
  const [transformMode, setTransformMode] = React.useState<'translate' | 'rotate' | 'scale'>('translate');
  const [isDragging, setIsDragging] = React.useState(false);
  const [startPos, setStartPos] = React.useState({ x: 0, y: 0 });

  const handleMouseDown = (e: React.MouseEvent, mode: 'translate' | 'rotate' | 'scale') => {
    e.preventDefault();
    setTransformMode(mode);
    setIsDragging(true);
    setStartPos({ x: e.clientX, y: e.clientY });
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (!isDragging) return;

    const deltaX = e.clientX - startPos.x;
    const deltaY = e.clientY - startPos.y;

    const values: Vector3 = {
      x: deltaX * 0.01,
      y: deltaY * 0.01,
      z: 0,
    };

    onTransform(transformMode, values);
    setStartPos({ x: e.clientX, y: e.clientY });
  };

  const handleMouseUp = () => {
    setIsDragging(false);
  };

  useEffect(() => {
    if (isDragging) {
      window.addEventListener('mousemove', handleMouseMove);
      window.addEventListener('mouseup', handleMouseUp);

      return () => {
        window.removeEventListener('mousemove', handleMouseMove);
        window.removeEventListener('mouseup', handleMouseUp);
      };
    }
  }, [isDragging, startPos, transformMode]);

  const selectedCount = selectionManager.getSelectedCount();
  if (selectedCount <= 1) return null;

  return (
    <div className="transform-handler-overlay">
      <div
        className="transform-button translate-button"
        onMouseDown={(e) => handleMouseDown(e, 'translate')}
        title="Translate (W)"
      >
        <svg width="20" height="20" viewBox="0 0 20 20">
          <path d="M10 0 L20 10 L10 20 L0 10 Z" fill="#4CAF50" />
        </svg>
      </div>

      <div
        className="transform-button rotate-button"
        onMouseDown={(e) => handleMouseDown(e, 'rotate')}
        title="Rotate (E)"
      >
        <svg width="20" height="20" viewBox="0 0 20 20">
          <path
            d="M10 0 A10 10 0 0 1 20 10"
            fill="none"
            stroke="#2196F3"
            strokeWidth="2"
          />
          <path d="M20 10 L15 5 M20 10 L15 15" stroke="#2196F3" strokeWidth="2" />
        </svg>
      </div>

      <div
        className="transform-button scale-button"
        onMouseDown={(e) => handleMouseDown(e, 'scale')}
        title="Scale (R)"
      >
        <svg width="20" height="20" viewBox="0 0 20 20">
          <rect x="2" y="2" width="16" height="16" fill="none" stroke="#FF9800" strokeWidth="2" />
          <circle cx="10" cy="10" r="3" fill="#FF9800" />
        </svg>
      </div>
    </div>
  );
}

/**
 * Selection Info Display
 */
export function SelectionInfo({ selectionManager }: { selectionManager: SelectionManager }) {
  const [selectionInfo, setSelectionInfo] = React.useState({
    count: 0,
    bounds: null as Bounds3D | null,
  });

  useEffect(() => {
    const unsubscribe = selectionManager.onSelectionChange((state) => {
      setSelectionInfo({
        count: state.selectedIds.length,
        bounds: state.selectionBounds,
      });
    });

    return () => {
      unsubscribe();
    };
  }, [selectionManager]);

  if (selectionInfo.count <= 1) return null;

  return (
    <div className="selection-info">
      <div className="selection-count">{selectionInfo.count} entities selected</div>
      {selectionInfo.bounds && (
        <div className="selection-bounds">
          Size: {selectionInfo.bounds.size.x.toFixed(2)} x{' '}
          {selectionInfo.bounds.size.y.toFixed(2)} x{' '}
          {selectionInfo.bounds.size.z.toFixed(2)}
        </div>
      )}
    </div>
  );
}
