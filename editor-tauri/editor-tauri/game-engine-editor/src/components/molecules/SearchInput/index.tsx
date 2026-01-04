import React, { InputHTMLAttributes, useState } from 'react';
import { Icon } from '../../atoms/Icon';
import { cn } from '../../../utils/cn';

/**
 * SearchInput size types
 */
export type SearchInputSize = 'sm' | 'md' | 'lg';

/**
 * SearchInput component props
 */
export interface SearchInputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'size'> {
  /**
   * Input size
   * @default 'md'
   */
  size?: SearchInputSize;

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
   * Show clear button
   * @default true
   */
  showClear?: boolean;

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

  /**
   * Search handler (called on Enter or after debounce)
   */
  onSearch?: (query: string) => void;

  /**
   * Clear handler
   */
  onClear?: () => void;

  /**
   * Debounce delay in milliseconds
   * @default 300
   */
  debounceDelay?: number;
}

/**
 * Size styles mapping
 */
const sizeStyles: Record<SearchInputSize, { container: string; input: string; icon: string }> = {
  sm: {
    container: 'h-8',
    input: 'pl-9 pr-8 px-3 py-1.5 text-sm',
    icon: 'w-4 h-4',
  },
  md: {
    container: 'h-10',
    input: 'pl-10 pr-10 px-4 py-2 text-sm',
    icon: 'w-5 h-5',
  },
  lg: {
    container: 'h-12',
    input: 'pl-12 pr-12 px-5 py-3 text-base',
    icon: 'w-6 h-6',
  },
};

/**
 * SearchInput component (Molecule)
 *
 * A search input component composed of Input + Icon atoms.
 * Features search icon, clear button, debounced search, and Enter key support.
 *
 * @example
 * ```tsx
 * <SearchInput
 *   label="Search users"
 *   placeholder="Enter name or email"
 *   value={searchQuery}
 *   onChange={setSearchQuery}
 *   onSearch={handleSearch}
 * />
 *
 * <SearchInput
 *   placeholder="Search..."
 *   onSearch={handleSearch}
 *   debounceDelay={500}
 *   showClear={true}
 * />
 *
 * <SearchInput
 *   label="Search products"
 *   error={hasError}
 *   errorMessage="Search query is too short"
 *   required
 * />
 * ```
 */
export const SearchInput = React.forwardRef<HTMLInputElement, SearchInputProps>(
  (
    {
      size = 'md',
      error = false,
      errorMessage,
      helpText,
      label,
      required = false,
      fullWidth = false,
      showClear = true,
      className,
      value,
      defaultValue = '',
      onChange,
      onSearch,
      onClear,
      debounceDelay = 300,
      placeholder = 'Search...',
      ...props
    },
    ref
  ) => {
    const [focused, setFocused] = useState(false);
    const [internalValue, setInternalValue] = useState(defaultValue);
    const [debounceTimer, setDebounceTimer] = useState<NodeJS.Timeout>();

    const currentValue = value !== undefined ? value : internalValue;

    // Debounced search
    const handleSearch = (query: string) => {
      if (debounceTimer) {
        clearTimeout(debounceTimer);
      }

      const timer = setTimeout(() => {
        onSearch?.(query);
      }, debounceDelay);

      setDebounceTimer(timer);
    };

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      const newValue = e.target.value;
      if (value === undefined) {
        setInternalValue(newValue);
      }
      onChange?.(newValue);
      handleSearch(newValue);
    };

    const handleClear = () => {
      if (value === undefined) {
        setInternalValue('');
      }
      onChange?.('');
      onClear?.();
      onSearch?.('');
    };

    const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === 'Enter') {
        if (debounceTimer) {
          clearTimeout(debounceTimer);
        }
        onSearch?.(currentValue);
      }
    };

    const hasValue = currentValue.length > 0;
    const sizeStyle = sizeStyles[size];

    return (
      <div className={cn('flex flex-col', fullWidth && 'w-full')}>
        {label && (
          <label className="block text-sm font-medium text-gray-700 mb-1">
            {label}
            {required && <span className="text-red-500 ml-1">*</span>}
          </label>
        )}

        <div className={cn('relative', sizeStyle.container, fullWidth && 'w-full')}>
          {/* Search icon */}
          <div
            className={cn(
              'absolute left-0 top-0 bottom-0 flex items-center justify-center pl-3',
              'pointer-events-none text-gray-400',
              focused && 'text-blue-500'
            )}
          >
            <Icon name="Search" size={size === 'sm' ? 16 : size === 'lg' ? 24 : 20} />
          </div>

          {/* Input */}
          <input
            ref={ref}
            value={currentValue}
            defaultValue={defaultValue}
            onChange={handleChange}
            onKeyDown={handleKeyDown}
            onFocus={(e) => {
              setFocused(true);
              props.onFocus?.(e);
            }}
            onBlur={(e) => {
              setFocused(false);
              props.onBlur?.(e);
            }}
            placeholder={placeholder}
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

          {/* Clear button */}
          {showClear && hasValue && (
            <button
              type="button"
              onClick={handleClear}
              className={cn(
                'absolute right-0 top-0 bottom-0 flex items-center justify-center pr-3',
                'text-gray-400 hover:text-gray-600',
                'focus:outline-none focus:ring-2 focus:ring-blue-500 rounded-full',
                'transition-colors duration-150',
                'disabled:opacity-50 disabled:cursor-not-allowed'
              )}
              aria-label="Clear search"
            >
              <Icon name="X" size={size === 'sm' ? 14 : size === 'lg' ? 20 : 16} />
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

SearchInput.displayName = 'SearchInput';
