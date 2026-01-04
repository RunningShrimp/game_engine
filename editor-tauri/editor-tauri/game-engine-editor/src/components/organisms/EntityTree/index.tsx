import React, { useState, useMemo, useCallback } from 'react';
import { Entity } from '../../types/engine';
import EntityTreeSearch from './EntityTreeSearch';
import EntityTreeContextMenu, { ContextMenuItem } from './EntityTreeContextMenu';

// Import EntityTreeItem - we'll need to refactor it slightly to work with the parent state
// For now, we'll use the original component structure with improved organization

export interface EntityTreeProps {
  entities: Entity[];
  selectedEntities: string[];
  onEntitySelect: (entityIds: string[]) => void;
  onEntityRename: (entityId: string, newName: string) => void;
  onEntityDelete: (entityId: string) => void;
  onEntityCreate: () => void;
  onEntityToggleVisibility: (entityId: string) => void;
  onEntityToggleLock: (entityId: string) => void;
  onEntityReorder?: (entityId: string, newIndex: number) => void;
  onEntityReparent?: (entityId: string, newParentId: string | null) => void;
  className?: string;
}

interface ContextMenuState {
  visible: boolean;
  x: number;
  y: number;
  entityId: string | null;
}

/**
 * EntityTree - Main entity hierarchy tree component
 * Organized organism component using atomic sub-components
 */
export const EntityTree: React.FC<EntityTreeProps> = ({
  entities,
  selectedEntities,
  onEntitySelect,
  onEntityRename,
  onEntityDelete,
  onEntityCreate,
  onEntityToggleVisibility,
  onEntityToggleLock,
  onEntityReorder,
  onEntityReparent,
  className = '',
}) => {
  // State management
  const [expandedEntities, setExpandedEntities] = useState<Set<string>>(new Set());
  const [editingEntity, setEditingEntity] = useState<string | null>(null);
  const [editName, setEditName] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [contextMenu, setContextMenu] = useState<ContextMenuState>({
    visible: false,
    x: 0,
    y: 0,
    entityId: null,
  });
  const [draggedEntity, setDraggedEntity] = useState<string | null>(null);
  const [dropTarget, setDropTarget] = useState<string | null>(null);

  // Filter entities based on search query
  const filteredEntities = useMemo(() => {
    if (!searchQuery.trim()) {
      return entities;
    }

    const filterTree = (entityList: Entity[]): Entity[] => {
      return entityList.reduce((acc: Entity[], entity) => {
        const matchesSearch = entity.name
          .toLowerCase()
          .includes(searchQuery.toLowerCase());
        const hasMatchingChildren = entity.children.length > 0;

        if (matchesSearch || hasMatchingChildren) {
          acc.push({
            ...entity,
            children: hasMatchingChildren ? filterTree(entity.children) : [],
          });
        }

        return acc;
      }, []);
    };

    return filterTree(entities);
  }, [entities, searchQuery]);

  // Expand entities that contain matched children
  useMemo(() => {
    if (searchQuery.trim()) {
      const expandMatching = (entityList: Entity[]) => {
        entityList.forEach((entity) => {
          const hasMatchingChildren = entity.children.some(
            (child) =>
              child.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
              child.children.length > 0
          );

          if (hasMatchingChildren) {
            setExpandedEntities((prev) => new Set([...prev, entity.id]));
            expandMatching(entity.children);
          }
        });
      };

      expandMatching(filteredEntities);
    }
  }, [searchQuery, filteredEntities]);

  // Context menu items
  const contextMenuItems: ContextMenuItem[] = [
    {
      id: 'rename',
      label: 'Rename',
      icon: (
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"
          />
        </svg>
      ),
      onClick: () => {
        if (contextMenu.entityId) {
          const entity = findEntity(entities, contextMenu.entityId);
          if (entity) startEditing(entity);
        }
      },
    },
    {
      id: 'duplicate',
      label: 'Duplicate',
      icon: (
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
          />
        </svg>
      ),
      onClick: () => {
        if (contextMenu.entityId) {
          console.log('Duplicate entity:', contextMenu.entityId);
          // TODO: Implement duplication
        }
      },
    },
    {
      id: 'toggle-visibility',
      label: 'Toggle Visibility',
      icon: (
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
          />
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
          />
        </svg>
      ),
      onClick: () => {
        if (contextMenu.entityId) {
          onEntityToggleVisibility(contextMenu.entityId);
        }
      },
    },
    {
      id: 'delete',
      label: 'Delete',
      icon: (
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
          />
        </svg>
      ),
      onClick: () => {
        if (contextMenu.entityId) {
          onEntityDelete(contextMenu.entityId);
        }
      },
      danger: true,
    },
  ];

  // Helper function to find entity by ID
  const findEntity = (entities: Entity[], entityId: string): Entity | null => {
    for (const entity of entities) {
      if (entity.id === entityId) return entity;
      if (entity.children.length > 0) {
        const found = findEntity(entity.children, entityId);
        if (found) return found;
      }
    }
    return null;
  };

  // Helper to get all entity IDs in order
  const getAllEntityIds = useCallback(
    (entities: Entity[], ids: string[] = []): string[] => {
      entities.forEach((entity) => {
        ids.push(entity.id);
        if (entity.children.length > 0) {
          getAllEntityIds(entity.children, ids);
        }
      });
      return ids;
    },
    []
  );

  // Toggle expand state
  const toggleExpand = useCallback((entityId: string) => {
    setExpandedEntities((prev) => {
      const newExpanded = new Set(prev);
      if (newExpanded.has(entityId)) {
        newExpanded.delete(entityId);
      } else {
        newExpanded.add(entityId);
      }
      return newExpanded;
    });
  }, []);

  // Start editing entity name
  const startEditing = useCallback((entity: Entity) => {
    setEditingEntity(entity.id);
    setEditName(entity.name);
    setContextMenu({ visible: false, x: 0, y: 0, entityId: null });
  }, []);

  // Finish editing
  const finishEditing = useCallback(() => {
    if (editingEntity && editName.trim()) {
      onEntityRename(editingEntity, editName.trim());
    }
    setEditingEntity(null);
    setEditName('');
  }, [editingEntity, editName, onEntityRename]);

  // Cancel editing
  const cancelEditing = useCallback(() => {
    setEditingEntity(null);
    setEditName('');
  }, []);

  // Handle context menu
  const handleContextMenu = useCallback(
    (e: React.MouseEvent, entityId: string) => {
      e.preventDefault();
      e.stopPropagation();

      if (!selectedEntities.includes(entityId)) {
        onEntitySelect([entityId]);
      }

      setContextMenu({
        visible: true,
        x: e.clientX,
        y: e.clientY,
        entityId,
      });
    },
    [selectedEntities, onEntitySelect]
  );

  // Handle entity click with multi-selection support
  const handleEntityClick = useCallback(
    (e: React.MouseEvent, entityId: string) => {
      e.stopPropagation();

      const ctrl = e.ctrlKey || e.metaKey;
      const shift = e.shiftKey;

      if (ctrl) {
        if (selectedEntities.includes(entityId)) {
          onEntitySelect(selectedEntities.filter((id) => id !== entityId));
        } else {
          onEntitySelect([...selectedEntities, entityId]);
        }
      } else if (shift && selectedEntities.length > 0) {
        const lastSelected = selectedEntities[selectedEntities.length - 1];
        const allEntityIds = getAllEntityIds(filteredEntities);
        const lastIndex = allEntityIds.indexOf(lastSelected);
        const currentIndex = allEntityIds.indexOf(entityId);

        if (lastIndex !== -1 && currentIndex !== -1) {
          const start = Math.min(lastIndex, currentIndex);
          const end = Math.max(lastIndex, currentIndex);
          const rangeIds = allEntityIds.slice(start, end + 1);
          onEntitySelect(rangeIds);
        }
      } else {
        onEntitySelect([entityId]);
      }
    },
    [selectedEntities, onEntitySelect, filteredEntities, getAllEntityIds]
  );

  // Drag and drop handlers
  const handleDragStart = useCallback(
    (e: React.DragEvent, entityId: string) => {
      setDraggedEntity(entityId);
      e.dataTransfer.effectAllowed = 'move';
    },
    []
  );

  const handleDragOver = useCallback((e: React.DragEvent, entityId: string) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    setDropTarget(entityId);
  }, []);

  const handleDragLeave = useCallback(() => {
    setDropTarget(null);
  }, []);

  const handleDrop = useCallback(
    async (e: React.DragEvent, targetEntityId: string) => {
      e.preventDefault();
      if (draggedEntity && draggedEntity !== targetEntityId && onEntityReparent) {
        await onEntityReparent(draggedEntity, targetEntityId);
        setExpandedEntities((prev) => new Set([...prev, targetEntityId]));
      }
      setDraggedEntity(null);
      setDropTarget(null);
    },
    [draggedEntity, onEntityReparent]
  );

  // Render single entity
  const renderEntity = useCallback(
    (entity: Entity, depth: number = 0): React.ReactNode => {
      const isSelected = selectedEntities.includes(entity.id);
      const isExpanded = expandedEntities.has(entity.id);
      const hasChildren = entity.children.length > 0;
      const isEditing = editingEntity === entity.id;
      const isDragTarget = dropTarget === entity.id;
      const isDragging = draggedEntity === entity.id;

      return (
        <div key={entity.id}>
          <div
            className={`flex items-center gap-1 py-1 px-2 hover:bg-slate-700 cursor-pointer relative ${
              isSelected ? 'bg-blue-600' : ''
            } ${isDragTarget ? 'border-t-2 border-blue-400' : ''} ${
              isDragging ? 'opacity-50' : ''
            }`}
            style={{ paddingLeft: `${depth * 16 + 8}px` }}
            draggable={!entity.locked}
            onDragStart={(e) => handleDragStart(e, entity.id)}
            onDragOver={(e) => handleDragOver(e, entity.id)}
            onDragLeave={handleDragLeave}
            onDrop={(e) => handleDrop(e, entity.id)}
            onClick={(e) => handleEntityClick(e, entity.id)}
            onContextMenu={(e) => handleContextMenu(e, entity.id)}
            role="treeitem"
            aria-selected={isSelected}
            aria-expanded={hasChildren ? isExpanded : undefined}
            aria-level={depth + 1}
          >
            {/* Expand/Collapse */}
            {hasChildren && (
              <button
                className="w-4 h-4 flex items-center justify-center text-slate-400 hover:text-slate-200"
                onClick={(e) => {
                  e.stopPropagation();
                  toggleExpand(entity.id);
                }}
                aria-label={isExpanded ? 'Collapse' : 'Expand'}
                aria-expanded={isExpanded}
              >
                <svg
                  className={`w-3 h-3 transition-transform ${isExpanded ? 'rotate-90' : ''}`}
                  fill="currentColor"
                  viewBox="0 0 20 20"
                >
                  <path d="M6 6L14 10L6 14V6Z" />
                </svg>
              </button>
            )}
            {!hasChildren && <div className="w-4" />}

            {/* Icon */}
            <svg className="w-4 h-4 text-slate-400" fill="currentColor" viewBox="0 0 20 20">
              <path d="M10 2L2 10V18H18V10L10 2ZM10 4L16 10V16H4V10L10 4Z" />
            </svg>

            {/* Name */}
            {isEditing ? (
              <input
                type="text"
                value={editName}
                onChange={(e) => setEditName(e.target.value)}
                onBlur={finishEditing}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') finishEditing();
                  if (e.key === 'Escape') cancelEditing();
                }}
                autoFocus
                className="flex-1 bg-slate-600 text-white text-sm px-1 py-0.5 rounded"
                onClick={(e) => e.stopPropagation()}
                aria-label="Edit entity name"
              />
            ) : (
              <span
                className="flex-1 text-sm text-slate-200 truncate"
                onDoubleClick={(e) => {
                  e.stopPropagation();
                  startEditing(entity);
                }}
              >
                {entity.name}
              </span>
            )}

            {/* Visibility Toggle */}
            <button
              className={`w-5 h-5 flex items-center justify-center rounded hover:bg-slate-600 ${
                entity.visible ? 'text-slate-300' : 'text-slate-600'
              }`}
              onClick={(e) => {
                e.stopPropagation();
                onEntityToggleVisibility(entity.id);
              }}
              title={entity.visible ? 'Visible' : 'Hidden'}
              aria-label={entity.visible ? 'Hide entity' : 'Show entity'}
              aria-pressed={entity.visible}
            >
              <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                {entity.visible ? (
                  <path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
                ) : (
                  <path
                    fillRule="evenodd"
                    d="M3.707 2.293a1 1 0 00-1.414 1.414l14 14a1 1 0 001.414-1.414l-1.473-1.473A10.014 10.014 0 0019.542 10C18.268 5.943 14.478 3 10 3a9.958 9.958 0 00-4.512 1.074l-1.78-1.781zm4.261 4.26l1.514 1.515a2.003 2.003 0 012.45 2.45l1.514 1.514a4 4 0 00-5.478-5.478z"
                    clipRule="evenodd"
                  />
                )}
              </svg>
            </button>

            {/* Lock Toggle */}
            <button
              className={`w-5 h-5 flex items-center justify-center rounded hover:bg-slate-600 ${
                entity.locked ? 'text-yellow-500' : 'text-slate-500'
              }`}
              onClick={(e) => {
                e.stopPropagation();
                onEntityToggleLock(entity.id);
              }}
              title={entity.locked ? 'Locked' : 'Unlocked'}
              aria-label={entity.locked ? 'Unlock entity' : 'Lock entity'}
              aria-pressed={entity.locked}
            >
              <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                {entity.locked ? (
                  <path
                    fillRule="evenodd"
                    d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z"
                    clipRule="evenodd"
                  />
                ) : (
                  <path d="M10 2a5 5 0 00-5 5v2a2 2 0 00-2 2v5a2 2 0 002 2h10a2 2 0 002-2v-5a2 2 0 00-2-2H7V7a3 3 0 015.905-.75 1 1 0 001.937-.5A5.002 5.002 0 0010 2z" />
                )}
              </svg>
            </button>
          </div>

          {/* Children */}
          {hasChildren && isExpanded && (
            <div role="group">
              {entity.children.map((child) => renderEntity(child, depth + 1))}
            </div>
          )}
        </div>
      );
    },
    [
      selectedEntities,
      expandedEntities,
      editingEntity,
      editName,
      dropTarget,
      draggedEntity,
      handleDragStart,
      handleDragOver,
      handleDragLeave,
      handleDrop,
      handleEntityClick,
      handleContextMenu,
      toggleExpand,
      startEditing,
      finishEditing,
      cancelEditing,
      onEntityToggleVisibility,
      onEntityToggleLock,
    ]
  );

  return (
    <div className={`flex flex-col h-full bg-slate-900 ${className}`}>
      {/* Header */}
      <div className="p-3 border-b border-slate-700">
        <div className="flex items-center justify-between mb-2">
          <h2 className="text-sm font-semibold text-slate-200">Scene Hierarchy</h2>
          <button
            onClick={onEntityCreate}
            className="px-2 py-1 text-xs bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
            title="Create New Entity"
            aria-label="Create new entity"
          >
            + New
          </button>
        </div>

        {/* Search */}
        <EntityTreeSearch
          value={searchQuery}
          onChange={setSearchQuery}
          placeholder="Search entities..."
        />
      </div>

      {/* Entity List */}
      <div
        className="flex-1 overflow-y-auto"
        role="tree"
        aria-label="Entity hierarchy"
        onDragOver={(e) => e.preventDefault()}
        onDrop={(e) => {
          e.preventDefault();
          if (draggedEntity && onEntityReparent) {
            onEntityReparent(draggedEntity, null);
          }
          setDraggedEntity(null);
        }}
      >
        {filteredEntities.length === 0 ? (
          <div className="p-4 text-center text-slate-500 text-sm">
            {searchQuery ? 'No entities found' : 'No entities in scene'}
          </div>
        ) : (
          filteredEntities.map((entity) => renderEntity(entity))
        )}
      </div>

      {/* Context Menu */}
      <EntityTreeContextMenu
        visible={contextMenu.visible}
        x={contextMenu.x}
        y={contextMenu.y}
        items={contextMenuItems}
        onClose={() =>
          setContextMenu({ visible: false, x: 0, y: 0, entityId: null })
        }
      />
    </div>
  );
};

export default EntityTree;
