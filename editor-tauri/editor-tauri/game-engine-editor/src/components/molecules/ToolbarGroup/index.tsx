import React from 'react';
import { cn } from '../../../utils/cn';
import { Divider } from '../../atoms';

export interface ToolbarGroupProps {
  children: React.ReactNode;
  showDivider?: boolean;
  className?: string;
}

export const ToolbarGroup: React.FC<ToolbarGroupProps> = ({
  children,
  showDivider = false,
  className,
}) => {
  return (
    <div className={cn('flex items-center gap-1', className)}>
      {children}
      {showDivider && <Divider orientation="vertical" className="mx-3" />}
    </div>
  );
};
