import React from 'react';
import { cn } from '../../lib/utils';

export interface PropertySectionProps {
  /** Section title */
  title: string;
  /** Additional badge text */
  badge?: string;
  /** Section children */
  children: React.ReactNode;
  /** Additional CSS classes */
  className?: string;
}

/**
 * PropertySection - A collapsible section for grouping related properties
 *
 * @example
 * ```tsx
 * <PropertySection title="Transform" badge="World">
 *   <TransformProperty position={...} rotation={...} />
 * </PropertySection>
 * ```
 */
export const PropertySection = React.memo<PropertySectionProps>(({
  title,
  badge,
  children,
  className,
}) => {
  return (
    <div className={cn('space-y-2', className)}>
      <div className="flex items-center justify-between">
        <h3 className="text-xs font-semibold text-slate-400 uppercase">{title}</h3>
        {badge && (
          <span className="text-xs text-slate-600 font-normal">{badge}</span>
        )}
      </div>
      <div className="space-y-2">
        {children}
      </div>
    </div>
  );
});

PropertySection.displayName = 'PropertySection';
