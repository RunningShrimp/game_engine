/**
 * UI Component Library
 *
 * A collection of reusable UI components for the application.
 * All components follow strict TypeScript typing, React best practices,
 * and include comprehensive JSDoc documentation.
 *
 * @example
 * ```tsx
 * import { Button, Spinner, Skeleton, EmptyState } from '@/components/ui';
 * ```
 */

// Button component
export { Button } from './Button';
export type { ButtonProps, ButtonVariant, ButtonSize } from './Button';

// Spinner component
export { Spinner } from './Spinner';
export type { SpinnerProps, SpinnerSize, SpinnerColor } from './Spinner';

// Skeleton component
export { Skeleton, CardSkeleton, TableSkeleton } from './Skeleton';
export type { SkeletonProps, SkeletonVariant } from './Skeleton';

// EmptyState component
export {
  EmptyState,
  NoDataEmptyState,
  NoSearchResultsEmptyState,
  ErrorEmptyState,
} from './EmptyState';
export type { EmptyStateProps } from './EmptyState';
