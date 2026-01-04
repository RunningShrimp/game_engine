import React, { useState, useRef, useCallback, useEffect, CSSProperties } from 'react';
import { PanelHandle } from './PanelHandle';
import './ResizablePanel.css';

export interface ResizablePanelProps {
  children: React.ReactNode;
  position?: 'left' | 'right';
  initialWidth?: number;
  minWidth?: number;
  maxWidth?: number;
  className?: string;
  onResize?: (width: number) => void;
  onResizeStart?: () => void;
  onResizeEnd?: () => void;
  resizeDebounce?: number; // Debounce delay in ms
  style?: CSSProperties;
}

export const ResizablePanel: React.FC<ResizablePanelProps> = ({
  children,
  position = 'left',
  initialWidth = 300,
  minWidth = 200,
  maxWidth = 800,
  className = '',
  onResize,
  onResizeStart,
  onResizeEnd,
  resizeDebounce = 0,
  style
}) => {
  const [width, setWidth] = useState(initialWidth);
  const [isDragging, setIsDragging] = useState(false);
  const resizeStartPos = useRef(0);
  const resizeStartWidth = useRef(0);
  const debounceTimerRef = useRef<NodeJS.Timeout | null>(null);

  // Clean up debounce timer on unmount
  useEffect(() => {
    return () => {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }
    };
  }, []);

  const handleResizeStart = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsDragging(true);
    resizeStartPos.current = e.clientX;
    resizeStartWidth.current = width;

    onResizeStart?.();

    // Prevent text selection during drag
    document.body.style.userSelect = 'none';
    document.body.style.cursor = position === 'left' ? 'ew-resize' : 'ew-resize';
  }, [width, position, onResizeStart]);

  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (!isDragging) return;

    const deltaX = position === 'left'
      ? e.clientX - resizeStartPos.current
      : resizeStartPos.current - e.clientX;

    const newWidth = Math.max(
      minWidth,
      Math.min(maxWidth, resizeStartWidth.current + deltaX)
    );

    setWidth(newWidth);

    // Handle debounced resize callback
    if (resizeDebounce > 0) {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }

      debounceTimerRef.current = setTimeout(() => {
        onResize?.(newWidth);
      }, resizeDebounce);
    } else {
      onResize?.(newWidth);
    }
  }, [isDragging, position, minWidth, maxWidth, onResize, resizeDebounce]);

  const handleMouseUp = useCallback(() => {
    if (!isDragging) return;

    setIsDragging(false);

    // Restore document styles
    document.body.style.userSelect = '';
    document.body.style.cursor = '';

    // Final resize callback
    onResize?.(width);
    onResizeEnd?.();
  }, [isDragging, width, onResize, onResizeEnd]);

  // Attach global mouse event listeners
  useEffect(() => {
    if (isDragging) {
      window.addEventListener('mousemove', handleMouseMove);
      window.addEventListener('mouseup', handleMouseUp);

      return () => {
        window.removeEventListener('mousemove', handleMouseMove);
        window.removeEventListener('mouseup', handleMouseUp);
      };
    }
  }, [isDragging, handleMouseMove, handleMouseUp]);

  const panelStyle: CSSProperties = {
    ...style,
    width: `${width}px`,
    minWidth: `${minWidth}px`,
    maxWidth: `${maxWidth}px`
  };

  return (
    <div
      className={`resizable-panel resizable-panel-${position} ${
        isDragging ? 'dragging' : ''
      } ${className}`}
      style={panelStyle}
    >
      {position === 'right' && (
        <PanelHandle
          position="right"
          isDragging={isDragging}
          onDragStart={handleResizeStart}
        />
      )}
      <div className="resizable-panel-content">
        {children}
      </div>
      {position === 'left' && (
        <PanelHandle
          position="left"
          isDragging={isDragging}
          onDragStart={handleResizeStart}
        />
      )}
    </div>
  );
};
