/**
 * Vector3Input Stories
 *
 * Stories for the Vector3Input component demonstrating various use cases.
 */

import type { Meta, StoryObj } from '@storybook/react';
import { Vector3Input } from './Vector3Input';

const meta: Meta<typeof Vector3Input> = {
  title: 'Molecules/Vector3Input',
  component: Vector3Input,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
  argTypes: {
    label: {
      control: 'text',
      description: 'Label for the input group',
    },
    step: {
      control: 'number',
      description: 'Step value for number inputs',
    },
    disabled: {
      control: 'boolean',
      description: 'Whether the inputs are disabled',
    },
    colorScheme: {
      control: 'select',
      options: ['rgb', 'slate'],
      description: 'Color scheme for axis labels',
    },
    precision: {
      control: 'number',
      description: 'Number of decimal places',
    },
  },
};

export default meta;
type Story = StoryObj<typeof Vector3Input>;

/**
 * Default Vector3Input for position editing
 */
export const Position: Story = {
  args: {
    label: 'Position',
    value: { x: 0, y: 0, z: 0 },
    onChange: (axis, value) => console.log(`${axis}: ${value}`),
    step: 0.1,
    colorScheme: 'rgb',
  },
};

/**
 * Vector3Input for rotation editing
 * Uses step of 1 for degree/radian values
 */
export const Rotation: Story = {
  args: {
    label: 'Rotation (Euler)',
    value: { x: 0, y: 90, z: 0 },
    onChange: (axis, value) => console.log(`${axis}: ${value}`),
    step: 1,
    colorScheme: 'rgb',
  },
};

/**
 * Vector3Input for scale editing
 * Uses smaller step and defaults to (1, 1, 1)
 */
export const Scale: Story = {
  args: {
    label: 'Scale',
    value: { x: 1, y: 1, z: 1 },
    onChange: (axis, value) => console.log(`${axis}: ${value}`),
    step: 0.01,
    colorScheme: 'rgb',
  },
};

/**
 * Disabled state
 */
export const Disabled: Story = {
  args: {
    label: 'Position (Locked)',
    value: { x: 10, y: 20, z: 30 },
    onChange: (axis, value) => console.log(`${axis}: ${value}`),
    step: 0.1,
    disabled: true,
  },
};

/**
 * Using slate color scheme
 */
export const SlateColorScheme: Story = {
  args: {
    label: 'Velocity',
    value: { x: 1.5, y: 2.3, z: 0.8 },
    onChange: (axis, value) => console.log(`${axis}: ${value}`),
    step: 0.1,
    colorScheme: 'slate',
  },
};

/**
 * With precision control
 */
export const WithPrecision: Story = {
  args: {
    label: 'Position (Precise)',
    value: { x: 1.23456789, y: 2.3456789, z: 3.456789 },
    onChange: (axis, value) => console.log(`${axis}: ${value}`),
    step: 0.001,
    precision: 3,
  },
};

/**
 * With min/max constraints
 */
export const WithConstraints: Story = {
  args: {
    label: 'Normalized Position',
    value: { x: 0.5, y: 0.5, z: 0.5 },
    onChange: (axis, value) => console.log(`${axis}: ${value}`),
    step: 0.01,
    min: 0,
    max: 1,
  },
};

/**
 * Interactive example with state
 */
export const Interactive = () => {
  const [position, setPosition] = React.useState({ x: 0, y: 0, z: 0 });

  return (
    <div className="p-4 bg-slate-900 rounded-lg space-y-4">
      <Vector3Input
        label="Position"
        value={position}
        onChange={(axis, value) => {
          setPosition({ ...position, [axis]: value });
        }}
        step={0.1}
      />

      <div className="text-slate-300 text-sm font-mono">
        Current Position: ({position.x.toFixed(2)}, {position.y.toFixed(2)}, {position.z.toFixed(2)})
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
