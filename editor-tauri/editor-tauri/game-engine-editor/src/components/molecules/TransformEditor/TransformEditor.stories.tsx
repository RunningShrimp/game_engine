/**
 * TransformEditor Stories
 *
 * Stories for the TransformEditor component demonstrating various transform editing scenarios.
 */

import type { Meta, StoryObj } from '@storybook/react';
import { TransformEditor } from './TransformEditor';
import type { Transform } from '../../../types/engine';

const meta: Meta<typeof TransformEditor> = {
  title: 'Molecules/TransformEditor',
  component: TransformEditor,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
  argTypes: {
    coordinateSpace: {
      control: 'select',
      options: ['world', 'local'],
      description: 'Coordinate space for editing',
    },
    disabled: {
      control: 'boolean',
      description: 'Whether the editor is disabled',
    },
    showCoordinateSpace: {
      control: 'boolean',
      description: 'Whether to show coordinate space toggle',
    },
  },
};

export default meta;
type Story = StoryObj<typeof TransformEditor>;

const defaultTransform: Transform = {
  position: { x: 0, y: 0, z: 0 },
  rotation: { x: 0, y: 0, z: 0 },
  scale: { x: 1, y: 1, z: 1 },
};

/**
 * Default TransformEditor in world space
 */
export const WorldSpace: Story = {
  args: {
    transform: defaultTransform,
    onChange: (transform) => console.log('Transform changed:', transform),
    coordinateSpace: 'world',
    showCoordinateSpace: true,
  },
};

/**
 * TransformEditor in local space
 */
export const LocalSpace: Story = {
  args: {
    transform: defaultTransform,
    onChange: (transform) => console.log('Transform changed:', transform),
    coordinateSpace: 'local',
    showCoordinateSpace: true,
  },
};

/**
 * With non-default transform values
 */
export const WithTransformValues: Story = {
  args: {
    transform: {
      position: { x: 10, y: 20, z: 30 },
      rotation: { x: 45, y: 90, z: 0 },
      scale: { x: 2, y: 2, z: 2 },
    },
    onChange: (transform) => console.log('Transform changed:', transform),
    coordinateSpace: 'world',
  },
};

/**
 * Disabled state
 */
export const Disabled: Story = {
  args: {
    transform: {
      position: { x: 10, y: 20, z: 30 },
      rotation: { x: 0, y: 90, z: 0 },
      scale: { x: 1.5, y: 1.5, z: 1.5 },
    },
    onChange: (transform) => console.log('Transform changed:', transform),
    disabled: true,
  },
};

/**
 * With custom step values
 */
export const CustomStepValues: Story = {
  args: {
    transform: defaultTransform,
    onChange: (transform) => console.log('Transform changed:', transform),
    coordinateSpace: 'world',
    stepValues: {
      position: 0.01,
      rotation: 0.1,
      scale: 0.001,
    },
  },
};

/**
 * Interactive example with state
 */
export const Interactive = () => {
  const [transform, setTransform] = React.useState<Transform>({
    position: { x: 0, y: 0, z: 0 },
    rotation: { x: 0, y: 0, z: 0 },
    scale: { x: 1, y: 1, z: 1 },
  });

  return (
    <div className="p-4 bg-slate-900 rounded-lg space-y-4">
      <TransformEditor
        transform={transform}
        onChange={setTransform}
        coordinateSpace="world"
        showCoordinateSpace={true}
      />

      <div className="text-slate-300 text-sm font-mono space-y-1">
        <div>Position: ({transform.position.x.toFixed(2)}, {transform.position.y.toFixed(2)}, {transform.position.z.toFixed(2)})</div>
        <div>Rotation: ({transform.rotation.x.toFixed(1)}°, {transform.rotation.y.toFixed(1)}°, {transform.rotation.z.toFixed(1)}°)</div>
        <div>Scale: ({transform.scale.x.toFixed(2)}, {transform.scale.y.toFixed(2)}, {transform.scale.z.toFixed(2)})</div>
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
