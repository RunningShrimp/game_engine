import React from 'react';
import { cn } from '../../lib/utils';

export interface PropertyValueProps {
  /** Value to display */
  children: React.ReactNode;
  /** Value variant */
  variant?: 'default' | 'muted' | 'accent' | 'success' | 'warning' | 'error';
  /** Additional CSS classes */
  className?: string;
  /** Truncate text */
  truncate?: boolean;
}

/**
 * PropertyValue - Display-only value component
 *
 * @example
 * ```tsx
 * <PropertyValue variant="muted">No value</PropertyValue>
 * <PropertyValue truncate>{longString}</PropertyValue>
 * ```
 */
export const PropertyValue = React.memo<PropertyValueProps>(({
  children,
  variant = 'default',
  className,
  truncate = false,
}) => {
  return (
    <div
      className={cn(
        'text-sm',
        {
          'text-slate-200': variant === 'default',
          'text-slate-500': variant === 'muted',
          'text-blue-400': variant === 'accent',
          'text-green-400': variant === 'success',
          'text-yellow-400': variant === 'warning',
          'text-red-400': variant === 'error',
        },
        truncate && 'truncate',
        className
      )}
    >
      {children}
    </div>
  );
});

PropertyValue.displayName = 'PropertyValue';
