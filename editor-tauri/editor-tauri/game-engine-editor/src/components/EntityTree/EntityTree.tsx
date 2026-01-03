import React, { useState, useRef, useEffect } from 'react';
import { Entity } from '../../types/engine';

interface EntityTreeProps {
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
}

interface ContextMenu {
  visible: boolean;
  x: number;
  y: number;
  entityId: string | null;
}

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
}) => {
  const [expandedEntities, setExpandedEntities] = useState<Set<string>>(new Set());
  const [editingEntity, setEditingEntity] = useState<string | null>(null);
  const [editName, setEditName] = useState('');
  const [contextMenu, setContextMenu] = useState<ContextMenu>({
    visible: false,
    x: 0,
    y: 0,
    entityId: null,
  });
  const [draggedEntity, setDraggedEntity] = useState<string | null>(null);
  const [dropTarget, setDropTarget] = useState<string | null>(null);

  const contextMenuRef = useRef<HTMLDivElement>(null);

  // Close context menu when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (contextMenuRef.current && !contextMenuRef.current.contains(event.target as Node)) {
        setContextMenu({ visible: false, x: 0, y: 0, entityId: null });
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const toggleExpand = (entityId: string) => {
    const newExpanded = new Set(expandedEntities);
    if (newExpanded.has(entityId)) {
      newExpanded.delete(entityId);
    } else {
      newExpanded.add(entityId);
    }
    setExpandedEntities(newExpanded);
  };

  const startEditing = (entity: Entity) => {
    setEditingEntity(entity.id);
    setEditName(entity.name);
    setContextMenu({ visible: false, x: 0, y: 0, entityId: null });
  };

  const finishEditing = () => {
    if (editingEntity && editName.trim()) {
      onEntityRename(editingEntity, editName.trim());
    }
    setEditingEntity(null);
    setEditName('');
  };

  const handleContextMenu = (e: React.MouseEvent, entityId: string) => {
    e.preventDefault();
    e.stopPropagation();

    // If right-clicking on non-selected entity, select it
    if (!selectedEntities.includes(entityId)) {
      onEntitySelect([entityId]);
    }

    setContextMenu({
      visible: true,
      x: e.clientX,
      y: e.clientY,
      entityId,
    });
  };

  const handleEntityClick = (e: React.MouseEvent, entityId: string) => {
    e.stopPropagation();

    const ctrl = e.ctrlKey || e.metaKey;
    const shift = e.shiftKey;

    if (ctrl) {
      // Ctrl+Click: Toggle selection
      if (selectedEntities.includes(entityId)) {
        onEntitySelect(selectedEntities.filter((id) => id !== entityId));
      } else {
        onEntitySelect([...selectedEntities, entityId]);
      }
    } else if (shift && selectedEntities.length > 0) {
      // Shift+Click: Range selection
      const lastSelected = selectedEntities[selectedEntities.length - 1];
      const allEntityIds = getAllEntityIds(entities);
      const lastIndex = allEntityIds.indexOf(lastSelected);
      const currentIndex = allEntityIds.indexOf(entityId);

      if (lastIndex !== -1 && currentIndex !== -1) {
        const start = Math.min(lastIndex, currentIndex);
        const end = Math.max(lastIndex, currentIndex);
        const rangeIds = allEntityIds.slice(start, end + 1);
        onEntitySelect(rangeIds);
      }
    } else {
      // Normal click: Single select
      onEntitySelect([entityId]);
    }
  };

  // Helper to get all entity IDs in order
  const getAllEntityIds = (entities: Entity[], ids: string[] = []): string[] => {
    entities.forEach((entity) => {
      ids.push(entity.id);
      if (entity.children.length > 0) {
        getAllEntityIds(entity.children, ids);
      }
    });
    return ids;
  };

  const handleDuplicate = () => {
    if (contextMenu.entityId) {
      // TODO: Implement duplication via App.tsx
      console.log('Duplicate entity:', contextMenu.entityId);
    }
    setContextMenu({ visible: false, x: 0, y: 0, entityId: null });
  };

  const handleDelete = () => {
    if (contextMenu.entityId) {
      onEntityDelete(contextMenu.entityId);
    }
    setContextMenu({ visible: false, x: 0, y: 0, entityId: null });
  };

  // Drag and drop handlers
  const handleDragStart = (e: React.DragEvent, entityId: string) => {
    setDraggedEntity(entityId);
    e.dataTransfer.effectAllowed = 'move';
  };

  const handleDragOver = (e: React.DragEvent, entityId: string) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    setDropTarget(entityId);
  };

  const handleDragLeave = () => {
    setDropTarget(null);
  };

  const handleDrop = async (e: React.DragEvent, targetEntityId: string) => {
    e.preventDefault();
    if (draggedEntity && draggedEntity !== targetEntityId && onEntityReparent) {
      await onEntityReparent(draggedEntity, targetEntityId);
      // Expand the target entity to show the dropped child
      setExpandedEntities((prev) => new Set([...prev, targetEntityId]));
    }
    setDraggedEntity(null);
    setDropTarget(null);
  };

  const renderEntity = (entity: Entity, depth: number = 0): React.ReactNode => {
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
        >
          {/* Expand/Collapse */}
          {hasChildren && (
            <button
              className="w-4 h-4 flex items-center justify-center text-slate-400 hover:text-slate-200"
              onClick={(e) => {
                e.stopPropagation();
                toggleExpand(entity.id);
              }}
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
                if (e.key === 'Escape') {
                  setEditingEntity(null);
                  setEditName('');
                }
              }}
              autoFocus
              className="flex-1 bg-slate-600 text-white text-sm px-1 py-0.5 rounded"
              onClick={(e) => e.stopPropagation()}
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
          <div>
            {entity.children.map((child) => renderEntity(child, depth + 1))}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="flex flex-col h-full bg-slate-900">
      {/* Header */}
      <div className="p-3 border-b border-slate-700">
        <div className="flex items-center justify-between mb-2">
          <h2 className="text-sm font-semibold text-slate-200">Scene Hierarchy</h2>
          <button
            onClick={onEntityCreate}
            className="px-2 py-1 text-xs bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
            title="Create New Entity"
          >
            + New
          </button>
        </div>
        <input
          type="text"
          placeholder="Search entities..."
          className="w-full px-2 py-1 text-sm bg-slate-800 border border-slate-700 rounded text-slate-200 placeholder-slate-500 focus:border-blue-500 focus:outline-none"
        />
      </div>

      {/* Entity List */}
      <div
        className="flex-1 overflow-y-auto"
        onDragOver={(e) => e.preventDefault()}
        onDrop={(e) => {
          e.preventDefault();
          if (draggedEntity && onEntityReparent) {
            onEntityReparent(draggedEntity, null); // Drop to root level
          }
          setDraggedEntity(null);
        }}
      >
        {entities.map((entity) => renderEntity(entity))}
      </div>

      {/* Context Menu */}
      {contextMenu.visible && (
        <div
          ref={contextMenuRef}
          className="fixed z-50 bg-slate-800 border border-slate-700 rounded shadow-xl py-1 min-w-48"
          style={{
            left: `${contextMenu.x}px`,
            top: `${contextMenu.y}px`,
          }}
        >
          <button
            className="w-full px-3 py-1.5 text-left text-sm text-slate-200 hover:bg-slate-700 flex items-center gap-2"
            onClick={() => {
              if (contextMenu.entityId) {
                const entity = entities.find((e) => e.id === contextMenu.entityId);
                if (entity) startEditing(entity);
              }
            }}
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
            </svg>
            Rename
          </button>
          <button
            className="w-full px-3 py-1.5 text-left text-sm text-slate-200 hover:bg-slate-700 flex items-center gap-2"
            onClick={handleDuplicate}
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
            Duplicate
          </button>
          <button
            className="w-full px-3 py-1.5 text-left text-sm text-slate-200 hover:bg-slate-700 flex items-center gap-2"
            onClick={() => {
              if (contextMenu.entityId) onEntityToggleVisibility(contextMenu.entityId);
              setContextMenu({ visible: false, x: 0, y: 0, entityId: null });
            }}
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
            </svg>
            Toggle Visibility
          </button>
          <div className="border-t border-slate-700 my-1" />
          <button
            className="w-full px-3 py-1.5 text-left text-sm text-red-400 hover:bg-slate-700 flex items-center gap-2"
            onClick={handleDelete}
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
            Delete
          </button>
        </div>
      )}
    </div>
  );
};
