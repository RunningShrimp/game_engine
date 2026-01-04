import React from 'react';
import { cn } from '../../utils/cn';

/**
 * Badge variant types
 */
export type BadgeVariant = 'default' | 'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'info' | 'outline';

/**
 * Badge size types
 */
export type BadgeSize = 'sm' | 'md' | 'lg';

/**
 * Badge component properties
 */
export interface BadgeProps {
  /** Badge content */
  children: React.ReactNode;
  /** Badge variant */
  variant?: BadgeVariant;
  /** Badge size */
  size?: BadgeSize;
  /** Additional CSS classes */
  className?: string;
  /** Icon to display */
  icon?: React.ReactNode;
  /** Whether to show a dot indicator */
  dot?: boolean;
  /** Accessibility label */
  label?: string;
}

/**
 * Badge component - Displays status indicators, tags, or labels
 *
 * @example
 * ```tsx
 * <Badge variant="success">Active</Badge>
 * <Badge variant="error" size="sm">Error</Badge>
 * <Badge variant="primary" icon={<Icon name="Star" />}>Featured</Badge>
 * <Badge variant="info" dot>New</Badge>
 * ```
 */
export const Badge = React.forwardRef<HTMLSpanElement, BadgeProps>(
  ({
    children,
    variant = 'default',
    size = 'md',
    className,
    icon,
    dot = false,
    label,
    ...props
  }, ref) => {
    const variantClasses: Record<BadgeVariant, string> = {
      default: 'bg-muted text-muted-foreground hover:bg-muted/80',
      primary: 'bg-primary text-primary-foreground hover:bg-primary/90',
      secondary: 'bg-secondary text-secondary-foreground hover:bg-secondary/80',
      success: 'bg-success text-success-foreground hover:bg-success/90',
      warning: 'bg-warning text-warning-foreground hover:bg-warning/90',
      error: 'bg-error text-error-foreground hover:bg-error/90',
      info: 'bg-info text-info-foreground hover:bg-info/90',
      outline: 'border border-border bg-transparent hover:bg-muted',
    };

    const sizeClasses: Record<BadgeSize, string> = {
      sm: 'px-2 py-0.5 text-xs',
      md: 'px-2.5 py-1 text-sm',
      lg: 'px-3 py-1.5 text-base',
    };

    return (
      <span
        ref={ref}
        className={cn(
          'inline-flex items-center gap-1.5 rounded-full font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2',
          variantClasses[variant],
          sizeClasses[size],
          className
        )}
        role="status"
        aria-label={label}
        {...props}
      >
        {dot && (
          <span
            className={cn(
              'h-2 w-2 rounded-full',
              variant === 'success' && 'bg-success-foreground',
              variant === 'error' && 'bg-error-foreground',
              variant === 'warning' && 'bg-warning-foreground',
              variant === 'info' && 'bg-info-foreground',
              variant === 'primary' && 'bg-primary-foreground',
              variant === 'default' && 'bg-foreground'
            )}
            aria-hidden="true"
          />
        )}
        {icon && <span className="flex-shrink-0">{icon}</span>}
        {children}
      </span>
    );
  }
);

Badge.displayName = 'Badge';
