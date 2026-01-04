/**
 * EntityInfo Component
 *
 * A molecule component for displaying and editing basic entity information.
 * Shows entity name, ID, visibility, and locked status with inline editing capabilities.
 *
 * @example
 * ```tsx
 * <EntityInfo
 *   entity={selectedEntity}
 *   onNameChange={(newName) => console.log('Name changed:', newName)}
 * />
 * ```
 */

import React, { useState, useEffect, KeyboardEvent } from 'react';
import { Text } from '../../atoms';
import { Input } from '../Input';
import { Checkbox } from '../Checkbox';
import type { Entity } from '../../../types/engine';

export interface EntityInfoProps {
  /**
   * The entity to display information for
   */
  entity: Entity;

  /**
   * Callback when entity name changes
   */
  onNameChange?: (newName: string) => void;

  /**
   * Whether name editing is enabled
   * @default true
   */
  allowNameEdit?: boolean;

  /**
   * Additional CSS classes
   */
  className?: string;

  /**
   * Whether to show entity ID
   * @default true
   */
  showId?: boolean;

  /**
   * Whether to show visibility toggle
   * @default true
   */
  showVisibility?: boolean;

  /**
   * Whether to show locked toggle
   * @default true
   */
  showLocked?: boolean;
}

export const EntityInfo: React.FC<EntityInfoProps> = ({
  entity,
  onNameChange,
  allowNameEdit = true,
  className = '',
  showId = true,
  showVisibility = true,
  showLocked = true,
}) => {
  const [entityName, setEntityName] = useState(entity.name);
  const [isEditingName, setIsEditingName] = useState(false);

  // Update local state when entity changes
  useEffect(() => {
    setEntityName(entity.name);
    setIsEditingName(false);
  }, [entity.id, entity.name]);

  // Handle name submission
  const handleNameSubmit = () => {
    const trimmedName = entityName.trim();
    if (onNameChange && trimmedName !== '' && trimmedName !== entity.name) {
      onNameChange(trimmedName);
    } else {
      setEntityName(entity.name);
    }
    setIsEditingName(false);
  };

  // Handle keyboard events during name editing
  const handleNameKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      handleNameSubmit();
    } else if (e.key === 'Escape') {
      setEntityName(entity.name);
      setIsEditingName(false);
    }
  };

  return (
    <div className={`entity-info space-y-2 ${className}`}>
      <Text
        variant="caption"
        weight="semibold"
        className="uppercase text-slate-400"
      >
        Entity
      </Text>

      <div className="space-y-2">
        {/* Name Field */}
        <div>
          <Text variant="caption" className="text-slate-400 block mb-1">
            Name
          </Text>

          {allowNameEdit && isEditingName ? (
            <Input
              value={entityName}
              onChange={(e) => setEntityName(e.target.value)}
              onKeyDown={handleNameKeyDown}
              onBlur={handleNameSubmit}
              autoFocus
              size="sm"
              className="w-full"
            />
          ) : (
            <div
              className={`
                w-full px-2 py-1 text-sm bg-slate-800 border
                border-slate-700 rounded text-slate-200
                ${allowNameEdit ? 'cursor-pointer hover:border-slate-600' : 'cursor-default'}
              `}
              onClick={() => allowNameEdit && setIsEditingName(true)}
            >
              {entity.name}
            </div>
          )}
        </div>

        {/* ID Field */}
        {showId && (
          <div>
            <Text variant="caption" className="text-slate-400 block mb-1">
              ID
            </Text>
            <Input
              value={entity.id}
              disabled
              size="sm"
              className="w-full cursor-not-allowed"
            />
          </div>
        )}

        {/* Visibility and Locked Toggles */}
        {(showVisibility || showLocked) && (
          <div className="flex gap-2">
            {/* Visibility */}
            {showVisibility && (
              <div className="flex-1">
                <Text variant="caption" className="text-slate-400 block mb-1">
                  Visible
                </Text>
                <div className="mt-1 flex items-center gap-2">
                  <Checkbox
                    checked={entity.visible}
                    disabled
                    size="sm"
                  />
                  <Text variant="caption" className="text-slate-500">
                    {entity.visible ? 'Yes' : 'No'}
                  </Text>
                </div>
              </div>
            )}

            {/* Locked */}
            {showLocked && (
              <div className="flex-1">
                <Text variant="caption" className="text-slate-400 block mb-1">
                  Locked
                </Text>
                <div className="mt-1 flex items-center gap-2">
                  <Checkbox
                    checked={entity.locked}
                    disabled
                    size="sm"
                  />
                  <Text variant="caption" className="text-slate-500">
                    {entity.locked ? 'Yes' : 'No'}
                  </Text>
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default EntityInfo;
