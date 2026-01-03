import React, { useState, useEffect } from 'react';
import { Entity, Transform, Component } from '../../types/engine';

interface PropertyInspectorProps {
  entities: Entity[];
  selectedEntities: string[];
  onTransformChange: (entityId: string, transform: Transform) => void;
  onComponentToggle?: (entityId: string, componentId: string, enabled: boolean) => void;
  onEntityRename?: (entityId: string, name: string) => void;
}

export const PropertyInspector: React.FC<PropertyInspectorProps> = ({
  entities,
  selectedEntities,
  onTransformChange,
  onComponentToggle,
  onEntityRename,
}) => {
  const selectedEntity =
    selectedEntities.length === 1
      ? entities.find((e) => e.id === selectedEntities[0])
      : null;

  // Local state for entity name editing
  const [entityName, setEntityName] = useState(selectedEntity?.name || '');
  const [isEditingName, setIsEditingName] = useState(false);

  // Update local state when selection changes
  useEffect(() => {
    if (selectedEntity) {
      setEntityName(selectedEntity.name);
      setIsEditingName(false);
    }
  }, [selectedEntity?.id]);

  if (!selectedEntity) {
    return (
      <div className="flex flex-col h-full bg-slate-900">
        <div className="p-3 border-b border-slate-700">
          <h2 className="text-sm font-semibold text-slate-200">Properties</h2>
        </div>
        <div className="flex-1 flex items-center justify-center text-slate-500">
          <p className="text-sm">No entity selected</p>
        </div>
      </div>
    );
  }

  const Vector3Input = ({
    label,
    value,
    onChange,
    step = 0.1,
  }: {
    label: string;
    value: { x: number; y: number; z: number };
    onChange: (axis: 'x' | 'y' | 'z', newValue: number) => void;
    step?: number;
  }) => (
    <div className="space-y-1">
      <label className="text-xs text-slate-400">{label}</label>
      <div className="flex gap-2">
        {(['x', 'y', 'z'] as const).map((axis) => (
          <div key={axis} className="flex-1">
            <label className={`text-xs font-semibold ${
              axis === 'x' ? 'text-red-400' :
              axis === 'y' ? 'text-green-400' :
              'text-blue-400'
            }`}>
              {axis.toUpperCase()}
            </label>
            <input
              type="number"
              step={step}
              value={value[axis]}
              onChange={(e) => onChange(axis, parseFloat(e.target.value) || 0)}
              className="w-full px-2 py-1 text-sm bg-slate-800 border border-slate-700 rounded text-slate-200 focus:border-blue-500 focus:outline-none"
            />
          </div>
        ))}
      </div>
    </div>
  );

  const handleNameSubmit = () => {
    if (onEntityRename && entityName.trim() !== '') {
      onEntityRename(selectedEntity.id, entityName.trim());
    } else {
      setEntityName(selectedEntity.name);
    }
    setIsEditingName(false);
  };

  const handleComponentToggle = (componentId: string, enabled: boolean) => {
    if (onComponentToggle) {
      onComponentToggle(selectedEntity.id, componentId, enabled);
    }
  };

  return (
    <div className="flex flex-col h-full bg-slate-900">
      {/* Header */}
      <div className="p-3 border-b border-slate-700">
        <h2 className="text-sm font-semibold text-slate-200">Properties</h2>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-3 space-y-4">
        {/* Entity Info */}
        <div className="space-y-2">
          <h3 className="text-xs font-semibold text-slate-400 uppercase">Entity</h3>
          <div className="space-y-2">
            <div>
              <label className="text-xs text-slate-400">Name</label>
              {isEditingName ? (
                <div className="flex gap-1">
                  <input
                    type="text"
                    value={entityName}
                    onChange={(e) => setEntityName(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') handleNameSubmit();
                      if (e.key === 'Escape') {
                        setEntityName(selectedEntity.name);
                        setIsEditingName(false);
                      }
                    }}
                    onBlur={handleNameSubmit}
                    autoFocus
                    className="flex-1 px-2 py-1 text-sm bg-slate-800 border border-slate-700 rounded text-slate-200 focus:border-blue-500 focus:outline-none"
                  />
                </div>
              ) : (
                <div
                  className="w-full px-2 py-1 text-sm bg-slate-800 border border-slate-700 rounded text-slate-200 cursor-pointer hover:border-slate-600"
                  onClick={() => setIsEditingName(true)}
                >
                  {selectedEntity.name}
                </div>
              )}
            </div>
            <div>
              <label className="text-xs text-slate-400">ID</label>
              <input
                type="text"
                value={selectedEntity.id}
                disabled
                className="w-full px-2 py-1 text-sm bg-slate-800 border border-slate-700 rounded text-slate-500 cursor-not-allowed"
              />
            </div>
            <div className="flex gap-2">
              <div className="flex-1">
                <label className="text-xs text-slate-400">Visible</label>
                <div className="mt-1 flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={selectedEntity.visible}
                    disabled
                    className="w-4 h-4"
                  />
                  <span className="text-xs text-slate-500">
                    {selectedEntity.visible ? 'Yes' : 'No'}
                  </span>
                </div>
              </div>
              <div className="flex-1">
                <label className="text-xs text-slate-400">Locked</label>
                <div className="mt-1 flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={selectedEntity.locked}
                    disabled
                    className="w-4 h-4"
                  />
                  <span className="text-xs text-slate-500">
                    {selectedEntity.locked ? 'Yes' : 'No'}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Transform */}
        <div className="space-y-3">
          <h3 className="text-xs font-semibold text-slate-400 uppercase flex items-center justify-between">
            <span>Transform</span>
            <span className="text-slate-600 font-normal">World</span>
          </h3>

          <Vector3Input
            label="Position"
            value={selectedEntity.transform.position}
            onChange={(axis, newValue) => {
              const newTransform: Transform = {
                ...selectedEntity.transform,
                position: { ...selectedEntity.transform.position, [axis]: newValue },
              };
              onTransformChange(selectedEntity.id, newTransform);
            }}
          />

          <Vector3Input
            label="Rotation (Euler)"
            value={{
              x: selectedEntity.transform.rotation.x,
              y: selectedEntity.transform.rotation.y,
              z: selectedEntity.transform.rotation.z,
            }}
            step={1}
            onChange={(axis, newValue) => {
              const newTransform: Transform = {
                ...selectedEntity.transform,
                rotation: { ...selectedEntity.transform.rotation, [axis]: newValue },
              };
              onTransformChange(selectedEntity.id, newTransform);
            }}
          />

          <Vector3Input
            label="Scale"
            value={selectedEntity.transform.scale}
            step={0.01}
            onChange={(axis, newValue) => {
              const newTransform: Transform = {
                ...selectedEntity.transform,
                scale: { ...selectedEntity.transform.scale, [axis]: newValue },
              };
              onTransformChange(selectedEntity.id, newTransform);
            }}
          />
        </div>

        {/* Components */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-semibold text-slate-400 uppercase">
              Components ({selectedEntity.components.length})
            </h3>
            <button className="text-xs text-blue-400 hover:text-blue-300 transition-colors">
              + Add Component
            </button>
          </div>

          {selectedEntity.components.length === 0 ? (
            <div className="text-center py-4 text-slate-600 text-sm">
              No components attached
            </div>
          ) : (
            selectedEntity.components.map((component) => (
              <ComponentItem
                key={component.id}
                component={component}
                onToggle={(enabled) => handleComponentToggle(component.id, enabled)}
              />
            ))
          )}
        </div>
      </div>
    </div>
  );
};

// Separate component for individual component items
interface ComponentItemProps {
  component: Component;
  onToggle: (enabled: boolean) => void;
}

const ComponentItem: React.FC<ComponentItemProps> = ({ component, onToggle }) => {
  const [isExpanded, setIsExpanded] = useState(true);

  return (
    <div className="bg-slate-800 rounded border border-slate-700 overflow-hidden">
      {/* Component Header */}
      <div className="flex items-center justify-between p-2 cursor-pointer hover:bg-slate-700 transition-colors">
        <div
          className="flex items-center gap-2 flex-1"
          onClick={() => setIsExpanded(!isExpanded)}
        >
          <svg
            className={`w-3 h-3 text-slate-400 transition-transform ${
              isExpanded ? 'rotate-90' : ''
            }`}
            fill="currentColor"
            viewBox="0 0 20 20"
          >
            <path d="M6 6L14 10L6 14V6Z" />
          </svg>
          <span className="text-sm text-slate-200 font-medium">
            {component.name}
          </span>
          <span className="text-xs text-slate-500 bg-slate-900 px-1.5 py-0.5 rounded">
            {component.type}
          </span>
        </div>
        <div className="flex items-center gap-2">
          <label className="flex items-center gap-1 text-xs text-slate-400 cursor-pointer">
            <input
              type="checkbox"
              checked={component.enabled}
              onChange={(e) => onToggle(e.target.checked)}
              className="w-3 h-3 rounded"
            />
            <span>Enabled</span>
          </label>
        </div>
      </div>

      {/* Component Properties */}
      {isExpanded && (
        <div className="px-2 pb-2 space-y-1">
          {Object.keys(component.properties).length === 0 ? (
            <div className="text-xs text-slate-600 py-2 px-1">
              No properties
            </div>
          ) : (
            Object.entries(component.properties).map(([key, value]) => (
              <div key={key} className="flex items-center gap-2 py-1">
                <label className="text-xs text-slate-400 w-1/2 truncate">
                  {key}
                </label>
                <div className="flex-1">
                  {typeof value === 'boolean' ? (
                    <input
                      type="checkbox"
                      checked={value}
                      disabled
                      className="w-3 h-3"
                    />
                  ) : typeof value === 'number' ? (
                    <input
                      type="number"
                      value={value}
                      disabled
                      className="w-full px-2 py-0.5 text-xs bg-slate-900 border border-slate-700 rounded text-slate-500"
                    />
                  ) : (
                    <input
                      type="text"
                      value={String(value)}
                      disabled
                      className="w-full px-2 py-0.5 text-xs bg-slate-900 border border-slate-700 rounded text-slate-500"
                    />
                  )}
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
};
