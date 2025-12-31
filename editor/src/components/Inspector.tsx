import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface TransformData {
  position: [number, number, number];
  rotation: [number, number, number];
  scale: [number, number, number];
}

interface ComponentData {
  type: string;
  [key: string]: any;
}

interface InspectorProps {
  entityId: number | null;
}

export const Inspector: React.FC<InspectorProps> = ({ entityId }) => {
  const [components, setComponents] = useState<ComponentData[]>([]);
  const [position, setPosition] = useState<[number, number, number]>([0, 0, 0]);
  const [rotation, setRotation] = useState<[number, number, number]>([0, 0, 0]);
  const [scale, setScale] = useState<[number, number, number]>([1, 1, 1]);

  useEffect(() => {
    if (entityId !== null) {
      loadComponents();
    }
  }, [entityId]);

  const loadComponents = async () => {
    if (entityId === null) return;

    try {
      const data = await invoke<ComponentData[]>('get_entity_components', {
        entityId
      });
      setComponents(data);

      // 查找Transform组件并更新状态
      const transform = data.find(c => c.type === 'Transform');
      if (transform) {
        setPosition(transform.position);
        setRotation(transform.rotation);
        setScale(transform.scale);
      }
    } catch (error) {
      console.error('Failed to load components:', error);
    }
  };

  const handleUpdateTransform = async () => {
    if (entityId === null) return;

    try {
      await invoke('update_transform', {
        entityId,
        position,
        rotation,
        scale
      });
    } catch (error) {
      console.error('Failed to update transform:', error);
    }
  };

  const renderVector3 = (
    value: [number, number, number],
    onChange: (value: [number, number, number]) => void,
    labels: string[] = ['X', 'Y', 'Z'],
    step: number = 0.1
  ) => (
    <div className="vector3">
      {value.map((v, i) => (
        <div key={i} className="vector3-field">
          <label>{labels[i]}</label>
          <input
            type="number"
            step={step}
            value={v}
            onChange={(e) => {
              const newValue = [...value] as [number, number, number];
              newValue[i] = parseFloat(e.target.value) || 0;
              onChange(newValue);
            }}
          />
        </div>
      ))}
    </div>
  );

  if (entityId === null) {
    return (
      <div className="inspector">
        <h3>Inspector</h3>
        <div className="no-selection">No entity selected</div>
      </div>
    );
  }

  return (
    <div className="inspector">
      <h3>Inspector</h3>
      <div className="entity-header">
        <span className="entity-id">Entity {entityId}</span>
      </div>

      {/* Transform Component */}
      <div className="section">
        <div className="section-header">
          <h4>Transform</h4>
        </div>

        <div className="section-content">
          <div className="field-group">
            <label>Position</label>
            {renderVector3(position, setPosition)}
          </div>

          <div className="field-group">
            <label>Rotation</label>
            {renderVector3(rotation, setRotation)}
          </div>

          <div className="field-group">
            <label>Scale</label>
            {renderVector3(scale, setScale, ['X', 'Y', 'Z'], 0.01)}
          </div>

          <button onClick={handleUpdateTransform} className="apply-button">
            Apply Transform
          </button>
        </div>
      </div>

      {/* Other Components */}
      {components
        .filter(c => c.type !== 'Transform')
        .map((component, index) => (
          <div key={index} className="section">
            <div className="section-header">
              <h4>{component.type}</h4>
            </div>
            <div className="section-content">
              <pre>{JSON.stringify(component, null, 2)}</pre>
            </div>
          </div>
        ))}

      {/* Add Component Button */}
      <div className="section">
        <button className="add-component-button">
          + Add Component
        </button>
      </div>
    </div>
  );
};
