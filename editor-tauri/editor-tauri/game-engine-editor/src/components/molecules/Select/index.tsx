import React, { SelectHTMLAttributes, useState } from 'react';
import { Icon } from '../../atoms/Icon';
import { cn } from '../../../utils/cn';

/**
 * Select size types
 */
export type SelectSize = 'sm' | 'md' | 'lg';

/**
 * Select option type
 */
export interface SelectOption {
  /**
   * Option value
   */
  value: string;

  /**
   * Option label
   */
  label: string;

  /**
   * Option is disabled
   */
  disabled?: boolean;
}

/**
 * Select component props
 */
export interface SelectProps extends Omit<SelectHTMLAttributes<HTMLSelectElement>, 'size'> {
  /**
   * Select size
   * @default 'md'
   */
  size?: SelectSize;

  /**
   * Icon name (from Lucide icons)
   */
  icon?: keyof typeof import('lucide-react');

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
   * Label for the select
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
   * Options array
   */
  options: SelectOption[];

  /**
   * Placeholder text
   */
  placeholder?: string;

  /**
   * Select value (controlled)
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
const sizeStyles: Record<SelectSize, { container: string; select: string; icon: string }> = {
  sm: {
    container: 'h-8',
    select: 'px-3 py-1.5 text-sm',
    icon: 'w-4 h-4',
  },
  md: {
    container: 'h-10',
    select: 'px-4 py-2 text-sm',
    icon: 'w-5 h-5',
  },
  lg: {
    container: 'h-12',
    select: 'px-5 py-3 text-base',
    icon: 'w-6 h-6',
  },
};

/**
 * Select component (Molecule)
 *
 * A dropdown select component composed of Icon + Select elements.
 * Supports labels, icons, error states, help text, and option groups.
 *
 * @example
 * ```tsx
 * const options = [
 *   { value: 'apple', label: 'Apple' },
 *   { value: 'banana', label: 'Banana' },
 *   { value: 'orange', label: 'Orange' },
 * ];
 *
 * <Select
 *   label="Fruit"
 *   icon="Apple"
 *   options={options}
 *   placeholder="Select a fruit"
 *   value={fruit}
 *   onChange={setFruit}
 *   required
 * />
 *
 * <Select
 *   label="Category"
 *   options={categories}
 *   error={hasError}
 *   errorMessage="Please select a category"
 * />
 * ```
 */
export const Select = React.forwardRef<HTMLSelectElement, SelectProps>(
  (
    {
      size = 'md',
      icon,
      error = false,
      errorMessage,
      helpText,
      label,
      required = false,
      fullWidth = false,
      options,
      placeholder,
      className,
      value,
      defaultValue,
      onChange,
      ...props
    },
    ref
  ) => {
    const [focused, setFocused] = useState(false);

    const handleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
      onChange?.(e.target.value);
    };

    const sizeStyle = sizeStyles[size];
    const hasIcon = !!icon;

    return (
      <div className={cn('flex flex-col', fullWidth && 'w-full')}>
        {label && (
          <label className="block text-sm font-medium text-gray-700 mb-1">
            {label}
            {required && <span className="text-red-500 ml-1">*</span>}
          </label>
        )}

        <div className={cn('relative', sizeStyle.container, fullWidth && 'w-full')}>
          {hasIcon && (
            <div
              className={cn(
                'absolute left-0 top-0 bottom-0 flex items-center justify-center pl-3',
                'pointer-events-none text-gray-400',
                focused && 'text-blue-500'
              )}
            >
              <Icon name={icon} size={size === 'sm' ? 16 : size === 'lg' ? 24 : 20} />
            </div>
          )}

          <select
            ref={ref}
            value={value}
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
            className={cn(
              // Base styles
              'block w-full border rounded-md appearance-none',
              'bg-white transition-all duration-200',
              // Focus states
              'focus:outline-none focus:ring-2 focus:ring-offset-0',
              // Error states
              error
                ? 'border-red-300 focus:border-red-500 focus:ring-red-500'
                : 'border-gray-300 focus:border-blue-500 focus:ring-blue-500',
              // Disabled states
              'disabled:bg-gray-100 disabled:cursor-not-allowed disabled:opacity-50',
              // Size styles
              sizeStyle.select,
              // Icon padding
              hasIcon && 'pl-10',
              'pr-10', // Space for chevron
              // Custom className
              className
            )}
            {...props}
          >
            {placeholder && (
              <option value="" disabled>
                {placeholder}
              </option>
            )}
            {options.map((option) => (
              <option
                key={option.value}
                value={option.value}
                disabled={option.disabled}
              >
                {option.label}
              </option>
            ))}
          </select>

          {/* Chevron icon */}
          <div className="absolute right-0 top-0 bottom-0 flex items-center justify-center pr-3 pointer-events-none text-gray-400">
            <Icon name="ChevronDown" size={size === 'sm' ? 14 : size === 'lg' ? 20 : 16} />
          </div>
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

Select.displayName = 'Select';
