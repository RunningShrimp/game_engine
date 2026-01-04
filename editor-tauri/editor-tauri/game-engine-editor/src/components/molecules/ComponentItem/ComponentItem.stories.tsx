/**
 * ComponentItem Stories
 *
 * Stories for the ComponentItem component demonstrating various component display scenarios.
 */

import type { Meta, StoryObj } from '@storybook/react';
import { ComponentItem } from './ComponentItem';
import type { Component } from '../../../types/engine';

const meta: Meta<typeof ComponentItem> = {
  title: 'Molecules/ComponentItem',
  component: ComponentItem,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
  argTypes: {
    allowPropertyEdit: {
      control: 'boolean',
      description: 'Whether property editing is enabled',
    },
    defaultExpanded: {
      control: 'boolean',
      description: 'Initial expanded state',
    },
    showTypeBadge: {
      control: 'boolean',
      description: 'Whether to show component type badge',
    },
    showEnabledToggle: {
      control: 'boolean',
      description: 'Whether to show the enabled checkbox',
    },
  },
};

export default meta;
type Story = StoryObj<typeof ComponentItem>;

const mockComponent: Component = {
  id: 'comp-123',
  name: 'MeshRenderer',
  type: 'renderer',
  enabled: true,
  properties: {
    mesh: 'Cube',
    material: 'DefaultMaterial',
    castShadows: true,
    receiveShadows: false,
    renderQueue: 2000,
  },
};

/**
 * Default ComponentItem with properties
 */
export const Default: Story = {
  args: {
    component: mockComponent,
    onToggle: (enabled) => console.log('Component toggled:', enabled),
    onPropertyChange: (key, value) => console.log('Property changed:', key, value),
    allowPropertyEdit: false,
    defaultExpanded: true,
  },
};

/**
 * Collapsed state
 */
export const Collapsed: Story = {
  args: {
    component: mockComponent,
    onToggle: (enabled) => console.log('Component toggled:', enabled),
    defaultExpanded: false,
  },
};

/**
 * Disabled component
 */
export const Disabled: Story = {
  args: {
    component: {
      ...mockComponent,
      enabled: false,
    },
    onToggle: (enabled) => console.log('Component toggled:', enabled),
  },
};

/**
 * With editable properties
 */
export const EditableProperties: Story = {
  args: {
    component: mockComponent,
    onToggle: (enabled) => console.log('Component toggled:', enabled),
    onPropertyChange: (key, value) => console.log('Property changed:', key, value),
    allowPropertyEdit: true,
  },
};

/**
 * Component with no properties
 */
export const NoProperties: Story = {
  args: {
    component: {
      id: 'comp-456',
      name: 'EmptyComponent',
      type: 'utility',
      enabled: true,
      properties: {},
    },
    onToggle: (enabled) => console.log('Component toggled:', enabled),
  },
};

/**
 * Without type badge
 */
export const NoTypeBadge: Story = {
  args: {
    component: mockComponent,
    onToggle: (enabled) => console.log('Component toggled:', enabled),
    showTypeBadge: false,
  },
};

/**
 * Without enabled toggle
 */
export const NoEnabledToggle: Story = {
  args: {
    component: mockComponent,
    showEnabledToggle: false,
  },
};

/**
 * Component with various property types
 */
export const MixedPropertyTypes: Story = {
  args: {
    component: {
      id: 'comp-789',
      name: 'Rigidbody',
      type: 'physics',
      enabled: true,
      properties: {
        mass: 1.5,
        isKinematic: false,
        collisionLayer: 'Default',
        drag: 0.5,
        angularDrag: 0.05,
        useGravity: true,
      },
    },
    onToggle: (enabled) => console.log('Component toggled:', enabled),
    onPropertyChange: (key, value) => console.log('Property changed:', key, value),
    allowPropertyEdit: true,
  },
};

/**
 * Component with nested object property
 */
export const NestedProperties: Story = {
  args: {
    component: {
      id: 'comp-999',
      name: 'Light',
      type: 'lighting',
      enabled: true,
      properties: {
        type: 'Point',
        intensity: 1.0,
        color: { r: 1, g: 0.5, b: 0.2 },
        range: 10,
        castShadows: true,
      },
    },
    onToggle: (enabled) => console.log('Component toggled:', enabled),
  },
};

/**
 * Interactive example with state
 */
export const Interactive = () => {
  const [component, setComponent] = React.useState<Component>(mockComponent);

  const handleToggle = (enabled: boolean) => {
    setComponent({ ...component, enabled });
  };

  const handlePropertyChange = (key: string, value: any) => {
    setComponent({
      ...component,
      properties: { ...component.properties, [key]: value },
    });
  };

  return (
    <div className="p-4 bg-slate-900 rounded-lg space-y-4">
      <ComponentItem
        component={component}
        onToggle={handleToggle}
        onPropertyChange={handlePropertyChange}
        allowPropertyEdit={true}
      />

      <div className="text-slate-300 text-sm">
        <strong>Component State:</strong>
        <div className="mt-2 font-mono text-xs">
          <div>Enabled: {component.enabled ? 'Yes' : 'No'}</div>
          <div>Properties: {Object.keys(component.properties).length} properties</div>
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
