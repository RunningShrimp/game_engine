/**
 * TransformEditor Component
 *
 * A molecule component for editing entity transform properties (Position, Rotation, Scale).
 * Provides a comprehensive interface for modifying 3D transforms with coordinate space options.
 *
 * @example
 * ```tsx
 * <TransformEditor
 *   transform={entity.transform}
 *   onChange={(newTransform) => console.log(newTransform)}
 *   coordinateSpace="world"
 * />
 * ```
 */

import React from 'react';
import { Text, Divider } from '../../atoms';
import { Vector3Input } from '../Vector3Input';
import type { Vector3InputProps } from '../Vector3Input';
import type { Transform } from '../../../types/engine';

export interface TransformEditorProps {
  /**
   * Current transform value
   */
  transform: Transform;

  /**
   * Callback when transform changes
   */
  onChange: (transform: Transform) => void;

  /**
   * Coordinate space for editing
   * @default 'world'
   */
  coordinateSpace?: 'world' | 'local';

  /**
   * Whether the editor is disabled
   * @default false
   */
  disabled?: boolean;

  /**
   * Additional CSS classes
   */
  className?: string;

  /**
   * Step values for position, rotation, scale
   */
  stepValues?: {
    position?: number;
    rotation?: number;
    scale?: number;
  };

  /**
   * Whether to show coordinate space toggle
   * @default false
   */
  showCoordinateSpace?: boolean;
}

export const TransformEditor: React.FC<TransformEditorProps> = ({
  transform,
  onChange,
  coordinateSpace = 'world',
  disabled = false,
  className = '',
  stepValues,
  showCoordinateSpace = false,
}) => {
  // Handle position change
  const handlePositionChange = (axis: 'x' | 'y' | 'z', newValue: number) => {
    onChange({
      ...transform,
      position: { ...transform.position, [axis]: newValue },
    });
  };

  // Handle rotation change
  const handleRotationChange = (axis: 'x' | 'y' | 'z', newValue: number) => {
    onChange({
      ...transform,
      rotation: { ...transform.rotation, [axis]: newValue },
    });
  };

  // Handle scale change
  const handleScaleChange = (axis: 'x' | 'y' | 'z', newValue: number) => {
    onChange({
      ...transform,
      scale: { ...transform.scale, [axis]: newValue },
    });
  };

  return (
    <div className={`transform-editor space-y-3 ${className}`}>
      {/* Header with coordinate space indicator */}
      <div className="flex items-center justify-between">
        <Text
          variant="caption"
          weight="semibold"
          className="uppercase text-slate-400"
        >
          Transform
        </Text>

        {(showCoordinateSpace || coordinateSpace) && (
          <Text
            variant="caption"
            className="text-slate-600 capitalize"
          >
            {coordinateSpace}
          </Text>
        )}
      </div>

      <Divider />

      {/* Position */}
      <Vector3Input
        label="Position"
        value={transform.position}
        onChange={handlePositionChange}
        step={stepValues?.position ?? 0.1}
        disabled={disabled}
      />

      {/* Rotation */}
      <Vector3Input
        label="Rotation (Euler)"
        value={{
          x: transform.rotation.x,
          y: transform.rotation.y,
          z: transform.rotation.z,
        }}
        onChange={handleRotationChange}
        step={stepValues?.rotation ?? 1}
        disabled={disabled}
      />

      {/* Scale */}
      <Vector3Input
        label="Scale"
        value={transform.scale}
        onChange={handleScaleChange}
        step={stepValues?.scale ?? 0.01}
        disabled={disabled}
      />
    </div>
  );
};

export default TransformEditor;
