import React from 'react';
import { Entity } from '../../../../types/engine';
import EntityTreeIcon from '../EntityTreeIcon';
import EntityTreeToggle from '../EntityTreeToggle';
import EntityTreeDragHandle from '../EntityTreeDragHandle';
import EntityTreeLabel from '../EntityTreeLabel';
import EntityTreeActions from '../EntityTreeActions';

export interface EntityTreeItemProps {
  entity: Entity;
  depth: number;
  isSelected: boolean;
  isExpanded: boolean;
  isEditing: boolean;
  editName: string;
  isDragTarget: boolean;
  isDragging: boolean;
  onToggleExpand: (entityId: string) => void;
  onEntityClick: (e: React.MouseEvent, entityId: string) => void;
  onContextMenu: (e: React.MouseEvent, entityId: string) => void;
  onDragStart: (e: React.DragEvent, entityId: string) => void;
  onDragOver: (e: React.DragEvent, entityId: string) => void;
  onDragLeave: () => void;
  onDrop: (e: React.DragEvent, entityId: string) => void;
  onStartEditing: (entity: Entity) => void;
  onEditNameChange: (name: string) => void;
  onEditFinish: () => void;
  onEditCancel: () => void;
  onToggleVisibility: (e: React.MouseEvent, entityId: string) => void;
  onToggleLock: (e: React.MouseEvent, entityId: string) => void;
}

/**
 * EntityTreeItem - Single entity tree item component
 * Renders a single entity in the hierarchy tree with all its controls
 */
export const EntityTreeItem: React.FC<EntityTreeItemProps> = ({
  entity,
  depth,
  isSelected,
  isExpanded,
  isEditing,
  editName,
  isDragTarget,
  isDragging,
  onToggleExpand,
  onEntityClick,
  onContextMenu,
  onDragStart,
  onDragOver,
  onDragLeave,
  onDrop,
  onStartEditing,
  onEditNameChange,
  onEditFinish,
  onEditCancel,
  onToggleVisibility,
  onToggleLock,
}) => {
  const hasChildren = entity.children.length > 0;

  return (
    <div>
      <div
        className={`flex items-center gap-1 py-1 px-2 hover:bg-slate-700 cursor-pointer relative ${
          isSelected ? 'bg-blue-600' : ''
        } ${isDragTarget ? 'border-t-2 border-blue-400' : ''} ${
          isDragging ? 'opacity-50' : ''
        }`}
        style={{ paddingLeft: `${depth * 16 + 8}px` }}
        draggable={!entity.locked}
        onDragStart={(e) => onDragStart(e, entity.id)}
        onDragOver={(e) => onDragOver(e, entity.id)}
        onDragLeave={onDragLeave}
        onDrop={(e) => onDrop(e, entity.id)}
        onClick={(e) => onEntityClick(e, entity.id)}
        onContextMenu={(e) => onContextMenu(e, entity.id)}
        role="treeitem"
        aria-selected={isSelected}
        aria-expanded={hasChildren ? isExpanded : undefined}
        aria-level={depth + 1}
      >
        {/* Expand/Collapse Toggle */}
        {hasChildren ? (
          <EntityTreeToggle
            expanded={isExpanded}
            onToggle={() => onToggleExpand(entity.id)}
          />
        ) : (
          <EntityTreeDragHandle />
        )}

        {/* Entity Icon */}
        <EntityTreeIcon />

        {/* Entity Name Label */}
        <EntityTreeLabel
          name={entity.name}
          isEditing={isEditing}
          editName={editName}
          onEditNameChange={onEditNameChange}
          onEditFinish={onEditFinish}
          onEditCancel={onEditCancel}
          onDoubleClick={() => onStartEditing(entity)}
        />

        {/* Action Buttons */}
        <EntityTreeActions
          visible={entity.visible}
          locked={entity.locked}
          onToggleVisibility={(e) => {
            e.stopPropagation();
            onToggleVisibility(e, entity.id);
          }}
          onToggleLock={(e) => {
            e.stopPropagation();
            onToggleLock(e, entity.id);
          }}
        />
      </div>

      {/* Children */}
      {hasChildren && isExpanded && (
        <div role="group">
          {entity.children.map((child) => (
            <EntityTreeItemWrapper
              key={child.id}
              entity={child}
              depth={depth + 1}
              // Pass down all handlers - they will be provided by parent wrapper
              {...{
                onToggleExpand,
                onEntityClick,
                onContextMenu,
                onDragStart,
                onDragOver,
                onDragLeave,
                onDrop,
                onStartEditing,
                onEditNameChange,
                onEditFinish,
                onEditCancel,
                onToggleVisibility,
                onToggleLock,
              }}
            />
          ))}
        </div>
      )}
    </div>
  );
};

// Wrapper component to handle recursive rendering with state
interface EntityTreeItemWrapperProps {
  entity: Entity;
  depth: number;
  onToggleExpand: (entityId: string) => void;
  onEntityClick: (e: React.MouseEvent, entityId: string) => void;
  onContextMenu: (e: React.MouseEvent, entityId: string) => void;
  onDragStart: (e: React.DragEvent, entityId: string) => void;
  onDragOver: (e: React.DragEvent, entityId: string) => void;
  onDragLeave: () => void;
  onDrop: (e: React.DragEvent, entityId: string) => void;
  onStartEditing: (entity: Entity) => void;
  onEditNameChange: (name: string) => void;
  onEditFinish: () => void;
  onEditCancel: () => void;
  onToggleVisibility: (e: React.MouseEvent, entityId: string) => void;
  onToggleLock: (e: React.MouseEvent, entityId: string) => void;
}

const EntityTreeItemWrapper: React.FC<EntityTreeItemWrapperProps> = (props) => {
  // These would come from the parent component's state
  // For now, we'll use placeholder values - the parent should manage this state
  const isSelected = false; // Would come from props
  const isExpanded = false; // Would come from props
  const isEditing = false; // Would come from props
  const editName = ''; // Would come from props
  const isDragTarget = false; // Would come from props
  const isDragging = false; // Would come from props

  return (
    <EntityTreeItem
      {...props}
      isSelected={isSelected}
      isExpanded={isExpanded}
      isEditing={isEditing}
      editName={editName}
      isDragTarget={isDragTarget}
      isDragging={isDragging}
    />
  );
};

export default EntityTreeItem;
