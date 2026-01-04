import React from 'react';
import { cn } from '../../lib/utils';

export interface LabelProps extends React.LabelHTMLAttributes<HTMLLabelElement> {
  /** Label variant */
  variant?: 'default' | 'muted' | 'accent';
  /** Label size */
  size?: 'xs' | 'sm' | 'md';
}

export const Label = React.forwardRef<HTMLLabelElement, LabelProps>(
  ({ className, variant = 'default', size = 'sm', children, ...props }, ref) => {
    return (
      <label
        ref={ref}
        className={cn(
          'font-medium',
          {
            'text-slate-200': variant === 'default',
            'text-slate-400': variant === 'muted',
            'text-blue-400': variant === 'accent',
          },
          {
            'text-xs': size === 'xs',
            'text-sm': size === 'sm',
            'text-base': size === 'md',
          },
          className
        )}
        {...props}
      >
        {children}
      </label>
    );
  }
);

Label.displayName = 'Label';
