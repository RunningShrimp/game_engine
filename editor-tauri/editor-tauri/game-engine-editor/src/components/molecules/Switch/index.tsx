import React, { InputHTMLAttributes, useState } from 'react';
import { Icon } from '../../atoms/Icon';
import { cn } from '../../../utils/cn';

/**
 * Switch size types
 */
export type SwitchSize = 'sm' | 'md' | 'lg';

/**
 * Switch component props
 */
export interface SwitchProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'size' | 'value' | 'onChange'> {
  /**
   * Switch size
   * @default 'md'
   */
  size?: SwitchSize;

  /**
   * Label text
   */
  label?: string;

  /**
   * Help/description text
   */
  helpText?: string;

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
   * Mark field as required
   * @default false
   */
  required?: boolean;

  /**
   * Switch value (controlled)
   */
  checked?: boolean;

  /**
   * Default checked state (uncontrolled)
   */
  defaultChecked?: boolean;

  /**
   * Change handler
   */
  onChange?: (checked: boolean) => void;
}

/**
 * Size styles mapping
 */
const sizeStyles: Record<SwitchSize, {
  container: string;
  track: string;
  thumb: string;
  thumbIcon: string;
  text: string;
}> = {
  sm: {
    container: 'h-5 w-9',
    track: 'rounded-full',
    thumb: 'w-3.5 h-3.5',
    thumbIcon: 'w-2 h-2',
    text: 'text-sm',
  },
  md: {
    container: 'h-6 w-11',
    track: 'rounded-full',
    thumb: 'w-5 h-5',
    thumbIcon: 'w-3 h-3',
    text: 'text-base',
  },
  lg: {
    container: 'h-7 w-13',
    track: 'rounded-full',
    thumb: 'w-6 h-6',
    thumbIcon: 'w-3.5 h-3.5',
    text: 'text-lg',
  },
};

/**
 * Switch component (Molecule)
 *
 * A toggle switch component with label support. Supports error states,
 * help text, and all standard input attributes.
 *
 * @example
 * ```tsx
 * <Switch
 *   label="Enable notifications"
 *   checked={enabled}
 *   onChange={setEnabled}
 *   helpText="Receive push notifications"
 * />
 *
 * <Switch
 *   label="Dark mode"
 *   defaultChecked
 * />
 *
 * <Switch
 *   label="Auto-save"
 *   error={hasError}
 *   errorMessage="Auto-save is required"
 *   required
 * />
 * ```
 */
export const Switch = React.forwardRef<HTMLInputElement, SwitchProps>(
  (
    {
      size = 'md',
      label,
      helpText,
      error = false,
      errorMessage,
      required = false,
      className,
      checked,
      defaultChecked = false,
      onChange,
      disabled,
      ...props
    },
    ref
  ) => {
    const [internalChecked, setInternalChecked] = useState(defaultChecked);

    const currentChecked = checked !== undefined ? checked : internalChecked;

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      const newChecked = e.target.checked;
      if (checked === undefined) {
        setInternalChecked(newChecked);
      }
      onChange?.(newChecked);
    };

    const sizeStyle = sizeStyles[size];

    return (
      <div className={cn('flex flex-col', disabled && 'opacity-50 cursor-not-allowed')}>
        <div className="flex items-center gap-3">
          <div className="relative inline-flex items-center">
            <input
              ref={ref}
              type="checkbox"
              checked={currentChecked}
              onChange={handleChange}
              disabled={disabled}
              className={cn(
                'peer sr-only', // Hide checkbox but keep it accessible
                className
              )}
              {...props}
            />

            {/* Track */}
            <div
              className={cn(
                // Base styles
                'block transition-all duration-200 ease-in-out cursor-pointer',
                'peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-offset-2',
                // Error states
                error
                  ? 'bg-gray-200 peer-focus:ring-red-500'
                  : 'bg-gray-200 peer-focus:ring-blue-500',
                // Checked states
                currentChecked && !error && 'bg-blue-600',
                currentChecked && error && 'bg-red-600',
                // Disabled states
                disabled && 'opacity-50 cursor-not-allowed',
                // Size styles
                sizeStyle.container,
                sizeStyle.track
              )}
            />

            {/* Thumb */}
            <div
              className={cn(
                'absolute left-0.5 top-0.5 bg-white border border-gray-300 rounded-full',
                'transition-all duration-200 ease-in-out pointer-events-none',
                'shadow-sm',
                // Checked state - move thumb to right
                currentChecked && 'translate-x-full border-white',
                // Size styles
                sizeStyle.thumb
              )}
            >
              {/* Check icon when checked */}
              {currentChecked && (
                <Icon
                  name="Check"
                  className={cn(
                    'text-blue-600 absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2',
                    sizeStyle.thumbIcon
                  )}
                  strokeWidth={3}
                />
              )}
            </div>
          </div>

          {label && (
            <div className="flex flex-col">
              <label
                className={cn(
                  'font-medium text-gray-700 cursor-pointer select-none',
                  error && currentChecked && 'text-red-600',
                  disabled && 'cursor-not-allowed',
                  sizeStyle.text
                )}
              >
                {label}
                {required && <span className="text-red-500 ml-1">*</span>}
              </label>
              {helpText && !error && (
                <p className="text-sm text-gray-500 mt-0.5">{helpText}</p>
              )}
              {error && errorMessage && (
                <p className="text-sm text-red-600 mt-0.5">{errorMessage}</p>
              )}
            </div>
          )}
        </div>

        {!label && error && errorMessage && (
          <p className="text-sm text-red-600 mt-1">{errorMessage}</p>
        )}

        {!label && helpText && !error && (
          <p className="text-sm text-gray-500 mt-1">{helpText}</p>
        )}
      </div>
    );
  }
);

Switch.displayName = 'Switch';
