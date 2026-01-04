/**
 * Vector3Input Component
 *
 * A molecule component for editing 3D vector values (Position, Rotation, Scale).
 * Displays three number inputs labeled X, Y, Z with color-coded labels.
 *
 * @example
 * ```tsx
 * <Vector3Input
 *   label="Position"
 *   value={{ x: 0, y: 0, z: 0 }}
 *   onChange={(axis, value) => console.log(`${axis}: ${value}`)}
 *   step={0.1}
 * />
 * ```
 */

import React from 'react';
import { Text } from '../../atoms';
import { NumberInput } from '../NumberInput';
import type { NumberInputProps } from '../NumberInput/NumberInput';

export interface Vector3InputProps {
  /**
   * Label for the input group
   */
  label: string;

  /**
   * Current vector value
   */
  value: {
    x: number;
    y: number;
    z: number;
  };

  /**
   * Callback when an axis value changes
   */
  onChange: (axis: 'x' | 'y' | 'z', newValue: number) => void;

  /**
   * Step value for number inputs
   * @default 0.1
   */
  step?: number;

  /**
   * Whether the inputs are disabled
   * @default false
   */
  disabled?: boolean;

  /**
   * Additional CSS classes
   */
  className?: string;

  /**
   * Color scheme for axis labels
   * @default 'rgb'
   */
  colorScheme?: 'rgb' | 'slate';

  /**
   * Minimum value for inputs
   */
  min?: number;

  /**
   * Maximum value for inputs
   */
  max?: number;

  /**
   * Number of decimal places to display
   */
  precision?: number;
}

export const Vector3Input: React.FC<Vector3InputProps> = ({
  label,
  value,
  onChange,
  step = 0.1,
  disabled = false,
  className = '',
  colorScheme = 'rgb',
  min,
  max,
  precision,
}) => {
  // Get color classes based on axis and color scheme
  const getAxisColorClass = (axis: 'x' | 'y' | 'z'): string => {
    if (colorScheme === 'slate') {
      return 'text-slate-400';
    }

    switch (axis) {
      case 'x':
        return 'text-red-400';
      case 'y':
        return 'text-green-400';
      case 'z':
        return 'text-blue-400';
    }
  };

  // Handle value change for a specific axis
  const handleAxisChange = (axis: 'x' | 'y' | 'z', newValue: number) => {
    if (onChange) {
      onChange(axis, newValue);
    }
  };

  // Apply precision if specified
  const formatValue = (val: number): number => {
    if (precision !== undefined) {
      return parseFloat(val.toFixed(precision));
    }
    return val;
  };

  return (
    <div className={`vector3-input space-y-1 ${className}`}>
      <Text
        variant="caption"
        className="text-slate-400"
      >
        {label}
      </Text>

      <div className="flex gap-2">
        {(['x', 'y', 'z'] as const).map((axis) => (
          <div key={axis} className="flex-1 flex flex-col">
            <Text
              variant="caption"
              weight="semibold"
              className={getAxisColorClass(axis)}
            >
              {axis.toUpperCase()}
            </Text>

            <NumberInput
              value={formatValue(value[axis])}
              onChange={(newValue) => handleAxisChange(axis, newValue)}
              step={step}
              disabled={disabled}
              min={min}
              max={max}
              precision={precision}
              size="sm"
              className="mt-1"
            />
          </div>
        ))}
      </div>
    </div>
  );
};

export default Vector3Input;
