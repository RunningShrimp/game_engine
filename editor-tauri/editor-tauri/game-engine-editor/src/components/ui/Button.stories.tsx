import type { Meta, StoryObj } from '@storybook/react';
import { Button } from './Button';

/**
 * Button Component Stories
 *
 * Comprehensive stories showcasing all Button variants, sizes, and states.
 */

const meta: Meta<typeof Button> = {
  title: 'Atoms/Button',
  component: Button,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: `
A versatile button component with support for multiple variants, sizes, and states.
Includes loading state with spinner and follows accessibility best practices.

**Features:**
- 5 variants: primary, secondary, outline, ghost, danger
- 5 sizes: xs, sm, md, lg, xl
- Loading state with spinner
- Full width option
- Full keyboard accessibility
        `,
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['primary', 'secondary', 'outline', 'ghost', 'danger'],
      description: 'Button visual variant',
    },
    size: {
      control: 'select',
      options: ['xs', 'sm', 'md', 'lg', 'xl'],
      description: 'Button size',
    },
    isLoading: {
      control: 'boolean',
      description: 'Show loading state with spinner',
    },
    disabled: {
      control: 'boolean',
      description: 'Disable the button',
    },
    fullWidth: {
      control: 'boolean',
      description: 'Button should take full width of parent',
    },
    children: {
      control: 'text',
      description: 'Button content',
    },
  },
};

export default meta;
type Story = StoryObj<typeof Button>;

/**
 * Default Button story - primary variant, medium size
 */
export const Default: Story = {
  args: {
    children: 'Click me',
    variant: 'primary',
    size: 'md',
  },
};

/**
 * All button variants
 */
export const Variants: Story = {
  render: () => (
    <div className="flex flex-wrap gap-4">
      <Button variant="primary">Primary</Button>
      <Button variant="secondary">Secondary</Button>
      <Button variant="outline">Outline</Button>
      <Button variant="ghost">Ghost</Button>
      <Button variant="danger">Danger</Button>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'All available button variants with default medium size.',
      },
    },
  },
};

/**
 * All button sizes
 */
export const Sizes: Story = {
  render: () => (
    <div className="flex flex-wrap items-center gap-4">
      <Button size="xs">Extra Small</Button>
      <Button size="sm">Small</Button>
      <Button size="md">Medium</Button>
      <Button size="lg">Large</Button>
      <Button size="xl">Extra Large</Button>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'All available button sizes with primary variant.',
      },
    },
  },
};

/**
 * Loading state
 */
export const Loading: Story = {
  render: () => (
    <div className="flex flex-wrap gap-4">
      <Button isLoading>Loading Primary</Button>
      <Button variant="secondary" isLoading>
        Loading Secondary
      </Button>
      <Button variant="outline" isLoading>
        Loading Outline
      </Button>
      <Button variant="danger" isLoading>
        Loading Danger
      </Button>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Buttons in loading state with spinner animation.',
      },
    },
  },
};

/**
 * Disabled state
 */
export const Disabled: Story = {
  render: () => (
    <div className="flex flex-wrap gap-4">
      <Button disabled>Disabled Primary</Button>
      <Button variant="secondary" disabled>
        Disabled Secondary
      </Button>
      <Button variant="outline" disabled>
        Disabled Outline
      </Button>
      <Button variant="ghost" disabled>
        Disabled Ghost
      </Button>
      <Button variant="danger" disabled>
        Disabled Danger
      </Button>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Buttons in disabled state with reduced opacity.',
      },
    },
  },
};

/**
 * Full width buttons
 */
export const FullWidth: Story = {
  render: () => (
    <div className="w-80 space-y-4">
      <Button fullWidth>Full Width Primary</Button>
      <Button variant="secondary" fullWidth>
        Full Width Secondary
      </Button>
      <Button variant="outline" fullWidth>
        Full Width Outline
      </Button>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Buttons that take the full width of their container.',
      },
    },
  },
};

/**
 * Interactive button with controls
 */
export const Interactive: Story = {
  args: {
    children: 'Interactive Button',
    variant: 'primary',
    size: 'md',
    isLoading: false,
    disabled: false,
    fullWidth: false,
  },
  parameters: {
    docs: {
      description: {
        story: 'Use the controls panel to interactively change button props.',
      },
    },
  },
};

/**
 * Button combinations
 */
export const Combinations: Story = {
  render: () => (
    <div className="space-y-8">
      <div>
        <h3 className="text-lg font-semibold mb-4">Primary Variants</h3>
        <div className="flex flex-wrap gap-4">
          <Button variant="primary" size="xs">
            XS Primary
          </Button>
          <Button variant="primary" size="sm">
            SM Primary
          </Button>
          <Button variant="primary" size="md">
            MD Primary
          </Button>
          <Button variant="primary" size="lg">
            LG Primary
          </Button>
          <Button variant="primary" size="xl">
            XL Primary
          </Button>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-4">Secondary Variants</h3>
        <div className="flex flex-wrap gap-4">
          <Button variant="secondary" size="xs">
            XS Secondary
          </Button>
          <Button variant="secondary" size="sm">
            SM Secondary
          </Button>
          <Button variant="secondary" size="md">
            MD Secondary
          </Button>
          <Button variant="secondary" size="lg">
            LG Secondary
          </Button>
          <Button variant="secondary" size="xl">
            XL Secondary
          </Button>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-4">Outline Variants</h3>
        <div className="flex flex-wrap gap-4">
          <Button variant="outline" size="xs">
            XS Outline
          </Button>
          <Button variant="outline" size="sm">
            SM Outline
          </Button>
          <Button variant="outline" size="md">
            MD Outline
          </Button>
          <Button variant="outline" size="lg">
            LG Outline
          </Button>
          <Button variant="outline" size="xl">
            XL Outline
          </Button>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-4">State Variations</h3>
        <div className="flex flex-wrap gap-4">
          <Button isLoading>Loading</Button>
          <Button disabled>Disabled</Button>
          <Button variant="danger" isLoading>
            Loading Danger
          </Button>
          <Button variant="danger" disabled>
            Disabled Danger
          </Button>
        </div>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Complete showcase of button combinations across variants, sizes, and states.',
      },
    },
  },
};

/**
 * With icons example
 */
export const WithIcons: Story = {
  render: () => (
    <div className="flex flex-wrap gap-4">
      <Button variant="primary">
        <svg
          className="w-4 h-4 mr-2"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M12 4v16m8-8H4"
          />
        </svg>
        Add New
      </Button>
      <Button variant="outline">
        <svg
          className="w-4 h-4 mr-2"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"
          />
        </svg>
        Upload
      </Button>
      <Button variant="danger">
        <svg
          className="w-4 h-4 mr-2"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
          />
        </svg>
        Delete
      </Button>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Buttons with icons alongside text.',
      },
    },
  },
};
