/**
 * EntityTree Component Usage Examples
 *
 * This file demonstrates how to use the EntityTree component
 * and its sub-components in various scenarios.
 */

import React, { useState } from 'react';
import { Entity } from '../../types/engine';
import EntityTree from './index';
import EntityTreeIcon from './EntityTreeIcon';
import EntityTreeToggle from './EntityTreeToggle';
import EntityTreeActions from './EntityTreeActions';
import EntityTreeLabel from './EntityTreeLabel';
import EntityTreeSearch from './EntityTreeSearch';

// Example 1: Basic Usage
export function BasicExample() {
  const [entities, setEntities] = useState<Entity[]>([
    {
      id: '1',
      name: 'Main Camera',
      transform: {
        position: { x: 0, y: 2, z: -5 },
        rotation: { x: 0, y: 0, z: 0, w: 1 },
        scale: { x: 1, y: 1, z: 1 },
      },
      components: [],
      children: [],
      visible: true,
      locked: false,
    },
  ]);

  const [selectedEntities, setSelectedEntities] = useState<string[]>([]);

  return (
    <div style={{ height: '400px' }}>
      <EntityTree
        entities={entities}
        selectedEntities={selectedEntities}
        onEntitySelect={setSelectedEntities}
        onEntityRename={(id, name) => {
          console.log('Rename:', id, 'to', name);
        }}
        onEntityDelete={(id) => {
          console.log('Delete:', id);
        }}
        onEntityCreate={() => {
          console.log('Create new entity');
        }}
        onEntityToggleVisibility={(id) => {
          console.log('Toggle visibility:', id);
        }}
        onEntityToggleLock={(id) => {
          console.log('Toggle lock:', id);
        }}
      />
    </div>
  );
}

// Example 2: Complex Hierarchy
export function ComplexHierarchyExample() {
  const [entities] = useState<Entity[]>([
    {
      id: '1',
      name: 'Scene Root',
      transform: {
        position: { x: 0, y: 0, z: 0 },
        rotation: { x: 0, y: 0, z: 0, w: 1 },
        scale: { x: 1, y: 1, z: 1 },
      },
      components: [],
      children: [
        {
          id: '2',
          name: 'Camera',
          transform: {
            position: { x: 0, y: 2, z: -5 },
            rotation: { x: 0, y: 0, z: 0, w: 1 },
            scale: { x: 1, y: 1, z: 1 },
          },
          components: [],
          children: [],
          parentId: '1',
          visible: true,
          locked: false,
        },
        {
          id: '3',
          name: 'Lights',
          transform: {
            position: { x: 0, y: 0, z: 0 },
            rotation: { x: 0, y: 0, z: 0, w: 1 },
            scale: { x: 1, y: 1, z: 1 },
          },
          components: [],
          children: [
            {
              id: '4',
              name: 'Directional Light',
              transform: {
                position: { x: 5, y: 10, z: 5 },
                rotation: { x: 0, y: 0, z: 0, w: 1 },
                scale: { x: 1, y: 1, z: 1 },
              },
              components: [],
              children: [],
              parentId: '3',
              visible: true,
              locked: false,
            },
            {
              id: '5',
              name: 'Point Light',
              transform: {
                position: { x: -5, y: 5, z: -5 },
                rotation: { x: 0, y: 0, z: 0, w: 1 },
                scale: { x: 1, y: 1, z: 1 },
              },
              components: [],
              children: [],
              parentId: '3',
              visible: true,
              locked: false,
            },
          ],
          parentId: '1',
          visible: true,
          locked: false,
        },
      ],
      visible: true,
      locked: false,
    },
  ]);

  const [selectedEntities, setSelectedEntities] = useState<string[]>([]);

  return (
    <div style={{ height: '600px' }}>
      <EntityTree
        entities={entities}
        selectedEntities={selectedEntities}
        onEntitySelect={setSelectedEntities}
        onEntityRename={(id, name) => {
          console.log('Rename:', id, 'to', name);
        }}
        onEntityDelete={(id) => {
          console.log('Delete:', id);
        }}
        onEntityCreate={() => {
          console.log('Create new entity');
        }}
        onEntityToggleVisibility={(id) => {
          console.log('Toggle visibility:', id);
        }}
        onEntityToggleLock={(id) => {
          console.log('Toggle lock:', id);
        }}
        onEntityReparent={(entityId, newParentId) => {
          console.log('Reparent:', entityId, 'to', newParentId || 'root');
        }}
      />
    </div>
  );
}

// Example 3: Using Individual Sub-Components
export function SubComponentsExample() {
  return (
    <div style={{ padding: '20px', gap: '20px', display: 'flex', flexDirection: 'column' }}>
      {/* EntityTreeIcon */}
      <div>
        <h3>EntityTreeIcon</h3>
        <EntityTreeIcon />
      </div>

      {/* EntityTreeToggle */}
      <div>
        <h3>EntityTreeToggle</h3>
        <EntityTreeToggle
          expanded={false}
          onToggle={() => console.log('Toggle')}
        />
      </div>

      {/* EntityTreeActions */}
      <div>
        <h3>EntityTreeActions</h3>
        <EntityTreeActions
          visible={true}
          locked={false}
          onToggleVisibility={(e) => console.log('Toggle visibility')}
          onToggleLock={(e) => console.log('Toggle lock')}
        />
      </div>

      {/* EntityTreeLabel */}
      <div>
        <h3>EntityTreeLabel</h3>
        <EntityTreeLabel
          name="Example Entity"
          isEditing={false}
          editName=""
          onEditNameChange={(name) => console.log('Edit name:', name)}
          onEditFinish={() => console.log('Finish editing')}
          onEditCancel={() => console.log('Cancel editing')}
          onDoubleClick={() => console.log('Double click')}
        />
      </div>

      {/* EntityTreeSearch */}
      <div>
        <h3>EntityTreeSearch</h3>
        <EntityTreeSearch
          value=""
          onChange={(value) => console.log('Search:', value)}
          placeholder="Search entities..."
        />
      </div>
    </div>
  );
}

// Example 4: With Custom State Management
export function CustomStateExample() {
  const [entities, setEntities] = useState<Entity[]>([]);
  const [selectedEntities, setSelectedEntities] = useState<string[]>([]);
  const [nextId, setNextId] = useState(1);

  const handleEntityCreate = () => {
    const newEntity: Entity = {
      id: String(nextId),
      name: `Entity ${nextId}`,
      transform: {
        position: { x: 0, y: 0, z: 0 },
        rotation: { x: 0, y: 0, z: 0, w: 1 },
        scale: { x: 1, y: 1, z: 1 },
      },
      components: [],
      children: [],
      visible: true,
      locked: false,
    };

    setEntities([...entities, newEntity]);
    setNextId(nextId + 1);
  };

  const handleEntityDelete = (entityId: string) => {
    setEntities(entities.filter((e) => e.id !== entityId));
    setSelectedEntities(selectedEntities.filter((id) => id !== entityId));
  };

  const handleEntityRename = (entityId: string, newName: string) => {
    setEntities(
      entities.map((entity) =>
        entity.id === entityId ? { ...entity, name: newName } : entity
      )
    );
  };

  const handleEntityToggleVisibility = (entityId: string) => {
    setEntities(
      entities.map((entity) =>
        entity.id === entityId
          ? { ...entity, visible: !entity.visible }
          : entity
      )
    );
  };

  const handleEntityToggleLock = (entityId: string) => {
    setEntities(
      entities.map((entity) =>
        entity.id === entityId
          ? { ...entity, locked: !entity.locked }
          : entity
      )
    );
  };

  const handleEntityReparent = (
    entityId: string,
    newParentId: string | null
  ) => {
    // Implementation would recursively move entity in hierarchy
    console.log('Reparent:', entityId, 'to', newParentId || 'root');
  };

  return (
    <div style={{ height: '500px' }}>
      <EntityTree
        entities={entities}
        selectedEntities={selectedEntities}
        onEntitySelect={setSelectedEntities}
        onEntityCreate={handleEntityCreate}
        onEntityDelete={handleEntityDelete}
        onEntityRename={handleEntityRename}
        onEntityToggleVisibility={handleEntityToggleVisibility}
        onEntityToggleLock={handleEntityToggleLock}
        onEntityReparent={handleEntityReparent}
      />
    </div>
  );
}

// Example 5: With Search and Filtering
export function SearchExample() {
  const [entities] = useState<Entity[]>([
    {
      id: '1',
      name: 'Main Camera',
      transform: {
        position: { x: 0, y: 2, z: -5 },
        rotation: { x: 0, y: 0, z: 0, w: 1 },
        scale: { x: 1, y: 1, z: 1 },
      },
      components: [],
      children: [],
      visible: true,
      locked: false,
    },
    {
      id: '2',
      name: 'Spot Light',
      transform: {
        position: { x: 5, y: 10, z: 5 },
        rotation: { x: 0, y: 0, z: 0, w: 1 },
        scale: { x: 1, y: 1, z: 1 },
      },
      components: [],
      children: [],
      visible: true,
      locked: false,
    },
    {
      id: '3',
      name: 'Point Light',
      transform: {
        position: { x: -5, y: 5, z: -5 },
        rotation: { x: 0, y: 0, z: 0, w: 1 },
        scale: { x: 1, y: 1, z: 1 },
      },
      components: [],
      children: [],
      visible: true,
      locked: false,
    },
  ]);

  const [selectedEntities, setSelectedEntities] = useState<string[]>([]);

  return (
    <div style={{ height: '400px' }}>
      <EntityTree
        entities={entities}
        selectedEntities={selectedEntities}
        onEntitySelect={setSelectedEntities}
        onEntityRename={(id, name) => console.log('Rename:', id, name)}
        onEntityDelete={(id) => console.log('Delete:', id)}
        onEntityCreate={() => console.log('Create entity')}
        onEntityToggleVisibility={(id) => console.log('Toggle visibility:', id)}
        onEntityToggleLock={(id) => console.log('Toggle lock:', id)}
      />
    </div>
  );
}

// Export all examples
export default {
  BasicExample,
  ComplexHierarchyExample,
  SubComponentsExample,
  CustomStateExample,
  SearchExample,
};
