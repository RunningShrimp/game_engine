import type { Meta, StoryObj } from '@storybook/react';
import { Spinner } from './Spinner';

/**
 * Spinner Component Stories
 *
 * Comprehensive stories showcasing all Spinner variants, sizes, and speeds.
 */

const meta: Meta<typeof Spinner> = {
  title: 'Atoms/Spinner',
  component: Spinner,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: `
A loading spinner component with configurable size, color, and animation speed.
Used to indicate loading states throughout the application.

**Features:**
- 5 sizes: xs, sm, md, lg, xl
- 4 colors: primary, secondary, white, currentColor
- 3 animation speeds: slow, normal, fast
- Full accessibility support with aria-label
        `,
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    size: {
      control: 'select',
      options: ['xs', 'sm', 'md', 'lg', 'xl'],
      description: 'Spinner size',
    },
    color: {
      control: 'select',
      options: ['primary', 'secondary', 'white', 'currentColor'],
      description: 'Spinner color',
    },
    speed: {
      control: 'select',
      options: ['slow', 'normal', 'fast'],
      description: 'Animation speed',
    },
  },
};

export default meta;
type Story = StoryObj<typeof Spinner>;

/**
 * Default Spinner story - medium size, primary color, normal speed
 */
export const Default: Story = {
  args: {
    size: 'md',
    color: 'primary',
    speed: 'normal',
  },
};

/**
 * All spinner sizes
 */
export const Sizes: Story = {
  render: () => (
    <div className="flex flex-wrap items-center gap-8">
      <div className="flex flex-col items-center gap-2">
        <Spinner size="xs" />
        <span className="text-sm text-gray-600">XS</span>
      </div>
      <div className="flex flex-col items-center gap-2">
        <Spinner size="sm" />
        <span className="text-sm text-gray-600">SM</span>
      </div>
      <div className="flex flex-col items-center gap-2">
        <Spinner size="md" />
        <span className="text-sm text-gray-600">MD</span>
      </div>
      <div className="flex flex-col items-center gap-2">
        <Spinner size="lg" />
        <span className="text-sm text-gray-600">LG</span>
      </div>
      <div className="flex flex-col items-center gap-2">
        <Spinner size="xl" />
        <span className="text-sm text-gray-600">XL</span>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'All available spinner sizes with primary color.',
      },
    },
  },
};

/**
 * All spinner colors
 */
export const Colors: Story = {
  render: () => (
    <div className="flex flex-wrap items-center gap-8">
      <div className="flex flex-col items-center gap-2">
        <Spinner size="lg" color="primary" />
        <span className="text-sm text-gray-600">Primary</span>
      </div>
      <div className="flex flex-col items-center gap-2">
        <Spinner size="lg" color="secondary" />
        <span className="text-sm text-gray-600">Secondary</span>
      </div>
      <div className="flex flex-col items-center gap-2 bg-gray-800 p-4 rounded">
        <Spinner size="lg" color="white" />
        <span className="text-sm text-white">White</span>
      </div>
      <div className="flex flex-col items-center gap-2">
        <Spinner size="lg" color="currentColor" className="text-blue-600" />
        <span className="text-sm text-gray-600">Current Color</span>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'All available spinner colors with large size.',
      },
    },
  },
};

/**
 * All animation speeds
 */
export const Speeds: Story = {
  render: () => (
    <div className="flex flex-wrap items-center gap-8">
      <div className="flex flex-col items-center gap-2">
        <Spinner size="lg" speed="slow" />
        <span className="text-sm text-gray-600">Slow</span>
      </div>
      <div className="flex flex-col items-center gap-2">
        <Spinner size="lg" speed="normal" />
        <span className="text-sm text-gray-600">Normal</span>
      </div>
      <div className="flex flex-col items-center gap-2">
        <Spinner size="lg" speed="fast" />
        <span className="text-sm text-gray-600">Fast</span>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'All available animation speeds.',
      },
    },
  },
};

/**
 * Interactive spinner with controls
 */
export const Interactive: Story = {
  args: {
    size: 'md',
    color: 'primary',
    speed: 'normal',
  },
  parameters: {
    docs: {
      description: {
        story: 'Use the controls panel to interactively change spinner props.',
      },
    },
  },
};

/**
 * Spinner combinations
 */
export const Combinations: Story = {
  render: () => (
    <div className="space-y-8">
      <div>
        <h3 className="text-lg font-semibold mb-4">Primary Color Variations</h3>
        <div className="flex flex-wrap items-center gap-8">
          <Spinner size="xs" color="primary" />
          <Spinner size="sm" color="primary" />
          <Spinner size="md" color="primary" />
          <Spinner size="lg" color="primary" />
          <Spinner size="xl" color="primary" />
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-4">Secondary Color Variations</h3>
        <div className="flex flex-wrap items-center gap-8">
          <Spinner size="xs" color="secondary" />
          <Spinner size="sm" color="secondary" />
          <Spinner size="md" color="secondary" />
          <Spinner size="lg" color="secondary" />
          <Spinner size="xl" color="secondary" />
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-4">Speed Variations</h3>
        <div className="flex flex-wrap items-center gap-8">
          <div className="flex flex-col items-center gap-2">
            <Spinner size="lg" color="primary" speed="slow" />
            <span className="text-sm text-gray-600">Slow</span>
          </div>
          <div className="flex flex-col items-center gap-2">
            <Spinner size="lg" color="primary" speed="normal" />
            <span className="text-sm text-gray-600">Normal</span>
          </div>
          <div className="flex flex-col items-center gap-2">
            <Spinner size="lg" color="primary" speed="fast" />
            <span className="text-sm text-gray-600">Fast</span>
          </div>
        </div>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Complete showcase of spinner combinations across sizes, colors, and speeds.',
      },
    },
  },
};

/**
 * In context examples
 */
export const InContext: Story = {
  render: () => (
    <div className="space-y-4">
      <div className="flex items-center gap-4 p-4 border rounded">
        <Spinner size="sm" color="primary" />
        <span className="text-sm">Loading data...</span>
      </div>

      <div className="flex items-center gap-4 p-4 border rounded">
        <Spinner size="md" color="primary" />
        <span className="text-base">Processing your request...</span>
      </div>

      <div className="flex items-center justify-center gap-4 p-8 bg-gray-800 rounded">
        <Spinner size="lg" color="white" />
        <span className="text-white">Loading application...</span>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Spinners in realistic context with accompanying text.',
      },
    },
  },
};
