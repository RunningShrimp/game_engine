import type { Meta, StoryObj } from '@storybook/react';
import {
  EmptyState,
  NoDataEmptyState,
  NoSearchResultsEmptyState,
  ErrorEmptyState,
} from './EmptyState';
import {
  FolderOpenIcon,
  DocumentTextIcon,
  ExclamationCircleIcon,
} from '@heroicons/react/24/outline';

/**
 * EmptyState Component Stories
 *
 * Comprehensive stories showcasing all EmptyState variants and configurations.
 */

const meta: Meta<typeof EmptyState> = {
  title: 'Atoms/EmptyState',
  component: EmptyState,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: `
A component to display empty states when there is no content to show.
Provides visual feedback and optional actions to guide users.

**Features:**
- 3 sizes: sm, md, lg
- Custom icon support
- Optional action buttons
- Pre-configured variants for common use cases
- Fully responsive design
        `,
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    size: {
      control: 'select',
      options: ['sm', 'md', 'lg'],
      description: 'Size variant',
    },
    title: {
      control: 'text',
      description: 'Main heading text',
    },
    description: {
      control: 'text',
      description: 'Descriptive text below the title',
    },
  },
};

export default meta;
type Story = StoryObj<typeof EmptyState>;

/**
 * Default EmptyState story
 */
export const Default: Story = {
  args: {
    title: 'No content found',
    description: 'Get started by creating some content.',
    size: 'md',
  },
};

/**
 * All sizes
 */
export const Sizes: Story = {
  render: () => (
    <div className="space-y-8">
      <div className="p-8 border rounded">
        <h4 className="text-sm font-semibold mb-4 text-gray-500">Small</h4>
        <EmptyState
          size="sm"
          icon={<FolderOpenIcon />}
          title="No documents"
          description="Create your first document."
        />
      </div>

      <div className="p-8 border rounded">
        <h4 className="text-sm font-semibold mb-4 text-gray-500">Medium</h4>
        <EmptyState
          size="md"
          icon={<FolderOpenIcon />}
          title="No documents"
          description="Create your first document."
        />
      </div>

      <div className="p-8 border rounded">
        <h4 className="text-sm font-semibold mb-4 text-gray-500">Large</h4>
        <EmptyState
          size="lg"
          icon={<FolderOpenIcon />}
          title="No documents"
          description="Create your first document."
        />
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'All available sizes for the EmptyState component.',
      },
    },
  },
};

/**
 * With icon
 */
export const WithIcon: Story = {
  render: () => (
    <div className="p-8 border rounded max-w-md">
      <EmptyState
        icon={<FolderOpenIcon />}
        title="No documents found"
        description="You haven't created any documents yet. Create your first document to get started."
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'EmptyState with custom icon.',
      },
    },
  },
};

/**
 * With action button
 */
export const WithAction: Story = {
  render: () => (
    <div className="p-8 border rounded max-w-md">
      <EmptyState
        icon={<DocumentTextIcon />}
        title="No documents found"
        description="Get started by creating your first document."
        action={
          <button className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors">
            Create Document
          </button>
        }
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'EmptyState with action button.',
      },
    },
  },
};

/**
 * No data preset
 */
export const NoData: Story = {
  render: () => (
    <div className="p-8 border rounded max-w-md">
      <NoDataEmptyState
        onAction={() => alert('Action clicked!')}
        actionLabel="Add Data"
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Pre-configured "No Data" empty state with action button.',
      },
    },
  },
};

/**
 * No search results preset
 */
export const NoSearchResults: Story = {
  render: () => (
    <div className="p-8 border rounded max-w-md">
      <NoSearchResultsEmptyState />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Pre-configured "No Search Results" empty state.',
      },
    },
  },
};

/**
 * Error state preset
 */
export const ErrorState: Story = {
  render: () => (
    <div className="p-8 border rounded max-w-md">
      <ErrorEmptyState onRetry={() => alert('Retrying...')} />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Pre-configured error state with retry button.',
      },
    },
  },
};

/**
 * Interactive example with controls
 */
export const Interactive: Story = {
  args: {
    title: 'No content found',
    description: 'Get started by creating some content.',
    size: 'md',
  },
  parameters: {
    docs: {
      description: {
        story: 'Use the controls panel to interactively change EmptyState props.',
      },
    },
  },
};

/**
 * Use case examples
 */
export const UseCases: Story = {
  render: () => (
    <div className="space-y-8">
      <div>
        <h3 className="text-lg font-semibold mb-4">Empty Document List</h3>
        <div className="p-8 border rounded max-w-md">
          <EmptyState
            icon={<FolderOpenIcon />}
            title="No documents"
            description="You haven't created any documents yet. Create your first document to get started."
            action={
              <button className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors">
                Create Document
              </button>
            }
          />
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-4">Empty Search Results</h3>
        <div className="p-8 border rounded max-w-md">
          <NoSearchResultsEmptyState />
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-4">Error Loading Data</h3>
        <div className="p-8 border rounded max-w-md">
          <ErrorEmptyState onRetry={() => alert('Retrying...')} />
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-4">Empty Project List</h3>
        <div className="p-8 border rounded max-w-md">
          <EmptyState
            size="lg"
            icon={<ExclamationCircleIcon />}
            title="No projects found"
            description="Create a new project or import an existing one to get started."
            action={
              <button className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors">
                New Project
              </button>
            }
          />
        </div>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Real-world use case examples for EmptyState component.',
      },
    },
  },
};

/**
 * Minimal without icon
 */
export const Minimal: Story = {
  render: () => (
    <div className="p-8 border rounded max-w-md">
      <EmptyState
        title="No content"
        description="There is no content to display at this time."
      />
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story: 'Minimal EmptyState without icon.',
      },
    },
  },
};
