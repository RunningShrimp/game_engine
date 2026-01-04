import React from 'react';
import { cn } from '../../lib/utils';
import { Label } from '../atoms/Label';

export interface PropertyLabelProps {
  /** Label text */
  children: React.ReactNode;
  /** Label variant */
  variant?: 'default' | 'muted' | 'accent';
  /** Axis color (for vector components) */
  axis?: 'x' | 'y' | 'z' | 'w';
  /** Additional CSS classes */
  className?: string;
}

/**
 * PropertyLabel - A specialized label for property fields
 *
 * @example
 * ```tsx
 * <PropertyLabel axis="x">X</PropertyLabel>
 * <PropertyLabel variant="muted">Name</PropertyLabel>
 * ```
 */
export const PropertyLabel = React.memo<PropertyLabelProps>(({
  children,
  variant = 'muted',
  axis,
  className,
}) => {
  const axisColor = React.useMemo(() => {
    switch (axis) {
      case 'x':
        return 'text-red-400';
      case 'y':
        return 'text-green-400';
      case 'z':
        return 'text-blue-400';
      case 'w':
        return 'text-purple-400';
      default:
        return '';
    }
  }, [axis]);

  return (
    <Label
      variant={axis ? 'default' : variant}
      size="xs"
      className={cn(axis && axisColor, 'font-semibold', className)}
    >
      {children}
    </Label>
  );
});

PropertyLabel.displayName = 'PropertyLabel';
