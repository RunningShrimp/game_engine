import React, { useState, useRef, useEffect } from 'react';
import { cn } from '../../utils/cn';

/**
 * Tooltip position
 */
export type TooltipPosition = 'top' | 'bottom' | 'left' | 'right';

/**
 * Tooltip component properties
 */
export interface TooltipProps {
  /** Tooltip content */
  content: React.ReactNode;
  /** Tooltip position */
  position?: TooltipPosition;
  /** Trigger element */
  children: React.ReactElement;
  /** Delay in milliseconds before showing */
  delay?: number;
  /** Whether tooltip is disabled */
  disabled?: boolean;
  /** Additional CSS classes for tooltip */
  className?: string;
  /** Maximum width of tooltip */
  maxWidth?: number;
  /** Whether to show arrow indicator */
  showArrow?: boolean;
}

/**
 * Tooltip component - Displays additional information on hover
 *
 * @example
 * ```tsx
 * <Tooltip content="This is a tooltip">
 *   <button>Hover me</button>
 * </Tooltip>
 * <Tooltip content="Help text" position="right">
 *   <Icon name="HelpCircle" />
 * </Tooltip>
 * <Tooltip content="Disabled" disabled>
 *   <button>No tooltip</button>
 * </Tooltip>
 * ```
 */
export const Tooltip: React.FC<TooltipProps> = ({
  content,
  position = 'top',
  delay = 200,
  disabled = false,
  className,
  maxWidth = 200,
  showArrow = true,
  children,
}) => {
  const [isVisible, setIsVisible] = useState(false);
  const [calculatedPosition, setCalculatedPosition] = useState(position);
  const triggerRef = useRef<HTMLElement>(null);
  const tooltipRef = useRef<HTMLDivElement>(null);
  const timeoutRef = useRef<NodeJS.Timeout>();

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  const handleMouseEnter = () => {
    if (disabled) return;

    timeoutRef.current = setTimeout(() => {
      setIsVisible(true);
    }, delay);
  };

  const handleMouseLeave = () => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    setIsVisible(false);
  };

  const positionClasses: Record<TooltipPosition, string> = {
    top: 'bottom-full left-1/2 -translate-x-1/2 mb-2',
    bottom: 'top-full left-1/2 -translate-x-1/2 mt-2',
    left: 'right-full top-1/2 -translate-y-1/2 mr-2',
    right: 'left-full top-1/2 -translate-y-1/2 ml-2',
  };

  const arrowClasses: Record<TooltipPosition, string> = {
    top: 'top-full left-1/2 -translate-x-1/2 border-t-muted-foreground border-l-transparent border-r-transparent border-b-transparent',
    bottom: 'bottom-full left-1/2 -translate-x-1/2 border-b-muted-foreground border-l-transparent border-r-transparent border-t-transparent',
    left: 'left-full top-1/2 -translate-y-1/2 border-l-muted-foreground border-t-transparent border-b-transparent border-r-transparent',
    right: 'right-full top-1/2 -translate-y-1/2 border-r-muted-foreground border-t-transparent border-b-transparent border-l-transparent',
  };

  const triggerElement = React.cloneElement(children, {
    ref: triggerRef,
    onMouseEnter: handleMouseEnter,
    onMouseLeave: handleMouseLeave,
    onFocus: handleMouseEnter,
    onBlur: handleMouseLeave,
    'aria-describedby': isVisible ? 'tooltip-content' : undefined,
  });

  return (
    <>
      {triggerElement}
      {isVisible && !disabled && (
        <div
          ref={tooltipRef}
          role="tooltip"
          id="tooltip-content"
          className={cn(
            'absolute z-50 px-2.5 py-1.5 text-xs text-white bg-muted-foreground rounded-md shadow-lg pointer-events-none',
            positionClasses[calculatedPosition],
            className
          )}
          style={{ maxWidth: `${maxWidth}px` }}
        >
          {content}
          {showArrow && (
            <div
              className={cn(
                'absolute w-0 h-0 border-4',
                arrowClasses[calculatedPosition]
              )}
            />
          )}
        </div>
      )}
    </>
  );
};

Tooltip.displayName = 'Tooltip';
