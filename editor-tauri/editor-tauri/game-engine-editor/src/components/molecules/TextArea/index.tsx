import React, { TextareaHTMLAttributes, useState } from 'react';
import { cn } from '../../../utils/cn';

/**
 * TextArea size types
 */
export type TextAreaSize = 'sm' | 'md' | 'lg';

/**
 * TextArea component props
 */
export interface TextAreaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  /**
   * TextArea size
   * @default 'md'
   */
  size?: TextAreaSize;

  /**
   * Error state
   * @default false
   */
  error?: boolean;

  /**
   * Error message to display
   */
  errorMessage?: string;

  /**
   * Help/description text
   */
  helpText?: string;

  /**
   * Label for the textarea
   */
  label?: string;

  /**
   * Mark field as required
   * @default false
   */
  required?: boolean;

  /**
   * Full width
   * @default false
   */
  fullWidth?: boolean;

  /**
   * Character count (max length)
   */
  maxLength?: number;

  /**
   * Show character count
   * @default false
   */
  showCount?: boolean;

  /**
   * Minimum rows
   * @default 3
   */
  minRows?: number;

  /**
   * Maximum rows (before scrolling)
   */
  maxRows?: number;

  /**
   * TextArea value (controlled)
   */
  value?: string;

  /**
   * Default value (uncontrolled)
   */
  defaultValue?: string;

  /**
   * Change handler
   */
  onChange?: (value: string) => void;
}

/**
 * Size styles mapping
 */
const sizeStyles: Record<TextAreaSize, string> = {
  sm: 'px-3 py-1.5 text-sm',
  md: 'px-4 py-2 text-sm',
  lg: 'px-5 py-3 text-base',
};

/**
 * TextArea component (Molecule)
 *
 * A multi-line text input component. Supports labels, error states,
 * help text, character counting, and auto-resize.
 *
 * @example
 * ```tsx
 * <TextArea
 *   label="Description"
 *   placeholder="Enter a description"
 *   value={description}
 *   onChange={setDescription}
 *   rows={4}
 *   maxLength={500}
 *   showCount
 * />
 *
 * <TextArea
 *   label="Comments"
 *   error={hasError}
 *   errorMessage="Comments are required"
 *   required
 * />
 * ```
 */
export const TextArea = React.forwardRef<HTMLTextAreaElement, TextAreaProps>(
  (
    {
      size = 'md',
      error = false,
      errorMessage,
      helpText,
      label,
      required = false,
      fullWidth = false,
      maxLength,
      showCount = false,
      minRows = 3,
      maxRows,
      className,
      value,
      defaultValue,
      onChange,
      rows = minRows,
      ...props
    },
    ref
  ) => {
    const [focused, setFocused] = useState(false);
    const [internalValue, setInternalValue] = useState(defaultValue || '');

    const currentValue = value !== undefined ? value : internalValue;
    const charCount = currentValue.length;
    const remainingChars = maxLength ? maxLength - charCount : 0;

    const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      const newValue = e.target.value;
      if (value === undefined) {
        setInternalValue(newValue);
      }
      onChange?.(newValue);
    };

    return (
      <div className={cn('flex flex-col', fullWidth && 'w-full')}>
        {label && (
          <label className="block text-sm font-medium text-gray-700 mb-1">
            {label}
            {required && <span className="text-red-500 ml-1">*</span>}
          </label>
        )}

        <div className="relative">
          <textarea
            ref={ref}
            value={currentValue}
            defaultValue={defaultValue}
            onChange={handleChange}
            onFocus={(e) => {
              setFocused(true);
              props.onFocus?.(e);
            }}
            onBlur={(e) => {
              setFocused(false);
              props.onBlur?.(e);
            }}
            maxLength={maxLength}
            rows={rows}
            className={cn(
              // Base styles
              'block w-full border rounded-md resize-y transition-all duration-200',
              // Focus states
              'focus:outline-none focus:ring-2 focus:ring-offset-0',
              // Error states
              error
                ? 'border-red-300 focus:border-red-500 focus:ring-red-500'
                : 'border-gray-300 focus:border-blue-500 focus:ring-blue-500',
              // Disabled states
              'disabled:bg-gray-100 disabled:cursor-not-allowed disabled:opacity-50',
              // Size styles
              sizeStyles[size],
              // Custom className
              className
            )}
            {...props}
          />

          {showCount && maxLength && (
            <div
              className={cn(
                'absolute bottom-2 right-2 text-xs',
                remainingChars < 0 ? 'text-red-500' : 'text-gray-400'
              )}
            >
              {charCount}/{maxLength}
            </div>
          )}
        </div>

        {error && errorMessage && (
          <p className="mt-1 text-sm text-red-600">{errorMessage}</p>
        )}

        {helpText && !error && (
          <p className="mt-1 text-sm text-gray-500">{helpText}</p>
        )}
      </div>
    );
  }
);

TextArea.displayName = 'TextArea';
