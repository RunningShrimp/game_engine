import React from 'react';
import { cn } from '../../lib/utils';
import { Label } from '../atoms/Label';

export interface PropertyFieldProps {
  /** Field label */
  label: string;
  /** Field content */
  children: React.ReactNode;
  /** Label variant */
  labelVariant?: 'default' | 'muted' | 'accent';
  /** Additional CSS classes */
  className?: string;
  /** Label width (1-12) */
  labelWidth?: number;
}

/**
 * PropertyField - A labeled field for displaying and editing properties
 *
 * @example
 * ```tsx
 * <PropertyField label="Position" labelWidth={2}>
 *   <Vector3Editor value={...} onChange={...} />
 * </PropertyField>
 * ```
 */
export const PropertyField = React.memo<PropertyFieldProps>(({
  label,
  children,
  labelVariant = 'muted',
  className,
  labelWidth = 12,
}) => {
  return (
    <div className={cn('space-y-1', className)}>
      <Label variant={labelVariant} size="xs">
        {label}
      </Label>
      {children}
    </div>
  );
});

PropertyField.displayName = 'PropertyField';
