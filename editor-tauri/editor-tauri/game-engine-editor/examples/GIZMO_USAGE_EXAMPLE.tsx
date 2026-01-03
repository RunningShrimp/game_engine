/**
 * Gizmo System Usage Example
 *
 * This example demonstrates how to use the 3D Gizmo system
 * in the Tauri Game Engine Editor.
 */

import React, { useState } from 'react';
import { Viewport } from '../src/components/Viewport/Viewport';
import { Entity, TransformMode, Space, Vector3 } from '../src/types/engine';

// Sample entity data
const sampleEntities: Entity[] = [
  {
    id: 'entity-1',
    name: 'Cube',
    transform: {
      position: { x: 0, y: 0, z: 0 },
      rotation: { x: 0, y: 0, z: 0, w: 1 },
      scale: { x: 1, y: 1, z: 1 }
    },
    components: [],
    children: [],
    visible: true,
    locked: false
  },
  {
    id: 'entity-2',
    name: 'Sphere',
    transform: {
      position: { x: 2, y: 0, z: 0 },
      rotation: { x: 0, y: 0, z: 0, w: 1 },
      scale: { x: 1, y: 1, z: 1 }
    },
    components: [],
    children: [],
    visible: true,
    locked: false
  }
];

export const GizmoExample: React.FC = () => {
  // State management
  const [entities, setEntities] = useState<Entity[]>(sampleEntities);
  const [selectedEntities, setSelectedEntities] = useState<string[]>(['entity-1']);
  const [transformMode, setTransformMode] = useState<TransformMode>(TransformMode.Translate);
  const [space, setSpace] = useState<Space>(Space.World);
  const [snapEnabled, setSnapEnabled] = useState(true);
  const [snapValue, setSnapValue] = useState(1.0);
  const [showGrid, setShowGrid] = useState(true);
  const [showStats, setShowStats] = useState(true);

  // Handle entity transformation
  const handleEntityTransform = (entityId: string, transform: Partial<Vector3>) => {
    setEntities(prevEntities =>
      prevEntities.map(entity => {
        if (entity.id === entityId) {
          return {
            ...entity,
            transform: {
              ...entity.transform,
              position: {
                ...entity.transform.position,
                ...transform
              }
            }
          };
        }
        return entity;
      })
    );
  };

  // Transform mode handlers
  const handleTranslateMode = () => setTransformMode(TransformMode.Translate);
  const handleRotateMode = () => setTransformMode(TransformMode.Rotate);
  const handleScaleMode = () => setTransformMode(TransformMode.Scale);

  // Space toggle handler
  const handleToggleSpace = () => {
    setSpace(prev => prev === Space.World ? Space.Local : Space.World);
  };

  return (
    <div style={{ width: '100%', height: '100vh', display: 'flex', flexDirection: 'column' }}>
      {/* Toolbar */}
      <div style={{
        padding: '10px',
        background: '#1e293b',
        borderBottom: '1px solid #334155',
        display: 'flex',
        gap: '10px',
        alignItems: 'center'
      }}>
        {/* Transform Mode Buttons */}
        <div style={{ display: 'flex', gap: '5px' }}>
          <button
            onClick={handleTranslateMode}
            style={{
              padding: '8px 16px',
              background: transformMode === TransformMode.Translate ? '#3b82f6' : '#475569',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontWeight: transformMode === TransformMode.Translate ? 'bold' : 'normal'
            }}
          >
            Translate (W)
          </button>
          <button
            onClick={handleRotateMode}
            style={{
              padding: '8px 16px',
              background: transformMode === TransformMode.Rotate ? '#3b82f6' : '#475569',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontWeight: transformMode === TransformMode.Rotate ? 'bold' : 'normal'
            }}
          >
            Rotate (E)
          </button>
          <button
            onClick={handleScaleMode}
            style={{
              padding: '8px 16px',
              background: transformMode === TransformMode.Scale ? '#3b82f6' : '#475569',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontWeight: transformMode === TransformMode.Scale ? 'bold' : 'normal'
            }}
          >
            Scale (R)
          </button>
        </div>

        <div style={{ width: '1px', height: '30px', background: '#334155' }} />

        {/* Space Toggle */}
        <button
          onClick={handleToggleSpace}
          style={{
            padding: '8px 16px',
            background: '#475569',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer'
          }}
        >
          {space === Space.World ? 'World' : 'Local'}
        </button>

        <div style={{ width: '1px', height: '30px', background: '#334155' }} />

        {/* Snap Controls */}
        <label style={{ color: 'white', display: 'flex', alignItems: 'center', gap: '5px' }}>
          <input
            type="checkbox"
            checked={snapEnabled}
            onChange={(e) => setSnapEnabled(e.target.checked)}
          />
          Snap
        </label>

        <input
          type="number"
          value={snapValue}
          onChange={(e) => setSnapValue(parseFloat(e.target.value) || 0.1)}
          step="0.1"
          min="0.1"
          style={{
            padding: '5px',
            width: '60px',
            background: '#475569',
            color: 'white',
            border: '1px solid #64748b',
            borderRadius: '4px'
          }}
        />

        <div style={{ width: '1px', height: '30px', background: '#334155' }} />

        {/* View Options */}
        <label style={{ color: 'white', display: 'flex', alignItems: 'center', gap: '5px' }}>
          <input
            type="checkbox"
            checked={showGrid}
            onChange={(e) => setShowGrid(e.target.checked)}
          />
          Grid
        </label>

        <label style={{ color: 'white', display: 'flex', alignItems: 'center', gap: '5px' }}>
          <input
            type="checkbox"
            checked={showStats}
            onChange={(e) => setShowStats(e.target.checked)}
          />
          Stats
        </label>
      </div>

      {/* Entity List */}
      <div style={{
        padding: '10px',
        background: '#1e293b',
        borderBottom: '1px solid #334155',
        display: 'flex',
        gap: '10px'
      }}>
        {entities.map(entity => (
          <button
            key={entity.id}
            onClick={() => setSelectedEntities([entity.id])}
            style={{
              padding: '6px 12px',
              background: selectedEntities.includes(entity.id) ? '#3b82f6' : '#475569',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer'
            }}
          >
            {entity.name}
          </button>
        ))}
      </div>

      {/* Viewport */}
      <div style={{ flex: 1, position: 'relative' }}>
        <Viewport
          entities={entities}
          selectedEntities={selectedEntities}
          transformMode={transformMode}
          space={space}
          gridSize={10}
          snapEnabled={snapEnabled}
          snapValue={snapValue}
          showGrid={showGrid}
          showStats={showStats}
          onEntityTransform={handleEntityTransform}
        />
      </div>

      {/* Info Panel */}
      <div style={{
        position: 'absolute',
        bottom: '20px',
        left: '20px',
        padding: '15px',
        background: 'rgba(30, 41, 59, 0.9)',
        border: '1px solid #475569',
        borderRadius: '8px',
        color: 'white',
        fontSize: '12px',
        maxWidth: '300px'
      }}>
        <h3 style={{ margin: '0 0 10px 0', fontSize: '14px' }}>Gizmo Controls</h3>
        <div style={{ lineHeight: '1.6' }}>
          <div><strong>W:</strong> Translate mode</div>
          <div><strong>E:</strong> Rotate mode</div>
          <div><strong>R:</strong> Scale mode</div>
          <div style={{ marginTop: '10px' }}>
            <strong>Click & Drag</strong> gizmo axes to transform
          </div>
          <div>
            <strong>Toggle Snap</strong> for precise positioning
          </div>
        </div>
      </div>
    </div>
  );
};

export default GizmoExample;

/**
 * Advanced Usage Example: Custom Gizmo Configuration
 */
export const AdvancedGizmoExample: React.FC = () => {
  // This example shows how to customize gizmo behavior
  const [customSnapValue, setCustomSnapValue] = useState(0.5);

  const handleEntityTransform = (entityId: string, transform: Partial<Vector3>) => {
    console.log('Custom transform handler:', { entityId, transform });
    // Implement custom logic here
    // - Send to backend via Tauri API
    // - Apply physics constraints
    // - Update undo/redo history
    // - Trigger animation
  };

  return (
    <div>
      {/* Your custom implementation */}
      <Viewport
        entities={sampleEntities}
        selectedEntities={['entity-1']}
        transformMode={TransformMode.Translate}
        space={Space.World}
        gridSize={10}
        snapEnabled={true}
        snapValue={customSnapValue}
        showGrid={true}
        showStats={false}
        onEntityTransform={handleEntityTransform}
      />
    </div>
  );
};

/**
 * Keyboard Shortcuts Integration
 */
export const KeyboardShortcutsExample: React.FC = () => {
  const [transformMode, setTransformMode] = useState<TransformMode>(TransformMode.Translate);

  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Check if not typing in an input
      if ((e.target as HTMLElement).tagName === 'INPUT') return;

      switch (e.key.toLowerCase()) {
        case 'w':
          setTransformMode(TransformMode.Translate);
          break;
        case 'e':
          setTransformMode(TransformMode.Rotate);
          break;
        case 'r':
          setTransformMode(TransformMode.Scale);
          break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  return (
    <Viewport
      entities={sampleEntities}
      selectedEntities={['entity-1']}
      transformMode={transformMode}
      space={Space.World}
      gridSize={10}
      snapEnabled={false}
      snapValue={1.0}
      showGrid={true}
      showStats={true}
    />
  );
};
