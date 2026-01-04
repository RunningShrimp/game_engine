import React, { RadioHTMLAttributes, useState } from 'react';
import { Icon } from '../../atoms/Icon';
import { cn } from '../../../utils/cn';

/**
 * Radio size types
 */
export type RadioSize = 'sm' | 'md' | 'lg';

/**
 * Radio option type
 */
export interface RadioOption {
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

  /**
   * Help text for this option
   */
  helpText?: string;
}

/**
 * Radio component props
 */
export interface RadioProps extends Omit<RadioHTMLAttributes<HTMLInputElement>, 'size' | 'value' | 'onChange'> {
  /**
   * Radio size
   * @default 'md'
   */
  size?: RadioSize;

  /**
   * Radio options
   */
  options: RadioOption[];

  /**
   * Layout orientation
   * @default 'vertical'
   */
  orientation?: 'vertical' | 'horizontal';

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
   * Group label
   */
  label?: string;

  /**
   * Group help text
   */
  helpText?: string;

  /**
   * Mark field as required
   * @default false
   */
  required?: boolean;

  /**
   * Radio value (controlled)
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
const sizeStyles: Record<RadioSize, { container: string; circle: string; inner: string; text: string }> = {
  sm: {
    container: 'h-4 w-4',
    circle: 'rounded-full',
    inner: 'w-2 h-2',
    text: 'text-sm',
  },
  md: {
    container: 'h-5 w-5',
    circle: 'rounded-full',
    inner: 'w-2.5 h-2.5',
    text: 'text-base',
  },
  lg: {
    container: 'h-6 w-6',
    circle: 'rounded-full',
    inner: 'w-3 h-3',
    text: 'text-lg',
  },
};

/**
 * Radio component (Molecule)
 *
 * A radio group component with support for multiple options, labels,
 * error states, and help text.
 *
 * @example
 * ```tsx
 * const options = [
 *   { value: 'apple', label: 'Apple' },
 *   { value: 'banana', label: 'Banana' },
 *   { value: 'orange', label: 'Orange' },
 * ];
 *
 * <Radio
 *   label="Select a fruit"
 *   options={options}
 *   value={fruit}
 *   onChange={setFruit}
 *   required
 * />
 *
 * <Radio
 *   label="Notification preference"
 *   options={notificationOptions}
 *   orientation="horizontal"
 *   helpText="Choose how you want to be notified"
 * />
 * ```
 */
export const Radio = React.forwardRef<HTMLDivElement, RadioProps>(
  (
    {
      size = 'md',
      options,
      orientation = 'vertical',
      error = false,
      errorMessage,
      label,
      helpText,
      required = false,
      className,
      value,
      defaultValue,
      onChange,
      name,
      ...props
    },
    ref
  ) => {
    const [internalValue, setInternalValue] = useState(defaultValue || '');
    const radioGroupName = name || `radio-${Math.random().toString(36).substr(2, 9)}`;

    const currentValue = value !== undefined ? value : internalValue;

    const handleChange = (newValue: string) => {
      if (value === undefined) {
        setInternalValue(newValue);
      }
      onChange?.(newValue);
    };

    const sizeStyle = sizeStyles[size];

    return (
      <div ref={ref} className={cn('flex flex-col', className)}>
        {label && (
          <label className="block text-sm font-medium text-gray-700 mb-2">
            {label}
            {required && <span className="text-red-500 ml-1">*</span>}
          </label>
        )}

        <div
          className={cn(
            'flex gap-4',
            orientation === 'vertical' ? 'flex-col' : 'flex-row flex-wrap'
          )}
        >
          {options.map((option) => {
            const isChecked = currentValue === option.value;
            const isDisabled = option.disabled || props.disabled;

            return (
              <div
                key={option.value}
                className={cn(
                  'flex items-start gap-3',
                  isDisabled && 'opacity-50 cursor-not-allowed'
                )}
              >
                <div className="relative flex items-center pt-0.5">
                  <input
                    type="radio"
                    name={radioGroupName}
                    value={option.value}
                    checked={isChecked}
                    disabled={isDisabled}
                    onChange={() => handleChange(option.value)}
                    className={cn(
                      // Base styles
                      'border appearance-none cursor-pointer transition-all duration-200',
                      // Focus states
                      'focus:outline-none focus:ring-2 focus:ring-offset-0',
                      // Error states
                      error
                        ? 'border-red-300 focus:border-red-500 focus:ring-red-500'
                        : 'border-gray-300 focus:border-blue-500 focus:ring-blue-500',
                      // Checked states
                      isChecked && 'border-blue-600',
                      // Disabled states
                      'disabled:bg-gray-100 disabled:cursor-not-allowed',
                      // Size styles
                      sizeStyle.container,
                      sizeStyle.circle
                    )}
                    {...props}
                  />

                  {/* Inner circle for checked state */}
                  {isChecked && (
                    <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
                      <div
                        className={cn(
                          'bg-blue-600 rounded-full',
                          sizeStyle.inner
                        )}
                      />
                    </div>
                  )}
                </div>

                <div className="flex flex-col">
                  <label
                    className={cn(
                      'font-medium text-gray-700 cursor-pointer select-none',
                      error && isChecked && 'text-red-600',
                      isDisabled && 'cursor-not-allowed',
                      sizeStyle.text
                    )}
                  >
                    {option.label}
                  </label>
                  {option.helpText && !error && (
                    <p className="text-sm text-gray-500 mt-0.5">{option.helpText}</p>
                  )}
                </div>
              </div>
            );
          })}
        </div>

        {error && errorMessage && (
          <p className="mt-2 text-sm text-red-600">{errorMessage}</p>
        )}

        {helpText && !error && (
          <p className="mt-2 text-sm text-gray-500">{helpText}</p>
        )}
      </div>
    );
  }
);

Radio.displayName = 'Radio';
