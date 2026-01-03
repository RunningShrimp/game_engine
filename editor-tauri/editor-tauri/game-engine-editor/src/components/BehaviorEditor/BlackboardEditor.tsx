/**
 * Blackboard Editor Component
 * Manages blackboard variables for behavior trees
 */

import React, { useState, useEffect } from 'react';
import { Blackboard, BlackboardVariable } from '../../types/behavior';
import './BlackboardEditor.css';

interface BlackboardEditorProps {
  blackboard: Blackboard;
  onBlackboardUpdate: (blackboard: Blackboard) => void;
  isOpen: boolean;
  onToggle: () => void;
}

export const BlackboardEditor: React.FC<BlackboardEditorProps> = ({
  blackboard,
  onBlackboardUpdate,
  isOpen,
  onToggle,
}) => {
  const [variables, setVariables] = useState<Record<string, BlackboardVariable>>({});
  const [newVarName, setNewVarName] = useState('');
  const [newVarType, setNewVarType] = useState<BlackboardVariable['type']>('boolean');
  const [newVarValue, setNewVarValue] = useState<any>('');
  const [filter, setFilter] = useState('');

  useEffect(() => {
    setVariables(blackboard.variables);
  }, [blackboard]);

  const handleAddVariable = () => {
    if (!newVarName.trim()) return;

    const parsedValue = parseValue(newVarValue, newVarType);
    const newVariable: BlackboardVariable = {
      name: newVarName,
      type: newVarType,
      value: parsedValue,
    };

    const updatedVariables = {
      ...variables,
      [newVarName]: newVariable,
    };

    setVariables(updatedVariables);
    onBlackboardUpdate({ variables: updatedVariables });

    // Reset form
    setNewVarName('');
    setNewVarValue('');
  };

  const handleRemoveVariable = (varName: string) => {
    const updatedVariables = { ...variables };
    delete updatedVariables[varName];

    setVariables(updatedVariables);
    onBlackboardUpdate({ variables: updatedVariables });
  };

  const handleUpdateVariable = (varName: string, value: any) => {
    const variable = variables[varName];
    if (!variable) return;

    const updatedVariable = { ...variable, value };
    const updatedVariables = {
      ...variables,
      [varName]: updatedVariable,
    };

    setVariables(updatedVariables);
    onBlackboardUpdate({ variables: updatedVariables });
  };

  const parseValue = (value: string, type: BlackboardVariable['type']): any => {
    switch (type) {
      case 'boolean':
        return value === 'true';
      case 'number':
        return parseFloat(value) || 0;
      case 'object':
        try {
          return JSON.parse(value);
        } catch {
          return {};
        }
      case 'vector3':
        try {
          const parsed = JSON.parse(value);
          return Array.isArray(parsed) ? parsed : [0, 0, 0];
        } catch {
          return [0, 0, 0];
        }
      default:
        return value;
    }
  };

  const formatValue = (value: any, type: BlackboardVariable['type']): string => {
    switch (type) {
      case 'object':
      case 'vector3':
        return JSON.stringify(value);
      default:
        return String(value);
    }
  };

  const renderValueEditor = (varName: string) => {
    const variable = variables[varName];
    if (!variable) return null;

    switch (variable.type) {
      case 'boolean':
        return (
          <label className="variable-toggle">
            <input
              type="checkbox"
              checked={variable.value}
              onChange={(e) => handleUpdateVariable(varName, e.target.checked)}
            />
            <span className="toggle-slider"></span>
          </label>
        );

      case 'number':
        return (
          <input
            type="number"
            value={variable.value}
            onChange={(e) => handleUpdateVariable(varName, parseFloat(e.target.value) || 0)}
            className="variable-input"
          />
        );

      case 'object':
      case 'vector3':
        return (
          <textarea
            value={formatValue(variable.value, variable.type)}
            onChange={(e) => {
              try {
                const parsed = JSON.parse(e.target.value);
                handleUpdateVariable(varName, parsed);
              } catch {
                // Invalid JSON, ignore
              }
            }}
            className="variable-textarea"
            rows={variable.type === 'vector3' ? 1 : 3}
          />
        );

      default:
        return (
          <input
            type="text"
            value={variable.value}
            onChange={(e) => handleUpdateVariable(varName, e.target.value)}
            className="variable-input"
          />
        );
    }
  };

  const filteredVariables = Object.entries(variables).filter(([name]) =>
    name.toLowerCase().includes(filter.toLowerCase())
  );

  if (!isOpen) {
    return (
      <button className="blackboard-toggle" onClick={onToggle} title="Blackboard">
        <span>📋</span>
      </button>
    );
  }

  return (
    <div className="blackboard-editor">
      <div className="panel-header">
        <h3>Blackboard</h3>
        <button className="close-button" onClick={onToggle}>
          ×
        </button>
      </div>

      {/* Search */}
      <div className="blackboard-search">
        <input
          type="text"
          placeholder="Search variables..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          className="search-input"
        />
      </div>

      {/* Variables List */}
      <div className="variables-list">
        {filteredVariables.length === 0 ? (
          <div className="empty-state">
            <p>No variables yet</p>
            <p className="hint">Add variables to store data for your behavior tree</p>
          </div>
        ) : (
          filteredVariables.map(([name, variable]) => (
            <div key={name} className="variable-item">
              <div className="variable-header">
                <span className="variable-name">{name}</span>
                <span className={`variable-type type-${variable.type}`}>
                  {variable.type}
                </span>
                <button
                  className="delete-button"
                  onClick={() => handleRemoveVariable(name)}
                  title="Remove variable"
                >
                  🗑️
                </button>
              </div>
              <div className="variable-value">
                {renderValueEditor(name)}
              </div>
              {variable.description && (
                <div className="variable-description">{variable.description}</div>
              )}
            </div>
          ))
        )}
      </div>

      {/* Add Variable Form */}
      <div className="add-variable-form">
        <div className="form-header">Add Variable</div>
        <div className="form-row">
          <input
            type="text"
            placeholder="Variable name"
            value={newVarName}
            onChange={(e) => setNewVarName(e.target.value)}
            className="form-input"
          />
          <select
            value={newVarType}
            onChange={(e) => setNewVarType(e.target.value as any)}
            className="form-select"
          >
            <option value="boolean">Boolean</option>
            <option value="number">Number</option>
            <option value="string">String</option>
            <option value="object">Object</option>
            <option value="vector3">Vector3</option>
          </select>
        </div>
        <div className="form-row">
          <input
            type="text"
            placeholder="Initial value"
            value={newVarValue}
            onChange={(e) => setNewVarValue(e.target.value)}
            className="form-input"
          />
          <button
            className="add-button"
            onClick={handleAddVariable}
            disabled={!newVarName.trim()}
          >
            Add
          </button>
        </div>
      </div>

      {/* Variable Count */}
      <div className="variables-count">
        {Object.keys(variables).length} variable{Object.keys(variables).length !== 1 ? 's' : ''}
      </div>
    </div>
  );
};

export default BlackboardEditor;
