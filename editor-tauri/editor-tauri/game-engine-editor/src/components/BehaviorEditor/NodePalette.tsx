/**
 * Node Palette Component
 * Displays available node types that can be dragged into the tree
 */

import React, { useState } from 'react';
import { NodeType, NodeCategory, NodeTemplate, NodeParameter } from '../../types/behavior';
import './NodePalette.css';

interface NodePaletteProps {
  onNodeTypeSelect: (type: NodeType) => void;
  isOpen: boolean;
  onToggle: () => void;
}

// Define all available node templates
const nodeTemplates: NodeTemplate[] = [
  // Composite Nodes
  {
    type: NodeType.Sequence,
    category: NodeCategory.Composite,
    name: 'Sequence',
    description: 'Execute children in order until one fails',
    icon: '➡️',
    defaultParameters: [],
    allowChildren: true,
  },
  {
    type: NodeType.Selector,
    category: NodeCategory.Composite,
    name: 'Selector',
    description: 'Execute children until one succeeds',
    icon: '⭕',
    defaultParameters: [],
    allowChildren: true,
  },
  {
    type: NodeType.Parallel,
    category: NodeCategory.Composite,
    name: 'Parallel',
    description: 'Execute all children simultaneously',
    icon: '⫴',
    defaultParameters: [
      {
        id: 'successMode',
        name: 'Success Mode',
        dataType: 'enum',
        value: 'all',
        enumValues: ['all', 'any'],
      },
    ],
    allowChildren: true,
  },

  // Decorator Nodes
  {
    type: NodeType.Inverter,
    category: NodeCategory.Decorator,
    name: 'Inverter',
    description: 'Invert the child\'s result',
    icon: '🔄',
    defaultParameters: [],
    allowChildren: true,
    maxChildren: 1,
  },
  {
    type: NodeType.Repeater,
    category: NodeCategory.Decorator,
    name: 'Repeater',
    description: 'Repeat child N times',
    icon: '🔁',
    defaultParameters: [
      {
        id: 'count',
        name: 'Repeat Count',
        dataType: 'number',
        value: 3,
        min: 1,
        max: 100,
        step: 1,
      },
    ],
    allowChildren: true,
    maxChildren: 1,
  },
  {
    type: NodeType.Cooldown,
    category: NodeCategory.Decorator,
    name: 'Cooldown',
    description: 'Add cooldown between executions',
    icon: '⏱️',
    defaultParameters: [
      {
        id: 'duration',
        name: 'Cooldown (seconds)',
        dataType: 'number',
        value: 1.0,
        min: 0.1,
        max: 60,
        step: 0.1,
      },
    ],
    allowChildren: true,
    maxChildren: 1,
  },
  {
    type: NodeType.AlwaysSucceed,
    category: NodeCategory.Decorator,
    name: 'Always Succeed',
    description: 'Always return success',
    icon: '✅',
    defaultParameters: [],
    allowChildren: true,
    maxChildren: 1,
  },
  {
    type: NodeType.AlwaysFail,
    category: NodeCategory.Decorator,
    name: 'Always Fail',
    description: 'Always return failure',
    icon: '❌',
    defaultParameters: [],
    allowChildren: true,
    maxChildren: 1,
  },

  // Condition Nodes
  {
    type: NodeType.Condition,
    category: NodeCategory.Condition,
    name: 'Condition',
    description: 'Check a blackboard condition',
    icon: '❓',
    defaultParameters: [
      {
        id: 'variable',
        name: 'Variable',
        dataType: 'string',
        value: '',
      },
      {
        id: 'operator',
        name: 'Operator',
        dataType: 'enum',
        value: 'equals',
        enumValues: ['equals', 'not_equals', 'greater', 'less', 'contains'],
      },
      {
        id: 'value',
        name: 'Value',
        dataType: 'object',
        value: '',
      },
    ],
    allowChildren: false,
  },
  {
    type: NodeType.Check,
    category: NodeCategory.Condition,
    name: 'Check',
    description: 'Check state/value',
    icon: '✓',
    defaultParameters: [
      {
        id: 'expression',
        name: 'Expression',
        dataType: 'string',
        value: '',
      },
    ],
    allowChildren: false,
  },

  // Action Nodes
  {
    type: NodeType.Action,
    category: NodeCategory.Action,
    name: 'Action',
    description: 'Execute custom action',
    icon: '⚡',
    defaultParameters: [
      {
        id: 'actionName',
        name: 'Action Name',
        dataType: 'string',
        value: '',
      },
      {
        id: 'parameters',
        name: 'Parameters',
        dataType: 'object',
        value: {},
      },
    ],
    allowChildren: false,
  },
  {
    type: NodeType.Wait,
    category: NodeCategory.Action,
    name: 'Wait',
    description: 'Wait for specified time',
    icon: '⏸️',
    defaultParameters: [
      {
        id: 'duration',
        name: 'Duration (seconds)',
        dataType: 'number',
        value: 1.0,
        min: 0.1,
        max: 60,
        step: 0.1,
      },
    ],
    allowChildren: false,
  },
  {
    type: NodeType.Log,
    category: NodeCategory.Action,
    name: 'Log',
    description: 'Log message to console',
    icon: '📝',
    defaultParameters: [
      {
        id: 'message',
        name: 'Message',
        dataType: 'string',
        value: 'Hello from behavior tree',
      },
      {
        id: 'level',
        name: 'Log Level',
        dataType: 'enum',
        value: 'info',
        enumValues: ['debug', 'info', 'warning', 'error'],
      },
    ],
    allowChildren: false,
  },
];

// Group templates by category
const groupedTemplates = {
  [NodeCategory.Composite]: nodeTemplates.filter(t => t.category === NodeCategory.Composite),
  [NodeCategory.Decorator]: nodeTemplates.filter(t => t.category === NodeCategory.Decorator),
  [NodeCategory.Condition]: nodeTemplates.filter(t => t.category === NodeCategory.Condition),
  [NodeCategory.Action]: nodeTemplates.filter(t => t.category === NodeCategory.Action),
};

const categoryIcons: Record<NodeCategory, string> = {
  [NodeCategory.Composite]: '📦',
  [NodeCategory.Decorator]: '🎨',
  [NodeCategory.Condition]: '❓',
  [NodeCategory.Action]: '⚡',
};

export const NodePalette: React.FC<NodePaletteProps> = ({
  onNodeTypeSelect,
  isOpen,
  onToggle,
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [expandedCategories, setExpandedCategories] = useState<Set<NodeCategory>>(
    new Set([NodeCategory.Composite, NodeCategory.Action])
  );

  const toggleCategory = (category: NodeCategory) => {
    setExpandedCategories(prev => {
      const newSet = new Set(prev);
      if (newSet.has(category)) {
        newSet.delete(category);
      } else {
        newSet.add(category);
      }
      return newSet;
    });
  };

  const filteredTemplates = nodeTemplates.filter(template =>
    template.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    template.description.toLowerCase().includes(searchQuery.toLowerCase())
  );

  if (!isOpen) {
    return (
      <button className="node-palette-toggle" onClick={onToggle}>
        <span>📦</span>
      </button>
    );
  }

  return (
    <div className="node-palette">
      <div className="palette-header">
        <h3>Node Palette</h3>
        <button className="close-button" onClick={onToggle}>
          ×
        </button>
      </div>

      {/* Search */}
      <div className="palette-search">
        <input
          type="text"
          placeholder="Search nodes..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="search-input"
        />
      </div>

      {/* Node List */}
      <div className="palette-nodes">
        {searchQuery ? (
          // Show filtered results
          <div className="node-category">
            <div className="node-list">
              {filteredTemplates.map(template => (
                <div
                  key={template.type}
                  className="palette-node"
                  onClick={() => onNodeTypeSelect(template.type)}
                  draggable
                  onDragStart={(e) => {
                    e.dataTransfer.setData('nodeType', template.type);
                  }}
                >
                  <span className="node-icon">{template.icon}</span>
                  <div className="node-info">
                    <div className="node-name">{template.name}</div>
                    <div className="node-description">{template.description}</div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        ) : (
          // Show grouped by category
          Object.entries(groupedTemplates).map(([category, templates]) => (
            <div key={category} className="node-category">
              <div
                className="category-header"
                onClick={() => toggleCategory(category as NodeCategory)}
              >
                <span className="category-icon">
                  {categoryIcons[category as NodeCategory]}
                </span>
                <span className="category-name">{category}</span>
                <span className="category-arrow">
                  {expandedCategories.has(category as NodeCategory) ? '▼' : '▶'}
                </span>
              </div>
              {expandedCategories.has(category as NodeCategory) && (
                <div className="node-list">
                  {templates.map(template => (
                    <div
                      key={template.type}
                      className="palette-node"
                      onClick={() => onNodeTypeSelect(template.type)}
                      draggable
                      onDragStart={(e) => {
                        e.dataTransfer.setData('nodeType', template.type);
                      }}
                    >
                      <span className="node-icon">{template.icon}</span>
                      <div className="node-info">
                        <div className="node-name">{template.name}</div>
                        <div className="node-description">{template.description}</div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default NodePalette;
