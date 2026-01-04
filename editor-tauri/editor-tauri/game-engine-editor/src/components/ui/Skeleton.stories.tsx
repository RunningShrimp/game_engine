import type { Meta, StoryObj } from '@storybook/react';
import { Skeleton, CardSkeleton, TableSkeleton } from './Skeleton';

/**
 * Skeleton Component Stories
 *
 * Comprehensive stories showcasing all Skeleton variants and configurations.
 */

const meta: Meta<typeof Skeleton> = {
  title: 'Atoms/Skeleton',
  component: Skeleton,
  parameters: {
    layout: 'padded',
    docs: {
      description: {
        component: `
A placeholder component that displays a loading animation to indicate that content is being loaded.
Supports text, rectangular, and circular variants with customizable dimensions.

**Features:**
- 3 variants: text, rectangular, circular
- Customizable width and height
- Multi-line text support
- Shimmer animation
- Pre-configured Card and Table skeletons
- Accessible with aria-hidden
        `,
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['text', 'rectangular', 'circular'],
      description: 'Shape variant of the skeleton',
    },
    width: {
      control: 'text',
      description: 'Width of the skeleton (e.g., "100px", "50%", "full")',
    },
    height: {
      control: 'text',
      description: 'Height of the skeleton (e.g., "20px", "100%", "auto")',
    },
    lines: {
      control: 'number',
      description: 'Number of skeleton lines (for text variant)',
    },
    animate: {
      control: 'boolean',
      description: 'Enable shimmer animation',
    },
  },
};

export default meta;
type Story = StoryObj<typeof Skeleton>;

/**
 * Default Skeleton story
 */
export const Default: Story = {
  args: {
    variant: 'rectangular',
    width: '200px',
    height: '40px',
  },
};

/**
 * All variants
 */
export const Variants: Story = {
  render: () => (
    <div className="space-y-8">
      <div>
        <h3 className="text-lg font-semibold mb-4">Text Variant</h3>
        <div className="space-y-2">
          <Skeleton variant="text" width="100%" height="20px" />
          <Skeleton variant="text" width="80%" height="20px" />
          <Skeleton variant="text" width="60%" height="20px" />
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-4">Rectangular Variant</h3>
        <div className="space-y-4">
          <Skeleton variant="rectangular" width="100%" height="100px" />
          <Skeleton variant="rectangular" width="100%" height="60px" />
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-4">Circular Variant</h3>
        <div className="flex items-center gap-4">
          <Skeleton variant="circular" width={40} height={40} />
          <Skeleton variant="circular" width={60} height={60} />
          <Skeleton variant="circular" width={80} height={80} />
        </div>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'All available skeleton variants.',
      },
    },
  },
};

/**
 * Multi-line text
 */
export const MultiLineText: Story = {
  render: () => (
    <div className="space-y-8">
      <div>
        <h4 className="text-sm font-semibold mb-4 text-gray-500">3 Lines</h4>
        <Skeleton variant="text" lines={3} />
      </div>

      <div>
        <h4 className="text-sm font-semibold mb-4 text-gray-500">5 Lines</h4>
        <Skeleton variant="text" lines={5} />
      </div>

      <div>
        <h4 className="text-sm font-semibold mb-4 text-gray-500">8 Lines</h4>
        <Skeleton variant="text" lines={8} />
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Multi-line text skeletons with varying line counts.',
      },
    },
  },
};

/**
 * Without animation
 */
export const WithoutAnimation: Story = {
  render: () => (
    <div className="space-y-4">
      <Skeleton variant="text" width="100%" height="20px" animate={false} />
      <Skeleton variant="rectangular" width="100%" height="100px" animate={false} />
      <Skeleton variant="circular" width={60} height={60} animate={false} />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Skeletons without shimmer animation.',
      },
    },
  },
};

/**
 * Card skeleton preset
 */
export const CardSkeletons: Story = {
  render: () => (
    <div className="space-y-4">
      <CardSkeleton />
      <CardSkeleton />
      <CardSkeleton />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Pre-configured card skeleton for loading card content.',
      },
    },
  },
};

/**
 * Table skeleton preset
 */
export const TableSkeletons: Story = {
  render: () => (
    <div className="p-4 border rounded">
      <TableSkeleton rows={5} columns={4} />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Pre-configured table skeleton for loading table rows.',
      },
    },
  },
};

/**
 * Interactive skeleton
 */
export const Interactive: Story = {
  args: {
    variant: 'rectangular',
    width: '200px',
    height: '40px',
    animate: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Use the controls panel to interactively change skeleton props.',
      },
    },
  },
};

/**
 * Loading patterns
 */
export const LoadingPatterns: Story = {
  render: () => (
    <div className="space-y-8">
      <div>
        <h3 className="text-lg font-semibold mb-4">Card List Loading</h3>
        <div className="grid grid-cols-2 gap-4">
          <CardSkeleton />
          <CardSkeleton />
          <CardSkeleton />
          <CardSkeleton />
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-4">User Profile Loading</h3>
        <div className="p-4 border rounded max-w-md">
          <div className="flex items-center space-x-4">
            <Skeleton variant="circular" width={80} height={80} />
            <div className="flex-1 space-y-3">
              <Skeleton variant="text" width="60%" height={24} />
              <Skeleton variant="text" width="40%" height={16} />
              <Skeleton variant="text" width="80%" height={16} />
            </div>
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-4">Article Loading</h3>
        <div className="p-4 border rounded max-w-2xl">
          <Skeleton variant="rectangular" width="100%" height={200} className="mb-4" />
          <Skeleton variant="text" lines={2} />
          <div className="mt-4 space-y-2">
            <Skeleton variant="text" lines={1} />
            <Skeleton variant="text" lines={1} />
            <Skeleton variant="text" lines={1} />
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-4">Dashboard Loading</h3>
        <div className="grid grid-cols-3 gap-4">
          <div className="p-4 border rounded">
            <Skeleton variant="rectangular" width="100%" height={100} />
            <Skeleton variant="text" width="60%" height={20} className="mt-4" />
          </div>
          <div className="p-4 border rounded">
            <Skeleton variant="rectangular" width="100%" height={100} />
            <Skeleton variant="text" width="60%" height={20} className="mt-4" />
          </div>
          <div className="p-4 border rounded">
            <Skeleton variant="rectangular" width="100%" height={100} />
            <Skeleton variant="text" width="60%" height={20} className="mt-4" />
          </div>
        </div>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Real-world loading patterns for different UI scenarios.',
      },
    },
  },
};

/**
 * Custom dimensions
 */
export const CustomDimensions: Story = {
  render: () => (
    <div className="space-y-4">
      <div>
        <h4 className="text-sm font-semibold mb-2 text-gray-500">Small Text</h4>
        <Skeleton variant="text" width="150px" height="14px" />
      </div>

      <div>
        <h4 className="text-sm font-semibold mb-2 text-gray-500">Medium Text</h4>
        <Skeleton variant="text" width="200px" height="18px" />
      </div>

      <div>
        <h4 className="text-sm font-semibold mb-2 text-gray-500">Large Text</h4>
        <Skeleton variant="text" width="250px" height="24px" />
      </div>

      <div>
        <h4 className="text-sm font-semibold mb-2 text-gray-500">Custom Rectangle</h4>
        <Skeleton variant="rectangular" width="300px" height="150px" />
      </div>

      <div>
        <h4 className="text-sm font-semibold mb-2 text-gray-500">Full Width</h4>
        <Skeleton variant="rectangular" width="100%" height="80px" />
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Skeletons with custom width and height dimensions.',
      },
    },
  },
};
