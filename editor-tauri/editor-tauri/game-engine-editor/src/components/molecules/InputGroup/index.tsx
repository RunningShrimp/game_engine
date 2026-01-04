import React, { HTMLAttributes, cloneElement, ReactElement, isValidElement } from 'react';
import { cn } from '../../../utils/cn';

/**
 * InputGroup item type
 */
export type InputGroupItem = ReactElement;

/**
 * InputGroup component props
 */
export interface InputGroupProps extends HTMLAttributes<HTMLDivElement> {
  /**
   * Input elements to group together
   */
  children: ReactElement | ReactElement[];

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
   * Label for the group
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
   * Separator between inputs
   * @default '-'
   */
  separator?: '-' | '.' | ':' | '/' | 'none';

  /**
   * Layout direction
   * @default 'horizontal'
   */
  direction?: 'horizontal' | 'vertical';
}

/**
 * Separator components
 */
const Separators: Record<string, React.ReactNode> = {
  '-': <span className="text-gray-500 font-medium">-</span>,
  '.': <span className="text-gray-500 font-medium">.</span>,
  ':': <span className="text-gray-500 font-medium">:</span>,
  '/': <span className="text-gray-500 font-medium">/</span>,
  'none': null,
};

/**
 * InputGroup component (Molecule)
 *
 * Groups multiple input components together with separators.
 * Commonly used for dates, phone numbers, IP addresses, etc.
 *
 * @example
 * ```tsx
 * <InputGroup
 *   label="Date of Birth"
 *   separator="/"
 * >
 *   <Input type="text" placeholder="MM" maxLength={2} />
 *   <Input type="text" placeholder="DD" maxLength={2} />
 *   <Input type="text" placeholder="YYYY" maxLength={4} />
 * </InputGroup>
 *
 * <InputGroup
 *   label="IP Address"
 *   separator="."
 * >
 *   <Input type="text" placeholder="000" maxLength={3} />
 *   <Input type="text" placeholder="000" maxLength={3} />
 *   <Input type="text" placeholder="000" maxLength={3} />
 *   <Input type="text" placeholder="000" maxLength={3} />
 * </InputGroup>
 *
 * <InputGroup
 *   label="Time"
 *   separator=":"
 * >
 *   <NumberInput min={0} max={23} placeholder="HH" />
 *   <NumberInput min={0} max={59} placeholder="MM" />
 *   <NumberInput min={0} max={59} placeholder="SS" />
 * </InputGroup>
 * ```
 */
export const InputGroup = React.forwardRef<HTMLDivElement, InputGroupProps>(
  (
    {
      children,
      error = false,
      errorMessage,
      helpText,
      label,
      required = false,
      fullWidth = false,
      separator = '-',
      direction = 'horizontal',
      className,
      ...props
    },
    ref
  ) => {
    // Convert single child to array
    const childArray = Array.isArray(children) ? children : [children];

    // Filter out invalid elements
    const validChildren = childArray.filter((child) => isValidElement(child));

    if (validChildren.length === 0) {
      console.warn('InputGroup: No valid children provided');
      return null;
    }

    // Clone children with modified props
    const enhancedChildren = validChildren.map((child, index) => {
      const baseProps = {
        key: index,
        // Remove label from individual inputs (we have group label)
        label: undefined,
        // Remove error state (we handle it at group level)
        error: undefined,
        errorMessage: undefined,
        // Pass through error state if needed
        className: cn(
          // Remove margin/radius based on position
          index === 0 && direction === 'horizontal' && 'rounded-l-none rounded-r-md',
          index === validChildren.length - 1 && direction === 'horizontal' && 'rounded-l-md rounded-r-none',
          index !== 0 && index !== validChildren.length - 1 && direction === 'horizontal' && 'rounded-none',
          direction === 'vertical' && 'rounded-md'
        ),
      };

      return cloneElement(child, baseProps);
    });

    const separatorNode = Separators[separator];

    return (
      <div ref={ref} className={cn('flex flex-col', fullWidth && 'w-full', className)}>
        {label && (
          <label className="block text-sm font-medium text-gray-700 mb-1">
            {label}
            {required && <span className="text-red-500 ml-1">*</span>}
          </label>
        )}

        <div
          className={cn(
            'flex',
            direction === 'horizontal' ? 'flex-row gap-1' : 'flex-col gap-2',
            error && 'ring-2 ring-red-500 rounded-md p-0.5'
          )}
          {...props}
        >
          {enhancedChildren.map((child, index) => (
            <React.Fragment key={index}>
              {index > 0 && separator !== 'none' && (
                <div className="flex items-center justify-center">
                  {separatorNode}
                </div>
              )}
              {child}
            </React.Fragment>
          ))}
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

InputGroup.displayName = 'InputGroup';
