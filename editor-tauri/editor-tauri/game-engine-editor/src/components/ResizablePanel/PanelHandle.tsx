import React, { useState } from 'react';
import './ResizablePanel.css';

export interface PanelHandleProps {
  position: 'left' | 'right';
  isDragging: boolean;
  onDragStart: (e: React.MouseEvent) => void;
}

export const PanelHandle: React.FC<PanelHandleProps> = ({
  position,
  isDragging,
  onDragStart
}) => {
  const [isHovered, setIsHovered] = useState(false);

  return (
    <div
      className={`resizable-panel-handle resizable-panel-handle-${position} ${
        isDragging ? 'dragging' : ''
      } ${isHovered ? 'hovered' : ''}`}
      onMouseDown={onDragStart}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      <div className="handle-grip">
        <div className="grip-line"></div>
        <div className="grip-line"></div>
        <div className="grip-line"></div>
      </div>
    </div>
  );
};
