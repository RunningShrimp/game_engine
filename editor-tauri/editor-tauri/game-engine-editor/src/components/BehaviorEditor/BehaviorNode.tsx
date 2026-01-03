/**
 * Behavior Node Component
 * Renders individual nodes in the behavior tree with drag-and-drop support
 */

import React, { useRef, useEffect, useState } from 'react';
import { BehaviorNode as BehaviorNodeType, NodeType, NodeStatus } from '../../types/behavior';
import './BehaviorNode.css';

interface BehaviorNodeProps {
  node: BehaviorNodeType;
  position: { x: number; y: number };
  scale: number;
  selected: boolean;
  onSelect: (addToSelection: boolean) => void;
  onUpdate: (updates: Partial<BehaviorNodeType>) => void;
  onDelete: () => void;
  onDrag: (deltaX: number, deltaY: number) => void;
  onChildDrop?: (parentId: string, childId: string) => void;
  canHaveChildren?: boolean;
}

export const BehaviorNode: React.FC<BehaviorNodeProps> = ({
  node,
  position,
  scale,
  selected,
  onSelect,
  onUpdate,
  onDelete,
  onDrag,
  onChildDrop,
  canHaveChildren = true,
}) => {
  const nodeRef = useRef<HTMLDivElement>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const [isOver, setIsOver] = useState(false);

  // Get node icon
  const getNodeIcon = () => {
    switch (node.type) {
      case NodeType.Sequence:
        return '➡️';
      case NodeType.Selector:
        return '⭕';
      case NodeType.Parallel:
        return '⫴';
      case NodeType.Inverter:
        return '🔄';
      case NodeType.Repeater:
        return '🔁';
      case NodeType.Cooldown:
        return '⏱️';
      case NodeType.AlwaysSucceed:
        return '✅';
      case NodeType.AlwaysFail:
        return '❌';
      case NodeType.Condition:
      case NodeType.Check:
        return '❓';
      case NodeType.Action:
        return '⚡';
      case NodeType.Wait:
        return '⏸️';
      case NodeType.Log:
        return '📝';
      default:
        return '📦';
    }
  };

  // Get node shape class
  const getNodeShapeClass = () => {
    switch (node.type) {
      case NodeType.Sequence:
      case NodeType.Selector:
      case NodeType.Parallel:
        return 'node-composite';
      case NodeType.Inverter:
      case NodeType.Repeater:
      case NodeType.Cooldown:
      case NodeType.AlwaysSucceed:
      case NodeType.AlwaysFail:
        return 'node-decorator';
      case NodeType.Condition:
      case NodeType.Check:
        return 'node-condition';
      case NodeType.Action:
      case NodeType.Wait:
      case NodeType.Log:
        return 'node-action';
      default:
        return 'node-default';
    }
  };

  // Get status indicator
  const getStatusIndicator = () => {
    switch (node.status) {
      case NodeStatus.Running:
        return '🟡';
      case NodeStatus.Success:
        return '🟢';
      case NodeStatus.Failure:
        return '🔴';
      default:
        return '⚪';
    }
  };

  // Handle mouse down for dragging
  const handleMouseDown = (e: React.MouseEvent) => {
    if (e.button !== 0) return; // Only left click
    e.stopPropagation();

    const addToSelection = e.ctrlKey || e.metaKey;
    onSelect(addToSelection);

    setIsDragging(true);
    setDragStart({ x: e.clientX, y: e.clientY });
  };

  // Handle mouse move for dragging
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isDragging) return;

      const deltaX = e.clientX - dragStart.x;
      const deltaY = e.clientY - dragStart.y;

      onDrag(deltaX, deltaY);
      setDragStart({ x: e.clientX, y: e.clientY });
    };

    const handleMouseUp = () => {
      setIsDragging(false);
    };

    if (isDragging) {
      window.addEventListener('mousemove', handleMouseMove);
      window.addEventListener('mouseup', handleMouseUp);
    }

    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isDragging, dragStart, onDrag]);

  // Handle drop for child nodes
  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    if (canHaveChildren) {
      setIsOver(true);
    }
  };

  const handleDragLeave = () => {
    setIsOver(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsOver(false);

    if (canHaveChildren && onChildDrop) {
      const childId = e.dataTransfer.getData('nodeId');
      if (childId && childId !== node.id) {
        onChildDrop(node.id, childId);
      }
    }
  };

  return (
    <div
      ref={nodeRef}
      className={`behavior-node ${getNodeShapeClass()} ${selected ? 'selected' : ''} ${isOver ? 'drag-over' : ''}`}
      style={{
        position: 'absolute',
        left: position.x,
        top: position.y,
        transform: `scale(${scale})`,
        transformOrigin: 'top left',
        cursor: isDragging ? 'grabbing' : 'grab',
      }}
      onMouseDown={handleMouseDown}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      draggable={canHaveChildren}
      onDragStart={(e) => {
        e.dataTransfer.setData('nodeId', node.id);
      }}
    >
      {/* Status Indicator */}
      {node.status && node.status !== NodeStatus.Idle && (
        <div className="node-status">{getStatusIndicator()}</div>
      )}

      {/* Node Header */}
      <div className="node-header">
        <span className="node-icon">{getNodeIcon()}</span>
        <span className="node-name">{node.name}</span>
      </div>

      {/* Node Type Badge */}
      <div className="node-type-badge">{node.type}</div>

      {/* Execution Count */}
      {node.executionCount !== undefined && node.executionCount > 0 && (
        <div className="node-execution-count">
          Executed: {node.executionCount}
        </div>
      )}

      {/* Child Count Indicator */}
      {node.children.length > 0 && (
        <div className="node-child-count">
          {node.children.length} child{node.children.length > 1 ? 'ren' : ''}
        </div>
      )}

      {/* Connection Point (Bottom) */}
      {canHaveChildren && (
        <div className="connection-point connection-point-bottom" />
      )}

      {/* Connection Point (Top - for non-root nodes) */}
      <div className="connection-point connection-point-top" />
    </div>
  );
};

export default BehaviorNode;
