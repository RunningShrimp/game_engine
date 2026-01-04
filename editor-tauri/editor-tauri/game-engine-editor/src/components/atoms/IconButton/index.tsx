import React from 'react';
import { Icon } from '../Icon';
import { cn } from '../../lib/utils';

export interface IconButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  /** Icon name from Lucide icons */
  iconName: keyof import('lucide-react');
  /** Icon size */
  iconSize?: number;
  /** Button variant */
  variant?: 'default' | 'ghost' | 'outline';
  /** Button size */
  size?: 'sm' | 'md' | 'lg';
}

export const IconButton = React.forwardRef<HTMLButtonElement, IconButtonProps>(
  ({ className, iconName, iconSize = 16, variant = 'ghost', size = 'sm', children, ...props }, ref) => {
    return (
      <button
        ref={ref}
        className={cn(
          'inline-flex items-center justify-center rounded transition-colors',
          'focus:outline-none focus:ring-2 focus:ring-blue-500',
          'disabled:opacity-50 disabled:cursor-not-allowed',
          {
            'bg-slate-800 hover:bg-slate-700 text-slate-200': variant === 'default',
            'hover:bg-slate-800 text-slate-400 hover:text-slate-200': variant === 'ghost',
            'border border-slate-700 hover:bg-slate-800 text-slate-400 hover:text-slate-200': variant === 'outline',
          },
          {
            'p-1': size === 'sm',
            'p-2': size === 'md',
            'p-3': size === 'lg',
          },
          className
        )}
        {...props}
      >
        <Icon name={iconName} size={iconSize} />
        {children}
      </button>
    );
  }
);

IconButton.displayName = 'IconButton';
