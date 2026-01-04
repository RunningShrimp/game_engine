import React from 'react';
import { cn } from '../../lib/utils';

export interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  /** Input variant */
  variant?: 'default' | 'muted' | 'accent';
  /** Input size */
  size?: 'sm' | 'md';
  /** Full width */
  fullWidth?: boolean;
}

export const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, variant = 'default', size = 'sm', fullWidth = false, ...props }, ref) => {
    return (
      <input
        ref={ref}
        className={cn(
          'rounded border transition-colors',
          'focus:border-blue-500 focus:outline-none',
          {
            'bg-slate-800 border-slate-700 text-slate-200': variant === 'default',
            'bg-slate-900 border-slate-700 text-slate-500 cursor-not-allowed': variant === 'muted',
            'bg-slate-800 border-blue-500 text-blue-400': variant === 'accent',
          },
          {
            'px-2 py-1 text-sm': size === 'sm',
            'px-3 py-2 text-base': size === 'md',
          },
          {
            'w-full': fullWidth,
          },
          className
        )}
        {...props}
      />
    );
  }
);

Input.displayName = 'Input';
