import React from 'react';
import { cn } from '../../utils/cn';

/**
 * Text variant types
 */
export type TextVariant = 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6' | 'p' | 'span' | 'label' | 'small' | 'code';

/**
 * Text component properties
 */
export interface TextProps {
  /** Text variant/element type */
  variant?: TextVariant;
  /** Text content */
  children: React.ReactNode;
  /** Additional CSS classes */
  className?: string;
  /** Text alignment */
  align?: 'left' | 'center' | 'right' | 'justify';
  /** Text color variant */
  color?: 'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'info' | 'default';
  /** Font weight */
  weight?: 'normal' | 'medium' | 'semibold' | 'bold';
  /** Whether text should truncate with ellipsis */
  truncate?: boolean;
  /** Number of lines to show (requires -webkit-line-clamp) */
  lines?: number;
  /** Whether to inherit parent font size */
  inherit?: boolean;
}

/**
 * Text component - Provides consistent typography throughout the application
 *
 * @example
 * ```tsx
 * <Text variant="h1">Heading 1</Text>
 * <Text variant="p">Paragraph text</Text>
 * <Text variant="label" color="primary">Label</Text>
 * <Text variant="code" weight="medium">Code snippet</Text>
 * <Text truncate lines={2}>Long text that will be truncated</Text>
 * ```
 */
export const Text = React.forwardRef<HTMLElement, TextProps>(
  ({
    variant = 'p',
    children,
    className,
    align = 'left',
    color = 'default',
    weight = 'normal',
    truncate = false,
    lines,
    inherit = false,
    ...props
  }, ref) => {
    const colorClasses = {
      primary: 'text-primary',
      secondary: 'text-secondary',
      success: 'text-success',
      warning: 'text-warning',
      error: 'text-error',
      info: 'text-info',
      default: 'text-foreground',
    };

    const weightClasses = {
      normal: 'font-normal',
      medium: 'font-medium',
      semibold: 'font-semibold',
      bold: 'font-bold',
    };

    const alignClasses = {
      left: 'text-left',
      center: 'text-center',
      right: 'text-right',
      justify: 'text-justify',
    };

    const variantStyles: Record<TextVariant, string> = {
      h1: 'text-4xl font-bold',
      h2: 'text-3xl font-bold',
      h3: 'text-2xl font-semibold',
      h4: 'text-xl font-semibold',
      h5: 'text-lg font-semibold',
      h6: 'text-base font-semibold',
      p: 'text-base leading-relaxed',
      span: 'text-base',
      label: 'text-sm font-medium',
      small: 'text-xs',
      code: 'font-mono text-sm bg-muted px-1.5 py-0.5 rounded',
    };

    const truncateClasses = truncate
      ? lines
        ? `display: -webkit-box; -webkit-line-clamp: ${lines}; -webkit-box-orient: vertical; overflow: hidden;`
        : 'truncate'
      : '';

    const Component = variant;

    return (
      <Component
        ref={ref as any}
        className={cn(
          variantStyles[variant],
          colorClasses[color],
          weightClasses[weight],
          alignClasses[align],
          !inherit && 'text-foreground',
          className
        )}
        style={lines && truncate ? { display: '-webkit-box', WebkitLineClamp: lines, WebkitBoxOrient: 'vertical', overflow: 'hidden' } : undefined}
        {...props}
      >
        {children}
      </Component>
    );
  }
);

Text.displayName = 'Text';
