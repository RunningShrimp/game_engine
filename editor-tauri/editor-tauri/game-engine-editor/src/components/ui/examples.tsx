/**
 * UI Components Examples
 *
 * This file demonstrates usage of all UI components.
 * Use this as a reference for implementing components in your application.
 */

import React, { useState } from 'react';
import {
  Button,
  Spinner,
  Skeleton,
  EmptyState,
  NoDataEmptyState,
  NoSearchResultsEmptyState,
  ErrorEmptyState,
  CardSkeleton,
  TableSkeleton,
} from './index';
import { FolderOpenIcon, SearchIcon, AlertCircleIcon } from 'lucide-react';

/**
 * Button Examples
 */
export const ButtonExamples = () => {
  const [isLoading, setIsLoading] = useState(false);

  const handleLoading = () => {
    setIsLoading(true);
    setTimeout(() => setIsLoading(false), 2000);
  };

  return (
    <div className="space-y-8 p-6">
      {/* Variants */}
      <section>
        <h2 className="text-xl font-semibold mb-4">Button Variants</h2>
        <div className="flex flex-wrap gap-4">
          <Button variant="primary">Primary</Button>
          <Button variant="secondary">Secondary</Button>
          <Button variant="outline">Outline</Button>
          <Button variant="ghost">Ghost</Button>
          <Button variant="danger">Danger</Button>
        </div>
      </section>

      {/* Sizes */}
      <section>
        <h2 className="text-xl font-semibold mb-4">Button Sizes</h2>
        <div className="flex items-center gap-4">
          <Button size="xs">Extra Small</Button>
          <Button size="sm">Small</Button>
          <Button size="md">Medium</Button>
          <Button size="lg">Large</Button>
          <Button size="xl">Extra Large</Button>
        </div>
      </section>

      {/* States */}
      <section>
        <h2 className="text-xl font-semibold mb-4">Button States</h2>
        <div className="flex flex-wrap gap-4">
          <Button>Default</Button>
          <Button disabled>Disabled</Button>
          <Button isLoading>Loading</Button>
          <Button isLoading={isLoading} onClick={handleLoading}>
            Click to Load
          </Button>
        </div>
      </section>

      {/* Full Width */}
      <section>
        <h2 className="text-xl font-semibold mb-4">Full Width Button</h2>
        <Button fullWidth>Full Width Button</Button>
      </section>
    </div>
  );
};

/**
 * Spinner Examples
 */
export const SpinnerExamples = () => {
  return (
    <div className="space-y-8 p-6">
      {/* Sizes */}
      <section>
        <h2 className="text-xl font-semibold mb-4">Spinner Sizes</h2>
        <div className="flex items-center gap-6">
          <div className="text-center">
            <Spinner size="xs" />
            <p className="text-sm mt-2">XS</p>
          </div>
          <div className="text-center">
            <Spinner size="sm" />
            <p className="text-sm mt-2">SM</p>
          </div>
          <div className="text-center">
            <Spinner size="md" />
            <p className="text-sm mt-2">MD</p>
          </div>
          <div className="text-center">
            <Spinner size="lg" />
            <p className="text-sm mt-2">LG</p>
          </div>
          <div className="text-center">
            <Spinner size="xl" />
            <p className="text-sm mt-2">XL</p>
          </div>
        </div>
      </section>

      {/* Colors */}
      <section>
        <h2 className="text-xl font-semibold mb-4">Spinner Colors</h2>
        <div className="flex items-center gap-6">
          <div className="text-center">
            <Spinner size="lg" color="primary" />
            <p className="text-sm mt-2">Primary</p>
          </div>
          <div className="text-center">
            <Spinner size="lg" color="secondary" />
            <p className="text-sm mt-2">Secondary</p>
          </div>
          <div className="text-center bg-blue-600 p-4 rounded">
            <Spinner size="lg" color="white" />
            <p className="text-sm mt-2 text-white">White</p>
          </div>
          <div className="text-center">
            <Spinner size="lg" color="currentColor" />
            <p className="text-sm mt-2">Current</p>
          </div>
        </div>
      </section>

      {/* Speeds */}
      <section>
        <h2 className="text-xl font-semibold mb-4">Spinner Speeds</h2>
        <div className="flex items-center gap-6">
          <div className="text-center">
            <Spinner size="lg" speed="slow" />
            <p className="text-sm mt-2">Slow</p>
          </div>
          <div className="text-center">
            <Spinner size="lg" speed="normal" />
            <p className="text-sm mt-2">Normal</p>
          </div>
          <div className="text-center">
            <Spinner size="lg" speed="fast" />
            <p className="text-sm mt-2">Fast</p>
          </div>
        </div>
      </section>
    </div>
  );
};

/**
 * Skeleton Examples
 */
export const SkeletonExamples = () => {
  return (
    <div className="space-y-8 p-6">
      {/* Text Skeleton */}
      <section>
        <h2 className="text-xl font-semibold mb-4">Text Skeleton</h2>
        <div className="space-y-2">
          <Skeleton variant="text" width="100%" />
          <Skeleton variant="text" width="80%" />
          <Skeleton variant="text" width="60%" />
        </div>
      </section>

      {/* Multiple Lines */}
      <section>
        <h2 className="text-xl font-semibold mb-4">Multiple Lines</h2>
        <Skeleton variant="text" lines={4} />
      </section>

      {/* Circular Skeleton */}
      <section>
        <h2 className="text-xl font-semibold mb-4">Circular (Avatar)</h2>
        <div className="flex items-center gap-4">
          <Skeleton variant="circular" width="40" height="40" />
          <div className="flex-1 space-y-2">
            <Skeleton variant="text" width="40%" />
            <Skeleton variant="text" width="60%" />
          </div>
        </div>
      </section>

      {/* Rectangular Skeleton */}
      <section>
        <h2 className="text-xl font-semibold mb-4">Rectangular (Image/Card)</h2>
        <div className="grid grid-cols-2 gap-4">
          <Skeleton variant="rectangular" width="100%" height={150} />
          <Skeleton variant="rectangular" width="100%" height={150} />
        </div>
      </section>

      {/* Card Skeleton */}
      <section>
        <h2 className="text-xl font-semibold mb-4">Card Skeleton</h2>
        <div className="border rounded-lg max-w-md">
          <CardSkeleton />
        </div>
      </section>

      {/* Table Skeleton */}
      <section>
        <h2 className="text-xl font-semibold mb-4">Table Skeleton</h2>
        <div className="border rounded-lg p-4 max-w-2xl">
          <TableSkeleton rows={5} columns={4} />
        </div>
      </section>

      {/* Without Animation */}
      <section>
        <h2 className="text-xl font-semibold mb-4">Without Animation</h2>
        <Skeleton variant="rectangular" width="100%" height={100} animate={false} />
      </section>
    </div>
  );
};

/**
 * EmptyState Examples
 */
export const EmptyStateExamples = () => {
  const handleRetry = () => {
    console.log('Retry clicked');
  };

  const handleCreate = () => {
    console.log('Create clicked');
  };

  return (
    <div className="space-y-8 p-6">
      {/* Custom Empty State */}
      <section>
        <h2 className="text-xl font-semibold mb-4">Custom Empty State</h2>
        <div className="border rounded-lg p-8">
          <EmptyState
            icon={<FolderOpenIcon />}
            title="No documents found"
            description="Create your first document to get started with the project."
            action={
              <Button onClick={handleCreate}>Create Document</Button>
            }
          />
        </div>
      </section>

      {/* Sizes */}
      <section>
        <h2 className="text-xl font-semibold mb-4">Empty State Sizes</h2>
        <div className="grid grid-cols-3 gap-4">
          <div className="border rounded-lg p-4">
            <EmptyState
              size="sm"
              icon={<FolderOpenIcon />}
              title="Small"
            />
          </div>
          <div className="border rounded-lg p-4">
            <EmptyState
              size="md"
              icon={<FolderOpenIcon />}
              title="Medium"
            />
          </div>
          <div className="border rounded-lg p-4">
            <EmptyState
              size="lg"
              icon={<FolderOpenIcon />}
              title="Large"
            />
          </div>
        </div>
      </section>

      {/* Pre-configured Variants */}
      <section>
        <h2 className="text-xl font-semibold mb-4">Pre-configured Variants</h2>
        <div className="grid grid-cols-1 gap-6">
          {/* No Data */}
          <div className="border rounded-lg p-8">
            <NoDataEmptyState
              onAction={handleCreate}
              actionLabel="Add Data"
            />
          </div>

          {/* No Search Results */}
          <div className="border rounded-lg p-8">
            <NoSearchResultsEmptyState />
          </div>

          {/* Error */}
          <div className="border rounded-lg p-8">
            <ErrorEmptyState onRetry={handleRetry} />
          </div>
        </div>
      </section>

      {/* Without Icon */}
      <section>
        <h2 className="text-xl font-semibold mb-4">Without Icon</h2>
        <div className="border rounded-lg p-8">
          <EmptyState
            title="Simple Empty State"
            description="Just text without an icon."
          />
        </div>
      </section>
    </div>
  );
};

/**
 * Complete UI Components Demo
 */
export const UIDemo = () => {
  const [activeTab, setActiveTab] = useState<'buttons' | 'spinners' | 'skeletons' | 'emptystates'>('buttons');

  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white shadow-sm">
        <div className="max-w-7xl mx-auto px-4 py-6">
          <h1 className="text-3xl font-bold text-gray-900">UI Components Demo</h1>
          <p className="text-gray-600 mt-2">
            Examples and usage demonstrations for all UI components
          </p>
        </div>
      </header>

      <nav className="bg-white border-b">
        <div className="max-w-7xl mx-auto px-4">
          <div className="flex gap-4">
            <button
              onClick={() => setActiveTab('buttons')}
              className={`px-4 py-3 font-medium border-b-2 transition-colors ${
                activeTab === 'buttons'
                  ? 'border-blue-600 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700'
              }`}
            >
              Buttons
            </button>
            <button
              onClick={() => setActiveTab('spinners')}
              className={`px-4 py-3 font-medium border-b-2 transition-colors ${
                activeTab === 'spinners'
                  ? 'border-blue-600 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700'
              }`}
            >
              Spinners
            </button>
            <button
              onClick={() => setActiveTab('skeletons')}
              className={`px-4 py-3 font-medium border-b-2 transition-colors ${
                activeTab === 'skeletons'
                  ? 'border-blue-600 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700'
              }`}
            >
              Skeletons
            </button>
            <button
              onClick={() => setActiveTab('emptystates')}
              className={`px-4 py-3 font-medium border-b-2 transition-colors ${
                activeTab === 'emptystates'
                  ? 'border-blue-600 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700'
              }`}
            >
              Empty States
            </button>
          </div>
        </div>
      </nav>

      <main className="max-w-7xl mx-auto px-4 py-8">
        {activeTab === 'buttons' && <ButtonExamples />}
        {activeTab === 'spinners' && <SpinnerExamples />}
        {activeTab === 'skeletons' && <SkeletonExamples />}
        {activeTab === 'emptystates' && <EmptyStateExamples />}
      </main>
    </div>
  );
};

export default UIDemo;
