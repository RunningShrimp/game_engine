/**
 * ComponentItem Component
 *
 * A molecule component for displaying and managing a single component attached to an entity.
 * Shows component name, type, enabled state, and properties in a collapsible panel.
 *
 * @example
 * ```tsx
 * <ComponentItem
 *   component={meshRendererComponent}
 *   onToggle={(enabled) => console.log('Component toggled:', enabled)}
 *   onPropertyChange={(key, value) => console.log('Property changed:', key, value)}
 * />
 * ```
 */

import React, { useState } from 'react';
import { Text, Icon, Badge } from '../../atoms';
import { Checkbox } from '../Checkbox';
import type { Component } from '../../../types/engine';

export interface ComponentItemProps {
  /**
   * The component to display
   */
  component: Component;

  /**
   * Callback when component enabled state is toggled
   */
  onToggle?: (enabled: boolean) => void;

  /**
   * Callback when a property value changes
   */
  onPropertyChange?: (propertyKey: string, value: any) => void;

  /**
   * Whether property editing is enabled
   * @default false
   */
  allowPropertyEdit?: boolean;

  /**
   * Initial expanded state
   * @default true
   */
  defaultExpanded?: boolean;

  /**
   * Additional CSS classes
   */
  className?: string;

  /**
   * Whether to show component type badge
   * @default true
   */
  showTypeBadge?: boolean;

  /**
   * Whether to show the enabled checkbox
   * @default true
   */
  showEnabledToggle?: boolean;
}

export const ComponentItem: React.FC<ComponentItemProps> = ({
  component,
  onToggle,
  onPropertyChange,
  allowPropertyEdit = false,
  defaultExpanded = true,
  className = '',
  showTypeBadge = true,
  showEnabledToggle = true,
}) => {
  const [isExpanded, setIsExpanded] = useState(defaultExpanded);

  // Handle property value change
  const handlePropertyChange = (key: string, currentValue: any, newValue: any) => {
    if (onPropertyChange && allowPropertyEdit) {
      onPropertyChange(key, newValue);
    }
  };

  // Render property input based on value type
  const renderPropertyValue = (key: string, value: any) => {
    const inputProps = {
      value,
      onChange: (newValue: any) => handlePropertyChange(key, value, newValue),
    };

    if (!allowPropertyEdit) {
      // Read-only display
      return (
        <div className="text-xs text-slate-500 truncate">
          {formatValue(value)}
        </div>
      );
    }

    // Editable input based on type
    switch (typeof value) {
      case 'boolean':
        return <Checkbox {...inputProps} checked={value} size="sm" />;

      case 'number':
        return (
          <input
            type="number"
            value={value}
            onChange={(e) => inputProps.onChange(parseFloat(e.target.value) || 0)}
            className="w-full px-2 py-0.5 text-xs bg-slate-900 border border-slate-700 rounded text-slate-200 focus:border-blue-500 focus:outline-none"
          />
        );

      case 'string':
        return (
          <input
            type="text"
            value={value}
            onChange={(e) => inputProps.onChange(e.target.value)}
            className="w-full px-2 py-0.5 text-xs bg-slate-900 border border-slate-700 rounded text-slate-200 focus:border-blue-500 focus:outline-none"
          />
        );

      default:
        return (
          <div className="text-xs text-slate-500 truncate">
            {formatValue(value)}
          </div>
        );
    }
  };

  // Format value for display
  const formatValue = (value: any): string => {
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (typeof value === 'boolean') return value ? 'true' : 'false';
    if (typeof value === 'object') return JSON.stringify(value);
    return String(value);
  };

  const propertyCount = Object.keys(component.properties).length;

  return (
    <div className={`component-item bg-slate-800 rounded border border-slate-700 overflow-hidden ${className}`}>
      {/* Component Header */}
      <div className="flex items-center justify-between p-2 cursor-pointer hover:bg-slate-700 transition-colors">
        <div
          className="flex items-center gap-2 flex-1"
          onClick={() => setIsExpanded(!isExpanded)}
        >
          {/* Expand/Collapse Icon */}
          <Icon
            name="chevron-right"
            size="xs"
            className={`text-slate-400 transition-transform ${isExpanded ? 'rotate-90' : ''}`}
          />

          {/* Component Name */}
          <Text
            variant="body2"
            weight="medium"
            className="text-slate-200"
          >
            {component.name}
          </Text>

          {/* Type Badge */}
          {showTypeBadge && (
            <Badge variant="secondary" size="sm">
              {component.type}
            </Badge>
          )}
        </div>

        {/* Enabled Toggle */}
        {showEnabledToggle && onToggle && (
          <div
            className="flex items-center gap-1"
            onClick={(e) => e.stopPropagation()}
          >
            <Checkbox
              checked={component.enabled}
              onChange={(checked) => onToggle(checked)}
              size="sm"
              label="Enabled"
            />
          </div>
        )}
      </div>

      {/* Component Properties */}
      {isExpanded && (
        <div className="px-2 pb-2 space-y-1">
          {propertyCount === 0 ? (
            <div className="text-xs text-slate-600 py-2 px-1">
              No properties
            </div>
          ) : (
            Object.entries(component.properties).map(([key, value]) => (
              <div key={key} className="flex items-center gap-2 py-1">
                <Text
                  variant="caption"
                  className="text-slate-400 w-1/2 truncate"
                >
                  {key}
                </Text>
                <div className="flex-1">
                  {renderPropertyValue(key, value)}
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
};

export default ComponentItem;
