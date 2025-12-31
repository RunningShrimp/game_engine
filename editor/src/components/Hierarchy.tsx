import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface Entity {
  id: number;
  name: string;
  children: Entity[];
}

interface HierarchyProps {
  selectedEntity?: number | null;
  onEntitySelect?: (entityId: number) => void;
}

export const Hierarchy: React.FC<HierarchyProps> = ({
  selectedEntity,
  onEntitySelect
}) => {
  const [entities, setEntities] = useState<Entity[]>([]);
  const [expanded, setExpanded] = useState<Set<number>>(new Set());
  const [filter, setFilter] = useState('');

  useEffect(() => {
    refreshEntities();
    // 每秒刷新实体列表
    const interval = setInterval(refreshEntities, 1000);
    return () => clearInterval(interval);
  }, []);

  const refreshEntities = async () => {
    try {
      const data = await invoke<Entity[]>('get_entities');
      setEntities(data);
    } catch (error) {
      console.error('Failed to fetch entities:', error);
    }
  };

  const toggleExpand = (entityId: number) => {
    setExpanded(prev => {
      const next = new Set(prev);
      if (next.has(entityId)) {
        next.delete(entityId);
      } else {
        next.add(entityId);
      }
      return next;
    });
  };

  const renderEntity = (entity: Entity, level: number = 0): React.ReactNode => {
    const padding = level * 20;
    const isExpanded = expanded.has(entity.id);
    const isSelected = selectedEntity === entity.id;
    const hasChildren = entity.children.length > 0;
    const matchesFilter = filter === '' ||
      entity.name.toLowerCase().includes(filter.toLowerCase());

    if (!matchesFilter) return null;

    return (
      <div key={entity.id} className="entity-node">
        <div
          className={`entity-item ${isSelected ? 'selected' : ''}`}
          style={{ paddingLeft: padding }}
          onClick={(e) => {
            e.stopPropagation();
            if (onEntitySelect) {
              onEntitySelect(entity.id);
            }
          }}
        >
          {hasChildren && (
            <span
              className="expand-icon"
              onClick={(e) => {
                e.stopPropagation();
                toggleExpand(entity.id);
              }}
            >
              {isExpanded ? '▼' : '▶'}
            </span>
          )}
          {!hasChildren && <span className="expand-icon-placeholder"></span>}
          <span className="entity-icon">📦</span>
          <span className="entity-name">{entity.name}</span>
        </div>
        {isExpanded && entity.children.map(child => renderEntity(child, level + 1))}
      </div>
    );
  };

  const handleCreateEntity = async () => {
    try {
      const entityId = await invoke<number>('create_entity', {
        name: 'New Entity',
        parentId: null
      });
      await refreshEntities();
      if (onEntitySelect) {
        onEntitySelect(entityId);
      }
    } catch (error) {
      console.error('Failed to create entity:', error);
    }
  };

  const handleDeleteEntity = async () => {
    if (selectedEntity === null || selectedEntity === undefined) return;

    try {
      await invoke('delete_entity', { entityId: selectedEntity });
      await refreshEntities();
    } catch (error) {
      console.error('Failed to delete entity:', error);
    }
  };

  return (
    <div className="hierarchy">
      <div className="hierarchy-header">
        <h3>Scene Hierarchy</h3>
        <div className="hierarchy-actions">
          <button onClick={handleCreateEntity} title="Create Entity">+</button>
          <button
            onClick={handleDeleteEntity}
            title="Delete Entity"
            disabled={selectedEntity === null || selectedEntity === undefined}
          >
            -
          </button>
        </div>
      </div>
      <input
        type="text"
        placeholder="Filter..."
        value={filter}
        onChange={(e) => setFilter(e.target.value)}
        className="entity-filter"
      />
      <div className="entity-list">
        {entities.map(entity => renderEntity(entity))}
      </div>
    </div>
  );
};
