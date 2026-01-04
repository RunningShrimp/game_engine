import React, { HTMLAttributes } from 'react';
import { cn } from '../../utils/cn';

/**
 * Skeleton variant types
 */
export type SkeletonVariant = 'text' | 'rectangular' | 'circular';

/**
 * Skeleton component props
 */
export interface SkeletonProps extends HTMLAttributes<HTMLDivElement> {
  /**
   * Shape variant of the skeleton
   * @default 'rectangular'
   */
  variant?: SkeletonVariant;

  /**
   * Width of the skeleton
   * @example '100px', '50%', 'full'
   */
  width?: string;

  /**
   * Height of the skeleton
   * @example '20px', '100%', 'auto'
   */
  height?: string;

  /**
   * Number of skeleton lines (for text variant)
   * @default 1
   */
  lines?: number;

  /**
   * Enable shimmer animation
   * @default true
   */
  animate?: boolean;
}

/**
 * Variant styles mapping
 */
const variantStyles: Record<SkeletonVariant, string> = {
  text: 'rounded',
  rectangular: 'rounded-md',
  circular: 'rounded-full',
};

/**
 * Animation class
 */
const animationClass = 'animate-pulse';

/**
 * Skeleton component - Loading placeholder
 *
 * A placeholder component that displays a loading animation
 * to indicate that content is being loaded.
 *
 * @example
 * ```tsx
 * // Text skeleton
 * <Skeleton variant="text" width="100%" height="20px" />
 *
 * // Circular avatar skeleton
 * <Skeleton variant="circular" width="40px" height="40px" />
 *
 * // Card skeleton
 * <Skeleton variant="rectangular" width="100%" height="200px" />
 *
 * // Multiple text lines
 * <Skeleton variant="text" lines={3} />
 * ```
 */
export const Skeleton = React.forwardRef<HTMLDivElement, SkeletonProps>(
  (
    {
      variant = 'rectangular',
      width,
      height,
      lines = 1,
      animate = true,
      className,
      style,
      ...props
    },
    ref
  ) => {
    const skeletonStyle = {
      width,
      height,
      ...style,
    };

    // For text variant with multiple lines
    if (variant === 'text' && lines > 1) {
      return (
        <div ref={ref} className={cn('space-y-2', className)} {...props}>
          {Array.from({ length: lines }).map((_, index) => (
            <div
              key={index}
              className={cn(
                'bg-muted',
                variantStyles[variant],
                animate && animationClass
              )}
              style={{
                width: index === lines - 1 ? '60%' : '100%',
                height: height || '1rem',
              }}
              aria-hidden="true"
            />
          ))}
        </div>
      );
    }

    return (
      <div
        ref={ref}
        className={cn(
          'bg-muted',
          variantStyles[variant],
          animate && animationClass,
          className
        )}
        style={skeletonStyle}
        aria-hidden="true"
        {...props}
      />
    );
  }
);

Skeleton.displayName = 'Skeleton';

/**
 * Card skeleton component
 * Pre-configured skeleton for card content
 */
export const CardSkeleton = React.forwardRef<
  HTMLDivElement,
  { className?: string }
>(({ className }, ref) => (
  <div ref={ref} className={cn('p-4', className)}>
    <div className="flex items-center space-x-4">
      <Skeleton variant="circular" width="40px" height="40px" />
      <div className="flex-1 space-y-2">
        <Skeleton variant="text" width="40%" height="20px" />
        <Skeleton variant="text" width="60%" height="16px" />
      </div>
    </div>
  </div>
));

CardSkeleton.displayName = 'CardSkeleton';

/**
 * Table skeleton component
 * Pre-configured skeleton for table rows
 */
export const TableSkeleton = React.forwardRef<
  HTMLDivElement,
  { rows?: number; columns?: number; className?: string }
>(({ rows = 5, columns = 4, className }, ref) => (
  <div ref={ref} className={cn('space-y-3', className)}>
    {Array.from({ length: rows }).map((_, rowIndex) => (
      <div key={rowIndex} className="flex space-x-4">
        {Array.from({ length: columns }).map((_, colIndex) => (
          <Skeleton
            key={colIndex}
            variant="text"
            width={`${100 / columns}%`}
            height="20px"
          />
        ))}
      </div>
    ))}
  </div>
));

TableSkeleton.displayName = 'TableSkeleton';
