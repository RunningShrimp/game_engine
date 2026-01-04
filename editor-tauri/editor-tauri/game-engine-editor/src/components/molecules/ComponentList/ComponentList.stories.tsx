/**
 * ComponentList Stories
 *
 * Stories for the ComponentList component demonstrating various component list scenarios.
 */

import type { Meta, StoryObj } from '@storybook/react';
import { ComponentList } from './ComponentList';
import type { Component } from '../../../types/engine';

const meta: Meta<typeof ComponentList> = {
  title: 'Molecules/ComponentList',
  component: ComponentList,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
  argTypes: {
    allowPropertyEdit: {
      control: 'boolean',
      description: 'Whether property editing is enabled',
    },
    showAddButton: {
      control: 'boolean',
      description: 'Whether to show add component button',
    },
    showRemoveButtons: {
      control: 'boolean',
      description: 'Whether to show remove buttons',
    },
    emptyText: {
      control: 'text',
      description: 'Text to display when no components',
    },
  },
};

export default meta;
type Story = StoryObj<typeof ComponentList>;

const mockComponents: Component[] = [
  {
    id: 'comp-1',
    name: 'MeshRenderer',
    type: 'renderer',
    enabled: true,
    properties: {
      mesh: 'Cube',
      material: 'DefaultMaterial',
      castShadows: true,
    },
  },
  {
    id: 'comp-2',
    name: 'Rigidbody',
    type: 'physics',
    enabled: true,
    properties: {
      mass: 1.0,
      isKinematic: false,
      useGravity: true,
    },
  },
  {
    id: 'comp-3',
    name: 'Collider',
    type: 'physics',
    enabled: false,
    properties: {
      shape: 'Box',
      isTrigger: false,
    },
  },
];

/**
 * Default ComponentList with multiple components
 */
export const Default: Story = {
  args: {
    components: mockComponents,
    onToggle: (componentId, enabled) => console.log('Toggled:', componentId, enabled),
    onAddComponent: () => console.log('Add component'),
    onRemoveComponent: (componentId) => console.log('Remove:', componentId),
    showAddButton: true,
    showRemoveButtons: false,
  },
};

/**
 * Empty component list
 */
export const Empty: Story = {
  args: {
    components: [],
    onAddComponent: () => console.log('Add component'),
    emptyText: 'No components attached',
  },
};

/**
 * With editable properties
 */
export const EditableProperties: Story = {
  args: {
    components: mockComponents,
    onToggle: (componentId, enabled) => console.log('Toggled:', componentId, enabled),
    onPropertyChange: (componentId, key, value) =>
      console.log('Property changed:', componentId, key, value),
    allowPropertyEdit: true,
  },
};

/**
 * With remove buttons
 */
export const WithRemoveButtons: Story = {
  args: {
    components: mockComponents,
    onRemoveComponent: (componentId) => console.log('Remove:', componentId),
    showRemoveButtons: true,
  },
};

/**
 * Without add button
 */
export const NoAddButton: Story = {
  args: {
    components: mockComponents,
    showAddButton: false,
  },
};

/**
 * Single component
 */
export const SingleComponent: Story = {
  args: {
    components: [mockComponents[0]],
    onToggle: (componentId, enabled) => console.log('Toggled:', componentId, enabled),
  },
};

/**
 * Many components
 */
export const ManyComponents: Story = {
  args: {
    components: [
      ...mockComponents,
      {
        id: 'comp-4',
        name: 'Light',
        type: 'lighting',
        enabled: true,
        properties: { type: 'Point', intensity: 1.0 },
      },
      {
        id: 'comp-5',
        name: 'AudioSource',
        type: 'audio',
        enabled: true,
        properties: { volume: 1.0, loop: false },
      },
      {
        id: 'comp-6',
        name: 'ParticleSystem',
        type: 'effects',
        enabled: false,
        properties: { duration: 5.0, loop: true },
      },
    ],
    onToggle: (componentId, enabled) => console.log('Toggled:', componentId, enabled),
  },
};

/**
 * All components disabled
 */
export const AllDisabled: Story = {
  args: {
    components: mockComponents.map((comp) => ({ ...comp, enabled: false })),
    onToggle: (componentId, enabled) => console.log('Toggled:', componentId, enabled),
  },
};

/**
 * Interactive example with state
 */
export const Interactive = () => {
  const [components, setComponents] = React.useState<Component[]>(mockComponents);

  const handleToggle = (componentId: string, enabled: boolean) => {
    setComponents(
      components.map((comp) =>
        comp.id === componentId ? { ...comp, enabled } : comp
      )
    );
  };

  const handlePropertyChange = (componentId: string, key: string, value: any) => {
    setComponents(
      components.map((comp) =>
        comp.id === componentId
          ? { ...comp, properties: { ...comp.properties, [key]: value } }
          : comp
      )
    );
  };

  const handleAddComponent = () => {
    const newComponent: Component = {
      id: `comp-${Date.now()}`,
      name: 'NewComponent',
      type: 'utility',
      enabled: true,
      properties: {},
    };
    setComponents([...components, newComponent]);
  };

  const handleRemoveComponent = (componentId: string) => {
    setComponents(components.filter((comp) => comp.id !== componentId));
  };

  return (
    <div className="p-4 bg-slate-900 rounded-lg space-y-4">
      <ComponentList
        components={components}
        onToggle={handleToggle}
        onPropertyChange={handlePropertyChange}
        onAddComponent={handleAddComponent}
        onRemoveComponent={handleRemoveComponent}
        allowPropertyEdit={true}
        showRemoveButtons={true}
      />

      <div className="text-slate-300 text-sm">
        <strong>List State:</strong>
        <div className="mt-2 font-mono text-xs">
          <div>Total Components: {components.length}</div>
          <div>Enabled: {components.filter((c) => c.enabled).length}</div>
          <div>Disabled: {components.filter((c) => !c.enabled).length}</div>
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
