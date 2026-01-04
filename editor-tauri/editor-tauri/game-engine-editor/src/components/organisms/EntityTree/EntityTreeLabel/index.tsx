import React from 'react';

export interface EntityTreeLabelProps {
  name: string;
  isEditing: boolean;
  editName: string;
  onEditNameChange: (name: string) => void;
  onEditFinish: () => void;
  onEditCancel: () => void;
  onDoubleClick: () => void;
  className?: string;
}

/**
 * EntityTreeLabel - Entity name label component
 * Displays entity name with inline editing capability
 */
export const EntityTreeLabel: React.FC<EntityTreeLabelProps> = ({
  name,
  isEditing,
  editName,
  onEditNameChange,
  onEditFinish,
  onEditCancel,
  onDoubleClick,
  className = '',
}) => {
  if (isEditing) {
    return (
      <input
        type="text"
        value={editName}
        onChange={(e) => onEditNameChange(e.target.value)}
        onBlur={onEditFinish}
        onKeyDown={(e) => {
          if (e.key === 'Enter') onEditFinish();
          if (e.key === 'Escape') onEditCancel();
        }}
        autoFocus
        className={`flex-1 bg-slate-600 text-white text-sm px-1 py-0.5 rounded ${className}`}
        onClick={(e) => e.stopPropagation()}
        aria-label="Edit entity name"
      />
    );
  }

  return (
    <span
      className={`flex-1 text-sm text-slate-200 truncate ${className}`}
      onDoubleClick={(e) => {
        e.stopPropagation();
        onDoubleClick();
      }}
    >
      {name}
    </span>
  );
};

export default EntityTreeLabel;
