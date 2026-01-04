import React, { InputHTMLAttributes, useState } from 'react';
import { cn } from '../../../utils/cn';

/**
 * Slider size types
 */
export type SliderSize = 'sm' | 'md' | 'lg';

/**
 * Slider component props
 */
export interface SliderProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'size' | 'value' | 'onChange'> {
  /**
   * Slider size
   * @default 'md'
   */
  size?: SliderSize;

  /**
   * Label for the slider
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
   * Minimum value
   * @default 0
   */
  min?: number;

  /**
   * Maximum value
   * @default 100
   */
  max?: number;

  /**
   * Step value
   * @default 1
   */
  step?: number;

  /**
   * Show value tooltip
   * @default true
   */
  showValue?: boolean;

  /**
   * Value label format
   */
  valueFormat?: (value: number) => string;

  /**
   * Show marks at intervals
   */
  marks?: number[];

  /**
   * Full width
   * @default false
   */
  fullWidth?: boolean;

  /**
   * Slider value (controlled)
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
   * Change commit handler (on mouse up)
   */
  onChangeCommit?: (value: number) => void;
}

/**
 * Size styles mapping
 */
const sizeStyles: Record<SliderSize, { height: string; thumb: string }> = {
  sm: {
    height: 'h-1.5',
    thumb: 'w-3 h-3 -mt-1',
  },
  md: {
    height: 'h-2',
    thumb: 'w-4 h-4 -mt-1.5',
  },
  lg: {
    height: 'h-2.5',
    thumb: 'w-5 h-5 -mt-2',
  },
};

/**
 * Slider component (Molecule)
 *
 * A range slider component with support for labels, value display,
 * marks, error states, and help text.
 *
 * @example
 * ```tsx
 * <Slider
 *   label="Volume"
 *   value={volume}
 *   onChange={setVolume}
 *   min={0}
 *   max={100}
 *   showValue
 * />
 *
 * <Slider
 *   label="Opacity"
 *   value={opacity}
 *   onChange={setOpacity}
 *   min={0}
 *   max={1}
 *   step={0.1}
 *   valueFormat={(v) => `${Math.round(v * 100)}%`}
 *   marks={[0, 0.25, 0.5, 0.75, 1]}
 * />
 *
 * <Slider
 *   label="Rating"
 *   error={hasError}
 *   errorMessage="Rating must be between 1 and 5"
 *   min={1}
 *   max={5}
 *   required
 * />
 * ```
 */
export const Slider = React.forwardRef<HTMLInputElement, SliderProps>(
  (
    {
      size = 'md',
      label,
      helpText,
      error = false,
      errorMessage,
      required = false,
      min = 0,
      max = 100,
      step = 1,
      showValue = true,
      valueFormat,
      marks,
      fullWidth = false,
      className,
      value,
      defaultValue = min,
      onChange,
      onChangeCommit,
      disabled,
      ...props
    },
    ref
  ) => {
    const [internalValue, setInternalValue] = useState(defaultValue);
    const [isDragging, setIsDragging] = useState(false);

    const currentValue = value !== undefined ? value : internalValue;
    const percentage = ((currentValue - min) / (max - min)) * 100;

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      const newValue = parseFloat(e.target.value);
      if (value === undefined) {
        setInternalValue(newValue);
      }
      onChange?.(newValue);
    };

    const handleCommit = () => {
      setIsDragging(false);
      onChangeCommit?.(currentValue);
    };

    const formatValue = (val: number) => {
      if (valueFormat) {
        return valueFormat(val);
      }
      return val.toString();
    };

    const sizeStyle = sizeStyles[size];

    return (
      <div className={cn('flex flex-col', fullWidth && 'w-full')}>
        {label && (
          <div className="flex items-center justify-between mb-2">
            <label className="text-sm font-medium text-gray-700">
              {label}
              {required && <span className="text-red-500 ml-1">*</span>}
            </label>
            {showValue && (
              <span className={cn(
                'text-sm font-mono',
                error ? 'text-red-600' : 'text-gray-900'
              )}>
                {formatValue(currentValue)}
              </span>
            )}
          </div>
        )}

        <div className="relative pt-4 pb-2">
          {/* Track background */}
          <div className={cn(
            'absolute top-1/2 left-0 right-0 -translate-y-1/2 rounded-full',
            'bg-gray-200',
            sizeStyle.height
          )}>
            {/* Filled track */}
            <div
              className={cn(
                'absolute top-0 left-0 h-full rounded-full transition-all duration-150',
                error ? 'bg-red-600' : 'bg-blue-600'
              )}
              style={{ width: `${percentage}%` }}
            />
          </div>

          {/* Marks */}
          {marks && marks.length > 0 && (
            <div className="absolute top-1/2 left-0 right-0 -translate-y-1/2 flex justify-between px-0">
              {marks.map((mark, index) => {
                const markPercentage = ((mark - min) / (max - min)) * 100;
                return (
                  <div
                    key={index}
                    className={cn(
                      'absolute w-0.5 h-2 bg-gray-400 rounded-full -translate-x-1/2',
                      disabled && 'opacity-50'
                    )}
                    style={{ left: `${markPercentage}%` }}
                  />
                );
              })}
            </div>
          )}

          {/* Input */}
          <input
            ref={ref}
            type="range"
            min={min}
            max={max}
            step={step}
            value={currentValue}
            onChange={handleChange}
            onMouseDown={() => setIsDragging(true)}
            onMouseUp={handleCommit}
            onTouchStart={() => setIsDragging(true)}
            onTouchEnd={handleCommit}
            disabled={disabled}
            className={cn(
              'absolute top-1/2 left-0 right-0 -translate-y-1/2 w-full',
              'opacity-0 cursor-pointer',
              disabled && 'cursor-not-allowed'
            )}
            {...props}
          />

          {/* Thumb */}
          <div
            className={cn(
              'absolute top-1/2 -translate-y-1/2 rounded-full border-2 border-white shadow-lg',
              'pointer-events-none transition-all duration-150',
              'hover:scale-110 active:scale-95',
              error ? 'bg-red-600' : 'bg-blue-600',
              isDragging && 'scale-110',
              disabled && 'opacity-50',
              sizeStyle.thumb
            )}
            style={{ left: `calc(${percentage}% - ${size === 'sm' ? 6 : size === 'lg' ? 10 : 8}px)` }}
          />
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

Slider.displayName = 'Slider';
