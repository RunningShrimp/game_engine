import React, { HTMLAttributes } from 'react';
import { cn } from '../../utils/cn';

/**
 * Spinner size types
 */
export type SpinnerSize = 'xs' | 'sm' | 'md' | 'lg' | 'xl';

/**
 * Spinner color types
 */
export type SpinnerColor = 'primary' | 'secondary' | 'white' | 'currentColor';

/**
 * Spinner component props
 */
export interface SpinnerProps extends HTMLAttributes<HTMLDivElement> {
  /**
   * Spinner size
   * @default 'md'
   */
  size?: SpinnerSize;

  /**
   * Spinner color
   * @default 'primary'
   */
  color?: SpinnerColor;

  /**
   * Display speed of the animation
   * @default 'normal'
   */
  speed?: 'slow' | 'normal' | 'fast';
}

/**
 * Size styles mapping
 */
const sizeStyles: Record<SpinnerSize, string> = {
  xs: 'h-3 w-3 border',
  sm: 'h-4 w-4 border-2',
  md: 'h-6 w-6 border-2',
  lg: 'h-8 w-8 border-2',
  xl: 'h-12 w-12 border-3',
};

/**
 * Animation duration mapping
 */
const speedStyles: Record<'slow' | 'normal' | 'fast', string> = {
  slow: 'animate-spin-slow',
  normal: 'animate-spin',
  fast: 'animate-spin-fast',
};

/**
 * Color mapping for border styles
 */
const colorStyles: Record<SpinnerColor, string> = {
  primary: 'border-blue-600 border-t-transparent',
  secondary: 'border-gray-600 border-t-transparent',
  white: 'border-white border-t-transparent',
  currentColor: 'border-current border-t-transparent opacity-75',
};

/**
 * Spinner component - Loading spinner indicator
 *
 * A loading spinner component with configurable size and color.
 * Used to indicate loading states throughout the application.
 *
 * @example
 * ```tsx
 * <Spinner size="md" color="primary" />
 * <Spinner size="lg" color="white" />
 * <Spinner size="sm" speed="fast" />
 * ```
 */
export const Spinner = React.forwardRef<HTMLDivElement, SpinnerProps>(
  ({ size = 'md', color = 'primary', speed = 'normal', className, ...props }, ref) => {
    return (
      <div
        ref={ref}
        role="status"
        aria-label="Loading"
        className={cn(
          'inline-block rounded-full',
          sizeStyles[size],
          colorStyles[color],
          speedStyles[speed],
          className
        )}
        {...props}
      >
        <span className="sr-only">Loading...</span>
      </div>
    );
  }
);

Spinner.displayName = 'Spinner';
