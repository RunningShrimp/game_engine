import React, { InputHTMLAttributes, useState } from 'react';
import { Icon } from '../../atoms/Icon';
import { cn } from '../../../utils/cn';

/**
 * Input size types
 */
export type InputSize = 'sm' | 'md' | 'lg';

/**
 * Input component props
 */
export interface InputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'size'> {
  /**
   * Input size
   * @default 'md'
   */
  size?: InputSize;

  /**
   * Icon name (from Lucide icons) to display inside input
   */
  icon?: keyof typeof import('lucide-react');

  /**
   * Icon position
   * @default 'left'
   */
  iconPosition?: 'left' | 'right';

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
   * Label for the input
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
   * Input value (controlled)
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
const sizeStyles: Record<InputSize, { container: string; input: string; icon: string }> = {
  sm: {
    container: 'h-8',
    input: 'px-3 py-1.5 text-sm',
    icon: 'w-4 h-4',
  },
  md: {
    container: 'h-10',
    input: 'px-4 py-2 text-sm',
    icon: 'w-5 h-5',
  },
  lg: {
    container: 'h-12',
    input: 'px-5 py-3 text-base',
    icon: 'w-6 h-6',
  },
};

/**
 * Input component (Molecule)
 *
 * A text input component composed of Icon + Text input atoms.
 * Supports labels, icons, error states, help text, and all standard input attributes.
 *
 * @example
 * ```tsx
 * <Input
 *   label="Email"
 *   icon="Mail"
 *   placeholder="Enter your email"
 *   value={email}
 *   onChange={setEmail}
 *   required
 * />
 *
 * <Input
 *   label="Password"
 *   icon="Lock"
 *   type="password"
 *   error={hasError}
 *   errorMessage="Password is required"
 * />
 * ```
 */
export const Input = React.forwardRef<HTMLInputElement, InputProps>(
  (
    {
      size = 'md',
      icon,
      iconPosition = 'left',
      error = false,
      errorMessage,
      helpText,
      label,
      required = false,
      fullWidth = false,
      className,
      value,
      defaultValue,
      onChange,
      ...props
    },
    ref
  ) => {
    const [focused, setFocused] = useState(false);
    const [internalValue, setInternalValue] = useState(defaultValue || '');

    const currentValue = value !== undefined ? value : internalValue;
    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      const newValue = e.target.value;
      if (value === undefined) {
        setInternalValue(newValue);
      }
      onChange?.(newValue);
    };

    const sizeStyle = sizeStyles[size];
    const hasIcon = !!icon;
    const showIconLeft = hasIcon && iconPosition === 'left';
    const showIconRight = hasIcon && iconPosition === 'right';

    return (
      <div className={cn('flex flex-col', fullWidth && 'w-full')}>
        {label && (
          <label className="block text-sm font-medium text-gray-700 mb-1">
            {label}
            {required && <span className="text-red-500 ml-1">*</span>}
          </label>
        )}

        <div className={cn('relative', sizeStyle.container, fullWidth && 'w-full')}>
          {showIconLeft && (
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

          <input
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
            className={cn(
              // Base styles
              'block w-full border rounded-md transition-all duration-200',
              // Focus states
              'focus:outline-none focus:ring-2 focus:ring-offset-0',
              // Error states
              error
                ? 'border-red-300 focus:border-red-500 focus:ring-red-500'
                : 'border-gray-300 focus:border-blue-500 focus:ring-blue-500',
              // Disabled states
              'disabled:bg-gray-100 disabled:cursor-not-allowed disabled:opacity-50',
              // Size styles
              sizeStyle.input,
              // Icon padding
              showIconLeft && 'pl-10',
              showIconRight && 'pr-10',
              // Custom className
              className
            )}
            {...props}
          />

          {showIconRight && (
            <div
              className={cn(
                'absolute right-0 top-0 bottom-0 flex items-center justify-center pr-3',
                'pointer-events-none text-gray-400',
                focused && 'text-blue-500'
              )}
            >
              <Icon name={icon} size={size === 'sm' ? 16 : size === 'lg' ? 24 : 20} />
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

Input.displayName = 'Input';
