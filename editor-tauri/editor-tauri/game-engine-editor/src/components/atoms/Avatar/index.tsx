import React from 'react';
import { cn } from '../../utils/cn';

/**
 * Avatar size types
 */
export type AvatarSize = 'xs' | 'sm' | 'md' | 'lg' | 'xl';

/**
 * Avatar component properties
 */
export interface AvatarProps {
  /** Avatar image source */
  src?: string;
  /** Avatar alt text */
  alt?: string;
  /** Avatar size */
  size?: AvatarSize;
  /** Additional CSS classes */
  className?: string;
  /** Fallback initials (2 characters max) */
  initials?: string;
  /** Fallback icon */
  fallbackIcon?: React.ReactNode;
  /** Whether avatar is circular (default) or square */
  shape?: 'circle' | 'square';
  /** Border variant */
  variant?: 'none' | 'ring' | 'border';
  /** Online status indicator */
  status?: 'online' | 'offline' | 'away' | 'busy';
  /** Click handler */
  onClick?: () => void;
}

/**
 * Avatar component - Displays user profile images or initials
 *
 * @example
 * ```tsx
 * <Avatar src="/avatar.png" alt="User name" />
 * <Avatar initials="JD" size="lg" />
 * <Avatar fallbackIcon={<Icon name="User" />} status="online" />
 * <Avatar initials="AB" shape="square" variant="ring" />
 * ```
 */
export const Avatar = React.forwardRef<HTMLDivElement, AvatarProps>(
  ({
    src,
    alt = '',
    size = 'md',
    className,
    initials,
    fallbackIcon,
    shape = 'circle',
    variant = 'none',
    status,
    onClick,
    ...props
  }, ref) => {
    const [imgError, setImgError] = React.useState(false);

    const sizeClasses: Record<AvatarSize, string> = {
      xs: 'h-6 w-6 text-xs',
      sm: 'h-8 w-8 text-sm',
      md: 'h-10 w-10 text-base',
      lg: 'h-12 w-12 text-lg',
      xl: 'h-16 w-16 text-xl',
    };

    const variantClasses = {
      none: '',
      ring: 'ring-2 ring-ring ring-offset-2',
      border: 'border-2 border-border',
    };

    const shapeClasses = {
      circle: 'rounded-full',
      square: 'rounded-md',
    };

    const statusColors = {
      online: 'bg-success',
      offline: 'bg-muted',
      away: 'bg-warning',
      busy: 'bg-error',
    };

    const displayInitials = React.useMemo(() => {
      if (!initials) return '';
      const parts = initials.trim().split(' ');
      if (parts.length >= 2) {
        return (parts[0][0] + parts[1][0]).toUpperCase();
      }
      return initials.slice(0, 2).toUpperCase();
    }, [initials]);

    const content = React.useMemo(() => {
      if (src && !imgError) {
        return (
          <img
            src={src}
            alt={alt}
            className="h-full w-full object-cover"
            onError={() => setImgError(true)}
          />
        );
      }

      if (fallbackIcon) {
        return <span className="flex h-full w-full items-center justify-center">{fallbackIcon}</span>;
      }

      if (displayInitials) {
        return (
          <span className="flex h-full w-full items-center justify-center font-medium text-foreground">
            {displayInitials}
          </span>
        );
      }

      return (
        <span className="flex h-full w-full items-center justify-center">
          <svg
            className="h-1/2 w-1/2 text-muted-foreground"
            fill="currentColor"
            viewBox="0 0 24 24"
          >
            <path d="M24 20.993V24H0v-2.996A14.977 14.977 0 0112.004 15c4.904 0 9.26 2.354 11.996 5.993zM16.002 8.999a4 4 0 11-8 0 4 4 0 018 0z" />
          </svg>
        </span>
      );
    }, [src, imgError, fallbackIcon, displayInitials, alt]);

    return (
      <div
        ref={ref}
        className={cn(
          'relative inline-flex shrink-0 items-center justify-center overflow-hidden bg-muted',
          sizeClasses[size],
          shapeClasses[shape],
          variantClasses[variant],
          onClick && 'cursor-pointer hover:opacity-80 transition-opacity',
          className
        )}
        onClick={onClick}
        role="img"
        aria-label={alt || 'Avatar'}
        {...props}
      >
        {content}
        {status && (
          <span
            className={cn(
              'absolute bottom-0 right-0 h-3 w-3 rounded-full border-2 border-background',
              statusColors[status],
              size === 'xs' && 'h-2 w-2',
              size === 'xl' && 'h-4 w-4'
            )}
            aria-label={`Status: ${status}`}
          />
        )}
      </div>
    );
  }
);

Avatar.displayName = 'Avatar';
