/**
 * VirtualEntityTree Usage Example
 *
 * This example demonstrates how to use the VirtualEntityTree component
 * in your application.
 */

import React, { useState } from 'react';
import { VirtualEntityTree } from './VirtualEntityTree';
import { Entity } from '../../types/engine';

// Example: Generate mock entities for testing
const generateMockEntities = (count: number): Entity[] => {
  const entities: Entity[] = [];

  for (let i = 0; i < count; i++) {
    const entity: Entity = {
      id: `entity-${i}`,
      name: `Entity ${i}`,
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

    // Add some children to create hierarchy
    if (i % 5 === 0 && i < count - 1) {
      for (let j = 0; j < 3; j++) {
        const childIndex = i + j + 1;
        if (childIndex < count) {
          const child: Entity = {
            id: `entity-${childIndex}`,
            name: `Child ${j + 1} of Entity ${i}`,
            transform: {
              position: { x: 0, y: 0, z: 0 },
              rotation: { x: 0, y: 0, z: 0, w: 1 },
              scale: { x: 1, y: 1, z: 1 },
            },
            components: [],
            children: [],
            parentId: entity.id,
            visible: true,
            locked: false,
          };
          entity.children.push(child);
        }
      }
    }

    entities.push(entity);
  }

  return entities;
};

export const VirtualEntityTreeExample: React.FC = () => {
  const [entities] = useState<Entity[]>(() => generateMockEntities(100));
  const [selectedEntities, setSelectedEntities] = useState<string[]>([]);

  const handleEntitySelect = (entityIds: string[]) => {
    setSelectedEntities(entityIds);
    console.log('Selected entities:', entityIds);
  };

  const handleEntityRename = (entityId: string, newName: string) => {
    console.log('Rename entity:', entityId, 'to', newName);
    // Implement rename logic
  };

  const handleEntityDelete = (entityId: string) => {
    console.log('Delete entity:', entityId);
    // Implement delete logic
  };

  const handleEntityCreate = () => {
    console.log('Create new entity');
    // Implement create logic
  };

  const handleToggleVisibility = (entityId: string) => {
    console.log('Toggle visibility:', entityId);
    // Implement visibility toggle logic
  };

  const handleToggleLock = (entityId: string) => {
    console.log('Toggle lock:', entityId);
    // Implement lock toggle logic
  };

  const handleEntityReparent = async (entityId: string, newParentId: string | null) => {
    console.log('Reparent entity:', entityId, 'to', newParentId);
    // Implement reparent logic
  };

  return (
    <div style={{ height: '600px', width: '300px', border: '1px solid #ccc' }}>
      <VirtualEntityTree
        entities={entities}
        selectedEntities={selectedEntities}
        onEntitySelect={handleEntitySelect}
        onEntityRename={handleEntityRename}
        onEntityDelete={handleEntityDelete}
        onEntityCreate={handleEntityCreate}
        onEntityToggleVisibility={handleToggleVisibility}
        onEntityToggleLock={handleToggleLock}
        onEntityReparent={handleEntityReparent}
      />
    </div>
  );
};

/**
 * Performance Test Component
 *
 * Use this to test performance with different entity counts
 */
export const VirtualEntityTreePerformanceTest: React.FC<{ entityCount: number }> = ({
  entityCount,
}) => {
  const [entities] = useState<Entity[]>(() => generateMockEntities(entityCount));
  const [selectedEntities, setSelectedEntities] = useState<string[]>([]);

  return (
    <div>
      <h3>Performance Test: {entityCount} entities</h3>
      <div style={{ height: '500px', width: '300px', border: '1px solid #ccc' }}>
        <VirtualEntityTree
          entities={entities}
          selectedEntities={selectedEntities}
          onEntitySelect={setSelectedEntities}
          onEntityRename={(id, name) => console.log('Rename:', id, name)}
          onEntityDelete={(id) => console.log('Delete:', id)}
          onEntityCreate={() => console.log('Create')}
          onEntityToggleVisibility={(id) => console.log('Toggle visibility:', id)}
          onEntityToggleLock={(id) => console.log('Toggle lock:', id)}
          onEntityReparent={async (id, parent) => console.log('Reparent:', id, parent)}
        />
      </div>
      <p>Selected: {selectedEntities.length} entities</p>
    </div>
  );
};

export default VirtualEntityTreeExample;
