/**
 * Node Canvas Component
 * Handles the infinite canvas with pan, zoom, and node rendering
 */

import React, { useState, useRef, useCallback, useEffect } from 'react';
import { Material, MaterialNode, NodeConnection } from '../../types/material';
import { Node } from './Node';
import { ConnectionLine } from './ConnectionLine';
import './NodeCanvas.css';

interface NodeCanvasProps {
  material: Material;
  selectedNodes: Set<string>;
  onNodeSelect: (nodeId: string, addToSelection: boolean) => void;
  onNodeUpdate: (nodeId: string, updates: Partial<MaterialNode>) => void;
  onNodeDelete: (nodeId: string) => void;
  onConnectionAdd: (connection: NodeConnection) => void;
  onConnectionRemove: (connectionId: string) => void;
}

export const NodeCanvas: React.FC<NodeCanvasProps> = ({
  material,
  selectedNodes,
  onNodeSelect,
  onNodeUpdate,
  onNodeDelete,
  onConnectionAdd,
  onConnectionRemove,
}) => {
  // Canvas state
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [scale, setScale] = useState(1);
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const [connectingFrom, setConnectingFrom] = useState<{
    nodeId: string;
    portId: string;
    portType: 'input' | 'output';
  } | null>(null);

  // Refs
  const canvasRef = useRef<HTMLDivElement>(null);
  const dragStartOffset = useRef({ x: 0, y: 0 });
  const mousePosition = useRef({ x: 0, y: 0 });

  // Handle canvas panning
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.target === canvasRef.current || (e.target as HTMLElement).classList.contains('canvas-content')) {
      setIsDragging(true);
      setDragStart({ x: e.clientX - offset.x, y: e.clientY - offset.y });
      dragStartOffset.current = { x: e.clientX - offset.x, y: e.clientY - offset.y };
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
      const newScale = Math.max(0.1, Math.min(3, scale + delta));

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

  // Handle connection creation
  const handlePortMouseDown = useCallback((
    nodeId: string,
    portId: string,
    portType: 'input' | 'output'
  ) => {
    setConnectingFrom({ nodeId, portId, portType });
  }, []);

  const handlePortMouseUp = useCallback((
    nodeId: string,
    portId: string,
    portType: 'input' | 'output'
  ) => {
    if (!connectingFrom) return;

    // Validate connection (must be input to output or vice versa)
    if (connectingFrom.portType === portType) {
      setConnectingFrom(null);
      return;
    }

    // Determine from and to
    const from = connectingFrom.portType === 'output'
      ? { nodeId: connectingFrom.nodeId, portId: connectingFrom.portId }
      : { nodeId, portId };
    const to = connectingFrom.portType === 'input'
      ? { nodeId: connectingFrom.nodeId, portId: connectingFrom.portId }
      : { nodeId, portId };

    // Create connection
    const connection: NodeConnection = {
      id: `conn_${Date.now()}`,
      fromNodeId: from.nodeId,
      fromPortId: from.portId,
      toNodeId: to.nodeId,
      toPortId: to.portId,
    };

    onConnectionAdd(connection);
    setConnectingFrom(null);
  }, [connectingFrom, onConnectionAdd]);

  // Handle node dragging
  const handleNodeDrag = useCallback((nodeId: string, deltaX: number, deltaY: number) => {
    const node = material.nodes.find(n => n.id === nodeId);
    if (!node) return;

    onNodeUpdate(nodeId, {
      position: {
        x: node.position.x + deltaX / scale,
        y: node.position.y + deltaY / scale,
      },
    });
  }, [material.nodes, scale, onNodeUpdate]);

  // Transform coordinates for rendering
  const transformPoint = useCallback((x: number, y: number) => {
    return {
      x: x * scale + offset.x,
      y: y * scale + offset.y,
    };
  }, [scale, offset]);

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
              id="grid"
              width={50 * scale}
              height={50 * scale}
              patternUnits="userSpaceOnUse"
            >
              <path
                d={`M ${50 * scale} 0 L 0 0 0 ${50 * scale}`}
                fill="none"
                stroke="rgba(255,255,255,0.1)"
                strokeWidth="1"
              />
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#grid)" />
        </svg>

        {/* Connections */}
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
          {material.connections.map(conn => {
            const fromNode = material.nodes.find(n => n.id === conn.fromNodeId);
            const toNode = material.nodes.find(n => n.id === conn.toNodeId);
            if (!fromNode || !toNode) return null;

            const fromPos = transformPoint(fromNode.position.x, fromNode.position.y);
            const toPos = transformPoint(toNode.position.x, toNode.position.y);

            return (
              <ConnectionLine
                key={conn.id}
                from={{ x: fromPos.x, y: fromPos.y }}
                to={{ x: toPos.x, y: toPos.y }}
                onRemove={() => onConnectionRemove(conn.id)}
              />
            );
          })}

          {/* Temporary connection line while dragging */}
          {connectingFrom && (
            <ConnectionLine
              from={transformPoint(
                material.nodes.find(n => n.id === connectingFrom.nodeId)?.position.x || 0,
                material.nodes.find(n => n.id === connectingFrom.nodeId)?.position.y || 0
              )}
              to={mousePosition.current}
              isTemporary
            />
          )}
        </svg>

        {/* Nodes */}
        {material.nodes.map(node => {
          const pos = transformPoint(node.position.x, node.position.y);
          return (
            <Node
              key={node.id}
              node={node}
              position={pos}
              scale={scale}
              selected={selectedNodes.has(node.id)}
              onSelect={(addToSelection) => onNodeSelect(node.id, addToSelection)}
              onUpdate={(updates) => onNodeUpdate(node.id, updates)}
              onDelete={() => onNodeDelete(node.id)}
              onDrag={(deltaX, deltaY) => handleNodeDrag(node.id, deltaX, deltaY)}
              onPortMouseDown={(portId, portType) => handlePortMouseDown(node.id, portId, portType)}
              onPortMouseUp={(portId, portType) => handlePortMouseUp(node.id, portId, portType)}
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
        <div>🖱️ Middle Click / Space + Drag: Pan</div>
        <div>🔍 Ctrl + Scroll: Zoom</div>
        <div>⌫ Delete: Remove Selected</div>
        <div>Ctrl+C/V: Copy/Paste</div>
      </div>
    </div>
  );
};

export default NodeCanvas;
