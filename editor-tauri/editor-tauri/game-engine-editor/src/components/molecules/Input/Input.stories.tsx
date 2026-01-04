import type { Meta, StoryObj } from '@storybook/react';
import { Input } from './index';
import { useState } from 'react';

/**
 * Input Component Stories
 *
 * Comprehensive stories showcasing all Input variants, sizes, and states.
 */

const meta: Meta<typeof Input> = {
  title: 'Molecules/Input',
  component: Input,
  parameters: {
    layout: 'padded',
    docs: {
      description: {
        component: `
A text input component composed of Icon + Text input atoms.
Supports labels, icons, error states, help text, and all standard input attributes.

**Features:**
- 3 sizes: sm, md, lg
- Icon support (left or right position)
- Label and help text
- Error state with error message
- Controlled and uncontrolled modes
- Full keyboard accessibility
        `,
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    size: {
      control: 'select',
      options: ['sm', 'md', 'lg'],
      description: 'Input size',
    },
    icon: {
      control: 'text',
      description: 'Lucide icon name (e.g., "Mail", "Lock")',
    },
    iconPosition: {
      control: 'select',
      options: ['left', 'right'],
      description: 'Icon position',
    },
    error: {
      control: 'boolean',
      description: 'Error state',
    },
    errorMessage: {
      control: 'text',
      description: 'Error message to display',
    },
    helpText: {
      control: 'text',
      description: 'Help/description text',
    },
    label: {
      control: 'text',
      description: 'Label for the input',
    },
    required: {
      control: 'boolean',
      description: 'Mark field as required',
    },
    fullWidth: {
      control: 'boolean',
      description: 'Full width',
    },
  },
};

export default meta;
type Story = StoryObj<typeof Input>;

/**
 * Default Input story
 */
export const Default: Story = {
  args: {
    label: 'Email',
    placeholder: 'Enter your email',
    size: 'md',
  },
};

/**
 * All sizes
 */
export const Sizes: Story = {
  render: () => (
    <div className="space-y-4 max-w-md">
      <Input
        size="sm"
        label="Small Input"
        placeholder="Small input"
      />
      <Input
        size="md"
        label="Medium Input"
        placeholder="Medium input"
      />
      <Input
        size="lg"
        label="Large Input"
        placeholder="Large input"
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'All available input sizes.',
      },
    },
  },
};

/**
 * With icons
 */
export const WithIcons: Story = {
  render: () => (
    <div className="space-y-4 max-w-md">
      <Input
        label="Email"
        icon="Mail"
        placeholder="you@example.com"
      />
      <Input
        label="Password"
        icon="Lock"
        type="password"
        placeholder="Enter password"
      />
      <Input
        label="Search"
        icon="Search"
        placeholder="Search..."
      />
      <Input
        label="Website"
        icon="Globe"
        iconPosition="right"
        placeholder="https://example.com"
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Inputs with icons on left and right positions.',
      },
    },
  },
};

/**
 * Error state
 */
export const ErrorState: Story = {
  render: () => (
    <div className="space-y-4 max-w-md">
      <Input
        label="Email"
        icon="Mail"
        error
        errorMessage="Please enter a valid email address"
        placeholder="you@example.com"
      />
      <Input
        label="Password"
        icon="Lock"
        type="password"
        error
        errorMessage="Password must be at least 8 characters"
        placeholder="Enter password"
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Inputs in error state with error messages.',
      },
    },
  },
};

/**
 * With help text
 */
export const WithHelpText: Story = {
  render: () => (
    <div className="space-y-4 max-w-md">
      <Input
        label="Username"
        icon="User"
        helpText="Choose a unique username for your account"
        placeholder="Enter username"
      />
      <Input
        label="Password"
        icon="Lock"
        type="password"
        helpText="Must be at least 8 characters with letters and numbers"
        placeholder="Enter password"
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Inputs with help text for additional guidance.',
      },
    },
  },
};

/**
 * Required fields
 */
export const RequiredFields: Story = {
  render: () => (
    <div className="space-y-4 max-w-md">
      <Input
        label="Email"
        icon="Mail"
        required
        placeholder="you@example.com"
      />
      <Input
        label="Password"
        icon="Lock"
        type="password"
        required
        placeholder="Enter password"
      />
      <Input
        label="Full Name"
        icon="User"
        required
        placeholder="John Doe"
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Required input fields with asterisk indicator.',
      },
    },
  },
};

/**
 * Full width
 */
export const FullWidth: Story = {
  render: () => (
    <div className="space-y-4">
      <Input
        fullWidth
        label="Full Width Input"
        placeholder="This input takes full width"
      />
      <Input
        fullWidth
        label="Subject"
        placeholder="Email subject"
      />
      <Input
        fullWidth
        label="Message"
        placeholder="Your message"
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Full width inputs that expand to container width.',
      },
    },
  },
};

/**
 * Controlled input
 */
export const Controlled: Story = {
  render: () => {
    const [value, setValue] = useState('');

    return (
      <div className="space-y-4 max-w-md">
        <Input
          label="Controlled Input"
          icon="Edit"
          value={value}
          onChange={setValue}
          placeholder="Type something..."
        />
        <p className="text-sm text-gray-600">
          Current value: "{value || '(empty)'}"
        </p>
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Controlled input with React state.',
      },
    },
  },
};

/**
 * Disabled state
 */
export const Disabled: Story = {
  render: () => (
    <div className="space-y-4 max-w-md">
      <Input
        label="Disabled Input"
        icon="Lock"
        disabled
        placeholder="Cannot edit"
        defaultValue="Locked content"
      />
      <Input
        label="Read-only Input"
        icon="Eye"
        readOnly
        placeholder="Cannot edit"
        defaultValue="Read-only content"
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Disabled and read-only input states.',
      },
    },
  },
};

/**
 * Interactive example with controls
 */
export const Interactive: Story = {
  args: {
    label: 'Email',
    placeholder: 'Enter your email',
    size: 'md',
    error: false,
    required: false,
    fullWidth: false,
  },
  parameters: {
    docs: {
      description: {
        story: 'Use the controls panel to interactively change input props.',
      },
    },
  },
};

/**
 * Form example
 */
export const FormExample: Story = {
  render: () => {
    const [formData, setFormData] = useState({
      email: '',
      password: '',
      username: '',
    });

    return (
      <div className="max-w-md p-6 bg-white rounded-lg shadow">
        <h3 className="text-lg font-semibold mb-4">Sign Up Form</h3>
        <div className="space-y-4">
          <Input
            label="Username"
            icon="User"
            required
            helpText="Choose a unique username"
            placeholder="johndoe"
            value={formData.username}
            onChange={(value) => setFormData({ ...formData, username: value })}
          />
          <Input
            label="Email"
            icon="Mail"
            type="email"
            required
            placeholder="you@example.com"
            value={formData.email}
            onChange={(value) => setFormData({ ...formData, email: value })}
          />
          <Input
            label="Password"
            icon="Lock"
            type="password"
            required
            helpText="Must be at least 8 characters"
            placeholder="Enter password"
            value={formData.password}
            onChange={(value) => setFormData({ ...formData, password: value })}
          />
          <button className="w-full px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700">
            Sign Up
          </button>
        </div>
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Complete form example with multiple controlled inputs.',
      },
    },
  },
};

/**
 * Input combinations
 */
export const Combinations: Story = {
  render: () => (
    <div className="space-y-8">
      <div>
        <h3 className="text-lg font-semibold mb-4">Size Variations</h3>
        <div className="space-y-4 max-w-md">
          <Input size="sm" label="Small" placeholder="Small input" />
          <Input size="md" label="Medium" placeholder="Medium input" />
          <Input size="lg" label="Large" placeholder="Large input" />
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-4">With Icons</h3>
        <div className="space-y-4 max-w-md">
          <Input label="Email" icon="Mail" placeholder="you@example.com" />
          <Input label="Phone" icon="Phone" placeholder="+1 234 567 890" />
          <Input label="Location" icon="MapPin" placeholder="City, Country" />
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-4">States</h3>
        <div className="space-y-4 max-w-md">
          <Input label="Normal" placeholder="Normal input" />
          <Input
            label="Error"
            icon="AlertCircle"
            error
            errorMessage="This field is required"
            placeholder="Error input"
          />
          <Input
            label="With Help"
            icon="Info"
            helpText="Additional information about this field"
            placeholder="Input with help text"
          />
        </div>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Complete showcase of input combinations.',
      },
    },
  },
};
