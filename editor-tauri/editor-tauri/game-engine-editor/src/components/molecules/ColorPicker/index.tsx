import React, { InputHTMLAttributes, useState } from 'react';
import { Icon } from '../../atoms/Icon';
import { cn } from '../../../utils/cn';

/**
 * ColorPicker size types
 */
export type ColorPickerSize = 'sm' | 'md' | 'lg';

/**
 * Color format types
 */
export type ColorFormat = 'hex' | 'rgb' | 'hsl';

/**
 * ColorPicker component props
 */
export interface ColorPickerProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'size' | 'type' | 'value'> {
  /**
   * Input size
   * @default 'md'
   */
  size?: ColorPickerSize;

  /**
   * Color format
   * @default 'hex'
   */
  format?: ColorFormat;

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
   * Show color preview swatch
   * @default true
   */
  showPreview?: boolean;

  /**
   * Show preset colors
   */
  presetColors?: string[];

  /**
   * Input value (controlled) - should be a valid CSS color string
   */
  value?: string;

  /**
   * Default value (uncontrolled)
   */
  defaultValue?: string;

  /**
   * Change handler
   */
  onChange?: (color: string) => void;
}

/**
 * Default preset colors
 */
const defaultPresetColors = [
  '#000000',
  '#FFFFFF',
  '#EF4444', // red-500
  '#F59E0B', // amber-500
  '#10B981', // emerald-500
  '#3B82F6', // blue-500
  '#8B5CF6', // violet-500
  '#EC4899', // pink-500
];

/**
 * Size styles mapping
 */
const sizeStyles: Record<ColorPickerSize, {
  container: string;
  input: string;
  preview: string;
  swatch: string;
}> = {
  sm: {
    container: 'h-8',
    input: 'px-3 py-1.5 text-sm',
    preview: 'w-8 h-8',
    swatch: 'w-4 h-4',
  },
  md: {
    container: 'h-10',
    input: 'px-4 py-2 text-sm',
    preview: 'w-10 h-10',
    swatch: 'w-5 h-5',
  },
  lg: {
    container: 'h-12',
    input: 'px-5 py-3 text-base',
    preview: 'w-12 h-12',
    swatch: 'w-6 h-6',
  },
};

/**
 * ColorPicker component (Molecule)
 *
 * A color picker input component with preview swatch and preset colors.
 * Supports multiple color formats and custom color input.
 *
 * @example
 * ```tsx
 * <ColorPicker
 *   label="Primary color"
 *   value={color}
 *   onChange={setColor}
 *   format="hex"
 * />
 *
 * <ColorPicker
 *   label="Background color"
 *   defaultValue="#ffffff"
 *   presetColors={['#000000', '#FFFFFF', '#FF0000']}
 * />
 *
 * <ColorPicker
 *   label="Accent color"
 *   error={hasError}
 *   errorMessage="Please select a valid color"
 *   required
 * />
 * ```
 */
export const ColorPicker = React.forwardRef<HTMLInputElement, ColorPickerProps>(
  (
    {
      size = 'md',
      format = 'hex',
      error = false,
      errorMessage,
      helpText,
      label,
      required = false,
      fullWidth = false,
      showPreview = true,
      presetColors = defaultPresetColors,
      className,
      value,
      defaultValue = '#000000',
      onChange,
      disabled,
      ...props
    },
    ref
  ) => {
    const [focused, setFocused] = useState(false);
    const [internalValue, setInternalValue] = useState(defaultValue);
    const fileInputRef = React.useRef<HTMLInputElement>(null);

    const currentValue = value !== undefined ? value : internalValue;

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      const newValue = e.target.value;
      if (value === undefined) {
        setInternalValue(newValue);
      }
      onChange?.(newValue);
    };

    const handlePresetClick = (color: string) => {
      if (value === undefined) {
        setInternalValue(color);
      }
      onChange?.(color);
    };

    const sizeStyle = sizeStyles[size];

    return (
      <div className={cn('flex flex-col', fullWidth && 'w-full')}>
        {label && (
          <label className="block text-sm font-medium text-gray-700 mb-1">
            {label}
            {required && <span className="text-red-500 ml-1">*</span>}
          </label>
        )}

        <div className="flex gap-2">
          {/* Color preview swatch */}
          {showPreview && (
            <div
              className={cn(
                'flex-shrink-0 rounded-md border-2 border-gray-300 cursor-pointer',
                'transition-all duration-200',
                'hover:scale-105 active:scale-95',
                error && 'border-red-500',
                focused && 'ring-2 ring-blue-500 ring-offset-2',
                disabled && 'opacity-50 cursor-not-allowed',
                sizeStyle.preview
              )}
              style={{ backgroundColor: currentValue }}
              onClick={() => !disabled && fileInputRef.current?.click()}
            >
              {/* Color picker icon overlay */}
              <div className="absolute inset-0 flex items-center justify-center opacity-0 hover:opacity-100 transition-opacity">
                <Icon name="Palette" size={size === 'sm' ? 16 : size === 'lg' ? 24 : 20} className="text-white drop-shadow-md" />
              </div>
            </div>
          )}

          {/* Input field */}
          <div className={cn('relative flex-1', sizeStyle.container, fullWidth && 'w-full')}>
            <input
              ref={ref}
              type="color"
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
                // Custom className
                className
              )}
              {...props}
            />
          </div>
        </div>

        {/* Preset colors */}
        {presetColors && presetColors.length > 0 && (
          <div className="flex flex-wrap gap-2 mt-2">
            {presetColors.map((color, index) => (
              <button
                key={index}
                type="button"
                onClick={() => handlePresetClick(color)}
                disabled={disabled}
                className={cn(
                  'rounded-md border-2 border-gray-300 cursor-pointer',
                  'transition-all duration-200',
                  'hover:scale-110 active:scale-95',
                  'focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2',
                  disabled && 'opacity-50 cursor-not-allowed',
                  sizeStyle.swatch
                )}
                style={{ backgroundColor: color }}
                aria-label={`Select color ${color}`}
              />
            ))}
          </div>
        )}

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

ColorPicker.displayName = 'ColorPicker';
