/**
 * Node Canvas Component
 * Handles the infinite canvas with pan, zoom, and tree rendering
 */

import React, { useState, useRef, useCallback, useEffect } from 'react';
import { BehaviorTree, BehaviorNode as BehaviorNodeType } from '../../types/behavior';
import { BehaviorNode } from './BehaviorNode';
import { ConnectionLine } from './ConnectionLine';
import './NodeCanvas.css';

interface NodeCanvasProps {
  tree: BehaviorTree;
  selectedNodes: Set<string>;
  onNodeSelect: (nodeId: string, addToSelection: boolean) => void;
  onNodeUpdate: (nodeId: string, updates: Partial<BehaviorNodeType>) => void;
  onNodeDelete: (nodeId: string) => void;
  onAddChild: (parentId: string, childNode: BehaviorNodeType) => void;
  onRemoveChild: (parentId: string, childId: string) => void;
}

export const NodeCanvas: React.FC<NodeCanvasProps> = ({
  tree,
  selectedNodes,
  onNodeSelect,
  onNodeUpdate,
  onNodeDelete,
  onAddChild,
  onRemoveChild,
}) => {
  // Canvas state
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [scale, setScale] = useState(1);
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });

  // Refs
  const canvasRef = useRef<HTMLDivElement>(null);
  const mousePosition = useRef({ x: 0, y: 0 });

  // Handle canvas panning
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.target === canvasRef.current || (e.target as HTMLElement).classList.contains('canvas-content')) {
      setIsDragging(true);
      setDragStart({ x: e.clientX - offset.x, y: e.clientY - offset.y });
    }
  }, [offset]);

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    mousePosition.current = { x: e.clientX, y: e.clientY };

    if (isDragging) {
      const newOffset = {
        x: e.clientX - dragStart.x,
        y: e.clientY - dragStart.y,
      };
      setOffset(newOffset);
    }
  }, [isDragging, dragStart]);

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
  }, []);

  // Handle zoom
  const handleWheel = useCallback((e: React.WheelEvent) => {
    if (e.ctrlKey || e.metaKey) {
      e.preventDefault();
      const zoomSensitivity = 0.001;
      const delta = -e.deltaY * zoomSensitivity;
      const newScale = Math.max(0.25, Math.min(2, scale + delta));

      // Zoom towards mouse position
      const rect = canvasRef.current?.getBoundingClientRect();
      if (!rect) return;

      const mouseX = e.clientX - rect.left;
      const mouseY = e.clientY - rect.top;

      const scaleChange = newScale / scale;
      const newOffset = {
        x: mouseX - (mouseX - offset.x) * scaleChange,
        y: mouseY - (mouseY - offset.y) * scaleChange,
      };

      setScale(newScale);
      setOffset(newOffset);
    }
  }, [scale, offset]);

  // Transform coordinates for rendering
  const transformPoint = useCallback((x: number, y: number) => {
    return {
      x: x * scale + offset.x,
      y: y * scale + offset.y,
    };
  }, [scale, offset]);

  // Flatten tree to array of nodes for rendering
  const flattenTree = (node: BehaviorNodeType | null): BehaviorNodeType[] => {
    if (!node) return [];
    const nodes: BehaviorNodeType[] = [node];
    node.children.forEach(child => {
      nodes.push(...flattenTree(child));
    });
    return nodes;
  };

  // Get all connections in the tree
  const getConnections = () => {
    const connections: Array<{
      from: BehaviorNodeType;
      to: BehaviorNodeType;
      fromPos: { x: number; y: number };
      toPos: { x: number; y: number };
    }> = [];

    const traverse = (node: BehaviorNodeType) => {
      node.children.forEach(child => {
        const fromPos = transformPoint(node.position.x, node.position.y);
        const toPos = transformPoint(child.position.x, child.position.y);
        connections.push({
          from: node,
          to: child,
          fromPos: {
            x: fromPos.x + 70,
            y: fromPos.y + 40,
          },
          toPos: {
            x: toPos.x + 70,
            y: toPos.y,
          },
        });
        traverse(child);
      });
    };

    if (tree.root) {
      traverse(tree.root);
    }

    return connections;
  };

  return (
    <div
      ref={canvasRef}
      className="node-canvas"
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onWheel={handleWheel}
    >
      <div
        className="canvas-content"
        style={{
          transform: `translate(${offset.x}px, ${offset.y}px) scale(${scale})`,
          transformOrigin: '0 0',
        }}
      >
        {/* Grid background */}
        <svg
          className="canvas-grid"
          width="10000"
          height="10000"
          style={{
            position: 'absolute',
            left: -5000,
            top: -5000,
            pointerEvents: 'none',
          }}
        >
          <defs>
            <pattern
              id="grid-behavior"
              width={50 * scale}
              height={50 * scale}
              patternUnits="userSpaceOnUse"
            >
              <path
                d={`M ${50 * scale} 0 L 0 0 0 ${50 * scale}`}
                fill="none"
                stroke="rgba(255,255,255,0.05)"
                strokeWidth="1"
              />
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#grid-behavior)" />
        </svg>

        {/* Render connections */}
        <svg
          className="connections-layer"
          width="10000"
          height="10000"
          style={{
            position: 'absolute',
            left: -5000,
            top: -5000,
            pointerEvents: 'none',
          }}
        >
          {getConnections().map((conn, idx) => (
            <ConnectionLine
              key={`conn-${idx}`}
              from={conn.fromPos}
              to={conn.toPos}
              status={conn.to.status}
            />
          ))}
        </svg>

        {/* Render nodes */}
        {flattenTree(tree.root).map(node => {
          const pos = transformPoint(node.position.x, node.position.y);
          const canHaveChildren = [
            'sequence',
            'selector',
            'parallel',
            'inverter',
            'repeater',
            'cooldown',
            'always_succeed',
            'always_fail',
          ].includes(node.type);

          return (
            <BehaviorNode
              key={node.id}
              node={node}
              position={pos}
              scale={scale}
              selected={selectedNodes.has(node.id)}
              onSelect={(addToSelection) => onNodeSelect(node.id, addToSelection)}
              onUpdate={(updates) => onNodeUpdate(node.id, updates)}
              onDelete={() => onNodeDelete(node.id)}
              onDrag={(deltaX, deltaY) => {
                onNodeUpdate(node.id, {
                  position: {
                    x: node.position.x + deltaX / scale,
                    y: node.position.y + deltaY / scale,
                  },
                });
              }}
              onChildDrop={(parentId, childId) => {
                // Handle reparenting
                console.log(`Reparent ${childId} to ${parentId}`);
              }}
              canHaveChildren={canHaveChildren}
            />
          );
        })}
      </div>

      {/* Zoom indicator */}
      <div className="zoom-indicator">
        {Math.round(scale * 100)}%
      </div>

      {/* Controls hint */}
      <div className="canvas-hint">
        <div>🖱️ Drag: Pan Canvas</div>
        <div>🔍 Ctrl + Scroll: Zoom</div>
        <div>⌫ Delete: Remove Selected</div>
        <div>Ctrl+C/V: Copy/Paste</div>
      </div>
    </div>
  );
};

export default NodeCanvas;
