/**
 * Material Editor Component
 * Main component for the node-based material editor
 */

import React, { useState, useCallback, useRef, useEffect } from 'react';
import { Material, MaterialNode, NodeConnection, NodeType } from '../../types/material';
import { NodeCanvas } from './NodeCanvas';
import { NodePalette } from './NodePalette';
import { PreviewPanel } from './PreviewPanel';
import { MaterialManager } from './MaterialManager';
import './MaterialEditor.css';

interface MaterialEditorProps {
  initialMaterial?: Material;
  onMaterialChange?: (material: Material) => void;
}

export const MaterialEditor: React.FC<MaterialEditorProps> = ({
  initialMaterial,
  onMaterialChange,
}) => {
  // Material state
  const [material, setMaterial] = useState<Material>(
    initialMaterial || {
      id: Date.now().toString(),
      name: 'New Material',
      nodes: [],
      connections: [],
      previewMesh: 'sphere',
    }
  );

  // UI state
  const [selectedNodes, setSelectedNodes] = useState<Set<string>>(new Set());
  const [copiedNodes, setCopiedNodes] = useState<MaterialNode[]>([]);
  const [isPaletteOpen, setIsPaletteOpen] = useState(true);
  const [isManagerOpen, setIsManagerOpen] = useState(false);

  // Refs
  const materialRef = useRef(material);
  materialRef.current = material;

  // Update material and notify parent
  const updateMaterial = useCallback((updatedMaterial: Material) => {
    setMaterial(updatedMaterial);
    if (onMaterialChange) {
      onMaterialChange(updatedMaterial);
    }
  }, [onMaterialChange]);

  // Node operations
  const addNode = useCallback((node: MaterialNode) => {
    updateMaterial({
      ...material,
      nodes: [...material.nodes, node],
    });
  }, [material, updateMaterial]);

  const removeNode = useCallback((nodeId: string) => {
    updateMaterial({
      ...material,
      nodes: material.nodes.filter(n => n.id !== nodeId),
      connections: material.connections.filter(
        c => c.fromNodeId !== nodeId && c.toNodeId !== nodeId
      ),
    });
  }, [material, updateMaterial]);

  const updateNode = useCallback((nodeId: string, updates: Partial<MaterialNode>) => {
    updateMaterial({
      ...material,
      nodes: material.nodes.map(n =>
        n.id === nodeId ? { ...n, ...updates } : n
      ),
    });
  }, [material, updateMaterial]);

  // Connection operations
  const addConnection = useCallback((connection: NodeConnection) => {
    // Remove existing connection to the input port
    const existingConnection = material.connections.find(
      c => c.toPortId === connection.toPortId
    );

    const updatedConnections = existingConnection
      ? material.connections.filter(c => c.id !== existingConnection.id)
      : material.connections;

    updateMaterial({
      ...material,
      connections: [...updatedConnections, connection],
    });
  }, [material, updateMaterial]);

  const removeConnection = useCallback((connectionId: string) => {
    updateMaterial({
      ...material,
      connections: material.connections.filter(c => c.id !== connectionId),
    });
  }, [material, updateMaterial]);

  // Selection operations
  const selectNode = useCallback((nodeId: string, addToSelection: boolean = false) => {
    setSelectedNodes(prev => {
      const newSelection = addToSelection ? new Set<string>(prev) : new Set<string>();
      if (newSelection.has(nodeId)) {
        newSelection.delete(nodeId);
      } else {
        newSelection.add(nodeId);
      }
      return newSelection;
    });
  }, []);

  const clearSelection = useCallback(() => {
    setSelectedNodes(new Set<string>());
  }, []);

  // Delete selected nodes
  const deleteSelected = useCallback(() => {
    selectedNodes.forEach(nodeId => {
      removeNode(nodeId);
    });
    clearSelection();
  }, [selectedNodes, removeNode, clearSelection]);

  // Copy selected nodes
  const copySelected = useCallback(() => {
    const nodesToCopy = material.nodes.filter(n => selectedNodes.has(n.id));
    setCopiedNodes(nodesToCopy);
  }, [material.nodes, selectedNodes]);

  // Paste copied nodes
  const pasteNodes = useCallback(() => {
    if (copiedNodes.length === 0) return;

    const offsetX = 50;
    const offsetY = 50;
    const newNodes: MaterialNode[] = copiedNodes.map(node => ({
      ...node,
      id: `${node.id}_copy_${Date.now()}`,
      position: {
        x: node.position.x + offsetX,
        y: node.position.y + offsetY,
      },
      inputs: node.inputs.map(input => ({
        ...input,
        id: `${input.id}_copy`,
        nodeId: `${node.id}_copy_${Date.now()}`,
        connectedTo: null,
      })),
      outputs: node.outputs.map(output => ({
        ...output,
        id: `${output.id}_copy`,
        nodeId: `${node.id}_copy_${Date.now()}`,
      })),
    }));

    updateMaterial({
      ...material,
      nodes: [...material.nodes, ...newNodes],
    });
  }, [copiedNodes, material, updateMaterial]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Delete: Remove selected nodes
      if (e.key === 'Delete' || e.key === 'Backspace') {
        if (selectedNodes.size > 0) {
          e.preventDefault();
          deleteSelected();
        }
      }

      // Ctrl+C: Copy selected nodes
      if ((e.ctrlKey || e.metaKey) && e.key === 'c') {
        if (selectedNodes.size > 0) {
          e.preventDefault();
          copySelected();
        }
      }

      // Ctrl+V: Paste nodes
      if ((e.ctrlKey || e.metaKey) && e.key === 'v') {
        e.preventDefault();
        pasteNodes();
      }

      // Ctrl+S: Save material
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault();
        // Trigger save via MaterialManager
        const event = new CustomEvent('save-material');
        window.dispatchEvent(event);
      }

      // Escape: Clear selection
      if (e.key === 'Escape') {
        clearSelection();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [selectedNodes, deleteSelected, copySelected, pasteNodes, clearSelection]);

  return (
    <div className="material-editor">
      {/* Top toolbar */}
      <div className="material-editor-toolbar">
        <div className="toolbar-section">
          <button
            className="toolbar-button"
            onClick={() => setIsPaletteOpen(!isPaletteOpen)}
            title="Toggle Node Palette"
          >
            <span className="icon">📦</span>
            {isPaletteOpen ? 'Hide Palette' : 'Show Palette'}
          </button>
          <button
            className="toolbar-button"
            onClick={() => setIsManagerOpen(!isManagerOpen)}
            title="Material Manager"
          >
            <span className="icon">📁</span>
            Materials
          </button>
        </div>

        <div className="toolbar-section">
          <h2 className="material-name">{material.name}</h2>
        </div>

        <div className="toolbar-section">
          <button
            className="toolbar-button"
            onClick={() => {
              const event = new CustomEvent('save-material');
              window.dispatchEvent(event);
            }}
            title="Save Material (Ctrl+S)"
          >
            <span className="icon">💾</span>
            Save
          </button>
          <button
            className="toolbar-button"
            onClick={() => {
              const event = new CustomEvent('export-material');
              window.dispatchEvent(event);
            }}
            title="Export Material"
          >
            <span className="icon">📤</span>
            Export
          </button>
        </div>
      </div>

      {/* Main content */}
      <div className="material-editor-content">
        {/* Node Palette */}
        {isPaletteOpen && (
          <div className="material-editor-sidebar">
            <NodePalette
              onNodeTypeSelect={(nodeType) => {
                // Create a default node of the selected type
                const newNode: MaterialNode = {
                  id: `node_${Date.now()}`,
                  type: nodeType,
                  position: { x: 100, y: 100 },
                  inputs: getDefaultInputsForNodeType(nodeType),
                  outputs: getDefaultOutputsForNodeType(nodeType),
                  parameters: getDefaultParametersForNodeType(nodeType),
                };
                addNode(newNode);
              }}
            />
          </div>
        )}

        {/* Node Canvas */}
        <div className="material-editor-canvas">
          <NodeCanvas
            material={material}
            selectedNodes={selectedNodes}
            onNodeSelect={selectNode}
            onNodeUpdate={updateNode}
            onNodeDelete={removeNode}
            onConnectionAdd={addConnection}
            onConnectionRemove={removeConnection}
          />
        </div>

        {/* Preview Panel */}
        <div className="material-editor-preview">
          <PreviewPanel material={material} />
        </div>
      </div>

      {/* Material Manager Modal */}
      {isManagerOpen && (
        <MaterialManager
          material={material}
          onLoad={(loadedMaterial) => {
            updateMaterial(loadedMaterial);
            setIsManagerOpen(false);
          }}
          onClose={() => setIsManagerOpen(false)}
        />
      )}
    </div>
  );
};

// Helper functions for default node configuration
function getDefaultInputsForNodeType(nodeType: NodeType) {
  // Define default inputs based on node type
  switch (nodeType) {
    case NodeType.Multiply:
    case NodeType.Add:
    case NodeType.Subtract:
    case NodeType.Divide:
    case NodeType.Power:
    case NodeType.Min:
    case NodeType.Max:
      return [
        { id: 'a', name: 'A', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: true },
        { id: 'b', name: 'B', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: true },
      ];
    case NodeType.Lerp:
    case NodeType.Mix:
      return [
        { id: 'a', name: 'A', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: true },
        { id: 'b', name: 'B', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: true },
        { id: 'alpha', name: 'Alpha', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: true },
      ];
    case NodeType.PBRMaster:
      return [
        { id: 'baseColor', name: 'Base Color', dataType: 'color' as any, connectedTo: null, nodeId: '', isInput: true },
        { id: 'metallic', name: 'Metallic', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: true },
        { id: 'roughness', name: 'Roughness', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: true },
        { id: 'normal', name: 'Normal', dataType: 'vector3' as any, connectedTo: null, nodeId: '', isInput: true },
        { id: 'ao', name: 'AO', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: true },
      ];
    case NodeType.TextureSample:
      return [
        { id: 'uv', name: 'UV', dataType: 'vector2' as any, connectedTo: null, nodeId: '', isInput: true },
      ];
    case NodeType.NormalMap:
      return [
        { id: 'normal', name: 'Normal', dataType: 'vector3' as any, connectedTo: null, nodeId: '', isInput: true },
        { id: 'strength', name: 'Strength', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: true },
      ];
    default:
      return [];
  }
}

function getDefaultOutputsForNodeType(nodeType: NodeType) {
  switch (nodeType) {
    case NodeType.Multiply:
    case NodeType.Add:
    case NodeType.Subtract:
    case NodeType.Divide:
    case NodeType.Power:
    case NodeType.Min:
    case NodeType.Max:
    case NodeType.Sqrt:
    case NodeType.Abs:
      return [
        { id: 'result', name: 'Result', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: false },
      ];
    case NodeType.Lerp:
    case NodeType.Mix:
      return [
        { id: 'result', name: 'Result', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: false },
      ];
    case NodeType.TextureSample:
      return [
        { id: 'rgba', name: 'RGBA', dataType: 'vector4' as any, connectedTo: null, nodeId: '', isInput: false },
        { id: 'r', name: 'R', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: false },
        { id: 'g', name: 'G', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: false },
        { id: 'b', name: 'B', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: false },
        { id: 'a', name: 'A', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: false },
      ];
    case NodeType.UVCoordinate:
      return [
        { id: 'uv', name: 'UV', dataType: 'vector2' as any, connectedTo: null, nodeId: '', isInput: false },
      ];
    case NodeType.ColorInput:
      return [
        { id: 'color', name: 'Color', dataType: 'color' as any, connectedTo: null, nodeId: '', isInput: false },
      ];
    case NodeType.FloatInput:
      return [
        { id: 'value', name: 'Value', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: false },
      ];
    default:
      return [];
  }
}

function getDefaultParametersForNodeType(nodeType: NodeType) {
  switch (nodeType) {
    case NodeType.FloatInput:
      return [
        { id: 'value', name: 'Value', dataType: 'float' as any, value: 0.5, min: 0, max: 1, step: 0.01 },
      ];
    case NodeType.ColorInput:
      return [
        { id: 'r', name: 'R', dataType: 'float' as any, value: 1.0, min: 0, max: 1, step: 0.01 },
        { id: 'g', name: 'G', dataType: 'float' as any, value: 1.0, min: 0, max: 1, step: 0.01 },
        { id: 'b', name: 'B', dataType: 'float' as any, value: 1.0, min: 0, max: 1, step: 0.01 },
        { id: 'a', name: 'A', dataType: 'float' as any, value: 1.0, min: 0, max: 1, step: 0.01 },
      ];
    case NodeType.TextureSample:
      return [
        { id: 'texturePath', name: 'Texture Path', dataType: 'texture2d' as any, value: '' },
      ];
    case NodeType.NormalMap:
      return [
        { id: 'strength', name: 'Strength', dataType: 'float' as any, value: 1.0, min: 0, max: 2, step: 0.1 },
      ];
    case NodeType.PBRMaster:
      return [
        { id: 'baseColor', name: 'Base Color', dataType: 'color' as any, value: [1.0, 1.0, 1.0, 1.0] },
        { id: 'metallic', name: 'Metallic', dataType: 'float' as any, value: 0.0, min: 0, max: 1, step: 0.01 },
        { id: 'roughness', name: 'Roughness', dataType: 'float' as any, value: 0.5, min: 0, max: 1, step: 0.01 },
        { id: 'ao', name: 'AO', dataType: 'float' as any, value: 1.0, min: 0, max: 1, step: 0.01 },
      ];
    default:
      return [];
  }
}

export default MaterialEditor;
