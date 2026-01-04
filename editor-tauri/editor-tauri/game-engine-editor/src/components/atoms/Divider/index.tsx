import React from 'react';
import { cn } from '../../../utils/cn';

/**
 * Divider orientation
 */
export type DividerOrientation = 'horizontal' | 'vertical';

/**
 * Divider component properties
 */
export interface DividerProps {
  /** Divider orientation */
  orientation?: DividerOrientation;
  /** Additional CSS classes */
  className?: string;
  /** Text or content to display in the middle */
  children?: React.ReactNode;
  /** Whether the divider has label styling (only when children is present) */
  label?: boolean;
}

/**
 * Divider component - Visual separator between content sections
 *
 * @example
 * ```tsx
 * <Divider />
 * <Divider orientation="vertical" className="h-8" />
 * <Divider>Section Title</Divider>
 * <Divider label>Important Section</Divider>
 * ```
 */
export const Divider = React.forwardRef<HTMLDivElement, DividerProps>(
  ({ orientation = 'horizontal', className, children, label = false, ...props }, ref) => {
    if (children) {
      return (
        <div
          ref={ref}
          className={cn(
            'relative flex items-center',
            orientation === 'horizontal' ? 'w-full' : 'h-full flex-col',
            className
          )}
          role="separator"
          aria-orientation={orientation}
          {...props}
        >
          <div
            className={cn(
              'border-border',
              orientation === 'horizontal' ? 'flex-grow border-t' : 'flex-grow border-l'
            )}
          />
          <span
            className={cn(
              'shrink-0 px-4 text-sm',
              label ? 'font-medium text-foreground' : 'text-muted-foreground'
            )}
          >
            {children}
          </span>
          <div
            className={cn(
              'border-border',
              orientation === 'horizontal' ? 'flex-grow border-t' : 'flex-grow border-l'
            )}
          />
        </div>
      );
    }

    return (
      <div
        ref={ref}
        className={cn(
          'border-border',
          orientation === 'horizontal' ? 'w-full border-t' : 'h-full border-l',
          className
        )}
        role="separator"
        aria-orientation={orientation}
        {...props}
      />
    );
  }
);

Divider.displayName = 'Divider';
