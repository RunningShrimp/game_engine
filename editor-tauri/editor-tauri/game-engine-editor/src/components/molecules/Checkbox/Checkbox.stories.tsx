import type { Meta, StoryObj } from '@storybook/react';
import { Checkbox } from './index';
import { useState } from 'react';

/**
 * Checkbox Component Stories
 *
 * Comprehensive stories showcasing all Checkbox variants, sizes, and states.
 */

const meta: Meta<typeof Checkbox> = {
  title: 'Molecules/Checkbox',
  component: Checkbox,
  parameters: {
    layout: 'padded',
    docs: {
      description: {
        component: `
A checkbox input component with label support.
Supports indeterminate state, error states, help text, and all standard checkbox attributes.

**Features:**
- 3 sizes: sm, md, lg
- Label and help text
- Error state with error message
- Indeterminate (mixed) state
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
      description: 'Checkbox size',
    },
    label: {
      control: 'text',
      description: 'Label text',
    },
    helpText: {
      control: 'text',
      description: 'Help/description text',
    },
    error: {
      control: 'boolean',
      description: 'Error state',
    },
    errorMessage: {
      control: 'text',
      description: 'Error message to display',
    },
    indeterminate: {
      control: 'boolean',
      description: 'Indeterminate (mixed) state',
    },
    required: {
      control: 'boolean',
      description: 'Mark field as required',
    },
    checked: {
      control: 'boolean',
      description: 'Checkbox checked state (controlled)',
    },
  },
};

export default meta;
type Story = StoryObj<typeof Checkbox>;

/**
 * Default Checkbox story
 */
export const Default: Story = {
  args: {
    label: 'Accept terms and conditions',
    size: 'md',
  },
};

/**
 * All sizes
 */
export const Sizes: Story = {
  render: () => (
    <div className="space-y-4">
      <Checkbox
        size="sm"
        label="Small checkbox"
        helpText="This is a small checkbox"
      />
      <Checkbox
        size="md"
        label="Medium checkbox"
        helpText="This is a medium checkbox"
      />
      <Checkbox
        size="lg"
        label="Large checkbox"
        helpText="This is a large checkbox"
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'All available checkbox sizes.',
      },
    },
  },
};

/**
 * Unchecked, checked, and indeterminate
 */
export const States: Story = {
  render: () => (
    <div className="space-y-4">
      <Checkbox
        label="Unchecked"
        helpText="Checkbox is unchecked"
      />
      <Checkbox
        label="Checked"
        defaultChecked
        helpText="Checkbox is checked"
      />
      <Checkbox
        label="Indeterminate"
        indeterminate
        helpText="Checkbox is in indeterminate state"
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Different checkbox states: unchecked, checked, and indeterminate.',
      },
    },
  },
};

/**
 * Without label
 */
export const WithoutLabel: Story = {
  render: () => (
    <div className="flex items-center gap-6">
      <Checkbox />
      <Checkbox defaultChecked />
      <Checkbox indeterminate />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Checkboxes without labels for compact layouts.',
      },
    },
  },
};

/**
 * With help text
 */
export const WithHelpText: Story = {
  render: () => (
    <div className="space-y-4">
      <Checkbox
        label="Subscribe to newsletter"
        helpText="Get updates about new features and releases"
        defaultChecked
      />
      <Checkbox
        label="Enable notifications"
        helpText="Receive push notifications for important events"
      />
      <Checkbox
        label="Allow data collection"
        helpText="Help us improve our services by collecting anonymous usage data"
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Checkboxes with help text for additional information.',
      },
    },
  },
};

/**
 * Error state
 */
export const ErrorState: Story = {
  render: () => (
    <div className="space-y-4">
      <Checkbox
        label="I accept the terms and conditions"
        error
        errorMessage="You must accept the terms to continue"
      />
      <Checkbox
        label="I am 18 years or older"
        error
        errorMessage="You must be 18 or older to use this service"
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Checkboxes in error state with error messages.',
      },
    },
  },
};

/**
 * Required fields
 */
export const RequiredFields: Story = {
  render: () => (
    <div className="space-y-4">
      <Checkbox
        label="Accept terms and conditions"
        required
        helpText="You must accept the terms to continue"
      />
      <Checkbox
        label="Privacy policy agreement"
        required
        helpText="Please review and accept our privacy policy"
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Required checkbox fields with asterisk indicator.',
      },
    },
  },
};

/**
 * Controlled checkbox
 */
export const Controlled: Story = {
  render: () => {
    const [checked, setChecked] = useState(false);

    return (
      <div className="space-y-4 max-w-md">
        <Checkbox
          label="Controlled checkbox"
          checked={checked}
          onChange={setChecked}
          helpText={`Current state: ${checked ? 'checked' : 'unchecked'}`}
        />
        <button
          onClick={() => setChecked(!checked)}
          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
        >
          Toggle State
        </button>
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Controlled checkbox with React state.',
      },
    },
  },
};

/**
 * Disabled state
 */
export const Disabled: Story = {
  render: () => (
    <div className="space-y-4">
      <Checkbox
        label="Disabled unchecked"
        disabled
      />
      <Checkbox
        label="Disabled checked"
        disabled
        defaultChecked
      />
      <Checkbox
        label="Disabled indeterminate"
        disabled
        indeterminate
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Disabled checkbox states.',
      },
    },
  },
};

/**
 * Interactive example with controls
 */
export const Interactive: Story = {
  args: {
    label: 'Accept terms and conditions',
    size: 'md',
    error: false,
    required: false,
    indeterminate: false,
  },
  parameters: {
    docs: {
      description: {
        story: 'Use the controls panel to interactively change checkbox props.',
      },
    },
  },
};

/**
 * Checkbox group example
 */
export const CheckboxGroup: Story = {
  render: () => {
    const [preferences, setPreferences] = useState({
      newsletter: true,
      notifications: false,
      updates: true,
    });

    const handleChange = (key: string, checked: boolean) => {
      setPreferences({ ...preferences, [key]: checked });
    };

    return (
      <div className="max-w-md p-6 bg-white rounded-lg shadow">
        <h3 className="text-lg font-semibold mb-4">Communication Preferences</h3>
        <div className="space-y-4">
          <Checkbox
            label="Email newsletter"
            helpText="Weekly newsletter with product updates"
            checked={preferences.newsletter}
            onChange={(checked) => handleChange('newsletter', checked)}
          />
          <Checkbox
            label="Push notifications"
            helpText="Real-time notifications for important events"
            checked={preferences.notifications}
            onChange={(checked) => handleChange('notifications', checked)}
          />
          <Checkbox
            label="Product updates"
            helpText="Email notifications about new features"
            checked={preferences.updates}
            onChange={(checked) => handleChange('updates', checked)}
          />
        </div>
        <div className="mt-4 p-3 bg-gray-50 rounded text-sm">
          Selected: {Object.entries(preferences)
            .filter(([_, v]) => v)
            .map(([k]) => k)
            .join(', ') || 'None'}
        </div>
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Checkbox group for managing multiple related options.',
      },
    },
  },
};

/**
 * Select all example
 */
export const SelectAllExample: Story = {
  render: () => {
    const [items, setItems] = useState({
      all: false,
      item1: false,
      item2: false,
      item3: false,
    });

    const handleSelectAll = (checked: boolean) => {
      setItems({
        all: checked,
        item1: checked,
        item2: checked,
        item3: checked,
      });
    };

    const handleItemChange = (key: string, checked: boolean) => {
      const newItems = { ...items, [key]: checked };
      const allChecked = Object.entries(newItems)
        .filter(([k]) => k !== 'all')
        .every(([_, v]) => v);
      const someChecked = Object.entries(newItems)
        .filter(([k]) => k !== 'all')
        .some(([_, v]) => v);

      setItems({
        ...newItems,
        all: allChecked,
        ...(someChecked && !allChecked ? { indeterminate: true } : {}),
      });
    };

    const selectedCount = Object.entries(items)
      .filter(([k]) => k !== 'all')
      .filter(([_, v]) => v).length;

    return (
      <div className="max-w-md p-6 bg-white rounded-lg shadow">
        <h3 className="text-lg font-semibold mb-4">Select Items</h3>
        <div className="space-y-3">
          <Checkbox
            label="Select All"
            checked={items.all}
            indeterminate={!items.all && selectedCount > 0}
            onChange={handleSelectAll}
          />
          <hr />
          <Checkbox
            label="Item 1"
            checked={items.item1}
            onChange={(checked) => handleItemChange('item1', checked)}
          />
          <Checkbox
            label="Item 2"
            checked={items.item2}
            onChange={(checked) => handleItemChange('item2', checked)}
          />
          <Checkbox
            label="Item 3"
            checked={items.item3}
            onChange={(checked) => handleItemChange('item3', checked)}
          />
        </div>
        <div className="mt-4 text-sm text-gray-600">
          Selected: {selectedCount} / 3 items
        </div>
      </div>
    );
  },
  parameters: {
    docs: {
      description: {
        story: 'Select all functionality with indeterminate state.',
      },
    },
  },
};
