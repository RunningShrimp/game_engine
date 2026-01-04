import React, { HTMLAttributes } from 'react';
import { cn } from '../../../utils/cn';

/**
 * FormCard component props
 */
export interface FormCardProps extends HTMLAttributes<HTMLDivElement> {
  /**
   * Card title
   */
  title?: string;

  /**
   * Card description
   */
  description?: string;

  /**
   * Card actions (buttons, etc.)
   */
  actions?: React.ReactNode;

  /**
   * Loading state
   */
  isLoading?: boolean;

  /**
   * Error state
   */
  error?: boolean;

  /**
   * Children content
   */
  children: React.ReactNode;
}

/**
 * FormCard component (Organism)
 *
 * A complex form container component that organizes form sections with proper spacing,
 * headers, and actions. This is an organism-level component that combines molecules and atoms.
 *
 * @example
 * ```tsx
 * <FormCard
 *   title="User Settings"
 *   description="Manage your account settings"
 *   actions={<Button>Save</Button>}
 * >
 *   <Input label="Name" />
 *   <Input label="Email" />
 * </FormCard>
 * ```
 */
export const FormCard = ({
  title,
  description,
  actions,
  isLoading = false,
  error = false,
  children,
  className,
  ...props
}: FormCardProps) => {
  return (
    <div
      className={cn(
        'bg-white rounded-lg shadow-sm border',
        error ? 'border-red-300' : 'border-gray-200',
        className
      )}
      {...props}
    >
      {(title || description) && (
        <div className="p-6 border-b border-gray-200">
          {title && (
            <h2 className={cn('text-xl font-semibold', error ? 'text-red-600' : 'text-gray-900')}>
              {title}
            </h2>
          )}
          {description && (
            <p className="text-sm text-gray-500 mt-1">{description}</p>
          )}
        </div>
      )}

      <div className={cn('p-6 space-y-4', isLoading && 'opacity-50 pointer-events-none')}>
        {children}
      </div>

      {actions && (
        <div className="p-6 bg-gray-50 border-t border-gray-200">
          {actions}
        </div>
      )}
    </div>
  );
};

FormCard.displayName = 'FormCard';
