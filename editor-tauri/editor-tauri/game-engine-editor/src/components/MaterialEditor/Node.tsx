/**
 * Node Component
 * Renders a single material node with inputs, outputs, and parameters
 */

import React, { useState, useRef, useCallback } from 'react';
import { MaterialNode, NodeType } from '../../types/material';
import './Node.css';

interface NodeProps {
  node: MaterialNode;
  position: { x: number; y: number };
  scale: number;
  selected: boolean;
  onSelect: (addToSelection: boolean) => void;
  onUpdate: (updates: Partial<MaterialNode>) => void;
  onDelete: () => void;
  onDrag: (deltaX: number, deltaY: number) => void;
  onPortMouseDown: (portId: string, portType: 'input' | 'output') => void;
  onPortMouseUp: (portId: string, portType: 'input' | 'output') => void;
}

export const Node: React.FC<NodeProps> = ({
  node,
  position,
  scale,
  selected,
  onSelect,
  onUpdate,
  onDelete,
  onDrag,
  onPortMouseDown,
  onPortMouseUp,
}) => {
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const [editMode, setEditMode] = useState(false);
  const [tempLabel, setTempLabel] = useState(node.label || '');

  const nodeRef = useRef<HTMLDivElement>(null);

  // Get node color based on type
  const getNodeColor = useCallback(() => {
    switch (node.type) {
      case NodeType.TextureInput:
      case NodeType.TextureSample:
        return '#8b5cf6'; // Purple
      case NodeType.ColorInput:
        return '#ec4899'; // Pink
      case NodeType.FloatInput:
      case NodeType.Vector2Input:
      case NodeType.Vector3Input:
      case NodeType.Vector4Input:
        return '#3b82f6'; // Blue
      case NodeType.Multiply:
      case NodeType.Add:
      case NodeType.Subtract:
      case NodeType.Divide:
      case NodeType.Lerp:
      case NodeType.Mix:
        return '#f59e0b'; // Amber
      case NodeType.PBRMaster:
        return '#10b981'; // Green
      case NodeType.NormalMap:
        return '#06b6d4'; // Cyan
      case NodeType.UVCoordinate:
      case NodeType.TextureMapping:
        return '#a855f7'; // Purple
      case NodeType.MaterialOutput:
        return '#ef4444'; // Red
      default:
        return '#6b7280'; // Gray
    }
  }, [node.type]);

  // Handle node header drag
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.target !== nodeRef.current && !(e.target as HTMLElement).classList.contains('node-header')) {
      return;
    }

    e.stopPropagation();
    setIsDragging(true);
    setDragStart({ x: e.clientX, y: e.clientY });
  }, []);

  // Handle mouse move for dragging
  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (!isDragging) return;

    const deltaX = e.clientX - dragStart.x;
    const deltaY = e.clientY - dragStart.y;

    onDrag(deltaX, deltaY);
    setDragStart({ x: e.clientX, y: e.clientY });
  }, [isDragging, dragStart, onDrag]);

  // Handle mouse up
  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
  }, []);

  // Handle parameter change
  const handleParameterChange = useCallback((paramId: string, value: any) => {
    const updatedParameters = node.parameters.map(p =>
      p.id === paramId ? { ...p, value } : p
    );
    onUpdate({ parameters: updatedParameters });
  }, [node.parameters, onUpdate]);

  // Get display name for node
  const getDisplayName = useCallback(() => {
    return node.label || node.type.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
  }, [node.label, node.type]);

  return (
    <div
      ref={nodeRef}
      className={`node ${selected ? 'selected' : ''}`}
      style={{
        left: position.x,
        top: position.y,
        borderColor: getNodeColor(),
        transform: `scale(${scale})`,
        transformOrigin: 'top left',
      }}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onClick={(e) => {
        e.stopPropagation();
        onSelect(e.shiftKey);
      }}
    >
      {/* Node Header */}
      <div
        className="node-header"
        style={{ backgroundColor: getNodeColor() }}
        onDoubleClick={() => setEditMode(true)}
      >
        {editMode ? (
          <input
            type="text"
            className="node-label-input"
            value={tempLabel}
            onChange={(e) => setTempLabel(e.target.value)}
            onBlur={() => {
              onUpdate({ label: tempLabel });
              setEditMode(false);
            }}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                onUpdate({ label: tempLabel });
                setEditMode(false);
              } else if (e.key === 'Escape') {
                setTempLabel(node.label || '');
                setEditMode(false);
              }
            }}
            autoFocus
            onClick={(e) => e.stopPropagation()}
          />
        ) : (
          <span className="node-title">{getDisplayName()}</span>
        )}
      </div>

      {/* Node Body */}
      <div className="node-body">
        {/* Input Ports */}
        {node.inputs.map(input => (
          <div key={input.id} className="node-row">
            <div className="port-container input">
              <div
                className="port input-port"
                onMouseDown={(e) => {
                  e.stopPropagation();
                  onPortMouseDown(input.id, 'input');
                }}
                onMouseUp={(e) => {
                  e.stopPropagation();
                  onPortMouseUp(input.id, 'input');
                }}
              />
              <span className="port-label">{input.name}</span>
            </div>
          </div>
        ))}

        {/* Parameters */}
        {node.parameters.map(param => (
          <div key={param.id} className="node-row parameter">
            <label className="parameter-label">{param.name}:</label>
            <div className="parameter-control">
              {param.dataType === 'float' && (
                <input
                  type="number"
                  className="parameter-input"
                  value={param.value}
                  min={param.min}
                  max={param.max}
                  step={param.step || 0.01}
                  onChange={(e) => handleParameterChange(param.id, parseFloat(e.target.value))}
                  onClick={(e) => e.stopPropagation()}
                />
              )}
              {param.dataType === 'color' && (
                <input
                  type="color"
                  className="parameter-color"
                  value={Array.isArray(param.value)
                    ? `#${param.value.slice(0, 3).map(v => Math.round(v * 255).toString(16).padStart(2, '0')).join('')}`
                    : '#ffffff'}
                  onChange={(e) => {
                    const hex = e.target.value;
                    const r = parseInt(hex.slice(1, 3), 16) / 255;
                    const g = parseInt(hex.slice(3, 5), 16) / 255;
                    const b = parseInt(hex.slice(5, 7), 16) / 255;
                    const a = Array.isArray(param.value) ? param.value[3] : 1.0;
                    handleParameterChange(param.id, [r, g, b, a]);
                  }}
                  onClick={(e) => e.stopPropagation()}
                />
              )}
              {param.dataType === 'texture2d' && (
                <input
                  type="text"
                  className="parameter-input"
                  value={param.value}
                  placeholder="Texture path"
                  onChange={(e) => handleParameterChange(param.id, e.target.value)}
                  onClick={(e) => e.stopPropagation()}
                />
              )}
            </div>
          </div>
        ))}

        {/* Output Ports */}
        {node.outputs.map(output => (
          <div key={output.id} className="node-row">
            <div className="port-container output">
              <span className="port-label">{output.name}</span>
              <div
                className="port output-port"
                onMouseDown={(e) => {
                  e.stopPropagation();
                  onPortMouseDown(output.id, 'output');
                }}
                onMouseUp={(e) => {
                  e.stopPropagation();
                  onPortMouseUp(output.id, 'output');
                }}
              />
            </div>
          </div>
        ))}
      </div>

      {/* Resize handle */}
      <div className="node-resize-handle" />
    </div>
  );
};

export default Node;
