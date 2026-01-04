import React, { CheckboxHTMLAttributes, useState } from 'react';
import { Icon } from '../../atoms/Icon';
import { cn } from '../../../utils/cn';

/**
 * Checkbox size types
 */
export type CheckboxSize = 'sm' | 'md' | 'lg';

/**
 * Checkbox component props
 */
export interface CheckboxProps extends Omit<CheckboxHTMLAttributes<HTMLInputElement>, 'size' | 'value' | 'onChange'> {
  /**
   * Checkbox size
   * @default 'md'
   */
  size?: CheckboxSize;

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
   * Indeterminate state (mixed)
   * @default false
   */
  indeterminate?: boolean;

  /**
   * Mark field as required
   * @default false
   */
  required?: boolean;

  /**
   * Checkbox value (controlled)
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
const sizeStyles: Record<CheckboxSize, { container: string; box: string; icon: string; text: string }> = {
  sm: {
    container: 'h-4 w-4',
    box: 'rounded',
    icon: 'w-3 h-3',
    text: 'text-sm',
  },
  md: {
    container: 'h-5 w-5',
    box: 'rounded-md',
    icon: 'w-4 h-4',
    text: 'text-base',
  },
  lg: {
    container: 'h-6 w-6',
    box: 'rounded-lg',
    icon: 'w-5 h-5',
    text: 'text-lg',
  },
};

/**
 * Checkbox component (Molecule)
 *
 * A checkbox input component with label support. Supports indeterminate state,
 * error states, help text, and all standard checkbox attributes.
 *
 * @example
 * ```tsx
 * <Checkbox
 *   label="Accept terms and conditions"
 *   checked={accepted}
 *   onChange={setAccepted}
 *   required
 * />
 *
 * <Checkbox
 *   label="Subscribe to newsletter"
 *   helpText="Get updates about new features"
 *   defaultChecked
 * />
 *
 * <Checkbox
 *   label="Select all"
 *   indeterminate={someSelected}
 *   checked={allSelected}
 * />
 * ```
 */
export const Checkbox = React.forwardRef<HTMLInputElement, CheckboxProps>(
  (
    {
      size = 'md',
      label,
      helpText,
      error = false,
      errorMessage,
      indeterminate = false,
      required = false,
      className,
      checked,
      defaultChecked = false,
      onChange,
      ...props
    },
    ref
  ) => {
    const [internalChecked, setInternalChecked] = useState(defaultChecked);
    const checkboxRef = React.useRef<HTMLInputElement>(null);

    // Handle indeterminate state
    React.useEffect(() => {
      if (checkboxRef.current) {
        checkboxRef.current.indeterminate = indeterminate;
      }
    }, [indeterminate]);

    // Forward ref
    React.useImperativeHandle(ref, () => checkboxRef.current!);

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
      <div className="flex flex-col">
        <div className="flex items-start gap-3">
          <div className="relative flex items-center pt-0.5">
            <input
              ref={checkboxRef}
              type="checkbox"
              checked={currentChecked}
              onChange={handleChange}
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
                currentChecked && !indeterminate && 'bg-blue-600 border-blue-600',
                // Indeterminate states
                indeterminate && 'bg-blue-600 border-blue-600',
                // Disabled states
                'disabled:bg-gray-100 disabled:cursor-not-allowed disabled:opacity-50',
                // Size styles
                sizeStyle.container,
                sizeStyle.box,
                // Custom className
                className
              )}
              {...props}
            />

            {/* Check icon */}
            {currentChecked && !indeterminate && (
              <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
                <Icon
                  name="Check"
                  className={cn('text-white', sizeStyle.icon)}
                  strokeWidth={3}
                />
              </div>
            )}

            {/* Indeterminate icon */}
            {indeterminate && (
              <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
                <div className={cn('bg-white rounded-sm', size === 'sm' ? 'w-2 h-0.5' : size === 'lg' ? 'w-3 h-0.5' : 'w-2.5 h-0.5')} />
              </div>
            )}
          </div>

          {label && (
            <div className="flex flex-col">
              <label
                className={cn(
                  'font-medium text-gray-700 cursor-pointer select-none',
                  error && 'text-red-600',
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

Checkbox.displayName = 'Checkbox';
