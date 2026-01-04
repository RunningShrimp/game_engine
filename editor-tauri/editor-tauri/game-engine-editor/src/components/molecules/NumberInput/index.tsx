import React, { InputHTMLAttributes, useState } from 'react';
import { Icon } from '../../atoms/Icon';
import { cn } from '../../../utils/cn';

/**
 * NumberInput size types
 */
export type NumberInputSize = 'sm' | 'md' | 'lg';

/**
 * NumberInput component props
 */
export interface NumberInputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'size' | 'type'> {
  /**
   * Input size
   * @default 'md'
   */
  size?: NumberInputSize;

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
   * Show increment/decrement buttons
   * @default true
   */
  showControls?: boolean;

  /**
   * Minimum value
   */
  min?: number;

  /**
   * Maximum value
   */
  max?: number;

  /**
   * Step value for increment/decrement
   * @default 1
   */
  step?: number;

  /**
   * Precision (decimal places)
   */
  precision?: number;

  /**
   * Input value (controlled)
   */
  value?: number;

  /**
   * Default value (uncontrolled)
   */
  defaultValue?: number;

  /**
   * Change handler
   */
  onChange?: (value: number) => void;

  /**
   * Format value for display
   */
  formatValue?: (value: number) => string;

  /**
   * Parse value from input
   */
  parseValue?: (value: string) => number;
}

/**
 * Size styles mapping
 */
const sizeStyles: Record<NumberInputSize, { container: string; input: string; button: string }> = {
  sm: {
    container: 'h-8',
    input: 'px-2 py-1.5 text-sm',
    button: 'w-6',
  },
  md: {
    container: 'h-10',
    input: 'px-3 py-2 text-sm',
    button: 'w-8',
  },
  lg: {
    container: 'h-12',
    input: 'px-4 py-3 text-base',
    button: 'w-10',
  },
};

/**
 * NumberInput component (Molecule)
 *
 * A number input component with increment/decrement controls.
 * Supports min/max constraints, precision, and custom formatting.
 *
 * @example
 * ```tsx
 * <NumberInput
 *   label="Quantity"
 *   value={quantity}
 *   onChange={setQuantity}
 *   min={0}
 *   max={100}
 *   step={1}
 * />
 *
 * <NumberInput
 *   label="Price"
 *   value={price}
 *   onChange={setPrice}
 *   min={0}
 *   step={0.01}
 *   precision={2}
 *   formatValue={(v) => `\$${v.toFixed(2)}`}
 * />
 *
 * <NumberInput
 *   label="Rating"
 *   error={hasError}
 *   errorMessage="Rating must be between 1 and 5"
 *   min={1}
 *   max={5}
 *   required
 * />
 * ```
 */
export const NumberInput = React.forwardRef<HTMLInputElement, NumberInputProps>(
  (
    {
      size = 'md',
      error = false,
      errorMessage,
      helpText,
      label,
      required = false,
      fullWidth = false,
      showControls = true,
      min,
      max,
      step = 1,
      precision,
      className,
      value,
      defaultValue = 0,
      onChange,
      formatValue,
      parseValue,
      disabled,
      ...props
    },
    ref
  ) => {
    const [focused, setFocused] = useState(false);
    const [internalValue, setInternalValue] = useState(defaultValue);
    const [inputValue, setInputValue] = useState(() => {
      const val = value !== undefined ? value : defaultValue;
      return formatValue ? formatValue(val) : val.toString();
    });

    const currentValue = value !== undefined ? value : internalValue;

    // Update input value when controlled value changes
    React.useEffect(() => {
      if (value !== undefined) {
        setInputValue(formatValue ? formatValue(value) : value.toString());
      }
    }, [value, formatValue]);

    const clampValue = (val: number): number => {
      let clamped = val;
      if (min !== undefined) clamped = Math.max(min, clamped);
      if (max !== undefined) clamped = Math.min(max, clamped);
      if (precision !== undefined) clamped = Number(clamped.toFixed(precision));
      return clamped;
    };

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      const textValue = e.target.value;
      setInputValue(textValue);

      const numValue = parseValue ? parseValue(textValue) : parseFloat(textValue);

      if (!isNaN(numValue)) {
        const clampedValue = clampValue(numValue);
        if (value === undefined) {
          setInternalValue(clampedValue);
        }
        onChange?.(clampedValue);
      }
    };

    const handleBlur = (e: React.FocusEvent<HTMLInputElement>) => {
      const numValue = parseFloat(inputValue);
      if (!isNaN(numValue)) {
        const clampedValue = clampValue(numValue);
        const formattedValue = formatValue ? formatValue(clampedValue) : clampedValue.toString();
        setInputValue(formattedValue);
        if (value === undefined) {
          setInternalValue(clampedValue);
        }
        onChange?.(clampedValue);
      }
      setFocused(false);
      props.onBlur?.(e);
    };

    const increment = () => {
      const newValue = clampValue(currentValue + step);
      if (value === undefined) {
        setInternalValue(newValue);
      }
      const formattedValue = formatValue ? formatValue(newValue) : newValue.toString();
      setInputValue(formattedValue);
      onChange?.(newValue);
    };

    const decrement = () => {
      const newValue = clampValue(currentValue - step);
      if (value === undefined) {
        setInternalValue(newValue);
      }
      const formattedValue = formatValue ? formatValue(newValue) : newValue.toString();
      setInputValue(formattedValue);
      onChange?.(newValue);
    };

    const canIncrement = max === undefined || currentValue < max;
    const canDecrement = min === undefined || currentValue > min;

    const sizeStyle = sizeStyles[size];

    return (
      <div className={cn('flex flex-col', fullWidth && 'w-full')}>
        {label && (
          <label className="block text-sm font-medium text-gray-700 mb-1">
            {label}
            {required && <span className="text-red-500 ml-1">*</span>}
          </label>
        )}

        <div className={cn('relative flex', sizeStyle.container, fullWidth && 'w-full')}>
          {/* Decrement button */}
          {showControls && (
            <button
              type="button"
              onClick={decrement}
              disabled={disabled || !canDecrement}
              className={cn(
                'flex items-center justify-center border border-r-0 rounded-l-md',
                'bg-gray-50 hover:bg-gray-100',
                'focus:outline-none focus:ring-2 focus:ring-blue-500',
                'disabled:bg-gray-100 disabled:cursor-not-allowed disabled:opacity-50',
                'transition-colors duration-150',
                sizeStyle.button
              )}
              aria-label="Decrement value"
            >
              <Icon name="Minus" size={size === 'sm' ? 12 : size === 'lg' ? 18 : 14} />
            </button>
          )}

          {/* Input */}
          <input
            ref={ref}
            type="text"
            inputMode="numeric"
            value={inputValue}
            onChange={handleChange}
            onFocus={(e) => {
              setFocused(true);
              props.onFocus?.(e);
            }}
            onBlur={handleBlur}
            disabled={disabled}
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
              // Control buttons
              showControls && 'rounded-none border-l-0 border-r-0',
              !showControls && 'rounded-md',
              // Text alignment
              'text-center',
              // Custom className
              className
            )}
            {...props}
          />

          {/* Increment button */}
          {showControls && (
            <button
              type="button"
              onClick={increment}
              disabled={disabled || !canIncrement}
              className={cn(
                'flex items-center justify-center border border-l-0 rounded-r-md',
                'bg-gray-50 hover:bg-gray-100',
                'focus:outline-none focus:ring-2 focus:ring-blue-500',
                'disabled:bg-gray-100 disabled:cursor-not-allowed disabled:opacity-50',
                'transition-colors duration-150',
                sizeStyle.button
              )}
              aria-label="Increment value"
            >
              <Icon name="Plus" size={size === 'sm' ? 12 : size === 'lg' ? 18 : 14} />
            </button>
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

NumberInput.displayName = 'NumberInput';
