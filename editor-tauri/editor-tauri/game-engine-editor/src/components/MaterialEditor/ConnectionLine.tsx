/**
 * Connection Line Component
 * Renders a Bézier curve connection between two nodes
 */

import React, { useRef, useEffect } from 'react';
import './ConnectionLine.css';

interface ConnectionLineProps {
  from: { x: number; y: number };
  to: { x: number; y: number };
  isTemporary?: boolean;
  onRemove?: () => void;
}

export const ConnectionLine: React.FC<ConnectionLineProps> = ({
  from,
  to,
  isTemporary = false,
  onRemove,
}) => {
  const pathRef = useRef<SVGPathElement>(null);

  // Calculate control points for smooth Bézier curve
  const calculateControlPoints = () => {
    const dx = Math.abs(to.x - from.x);
    const controlOffset = Math.max(dx * 0.5, 50);

    return {
      cp1x: from.x + controlOffset,
      cp1y: from.y,
      cp2x: to.x - controlOffset,
      cp2y: to.y,
    };
  };

  const { cp1x, cp1y, cp2x, cp2y } = calculateControlPoints();

  // Generate path data
  const pathData = `M ${from.x} ${from.y} C ${cp1x} ${cp1y}, ${cp2x} ${cp2y}, ${to.x} ${to.y}`;

  // Handle right-click to remove connection
  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    if (onRemove) {
      onRemove();
    }
  };

  return (
    <g className={`connection-line ${isTemporary ? 'temporary' : ''}`}>
      <path
        ref={pathRef}
        d={pathData}
        className="connection-path"
        onContextMenu={handleContextMenu}
      />
      {!isTemporary && onRemove && (
        <circle
          cx={(from.x + to.x) / 2}
          cy={(from.y + to.y) / 2}
          r="6"
          className="connection-remove-button"
          onClick={onRemove}
        />
      )}
    </g>
  );
};

export default ConnectionLine;
