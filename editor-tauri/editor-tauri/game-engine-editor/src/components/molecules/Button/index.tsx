import React, { ButtonHTMLAttributes } from 'react';
import { Icon } from '../../atoms/Icon';
import { cn } from '../../../utils/cn';

/**
 * Button variant types
 */
export type ButtonVariant = 'primary' | 'secondary' | 'outline' | 'ghost' | 'danger';

/**
 * Button size types
 */
export type ButtonSize = 'xs' | 'sm' | 'md' | 'lg' | 'xl';

/**
 * Button component props
 */
export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  /**
   * Button visual variant
   * @default 'primary'
   */
  variant?: ButtonVariant;

  /**
   * Button size
   * @default 'md'
   */
  size?: ButtonSize;

  /**
   * Icon name (from Lucide icons)
   */
  icon?: keyof typeof import('lucide-react');

  /**
   * Icon position
   * @default 'left'
   */
  iconPosition?: 'left' | 'right';

  /**
   * Show loading state with spinner
   * @default false
   */
  isLoading?: boolean;

  /**
   * Disable the button
   * @default false
   */
  disabled?: boolean;

  /**
   * Button should take full width of parent
   * @default false
   */
  fullWidth?: boolean;

  /**
   * Error state
   * @default false
   */
  error?: boolean;

  /**
   * Child content to display
   */
  children?: React.ReactNode;
}

/**
 * Variant styles mapping
 */
const variantStyles: Record<ButtonVariant, { base: string; hover: string; focus: string }> = {
  primary: {
    base: 'bg-blue-600 text-white border-transparent',
    hover: 'hover:bg-blue-700',
    focus: 'focus:ring-blue-500',
  },
  secondary: {
    base: 'bg-gray-600 text-white border-transparent',
    hover: 'hover:bg-gray-700',
    focus: 'focus:ring-gray-500',
  },
  outline: {
    base: 'bg-transparent text-gray-700 border-gray-300',
    hover: 'hover:bg-gray-50',
    focus: 'focus:ring-gray-500',
  },
  ghost: {
    base: 'bg-transparent text-gray-700 border-transparent',
    hover: 'hover:bg-gray-100',
    focus: 'focus:ring-gray-500',
  },
  danger: {
    base: 'bg-red-600 text-white border-transparent',
    hover: 'hover:bg-red-700',
    focus: 'focus:ring-red-500',
  },
};

/**
 * Size styles mapping
 */
const sizeStyles: Record<ButtonSize, string> = {
  xs: 'px-2 py-1 text-xs font-medium rounded',
  sm: 'px-3 py-1.5 text-sm font-medium rounded',
  md: 'px-4 py-2 text-sm font-medium rounded-md',
  lg: 'px-5 py-2.5 text-base font-medium rounded-md',
  xl: 'px-6 py-3 text-base font-medium rounded-lg',
};

/**
 * Spinner component for loading state
 */
const ButtonSpinner = () => (
  <svg
    className="animate-spin h-4 w-4"
    xmlns="http://www.w3.org/2000/svg"
    fill="none"
    viewBox="0 0 24 24"
    aria-hidden="true"
  >
    <circle
      className="opacity-25"
      cx="12"
      cy="12"
      r="10"
      stroke="currentColor"
      strokeWidth="4"
    />
    <path
      className="opacity-75"
      fill="currentColor"
      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
    />
  </svg>
);

/**
 * Button component (Molecule)
 *
 * A versatile button component composed of Icon + Text/Content atoms.
 * Supports multiple variants, sizes, icons, and states.
 * Includes loading state with spinner and follows accessibility best practices.
 *
 * @example
 * ```tsx
 * <Button variant="primary" icon="Send">
 *   Submit
 * </Button>
 *
 * <Button variant="outline" icon="Download" iconPosition="right">
 *   Download
 * </Button>
 *
 * <Button variant="danger" isLoading>
 *   Deleting...
 * </Button>
 * ```
 */
export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      variant = 'primary',
      size = 'md',
      icon,
      iconPosition = 'left',
      isLoading = false,
      disabled = false,
      fullWidth = false,
      error = false,
      className,
      children,
      ...props
    },
    ref
  ) => {
    const isDisabled = disabled || isLoading;

    const variantStyle = variantStyles[variant];
    const errorStyle = error ? 'border-red-500 text-red-600' : '';

    return (
      <button
        ref={ref}
        disabled={isDisabled}
        className={cn(
          // Base styles
          'inline-flex items-center justify-center gap-2 border',
          'font-medium focus:outline-none focus:ring-2 focus:ring-offset-2',
          'transition-all duration-200',
          'disabled:opacity-50 disabled:cursor-not-allowed',
          // Variant styles
          variantStyle.base,
          variantStyle.hover,
          variantStyle.focus,
          // Size styles
          sizeStyles[size],
          // Width
          fullWidth && 'w-full',
          // Error state
          errorStyle,
          // Custom className
          className
        )}
        {...props}
      >
        {isLoading && <ButtonSpinner />}
        {!isLoading && icon && iconPosition === 'left' && (
          <Icon name={icon} size={16} />
        )}
        {children && <span>{children}</span>}
        {!isLoading && icon && iconPosition === 'right' && (
          <Icon name={icon} size={16} />
        )}
      </button>
    );
  }
);

Button.displayName = 'Button';
