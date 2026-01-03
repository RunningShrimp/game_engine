/**
 * Node Palette Component
 * Displays available nodes organized by category
 */

import React, { useState } from 'react';
import { NodeType } from '../../types/material';
import './NodePalette.css';

interface NodeCategory {
  name: string;
  nodeTypes: NodeType[];
}

const NODE_CATEGORIES: NodeCategory[] = [
  {
    name: 'Input',
    nodeTypes: [
      NodeType.TextureInput,
      NodeType.ColorInput,
      NodeType.FloatInput,
      NodeType.Vector2Input,
      NodeType.Vector3Input,
      NodeType.Vector4Input,
    ],
  },
  {
    name: 'Math',
    nodeTypes: [
      NodeType.Add,
      NodeType.Subtract,
      NodeType.Multiply,
      NodeType.Divide,
      NodeType.Lerp,
      NodeType.Mix,
      NodeType.Power,
      NodeType.Sqrt,
      NodeType.Abs,
      NodeType.Min,
      NodeType.Max,
      NodeType.Clamp,
      NodeType.Normalize,
    ],
  },
  {
    name: 'PBR',
    nodeTypes: [
      NodeType.PBRMaster,
      NodeType.Metallic,
      NodeType.Roughness,
      NodeType.NormalMap,
      NodeType.AmbientOcclusion,
      NodeType.Emission,
    ],
  },
  {
    name: 'Texture',
    nodeTypes: [
      NodeType.TextureSample,
      NodeType.TextureParameter,
    ],
  },
  {
    name: 'UV',
    nodeTypes: [
      NodeType.UVCoordinate,
      NodeType.TextureMapping,
      NodeType.UVScroll,
    ],
  },
  {
    name: 'Output',
    nodeTypes: [
      NodeType.MaterialOutput,
    ],
  },
];

interface NodePaletteProps {
  onNodeTypeSelect: (nodeType: NodeType) => void;
}

export const NodePalette: React.FC<NodePaletteProps> = ({
  onNodeTypeSelect,
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(
    new Set(NODE_CATEGORIES.map(c => c.name))
  );

  // Toggle category expansion
  const toggleCategory = (categoryName: string) => {
    setExpandedCategories(prev => {
      const newExpanded = new Set(prev);
      if (newExpanded.has(categoryName)) {
        newExpanded.delete(categoryName);
      } else {
        newExpanded.add(categoryName);
      }
      return newExpanded;
    });
  };

  // Filter categories and nodes based on search
  const filteredCategories = NODE_CATEGORIES.map(category => ({
    ...category,
    nodeTypes: category.nodeTypes.filter(nodeType =>
      nodeType.toLowerCase().includes(searchQuery.toLowerCase()) ||
      category.name.toLowerCase().includes(searchQuery.toLowerCase())
    ),
  })).filter(category => category.nodeTypes.length > 0);

  // Get icon for node type
  const getNodeIcon = (nodeType: NodeType): string => {
    if (nodeType.includes('texture') || nodeType.includes('Texture')) return '🖼️';
    if (nodeType.includes('color') || nodeType.includes('Color')) return '🎨';
    if (nodeType.includes('float') || nodeType.includes('Float') || nodeType.includes('vector')) return '🔢';
    if (['Add', 'Subtract', 'Multiply', 'Divide', 'Power', 'Sqrt'].includes(nodeType)) return '➕';
    if (['Lerp', 'Mix', 'Clamp', 'Min', 'Max'].includes(nodeType)) return '📊';
    if (nodeType.includes('PBR') || nodeType.includes('Metallic') || nodeType.includes('Roughness')) return '✨';
    if (nodeType.includes('Normal') || nodeType.includes('UV')) return '📐';
    if (nodeType.includes('Output')) return '📤';
    return '📦';
  };

  return (
    <div className="node-palette">
      <div className="palette-header">
        <h3>Node Palette</h3>
        <div className="palette-search">
          <input
            type="text"
            placeholder="Search nodes..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="search-input"
          />
        </div>
      </div>

      <div className="palette-content">
        {filteredCategories.map(category => (
          <div key={category.name} className="palette-category">
            <div
              className="category-header"
              onClick={() => toggleCategory(category.name)}
            >
              <span className={`category-arrow ${expandedCategories.has(category.name) ? 'expanded' : ''}`}>
                ▶
              </span>
              <span className="category-name">{category.name}</span>
              <span className="category-count">({category.nodeTypes.length})</span>
            </div>

            {expandedCategories.has(category.name) && (
              <div className="category-nodes">
                {category.nodeTypes.map(nodeType => (
                  <div
                    key={nodeType}
                    className="palette-node"
                    onClick={() => onNodeTypeSelect(nodeType)}
                    title={nodeType}
                  >
                    <span className="node-icon">{getNodeIcon(nodeType)}</span>
                    <span className="node-name">
                      {nodeType.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

export default NodePalette;
