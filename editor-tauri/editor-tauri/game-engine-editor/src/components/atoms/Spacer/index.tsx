import React from 'react';
import { cn } from '../../utils/cn';

/**
 * Spacer axis
 */
export type SpacerAxis = 'horizontal' | 'vertical';

/**
 * Spacer size presets
 */
export type SpacerSize = 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl' | 'custom';

/**
 * Spacer component properties
 */
export interface SpacerProps {
  /** Spacer axis */
  axis?: SpacerAxis;
  /** Spacer size preset */
  size?: SpacerSize;
  /** Custom size in pixels (only used when size is 'custom') */
  value?: number;
  /** Whether spacer can grow to fill available space */
  grow?: boolean;
  /** Additional CSS classes */
  className?: string;
}

/**
 * Spacer component - Creates consistent spacing between elements
 *
 * @example
 * ```tsx
 * <Spacer size="md" />
 * <Spacer axis="vertical" size="lg" />
 * <Spacer grow /> // Flex grow spacer
 * <Spacer size="custom" value={32} />
 * ```
 */
export const Spacer = React.forwardRef<HTMLDivElement, SpacerProps>(
  ({ axis = 'horizontal', size = 'md', value, grow = false, className, ...props }, ref) => {
    const sizeClasses: Record<SpacerSize, string> = {
      xs: 'size-1',
      sm: 'size-2',
      md: 'size-4',
      lg: 'size-6',
      xl: 'size-8',
      '2xl': 'size-12',
      custom: '',
    };

    const axisClasses = axis === 'horizontal' ? 'w-full h-0' : 'h-full w-0';

    const customStyle = size === 'custom' && value ? (
      axis === 'horizontal' ? { width: `${value}px` } : { height: `${value}px` }
    ) : undefined;

    return (
      <div
        ref={ref}
        className={cn(
          'shrink-0',
          !grow && sizeClasses[size],
          grow && 'flex-grow',
          !grow && axisClasses,
          className
        )}
        style={customStyle}
        aria-hidden="true"
        {...props}
      />
    );
  }
);

Spacer.displayName = 'Spacer';
