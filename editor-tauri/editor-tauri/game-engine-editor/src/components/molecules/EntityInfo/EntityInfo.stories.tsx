/**
 * EntityInfo Stories
 *
 * Stories for the EntityInfo component demonstrating various entity information display scenarios.
 */

import type { Meta, StoryObj } from '@storybook/react';
import { EntityInfo } from './EntityInfo';
import type { Entity } from '../../../types/engine';

const meta: Meta<typeof EntityInfo> = {
  title: 'Molecules/EntityInfo',
  component: EntityInfo,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
  argTypes: {
    allowNameEdit: {
      control: 'boolean',
      description: 'Whether name editing is enabled',
    },
    showId: {
      control: 'boolean',
      description: 'Whether to show entity ID',
    },
    showVisibility: {
      control: 'boolean',
      description: 'Whether to show visibility toggle',
    },
    showLocked: {
      control: 'boolean',
      description: 'Whether to show locked toggle',
    },
  },
};

export default meta;
type Story = StoryObj<typeof EntityInfo>;

const mockEntity: Entity = {
  id: 'entity-123',
  name: 'Cube',
  visible: true,
  locked: false,
  transform: {
    position: { x: 0, y: 0, z: 0 },
    rotation: { x: 0, y: 0, z: 0 },
    scale: { x: 1, y: 1, z: 1 },
  },
  components: [],
};

/**
 * Default EntityInfo with all fields visible
 */
export const Default: Story = {
  args: {
    entity: mockEntity,
    onNameChange: (newName) => console.log('Name changed to:', newName),
    allowNameEdit: true,
    showId: true,
    showVisibility: true,
    showLocked: true,
  },
};

/**
 * Entity with name editing disabled
 */
export const NoNameEdit: Story = {
  args: {
    entity: mockEntity,
    allowNameEdit: false,
  },
};

/**
 * Minimal view - only name and ID
 */
export const Minimal: Story = {
  args: {
    entity: mockEntity,
    showVisibility: false,
    showLocked: false,
  },
};

/**
 * Hidden and locked entity
 */
export const HiddenAndLocked: Story = {
  args: {
    entity: {
      ...mockEntity,
      visible: false,
      locked: true,
      name: 'Hidden Cube',
    },
  },
};

/**
 * Entity with long name
 */
export const LongName: Story = {
  args: {
    entity: {
      ...mockEntity,
      name: 'This is a very long entity name that might wrap or truncate depending on styling',
    },
  },
};

/**
 * Entity with special characters in name
 */
export const SpecialCharacters: Story = {
  args: {
    entity: {
      ...mockEntity,
      name: 'Entity_@#$%^&*()',
    },
  },
};

/**
 * Interactive example with state
 */
export const Interactive = () => {
  const [entity, setEntity] = React.useState<Entity>(mockEntity);

  const handleNameChange = (newName: string) => {
    setEntity({ ...entity, name: newName });
  };

  return (
    <div className="p-4 bg-slate-900 rounded-lg space-y-4">
      <EntityInfo
        entity={entity}
        onNameChange={handleNameChange}
      />

      <div className="text-slate-300 text-sm">
        <strong>Current State:</strong>
        <div className="mt-2 font-mono text-xs space-y-1">
          <div>Name: {entity.name}</div>
          <div>ID: {entity.id}</div>
          <div>Visible: {entity.visible ? 'Yes' : 'No'}</div>
          <div>Locked: {entity.locked ? 'Yes' : 'No'}</div>
        </div>
      </div>
    </div>
  );
};

Interactive.decorators = [
  (Story) => (
    <div className="w-full max-w-md">
      <Story />
    </div>
  ),
];
