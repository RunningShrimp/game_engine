import React, { ButtonHTMLAttributes, forwardRef } from 'react';
import { cn } from '../../utils/cn';

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
   * Child content to display
   */
  children: React.ReactNode;
}

/**
 * Variant styles mapping
 */
const variantStyles: Record<ButtonVariant, string> = {
  primary: 'bg-blue-600 text-white hover:bg-blue-700 focus:ring-blue-500 border-transparent',
  secondary: 'bg-gray-600 text-white hover:bg-gray-700 focus:ring-gray-500 border-transparent',
  outline: 'bg-transparent text-gray-700 hover:bg-gray-50 focus:ring-gray-500 border-gray-300',
  ghost: 'bg-transparent text-gray-700 hover:bg-gray-100 focus:ring-gray-500 border-transparent',
  danger: 'bg-red-600 text-white hover:bg-red-700 focus:ring-red-500 border-transparent',
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
    className="animate-spin -ml-1 mr-2 h-4 w-4"
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
 * Button component
 *
 * A versatile button component with support for multiple variants, sizes, and states.
 * Includes loading state with spinner and follows accessibility best practices.
 *
 * @example
 * ```tsx
 * <Button variant="primary" size="md" onClick={handleClick}>
 *   Click me
 * </Button>
 *
 * <Button variant="outline" isLoading>
 *   Loading...
 * </Button>
 *
 * <Button variant="danger" fullWidth>
 *   Delete
 * </Button>
 * ```
 */
export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      variant = 'primary',
      size = 'md',
      isLoading = false,
      disabled = false,
      fullWidth = false,
      className,
      children,
      ...props
    },
    ref
  ) => {
    const isDisabled = disabled || isLoading;

    return (
      <button
        ref={ref}
        disabled={isDisabled}
        className={cn(
          // Base styles
          'inline-flex items-center justify-center border',
          'font-medium focus:outline-none focus:ring-2 focus:ring-offset-2',
          'transition-colors duration-200',
          'disabled:opacity-50 disabled:cursor-not-allowed',
          // Variant styles
          variantStyles[variant],
          // Size styles
          sizeStyles[size],
          // Width
          fullWidth && 'w-full',
          // Custom className
          className
        )}
        {...props}
      >
        {isLoading && <ButtonSpinner />}
        {children}
      </button>
    );
  }
);

Button.displayName = 'Button';
