/**
 * Connection Line Component
 * Renders Bezier curves connecting parent and child nodes in behavior tree
 */

import React from 'react';

interface ConnectionLineProps {
  from: { x: number; y: number };
  to: { x: number; y: number };
  isTemporary?: boolean;
  isHighlighted?: boolean;
  status?: 'idle' | 'running' | 'success' | 'failure';
}

export const ConnectionLine: React.FC<ConnectionLineProps> = ({
  from,
  to,
  isTemporary = false,
  isHighlighted = false,
  status = 'idle',
}) => {
  // Calculate Bezier curve control points
  const controlOffset = Math.abs(to.y - from.y) * 0.5;

  const pathD = `
    M ${from.x} ${from.y}
    C ${from.x} ${from.y + controlOffset},
      ${to.x} ${to.y - controlOffset},
      ${to.x} ${to.y}
  `;

  // Determine stroke color based on status
  const getStrokeColor = () => {
    if (isTemporary) return 'rgba(255, 255, 255, 0.5)';
    if (isHighlighted) return '#00ff88';

    switch (status) {
      case 'running':
        return '#ffcc00';
      case 'success':
        return '#00ff88';
      case 'failure':
        return '#ff4444';
      default:
        return 'rgba(255, 255, 255, 0.3)';
    }
  };

  const getStrokeWidth = () => {
    if (isHighlighted || status !== 'idle') return 3;
    return 2;
  };

  return (
    <g className="connection-line">
      <path
        d={pathD}
        stroke={getStrokeColor()}
        strokeWidth={getStrokeWidth()}
        fill="none"
        strokeDasharray={isTemporary ? '5,5' : undefined}
        style={{
          transition: 'stroke 0.3s, stroke-width 0.3s',
        }}
      />

      {/* Arrow head */}
      {!isTemporary && (
        <polygon
          points={`
            ${to.x},
            ${to.y}
            ${to.x - 6},
            ${to.y - 10}
            ${to.x + 6},
            ${to.y - 10}
          `}
          fill={getStrokeColor()}
          style={{
            transition: 'fill 0.3s',
          }}
        />
      )}
    </g>
  );
};

export default ConnectionLine;
