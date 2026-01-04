import React from 'react';
import { cn } from '../../utils/cn';

/**
 * ProgressBar size types
 */
export type ProgressBarSize = 'sm' | 'md' | 'lg';

/**
 * ProgressBar variant types
 */
export type ProgressBarVariant = 'default' | 'primary' | 'success' | 'warning' | 'error';

/**
 * ProgressBar component properties
 */
export interface ProgressBarProps {
  /** Progress value (0-100) */
  value: number;
  /** Progress bar size */
  size?: ProgressBarSize;
  /** Progress bar variant */
  variant?: ProgressBarVariant;
  /** Additional CSS classes */
  className?: string;
  /** Whether to show percentage label */
  showLabel?: boolean;
  /** Custom label */
  label?: string;
  /** Whether progress bar is striped */
  striped?: boolean;
  /** Whether stripes are animated */
  animated?: boolean;
  /** Accessibility label */
  ariaLabel?: string;
  /** Maximum value (default 100) */
  max?: number;
}

/**
 * ProgressBar component - Displays progress of an operation
 *
 * @example
 * ```tsx
 * <ProgressBar value={50} />
 * <ProgressBar value={75} variant="success" showLabel />
 * <ProgressBar value={30} variant="warning" striped animated />
 * <ProgressBar value={90} size="lg" label="Processing..." />
 * ```
 */
export const ProgressBar = React.forwardRef<HTMLDivElement, ProgressBarProps>(
  ({
    value,
    size = 'md',
    variant = 'default',
    className,
    showLabel = false,
    label,
    striped = false,
    animated = false,
    ariaLabel = 'Progress',
    max = 100,
    ...props
  }, ref) => {
    const clampedValue = Math.min(100, Math.max(0, (value / max) * 100));
    const displayValue = Math.round(clampedValue);

    const sizeClasses: Record<ProgressBarSize, string> = {
      sm: 'h-1',
      md: 'h-2',
      lg: 'h-3',
    };

    const variantClasses: Record<ProgressBarVariant, string> = {
      default: 'bg-primary',
      primary: 'bg-primary',
      success: 'bg-success',
      warning: 'bg-warning',
      error: 'bg-error',
    };

    return (
      <div ref={ref} className={cn('w-full', className)} {...props}>
        <div className="relative w-full bg-muted rounded-full overflow-hidden">
          <div
            role="progressbar"
            aria-valuenow={value}
            aria-valuemin={0}
            aria-valuemax={max}
            aria-label={ariaLabel}
            className={cn(
              'h-full rounded-full transition-all duration-300 ease-out',
              sizeClasses[size],
              variantClasses[variant],
              striped && 'bg-[linear-gradient(45deg,rgba(255,255,255,.15)_25%,transparent_25%,transparent_50%,rgba(255,255,255,.15)_50%,rgba(255,255,255,.15)_75%,transparent_75%,transparent)] bg-[length:1rem_1rem]',
              animated && 'animate-[progress-stripes_1s_linear_infinite]'
            )}
            style={{ width: `${clampedValue}%` }}
          />
        </div>
        {(showLabel || label) && (
          <div className="mt-1 flex justify-between text-xs text-muted-foreground">
            <span>{label || ariaLabel}</span>
            <span>{displayValue}%</span>
          </div>
        )}
      </div>
    );
  }
);

ProgressBar.displayName = 'ProgressBar';
