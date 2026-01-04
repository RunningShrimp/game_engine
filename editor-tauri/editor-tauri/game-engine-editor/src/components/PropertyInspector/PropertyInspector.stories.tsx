/**
 * PropertyInspector Stories
 *
 * Stories for the PropertyInspector component demonstrating various property inspection scenarios.
 */

import type { Meta, StoryObj } from '@storybook/react';
import { PropertyInspector } from './PropertyInspector.refactored';
import type { Entity } from '../../types/engine';

const meta: Meta<typeof PropertyInspector> = {
  title: 'Organisms/PropertyInspector',
  component: PropertyInspector,
  parameters: {
    layout: 'fullscreen',
  },
  tags: ['autodocs'],
  argTypes: {
    allowPropertyEdit: {
      control: 'boolean',
      description: 'Whether to enable property editing',
    },
    coordinateSpace: {
      control: 'select',
      options: ['world', 'local'],
      description: 'Coordinate space for transform editing',
    },
  },
};

export default meta;
type Story = StoryObj<typeof PropertyInspector>;

// Mock entities
const mockEntities: Entity[] = [
  {
    id: 'entity-1',
    name: 'Cube',
    visible: true,
    locked: false,
    transform: {
      position: { x: 0, y: 0, z: 0 },
      rotation: { x: 0, y: 0, z: 0 },
      scale: { x: 1, y: 1, z: 1 },
    },
    components: [
      {
        id: 'comp-1',
        name: 'MeshRenderer',
        type: 'renderer',
        enabled: true,
        properties: {
          mesh: 'Cube',
          material: 'DefaultMaterial',
          castShadows: true,
          receiveShadows: false,
        },
      },
      {
        id: 'comp-2',
        name: 'Rigidbody',
        type: 'physics',
        enabled: true,
        properties: {
          mass: 1.0,
          useGravity: true,
          isKinematic: false,
        },
      },
    ],
  },
  {
    id: 'entity-2',
    name: 'Sphere',
    visible: true,
    locked: false,
    transform: {
      position: { x: 5, y: 2, z: 0 },
      rotation: { x: 45, y: 90, z: 0 },
      scale: { x: 2, y: 2, z: 2 },
    },
    components: [
      {
        id: 'comp-3',
        name: 'MeshRenderer',
        type: 'renderer',
        enabled: true,
        properties: {
          mesh: 'Sphere',
          material: 'GlowMaterial',
          castShadows: true,
        },
      },
    ],
  },
  {
    id: 'entity-3',
    name: 'Hidden Cube',
    visible: false,
    locked: true,
    transform: {
      position: { x: -3, y: 0, z: 2 },
      rotation: { x: 0, y: 0, z: 0 },
      scale: { x: 1, y: 1, z: 1 },
    },
    components: [],
  },
];

/**
 * Default PropertyInspector with an entity selected
 */
export const Default: Story = {
  args: {
    entities: mockEntities,
    selectedEntities: ['entity-1'],
    onTransformChange: (entityId, transform) =>
      console.log('Transform changed:', entityId, transform),
    onComponentToggle: (entityId, componentId, enabled) =>
      console.log('Component toggled:', entityId, componentId, enabled),
    onEntityRename: (entityId, name) =>
      console.log('Entity renamed:', entityId, name),
    coordinateSpace: 'world',
  },
};

/**
 * Empty state - no entity selected
 */
export const NoSelection: Story = {
  args: {
    entities: mockEntities,
    selectedEntities: [],
    onTransformChange: () => {},
  },
};

/**
 * Multiple selection (shows first entity)
 */
export const MultipleSelection: Story = {
  args: {
    entities: mockEntities,
    selectedEntities: ['entity-1', 'entity-2'],
    onTransformChange: (entityId, transform) =>
      console.log('Transform changed:', entityId, transform),
  },
};

/**
 * With editable properties
 */
export const EditableProperties: Story = {
  args: {
    entities: mockEntities,
    selectedEntities: ['entity-1'],
    onTransformChange: (entityId, transform) =>
      console.log('Transform changed:', entityId, transform),
    onComponentPropertyChange: (entityId, componentId, key, value) =>
      console.log('Property changed:', entityId, componentId, key, value),
    allowPropertyEdit: true,
  },
};

/**
 * Entity with no components
 */
export const NoComponents: Story = {
  args: {
    entities: [mockEntities[2]], // entity-3 has no components
    selectedEntities: ['entity-3'],
    onTransformChange: (entityId, transform) =>
      console.log('Transform changed:', entityId, transform),
  },
};

/**
 * Local coordinate space
 */
export const LocalSpace: Story = {
  args: {
    entities: mockEntities,
    selectedEntities: ['entity-1'],
    onTransformChange: (entityId, transform) =>
      console.log('Transform changed:', entityId, transform),
    coordinateSpace: 'local',
  },
};

/**
 * Entity with custom transform values
 */
export const CustomTransform: Story = {
  args: {
    entities: [
      {
        ...mockEntities[1], // Sphere entity
        transform: {
          position: { x: 10.5, y: -3.2, z: 7.8 },
          rotation: { x: 45, y: 90, z: 180 },
          scale: { x: 1.5, y: 2.0, z: 0.5 },
        },
      },
    ],
    selectedEntities: ['entity-2'],
    onTransformChange: (entityId, transform) =>
      console.log('Transform changed:', entityId, transform),
  },
};

/**
 * Hidden and locked entity
 */
export const HiddenAndLocked: Story = {
  args: {
    entities: [mockEntities[2]], // entity-3 is hidden and locked
    selectedEntities: ['entity-3'],
    onTransformChange: (entityId, transform) =>
      console.log('Transform changed:', entityId, transform),
  },
};

/**
 * Interactive example with state
 */
export const Interactive = () => {
  const [entities, setEntities] = React.useState<Entity[]>(mockEntities);
  const [selectedEntityId, setSelectedEntityId] = React.useState<string>('entity-1');

  const handleTransformChange = (entityId: string, transform: any) => {
    setEntities(
      entities.map((e) => (e.id === entityId ? { ...e, transform } : e))
    );
  };

  const handleComponentToggle = (
    entityId: string,
    componentId: string,
    enabled: boolean
  ) => {
    setEntities(
      entities.map((e) =>
        e.id === entityId
          ? {
              ...e,
              components: e.components.map((c) =>
                c.id === componentId ? { ...c, enabled } : c
              ),
            }
          : e
      )
    );
  };

  const handleEntityRename = (entityId: string, name: string) => {
    setEntities(
      entities.map((e) => (e.id === entityId ? { ...e, name } : e))
    );
  };

  const selectedEntity = entities.find((e) => e.id === selectedEntityId);

  return (
    <div className="flex h-screen bg-slate-950">
      {/* Sidebar for entity selection */}
      <div className="w-64 bg-slate-900 border-r border-slate-700 p-4">
        <h2 className="text-lg font-semibold text-slate-200 mb-4">Scene Entities</h2>
        <div className="space-y-2">
          {entities.map((entity) => (
            <button
              key={entity.id}
              onClick={() => setSelectedEntityId(entity.id)}
              className={`
                w-full text-left px-3 py-2 rounded transition-colors
                ${
                  selectedEntityId === entity.id
                    ? 'bg-blue-600 text-white'
                    : 'bg-slate-800 text-slate-300 hover:bg-slate-700'
                }
              `}
            >
              <div className="font-medium">{entity.name}</div>
              <div className="text-xs opacity-75">{entity.components.length} components</div>
            </button>
          ))}
        </div>

        {selectedEntity && (
          <div className="mt-6 p-3 bg-slate-800 rounded">
            <h3 className="text-sm font-semibold text-slate-300 mb-2">Selected Entity</h3>
            <div className="text-xs text-slate-400 space-y-1">
              <div>Name: {selectedEntity.name}</div>
              <div>Visible: {selectedEntity.visible ? 'Yes' : 'No'}</div>
              <div>Locked: {selectedEntity.locked ? 'Yes' : 'No'}</div>
            </div>
          </div>
        )}
      </div>

      {/* Property Inspector */}
      <div className="flex-1">
        <PropertyInspector
          entities={entities}
          selectedEntities={[selectedEntityId]}
          onTransformChange={handleTransformChange}
          onComponentToggle={handleComponentToggle}
          onEntityRename={handleEntityRename}
          allowPropertyEdit={true}
        />
      </div>
    </div>
  );
};
