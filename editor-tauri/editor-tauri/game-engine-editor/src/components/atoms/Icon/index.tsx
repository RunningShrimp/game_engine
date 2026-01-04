import React from 'react';
import * as LucideIcons from 'lucide-react';
import { cn } from '../../utils/cn';

/**
 * Icon component properties
 */
export interface IconProps {
  /** Icon name from Lucide icons */
  name: keyof typeof LucideIcons;
  /** Icon size in pixels */
  size?: number;
  /** Icon color (CSS color value) */
  color?: string;
  /** Additional CSS classes */
  className?: string;
  /** Accessibility label */
  label?: string;
  /** Stroke width */
  strokeWidth?: number;
}

/**
 * Icon component - Wraps Lucide icons for consistent usage
 *
 * @example
 * ```tsx
 * <Icon name="Play" size={24} />
 * <Icon name="Settings" color="red" className="text-red-500" />
 * <Icon name="User" label="User icon" />
 * ```
 */
export const Icon = React.forwardRef<SVGSVGElement, IconProps>(
  ({ name, size = 24, color, className, label, strokeWidth = 2, ...props }, ref) => {
    const LucideIcon = LucideIcons[name] as React.ComponentType<{
      size?: number;
      className?: string;
      strokeWidth?: number;
      style?: React.CSSProperties;
    }>;

    if (!LucideIcon) {
      console.warn(`Icon "${name}" not found in Lucide icons`);
      return null;
    }

    return (
      <LucideIcon
        ref={ref}
        size={size}
        strokeWidth={strokeWidth}
        className={cn('inline-block', className)}
        style={color ? { color } : undefined}
        aria-label={label}
        aria-hidden={!label}
        {...props}
      />
    );
  }
);

Icon.displayName = 'Icon';
