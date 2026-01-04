import React from 'react';
import { cn } from '../../../utils/cn';
import { Icon } from '../../atoms';
import type { IconName } from '../../atoms';

export interface ToolbarButtonProps {
  icon: IconName;
  title?: string;
  disabled?: boolean;
  active?: boolean;
  variant?: 'default' | 'active' | 'success' | 'warning' | 'danger';
  onClick?: () => void;
  className?: string;
}

export const ToolbarButton: React.FC<ToolbarButtonProps> = ({
  icon,
  title,
  disabled = false,
  active = false,
  variant = 'default',
  onClick,
  className,
}) => {
  const variantStyles: Record<typeof variant, string> = {
    default: 'bg-slate-700 text-slate-300 hover:bg-slate-600',
    active: 'bg-blue-500 text-white',
    success: 'bg-green-500 text-white',
    warning: 'bg-yellow-600 text-white hover:bg-yellow-700',
    danger: 'bg-red-600 text-white hover:bg-red-700',
  };

  const disabledStyles = 'bg-slate-800 text-slate-600 cursor-not-allowed';

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      title={title}
      className={cn(
        'px-3 py-1.5 rounded text-sm font-medium transition-colors',
        disabled ? disabledStyles : variantStyles[variant],
        className
      )}
    >
      <Icon name={icon} size={16} />
    </button>
  );
};
