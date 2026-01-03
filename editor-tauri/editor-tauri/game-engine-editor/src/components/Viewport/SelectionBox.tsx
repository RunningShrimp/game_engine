// Selection Box Component for Box Selection

import React, { useRef, useEffect } from 'react';
import { selectionManager } from '../../utils/SelectionManager';
import { Rect } from '../../types/selection';

interface SelectionBoxProps {
  viewportRef: React.RefObject<HTMLDivElement>;
  onSelectionStart?: () => void;
  onSelectionEnd?: (selectedIds: string[]) => void;
}

export function SelectionBox({
  viewportRef,
  onSelectionStart,
  onSelectionEnd,
}: SelectionBoxProps) {
  const [box, setBox] = React.useState<Rect | null>(null);
  const [isActive, setIsActive] = React.useState(false);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const viewport = viewportRef.current;
    if (!viewport) return;

    const handleMouseDown = (e: MouseEvent) => {
      // Check if Shift key is held and not Ctrl key (for box selection)
      if (e.shiftKey && !e.ctrlKey && e.button === 0) {
        e.preventDefault();
        e.stopPropagation();

        const rect = viewport.getBoundingClientRect();
        const startPoint = {
          x: e.clientX - rect.left,
          y: e.clientY - rect.top,
        };

        setBox({ start: startPoint, end: startPoint });
        setIsActive(true);
        selectionManager.startBoxSelection(startPoint);

        if (onSelectionStart) {
          onSelectionStart();
        }
      }
    };

    const handleMouseMove = (e: MouseEvent) => {
      if (!isActive || !box) return;

      const rect = viewport.getBoundingClientRect();
      const endPoint = {
        x: e.clientX - rect.left,
        y: e.clientY - rect.top,
      };

      setBox({ ...box, end: endPoint });
      selectionManager.updateBoxSelection(endPoint);
    };

    const handleMouseUp = (e: MouseEvent) => {
      if (!isActive || !box) return;

      const rect = viewport.getBoundingClientRect();
      const endPoint = {
        x: e.clientX - rect.left,
        y: e.clientY - rect.top,
      };

      // Get entities in box (this would need to be implemented
      // to project 3D positions to 2D screen space)
      const entitiesInBox = getEntitiesInBox(box, viewport);

      selectionManager.endBoxSelection(entitiesInBox, true);

      if (onSelectionEnd) {
        onSelectionEnd(entitiesInBox.map((e) => e.entityId));
      }

      setBox(null);
      setIsActive(false);
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      // Cancel box selection with Escape
      if (e.key === 'Escape' && isActive) {
        setBox(null);
        setIsActive(false);
      }
    };

    // Add event listeners
    viewport.addEventListener('mousedown', handleMouseDown);
    viewport.addEventListener('mousemove', handleMouseMove);
    viewport.addEventListener('mouseup', handleMouseUp);
    document.addEventListener('keydown', handleKeyDown);

    return () => {
      viewport.removeEventListener('mousedown', handleMouseDown);
      viewport.removeEventListener('mousemove', handleMouseMove);
      viewport.removeEventListener('mouseup', handleMouseUp);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [isActive, box, viewportRef, onSelectionStart, onSelectionEnd]);

  if (!box || !isActive) return null;

  const style: React.CSSProperties = {
    position: 'absolute',
    left: Math.min(box.start.x, box.end.x),
    top: Math.min(box.start.y, box.end.y),
    width: Math.abs(box.end.x - box.start.x),
    height: Math.abs(box.end.y - box.start.y),
    border: '1px dashed #00ff00',
    backgroundColor: 'rgba(0, 255, 0, 0.1)',
    pointerEvents: 'none',
    zIndex: 1000,
  };

  return <div style={style} className="selection-box" />;
}

/**
 * Get entities in selection box
 * This is a placeholder - in a real implementation,
 * you would project entity 3D positions to 2D screen space
 */
function getEntitiesInBox(box: Rect, viewport: HTMLElement): Array<{
  entityId: string;
  position: { x: number; y: number };
  bounds: Rect;
}> {
  // This would need to:
  // 1. Get all entities from the scene
  // 2. Project their 3D positions to 2D screen space
  // 3. Check if they're within the selection box

  // For now, return empty array
  return [];

  /*
  Real implementation would look something like:

  const entities = sceneManager.getAllEntities();
  const camera = sceneManager.getActiveCamera();
  const entityPositions: EntityScreenPos[] = [];

  entities.forEach(entity => {
    const screenPos = projectToScreen(entity.transform.position, camera, viewport);
    const bounds = calculateScreenBounds(entity, camera, viewport);

    entityPositions.push({
      entityId: entity.id,
      position: screenPos,
      bounds,
    });
  });

  return filterEntitiesInBox(entityPositions, box);
  */
}

/**
 * Project 3D position to 2D screen space
 */
function projectToScreen(
  position: { x: number; y: number; z: number },
  camera: any,
  viewport: HTMLElement
): { x: number; y: number } {
  // Implement projection logic
  // This would use the camera's view-projection matrix
  return { x: 0, y: 0 };
}

/**
 * Calculate screen bounds for an entity
 */
function calculateScreenBounds(
  entity: any,
  camera: any,
  viewport: HTMLElement
): Rect {
  // Implement bounds calculation
  // This would project the entity's bounding box to screen space
  return {
    start: { x: 0, y: 0 },
    end: { x: 0, y: 0 },
  };
}

/**
 * Filter entities by selection box
 */
function filterEntitiesInBox(
  entityPositions: Array<{
    entityId: string;
    position: { x: number; y: number };
    bounds: Rect;
  }>,
  box: Rect
): Array<{ entityId: string; position: { x: number; y: number }; bounds: Rect }> {
  const minX = Math.min(box.start.x, box.end.x);
  const maxX = Math.max(box.start.x, box.end.x);
  const minY = Math.min(box.start.y, box.end.y);
  const maxY = Math.max(box.start.y, box.end.y);

  return entityPositions.filter((pos) => {
    const x = pos.position.x;
    const y = pos.position.y;
    return x >= minX && x <= maxX && y >= minY && y <= maxY;
  });
}
