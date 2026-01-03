/**
 * Behavior Tree Manager Component
 * Manages behavior tree templates and provides save/load functionality
 */

import React, { useState } from 'react';
import { BehaviorTree, TreeTemplate, ValidationResult } from '../../types/behavior';
import './BehaviorTreeManager.css';

interface BehaviorTreeManagerProps {
  isOpen: boolean;
  trees: BehaviorTree[];
  onLoad: (tree: BehaviorTree) => void;
  onClose: () => void;
  onDelete?: (treeId: string) => void;
  onDuplicate?: (tree: BehaviorTree) => void;
  onValidate?: (tree: BehaviorTree) => ValidationResult;
}

// Example templates
const exampleTemplates: TreeTemplate[] = [
  {
    id: 'patrol',
    name: 'Patrol AI',
    description: 'Basic patrol behavior for AI agents',
    category: 'Movement',
    tree: {
      id: 'template_patrol',
      name: 'Patrol AI',
      root: {
        id: 'root_patrol',
        type: 'sequence' as any,
        name: 'Patrol Loop',
        position: { x: 100, y: 100 },
        children: [],
        parameters: [],
      },
      blackboard: { variables: {} },
      createdAt: Date.now(),
      updatedAt: Date.now(),
    },
  },
  {
    id: 'combat',
    name: 'Combat AI',
    description: 'Combat behavior with attack and flee logic',
    category: 'Combat',
    tree: {
      id: 'template_combat',
      name: 'Combat AI',
      root: {
        id: 'root_combat',
        type: 'selector' as any,
        name: 'Combat Selector',
        position: { x: 100, y: 100 },
        children: [],
        parameters: [],
      },
      blackboard: { variables: {} },
      createdAt: Date.now(),
      updatedAt: Date.now(),
    },
  },
];

export const BehaviorTreeManager: React.FC<BehaviorTreeManagerProps> = ({
  isOpen,
  trees,
  onLoad,
  onClose,
  onDelete,
  onDuplicate,
  onValidate,
}) => {
  const [activeTab, setActiveTab] = useState<'saved' | 'templates'>('saved');
  const [searchQuery, setSearchQuery] = useState('');

  if (!isOpen) return null;

  const filteredTrees = trees.filter(tree =>
    tree.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const filteredTemplates = exampleTemplates.filter(template =>
    template.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="behavior-tree-manager-overlay" onClick={onClose}>
      <div className="behavior-tree-manager" onClick={(e) => e.stopPropagation()}>
        {/* Header */}
        <div className="manager-header">
          <h2>Behavior Tree Manager</h2>
          <button className="close-button" onClick={onClose}>
            ×
          </button>
        </div>

        {/* Search */}
        <div className="manager-search">
          <input
            type="text"
            placeholder="Search trees..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="search-input"
          />
        </div>

        {/* Tabs */}
        <div className="manager-tabs">
          <button
            className={`tab-button ${activeTab === 'saved' ? 'active' : ''}`}
            onClick={() => setActiveTab('saved')}
          >
            Saved Trees ({trees.length})
          </button>
          <button
            className={`tab-button ${activeTab === 'templates' ? 'active' : ''}`}
            onClick={() => setActiveTab('templates')}
          >
            Templates ({exampleTemplates.length})
          </button>
        </div>

        {/* Content */}
        <div className="manager-content">
          {activeTab === 'saved' && (
            <div className="trees-list">
              {filteredTrees.length === 0 ? (
                <div className="empty-state">
                  <p>No saved trees</p>
                  <p className="hint">Create and save behavior trees to see them here</p>
                </div>
              ) : (
                filteredTrees.map(tree => (
                  <div key={tree.id} className="tree-item">
                    <div className="tree-info">
                      <h3 className="tree-name">{tree.name}</h3>
                      {tree.description && (
                        <p className="tree-description">{tree.description}</p>
                      )}
                      <div className="tree-meta">
                        <span className="tree-date">
                          Updated: {new Date(tree.updatedAt).toLocaleDateString()}
                        </span>
                      </div>
                    </div>
                    <div className="tree-actions">
                      <button
                        className="action-button load-button"
                        onClick={() => onLoad(tree)}
                      >
                        Load
                      </button>
                      {onDuplicate && (
                        <button
                          className="action-button duplicate-button"
                          onClick={() => onDuplicate(tree)}
                        >
                          Duplicate
                        </button>
                      )}
                      {onDelete && (
                        <button
                          className="action-button delete-button"
                          onClick={() => {
                            if (confirm(`Delete "${tree.name}"?`)) {
                              onDelete(tree.id);
                            }
                          }}
                        >
                          Delete
                        </button>
                      )}
                    </div>
                  </div>
                ))
              )}
            </div>
          )}

          {activeTab === 'templates' && (
            <div className="trees-list">
              {filteredTemplates.map(template => (
                <div key={template.id} className="tree-item template-item">
                  <div className="tree-info">
                    <span className="template-category">{template.category}</span>
                    <h3 className="tree-name">{template.name}</h3>
                    <p className="tree-description">{template.description}</p>
                  </div>
                  <div className="tree-actions">
                    <button
                      className="action-button load-button"
                      onClick={() => onLoad(template.tree)}
                    >
                      Use Template
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default BehaviorTreeManager;
