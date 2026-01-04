/**
 * ComponentList Component
 *
 * A molecule component for displaying and managing all components attached to an entity.
 * Provides a list interface with add/remove functionality.
 *
 * @example
 * ```tsx
 * <ComponentList
 *   components={entity.components}
 *   onToggle={(componentId, enabled) => console.log('Toggled:', componentId, enabled)}
 *   onAddComponent={() => console.log('Add component')}
 *   onRemoveComponent={(componentId) => console.log('Remove:', componentId)}
 * />
 * ```
 */

import React from 'react';
import { Text, Button } from '../../atoms';
import { ComponentItem } from '../ComponentItem';
import type { Component } from '../../../types/engine';

export interface ComponentListProps {
  /**
   * Array of components to display
   */
  components: Component[];

  /**
   * Callback when a component is toggled
   */
  onToggle?: (componentId: string, enabled: boolean) => void;

  /**
   * Callback when a component property changes
   */
  onPropertyChange?: (componentId: string, propertyKey: string, value: any) => void;

  /**
   * Callback when adding a new component
   */
  onAddComponent?: () => void;

  /**
   * Callback when removing a component
   */
  onRemoveComponent?: (componentId: string) => void;

  /**
   * Whether property editing is enabled
   * @default false
   */
  allowPropertyEdit?: boolean;

  /**
   * Whether to show add component button
   * @default true
   */
  showAddButton?: boolean;

  /**
   * Whether to show remove buttons
   * @default false
   */
  showRemoveButtons?: boolean;

  /**
   * Text to display when no components are attached
   * @default 'No components attached'
   */
  emptyText?: string;

  /**
   * Additional CSS classes
   */
  className?: string;
}

export const ComponentList: React.FC<ComponentListProps> = ({
  components,
  onToggle,
  onPropertyChange,
  onAddComponent,
  onRemoveComponent,
  allowPropertyEdit = false,
  showAddButton = true,
  showRemoveButtons = false,
  emptyText = 'No components attached',
  className = '',
}) => {
  // Handle component toggle
  const handleToggle = (componentId: string, enabled: boolean) => {
    if (onToggle) {
      onToggle(componentId, enabled);
    }
  };

  // Handle property change
  const handlePropertyChange = (componentId: string, propertyKey: string, value: any) => {
    if (onPropertyChange) {
      onPropertyChange(componentId, propertyKey, value);
    }
  };

  // Handle component removal
  const handleRemove = (componentId: string) => {
    if (onRemoveComponent) {
      onRemoveComponent(componentId);
    }
  };

  return (
    <div className={`component-list space-y-2 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <Text
          variant="caption"
          weight="semibold"
          className="uppercase text-slate-400"
        >
          Components ({components.length})
        </Text>

        {showAddButton && onAddComponent && (
          <Button
            variant="text"
            size="sm"
            onClick={onAddComponent}
            className="text-blue-400 hover:text-blue-300"
          >
            + Add Component
          </Button>
        )}
      </div>

      {/* Component List or Empty State */}
      {components.length === 0 ? (
        <div className="text-center py-4 text-slate-600 text-sm">
          {emptyText}
        </div>
      ) : (
        <div className="space-y-2">
          {components.map((component) => (
            <div key={component.id} className="relative group">
              <ComponentItem
                component={component}
                onToggle={(enabled) => handleToggle(component.id, enabled)}
                onPropertyChange={(key, value) =>
                  handlePropertyChange(component.id, key, value)
                }
                allowPropertyEdit={allowPropertyEdit}
              />

              {/* Remove Button (shown on hover) */}
              {showRemoveButtons && onRemoveComponent && (
                <button
                  onClick={() => handleRemove(component.id)}
                  className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity p-1 hover:bg-red-500/20 rounded"
                  title="Remove component"
                >
                  <svg className="w-4 h-4 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default ComponentList;
