/**
 * Properties Panel Component
 * Displays and edits properties of selected behavior tree nodes
 */

import React, { useState, useEffect } from 'react';
import { BehaviorNode as BehaviorNodeType, NodeParameter, NodeType } from '../../types/behavior';
import './PropertiesPanel.css';

interface PropertiesPanelProps {
  node: BehaviorNodeType | null;
  onNodeUpdate: (nodeId: string, updates: Partial<BehaviorNodeType>) => void;
  isOpen: boolean;
  onToggle: () => void;
}

export const PropertiesPanel: React.FC<PropertiesPanelProps> = ({
  node,
  onNodeUpdate,
  isOpen,
  onToggle,
}) => {
  const [localName, setLocalName] = useState('');
  const [localDescription, setLocalDescription] = useState('');
  const [localParameters, setLocalParameters] = useState<NodeParameter[]>([]);

  useEffect(() => {
    if (node) {
      setLocalName(node.name);
      setLocalDescription(node.description || '');
      setLocalParameters(node.parameters);
    }
  }, [node]);

  if (!isOpen) {
    return (
      <button className="properties-panel-toggle" onClick={onToggle}>
        <span>⚙️</span>
      </button>
    );
  }

  if (!node) {
    return (
      <div className="properties-panel">
        <div className="panel-header">
          <h3>Properties</h3>
          <button className="close-button" onClick={onToggle}>
            ×
          </button>
        </div>
        <div className="panel-empty">
          <p>No node selected</p>
          <p className="hint">Select a node to view and edit its properties</p>
        </div>
      </div>
    );
  }

  const handleNameChange = (name: string) => {
    setLocalName(name);
    onNodeUpdate(node.id, { name });
  };

  const handleDescriptionChange = (description: string) => {
    setLocalDescription(description);
    onNodeUpdate(node.id, { description });
  };

  const handleParameterChange = (paramId: string, value: any) => {
    const updatedParameters = localParameters.map(param =>
      param.id === paramId ? { ...param, value } : param
    );
    setLocalParameters(updatedParameters);
    onNodeUpdate(node.id, { parameters: updatedParameters });
  };

  const renderParameterEditor = (param: NodeParameter) => {
    switch (param.dataType) {
      case 'boolean':
        return (
          <label className="parameter-toggle">
            <input
              type="checkbox"
              checked={param.value}
              onChange={(e) => handleParameterChange(param.id, e.target.checked)}
            />
            <span className="toggle-slider"></span>
          </label>
        );

      case 'number':
        return (
          <div className="parameter-number">
            <input
              type="number"
              value={param.value}
              min={param.min}
              max={param.max}
              step={param.step || 0.01}
              onChange={(e) => handleParameterChange(param.id, parseFloat(e.target.value) || 0)}
            />
            {(param.min !== undefined || param.max !== undefined) && (
              <span className="parameter-hint">
                [{param.min ?? '-∞'}, {param.max ?? '∞'}]
              </span>
            )}
          </div>
        );

      case 'enum':
        return (
          <select
            value={param.value}
            onChange={(e) => handleParameterChange(param.id, e.target.value)}
            className="parameter-select"
          >
            {param.enumValues?.map(enumValue => (
              <option key={enumValue} value={enumValue}>
                {enumValue}
              </option>
            ))}
          </select>
        );

      case 'string':
        return (
          <input
            type="text"
            value={param.value}
            onChange={(e) => handleParameterChange(param.id, e.target.value)}
            className="parameter-input"
          />
        );

      case 'object':
        return (
          <textarea
            value={typeof param.value === 'object' ? JSON.stringify(param.value, null, 2) : param.value}
            onChange={(e) => {
              try {
                const parsed = JSON.parse(e.target.value);
                handleParameterChange(param.id, parsed);
              } catch {
                handleParameterChange(param.id, e.target.value);
              }
            }}
            className="parameter-textarea"
            rows={4}
          />
        );

      default:
        return <span className="parameter-value">{String(param.value)}</span>;
    }
  };

  return (
    <div className="properties-panel">
      <div className="panel-header">
        <h3>Properties</h3>
        <button className="close-button" onClick={onToggle}>
          ×
        </button>
      </div>

      <div className="panel-content">
        {/* Node Type Badge */}
        <div className="property-section">
          <div className="node-type-badge-large">
            <span className="node-icon">{getNodeIcon(node.type)}</span>
            <span className="node-type-name">{node.type}</span>
          </div>
        </div>

        {/* Basic Properties */}
        <div className="property-section">
          <label className="property-label">Name</label>
          <input
            type="text"
            value={localName}
            onChange={(e) => handleNameChange(e.target.value)}
            className="property-input"
          />
        </div>

        <div className="property-section">
          <label className="property-label">Description</label>
          <textarea
            value={localDescription}
            onChange={(e) => handleDescriptionChange(e.target.value)}
            className="property-textarea"
            rows={3}
            placeholder="Add a description..."
          />
        </div>

        {/* Node ID (Read-only) */}
        <div className="property-section">
          <label className="property-label">Node ID</label>
          <input
            type="text"
            value={node.id}
            readOnly
            className="property-input readonly"
          />
        </div>

        {/* Position (Read-only) */}
        <div className="property-section">
          <label className="property-label">Position</label>
          <div className="position-display">
            <span>X: {Math.round(node.position.x)}</span>
            <span>Y: {Math.round(node.position.y)}</span>
          </div>
        </div>

        {/* Parameters */}
        {localParameters.length > 0 && (
          <div className="property-section">
            <div className="section-header">Parameters</div>
            {localParameters.map(param => (
              <div key={param.id} className="parameter-item">
                <label className="parameter-label">{param.name}</label>
                {renderParameterEditor(param)}
                {param.dataType === 'string' && param.name !== 'Message' && (
                  <div className="parameter-hint">
                    Type: {param.dataType}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}

        {/* Debug Info */}
        {node.status && (
          <div className="property-section">
            <div className="section-header">Debug Info</div>
            <div className="debug-info">
              <div className="debug-row">
                <span className="debug-label">Status:</span>
                <span className={`debug-status debug-status-${node.status}`}>
                  {node.status}
                </span>
              </div>
              {node.executionCount !== undefined && (
                <div className="debug-row">
                  <span className="debug-label">Executions:</span>
                  <span>{node.executionCount}</span>
                </div>
              )}
              {node.lastExecutionTime !== undefined && (
                <div className="debug-row">
                  <span className="debug-label">Last Time:</span>
                  <span>{node.lastExecutionTime.toFixed(2)}ms</span>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Children Count */}
        {node.children.length > 0 && (
          <div className="property-section">
            <div className="section-header">Children</div>
            <div className="children-info">
              {node.children.length} child{node.children.length > 1 ? 'ren' : ''}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

// Helper function to get node icon
function getNodeIcon(type: NodeType): string {
  switch (type) {
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
}

export default PropertiesPanel;
