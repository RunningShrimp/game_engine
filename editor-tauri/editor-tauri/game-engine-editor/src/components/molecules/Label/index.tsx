import React, { LabelHTMLAttributes } from 'react';
import { cn } from '../../../utils/cn';

/**
 * Label component props
 */
export interface LabelProps extends LabelHTMLAttributes<HTMLLabelElement> {
  /**
   * Form element this label is associated with
   */
  htmlFor?: string;

  /**
   * Mark field as required
   * @default false
   */
  required?: boolean;

  /**
   * Error state
   * @default false
   */
  error?: boolean;

  /**
   * Help/description text
   */
  helpText?: string;

  /**
   * Label content
   */
  children: React.ReactNode;
}

/**
 * Label component
 *
 * A form label component with support for required indicators,
 * error states, and help text.
 *
 * @example
 * ```tsx
 * <Label htmlFor="name" required>
 *   Name
 * </Label>
 *
 * <Label htmlFor="email" error helpText="Must be a valid email">
 *   Email
 * </Label>
 * ```
 */
export const Label = React.forwardRef<HTMLLabelElement, LabelProps>(
  (
    {
      htmlFor,
      required = false,
      error = false,
      helpText,
      className,
      children,
      ...props
    },
    ref
  ) => {
    return (
      <div className="w-full">
        <label
          ref={ref}
          htmlFor={htmlFor}
          className={cn(
            'block text-sm font-medium mb-1',
            error ? 'text-red-600' : 'text-gray-700',
            'transition-colors duration-200',
            className
          )}
          {...props}
        >
          {children}
          {required && (
            <span className="text-red-500 ml-1" aria-label="required">
              *
            </span>
          )}
        </label>
        {helpText && (
          <p
            className={cn(
              'text-xs',
              error ? 'text-red-500' : 'text-gray-500',
              'mt-0.5'
            )}
          >
            {helpText}
          </p>
        )}
      </div>
    );
  }
);

Label.displayName = 'Label';
