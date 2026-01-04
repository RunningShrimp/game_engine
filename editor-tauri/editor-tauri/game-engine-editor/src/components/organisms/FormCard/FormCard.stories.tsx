import type { Meta, StoryObj } from '@storybook/react';
import { FormCard } from './index';
import { Input } from '../../molecules/Input';
import { Checkbox } from '../../molecules/Checkbox';
import { Button } from '../../atoms/Button';

/**
 * FormCard Component Stories
 *
 * A complex form container component that organizes form sections with proper spacing,
 * headers, and actions. This is an organism-level component that combines molecules and atoms.
 */

const meta: Meta<typeof FormCard> = {
  title: 'Organisms/FormCard',
  component: FormCard,
  parameters: {
    layout: 'padded',
    docs: {
      description: {
        component: `
A complex form container component that organizes form sections with proper spacing,
headers, and actions. This organism-level component combines multiple molecules and atoms.

**Features:**
- Sectioned form layout
- Integrated header and actions
- Responsive design
- Loading and error states
- Accessibility support
        `,
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    title: {
      control: 'text',
      description: 'Form card title',
    },
    description: {
      control: 'text',
      description: 'Form card description',
    },
    isLoading: {
      control: 'boolean',
      description: 'Loading state',
    },
  },
};

export default meta;
type Story = StoryObj<typeof FormCard>;

/**
 * Default FormCard
 */
export const Default: Story = {
  render: () => (
    <div className="w-full max-w-2xl">
      <div className="bg-white rounded-lg shadow-sm border border-gray-200">
        <div className="p-6 border-b border-gray-200">
          <h2 className="text-xl font-semibold text-gray-900">User Settings</h2>
          <p className="text-sm text-gray-500 mt-1">
            Manage your account settings and preferences
          </p>
        </div>
        <div className="p-6 space-y-6">
          <Input
            label="Full Name"
            icon="User"
            placeholder="Enter your full name"
            defaultValue="John Doe"
          />
          <Input
            label="Email"
            icon="Mail"
            type="email"
            placeholder="you@example.com"
            defaultValue="john@example.com"
          />
          <div className="pt-4 border-t border-gray-200">
            <Button>Save Changes</Button>
          </div>
        </div>
      </div>
    </div>
  ),
};

/**
 * With multiple sections
 */
export const WithSections: Story = {
  render: () => (
    <div className="w-full max-w-2xl space-y-6">
      {/* Personal Information Section */}
      <div className="bg-white rounded-lg shadow-sm border border-gray-200">
        <div className="p-6 border-b border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900">Personal Information</h3>
        </div>
        <div className="p-6 space-y-4">
          <Input label="Full Name" placeholder="John Doe" />
          <Input label="Email" type="email" placeholder="you@example.com" />
          <Input label="Phone" icon="Phone" placeholder="+1 234 567 890" />
        </div>
      </div>

      {/* Preferences Section */}
      <div className="bg-white rounded-lg shadow-sm border border-gray-200">
        <div className="p-6 border-b border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900">Preferences</h3>
        </div>
        <div className="p-6 space-y-4">
          <Checkbox
            label="Email notifications"
            helpText="Receive email updates about your account"
            defaultChecked
          />
          <Checkbox
            label="Marketing communications"
            helpText="Receive emails about new features and promotions"
          />
          <Checkbox
            label="Two-factor authentication"
            helpText="Add an extra layer of security to your account"
          />
        </div>
      </div>

      {/* Actions */}
      <div className="flex gap-3 justify-end">
        <Button variant="outline">Cancel</Button>
        <Button>Save All Changes</Button>
      </div>
    </div>
  ),
};

/**
 * Loading state
 */
export const Loading: Story = {
  render: () => (
    <div className="w-full max-w-2xl">
      <div className="bg-white rounded-lg shadow-sm border border-gray-200">
        <div className="p-6 border-b border-gray-200">
          <div className="animate-pulse">
            <div className="h-6 bg-gray-200 rounded w-1/3 mb-2"></div>
            <div className="h-4 bg-gray-200 rounded w-2/3"></div>
          </div>
        </div>
        <div className="p-6 space-y-4">
          <div className="animate-pulse space-y-3">
            <div className="h-4 bg-gray-200 rounded w-1/4"></div>
            <div className="h-10 bg-gray-200 rounded"></div>
            <div className="h-4 bg-gray-200 rounded w-1/6"></div>
            <div className="h-10 bg-gray-200 rounded"></div>
          </div>
        </div>
      </div>
    </div>
  ),
};

/**
 * With errors
 */
export const WithErrors: Story = {
  render: () => (
    <div className="w-full max-w-2xl">
      <div className="bg-white rounded-lg shadow-sm border border-gray-200">
        <div className="p-6 border-b border-gray-200">
          <h2 className="text-xl font-semibold text-red-600">Form Errors</h2>
          <p className="text-sm text-gray-500 mt-1">
            Please fix the errors below to continue
          </p>
        </div>
        <div className="p-6 space-y-6">
          <Input
            label="Email"
            icon="Mail"
            type="email"
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
          <div className="pt-4 border-t border-gray-200">
            <Button disabled>Save Changes</Button>
          </div>
        </div>
      </div>
    </div>
  ),
};

/**
 * Complete registration form
 */
export const RegistrationForm: Story = {
  render: () => (
    <div className="w-full max-w-2xl">
      <div className="bg-white rounded-lg shadow-sm border border-gray-200">
        <div className="p-6 border-b border-gray-200">
          <h2 className="text-xl font-semibold text-gray-900">Create Account</h2>
          <p className="text-sm text-gray-500 mt-1">
            Join the Game Engine Editor community
          </p>
        </div>
        <div className="p-6 space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <Input label="First Name" placeholder="John" />
            <Input label="Last Name" placeholder="Doe" />
          </div>
          <Input
            label="Email"
            icon="Mail"
            type="email"
            required
            placeholder="you@example.com"
          />
          <Input
            label="Password"
            icon="Lock"
            type="password"
            required
            helpText="Must be at least 8 characters with letters and numbers"
            placeholder="Create a password"
          />
          <Input
            label="Confirm Password"
            icon="Lock"
            type="password"
            required
            placeholder="Confirm your password"
          />

          <div className="pt-4 border-t border-gray-200">
            <Checkbox
              label="I accept the Terms of Service and Privacy Policy"
              required
            />
          </div>

          <div className="pt-4 space-y-3">
            <Button fullWidth size="lg">
              Create Account
            </Button>
            <p className="text-sm text-center text-gray-500">
              Already have an account?{' '}
              <a href="#" className="text-blue-600 hover:underline">
                Sign in
              </a>
            </p>
          </div>
        </div>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Complete registration form with validation and proper structure.',
      },
    },
  },
};

/**
 * Settings panel
 */
export const SettingsPanel: Story = {
  render: () => (
    <div className="w-full max-w-3xl space-y-6">
      {/* Profile Section */}
      <div className="bg-white rounded-lg shadow-sm border border-gray-200">
        <div className="p-6 border-b border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900">Profile Information</h3>
          <p className="text-sm text-gray-500 mt-1">
            Update your personal information
          </p>
        </div>
        <div className="p-6 space-y-4">
          <Input label="Display Name" placeholder="John Doe" />
          <Input
            label="Bio"
            helpText="Tell us a little about yourself"
            placeholder="I'm a game developer..."
          />
          <Input
            label="Website"
            icon="Globe"
            placeholder="https://yourwebsite.com"
          />
          <Input
            label="Location"
            icon="MapPin"
            placeholder="City, Country"
          />
        </div>
        <div className="p-6 bg-gray-50 border-t border-gray-200">
          <Button>Save Profile</Button>
        </div>
      </div>

      {/* Notifications Section */}
      <div className="bg-white rounded-lg shadow-sm border border-gray-200">
        <div className="p-6 border-b border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900">Notification Preferences</h3>
          <p className="text-sm text-gray-500 mt-1">
            Choose how you want to be notified
          </p>
        </div>
        <div className="p-6 space-y-4">
          <Checkbox
            label="Email notifications"
            helpText="Receive notifications via email"
            defaultChecked
          />
          <Checkbox
            label="Push notifications"
            helpText="Receive browser push notifications"
          />
          <Checkbox
            label="Product updates"
            helpText="Receive updates about new features"
            defaultChecked
          />
          <Checkbox
            label="Weekly digest"
            helpText="Get a weekly summary of activity"
          />
        </div>
        <div className="p-6 bg-gray-50 border-t border-gray-200">
          <Button>Save Preferences</Button>
        </div>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Settings panel with multiple sections for organizing preferences.',
      },
    },
  },
};

/**
 * Dark theme example
 */
export const DarkTheme: Story = {
  render: () => (
    <div className="w-full max-w-2xl bg-gray-900 p-8">
      <div className="bg-gray-800 rounded-lg shadow-sm border border-gray-700">
        <div className="p-6 border-b border-gray-700">
          <h2 className="text-xl font-semibold text-white">Dark Theme Form</h2>
          <p className="text-sm text-gray-400 mt-1">
            Form optimized for dark mode
          </p>
        </div>
        <div className="p-6 space-y-4">
          <Input
            label="Username"
            icon="User"
            placeholder="Enter username"
          />
          <Input
            label="Email"
            icon="Mail"
            type="email"
            placeholder="you@example.com"
          />
          <Checkbox
            label="Remember me"
            helpText="Keep me signed in on this device"
          />
          <Button fullWidth>Sign In</Button>
        </div>
      </div>
    </div>
  ),
  parameters: {
    backgrounds: {
      default: 'dark',
    },
    docs: {
      description: {
        story: 'Form with dark theme styling.',
      },
    },
  },
};
