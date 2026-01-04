import React from 'react';
import { cn } from '../../utils/cn';

/**
 * EmptyState component props
 */
export interface EmptyStateProps {
  /**
   * Icon or illustration to display
   */
  icon?: React.ReactNode;

  /**
   * Main heading text
   */
  title: string;

  /**
   * Descriptive text below the title
   */
  description?: string;

  /**
   * Action button to display
   */
  action?: React.ReactNode;

  /**
   * Additional CSS classes
   */
  className?: string;

  /**
   * Size variant
   * @default 'md'
   */
  size?: 'sm' | 'md' | 'lg';
}

/**
 * Size styles mapping
 */
const sizeStyles = {
  sm: {
    icon: 'h-12 w-12',
    title: 'text-lg',
    description: 'text-sm',
  },
  md: {
    icon: 'h-16 w-16',
    title: 'text-xl',
    description: 'text-base',
  },
  lg: {
    icon: 'h-24 w-24',
    title: 'text-2xl',
    description: 'text-lg',
  },
};

/**
 * EmptyState component
 *
 * A component to display empty states when there is no content to show.
 * Provides visual feedback and optional actions to guide users.
 *
 * @example
 * ```tsx
 * <EmptyState
 *   icon={<FolderOpenIcon />}
 *   title="No documents found"
 *   description="Create your first document to get started."
 *   action={<Button>Create Document</Button>}
 * />
 *
 * <EmptyState
 *   size="sm"
 *   title="No results"
 *   description="Try adjusting your search filters."
 * />
 * ```
 */
export const EmptyState = ({
  icon,
  title,
  description,
  action,
  className,
  size = 'md',
}: EmptyStateProps) => {
  const styles = sizeStyles[size];

  return (
    <div
      className={cn(
        'flex flex-col items-center justify-center text-center',
        'p-8',
        className
      )}
    >
      {icon && (
        <div
          className={cn(
            'mx-auto mb-4 text-gray-400',
            styles.icon
          )}
        >
          {icon}
        </div>
      )}

      <h3
        className={cn(
          'font-semibold text-gray-900 mb-2',
          styles.title
        )}
      >
        {title}
      </h3>

      {description && (
        <p
          className={cn(
            'text-gray-500 max-w-sm mb-6',
            styles.description
          )}
        >
          {description}
        </p>
      )}

      {action && <div>{action}</div>}
    </div>
  );
};

EmptyState.displayName = 'EmptyState';

/**
 * Pre-configured empty state for "no data"
 */
export const NoDataEmptyState = ({
  onAction,
  actionLabel = 'Add Data',
}: {
  onAction?: () => void;
  actionLabel?: string;
}) => (
  <EmptyState
    title="No data available"
    description="There is no data to display. Get started by adding some data."
    action={
      onAction ? (
        <button
          onClick={onAction}
          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
        >
          {actionLabel}
        </button>
      ) : undefined
    }
  />
);

NoDataEmptyState.displayName = 'NoDataEmptyState';

/**
 * Pre-configured empty state for "no search results"
 */
export const NoSearchResultsEmptyState = () => (
  <EmptyState
    title="No results found"
    description="We couldn't find anything matching your search. Try adjusting your filters or search terms."
  />
);

NoSearchResultsEmptyState.displayName = 'NoSearchResultsEmptyState';

/**
 * Pre-configured empty state for "error state"
 */
export const ErrorEmptyState = ({
  onRetry,
}: {
  onRetry?: () => void;
}) => (
  <EmptyState
    title="Something went wrong"
    description="An error occurred while loading the content. Please try again."
    action={
      onRetry ? (
        <button
          onClick={onRetry}
          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
        >
          Try Again
        </button>
      ) : undefined
    }
  />
);

ErrorEmptyState.displayName = 'ErrorEmptyState';
