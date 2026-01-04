import React from 'react';
import { cn } from '../../lib/utils';

export interface CheckboxProps extends React.InputHTMLAttributes<HTMLInputElement> {
  /** Label text */
  label?: string;
  /** Label position */
  labelPosition?: 'left' | 'right';
}

export const Checkbox = React.forwardRef<HTMLInputElement, CheckboxProps>(
  ({ className, label, labelPosition = 'right', disabled, ...props }, ref) => {
    const checkbox = (
      <input
        ref={ref}
        type="checkbox"
        disabled={disabled}
        className={cn(
          'rounded border-slate-700 bg-slate-800',
          'focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-slate-900',
          'disabled:cursor-not-allowed disabled:opacity-50',
          {
            'w-3 h-3': !className?.includes('w-'),
            'w-4 h-4': className?.includes('w-4'),
          },
          className
        )}
        {...props}
      />
    );

    if (!label) {
      return checkbox;
    }

    return (
      <label
        className={cn(
          'flex items-center gap-1 text-sm cursor-pointer',
          {
            'cursor-not-allowed opacity-50': disabled,
          }
        )}
      >
        {labelPosition === 'left' && (
          <>
            {label}
            {checkbox}
          </>
        )}
        {labelPosition === 'right' && (
          <>
            {checkbox}
            <span>{label}</span>
          </>
        )}
      </label>
    );
  }
);

Checkbox.displayName = 'Checkbox';
