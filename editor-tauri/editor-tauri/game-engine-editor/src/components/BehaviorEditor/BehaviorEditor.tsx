/**
 * Behavior Tree Editor Component
 * Main editor component that integrates all behavior tree editing features
 */

import React, { useState, useCallback, useEffect, useRef } from 'react';
import { BehaviorTree, BehaviorNode as BehaviorNodeType, NodeType, NodeStatus, Blackboard } from '../../types/behavior';
import { NodeCanvas } from './NodeCanvas';
import { NodePalette } from './NodePalette';
import { PropertiesPanel } from './PropertiesPanel';
import { BlackboardEditor } from './BlackboardEditor';
import { DebugPanel } from './DebugPanel';
import { BehaviorTreeManager } from './BehaviorTreeManager';
import { Toolbar } from './Toolbar';
import './BehaviorEditor.css';

interface BehaviorEditorProps {
  initialTree?: BehaviorTree;
  onTreeChange?: (tree: BehaviorTree) => void;
}

// Create a default root node
const createDefaultRoot = (): BehaviorNodeType => ({
  id: 'root',
  type: NodeType.Sequence,
  name: 'Root',
  description: 'Root node',
  position: { x: 100, y: 100 },
  children: [],
  parameters: [],
  status: NodeStatus.Idle,
});

export const BehaviorEditor: React.FC<BehaviorEditorProps> = ({
  initialTree,
  onTreeChange,
}) => {
  // Tree state
  const [tree, setTree] = useState<BehaviorTree>(
    initialTree || {
      id: 'tree_' + Date.now(),
      name: 'New Behavior Tree',
      description: 'A new behavior tree',
      root: createDefaultRoot(),
      blackboard: { variables: {} },
      createdAt: Date.now(),
      updatedAt: Date.now(),
    }
  );

  // UI state
  const [selectedNodes, setSelectedNodes] = useState<Set<string>>(new Set());
  const [selectedNode, setSelectedNode] = useState<BehaviorNodeType | null>(null);
  const [copiedNodes, setCopiedNodes] = useState<BehaviorNodeType[]>([]);
  const [isPaletteOpen, setIsPaletteOpen] = useState(true);
  const [isPropertiesOpen, setIsPropertiesOpen] = useState(true);
  const [isBlackboardOpen, setIsBlackboardOpen] = useState(false);
  const [isManagerOpen, setIsManagerOpen] = useState(false);

  // Debug state
  const [isDebugMode, setIsDebugMode] = useState(false);
  const [isRunning, setIsRunning] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const [currentNodeId, setCurrentNodeId] = useState<string | null>(null);
  const [nodeStates, setNodeStates] = useState<Record<string, NodeStatus>>({});
  const [breakpoints, setBreakpoints] = useState<Set<string>>(new Set());

  // Saved trees
  const [savedTrees, setSavedTrees] = useState<BehaviorTree[]>([]);

  // Refs
  const treeRef = useRef(tree);
  treeRef.current = tree;

  // Update tree and notify parent
  const updateTree = useCallback((updatedTree: BehaviorTree) => {
    const newTree = {
      ...updatedTree,
      updatedAt: Date.now(),
    };
    setTree(newTree);
    if (onTreeChange) {
      onTreeChange(newTree);
    }
  }, [onTreeChange]);

  // Helper function to find and update a node in the tree
  const updateNodeInTree = useCallback((
    node: BehaviorNodeType | null,
    nodeId: string,
    updates: Partial<BehaviorNodeType>
  ): BehaviorNodeType | null => {
    if (!node) return null;

    if (node.id === nodeId) {
      return { ...node, ...updates };
    }

    if (node.children) {
      return {
        ...node,
        children: node.children.map(child =>
          updateNodeInTree(child, nodeId, updates) || child
        ),
      };
    }

    return node;
  }, []);

  // Helper function to remove a node from the tree
  const removeNodeFromTree = useCallback((
    node: BehaviorNodeType | null,
    nodeId: string
  ): BehaviorNodeType | null => {
    if (!node) return null;

    if (node.id === nodeId) {
      return null;
    }

    if (node.children) {
      const filteredChildren = node.children
        .map(child => removeNodeFromTree(child, nodeId))
        .filter(Boolean) as BehaviorNodeType[];

      return {
        ...node,
        children: filteredChildren,
      };
    }

    return node;
  }, []);

  // Node operations
  const addNode = useCallback((nodeType: NodeType, parentNode?: BehaviorNodeType) => {
    const newNode: BehaviorNodeType = {
      id: `node_${Date.now()}`,
      type: nodeType,
      name: nodeType.charAt(0).toUpperCase() + nodeType.slice(1),
      position: { x: 100, y: 100 },
      children: [],
      parameters: [],
      status: NodeStatus.Idle,
    };

    if (parentNode) {
      const updatedRoot = updateNodeInTree(tree.root, parentNode.id, {
        children: [...parentNode.children, newNode],
      });
      if (updatedRoot) {
        updateTree({ ...tree, root: updatedRoot });
      }
    } else {
      updateTree({ ...tree, root: newNode });
    }

    setSelectedNode(newNode);
    setSelectedNodes(new Set([newNode.id]));
  }, [tree, updateNodeInTree, updateTree]);

  const removeNode = useCallback((nodeId: string) => {
    if (tree.root?.id === nodeId) {
      // Don't remove root, just clear children
      updateTree({
        ...tree,
        root: { ...tree.root!, children: [] },
      });
    } else {
      const updatedRoot = removeNodeFromTree(tree.root, nodeId);
      updateTree({ ...tree, root: updatedRoot || createDefaultRoot() });
    }

    if (selectedNode?.id === nodeId) {
      setSelectedNode(null);
    }
    setSelectedNodes(prev => {
      const newSet = new Set(prev);
      newSet.delete(nodeId);
      return newSet;
    });
  }, [tree, removeNodeFromTree, updateTree, selectedNode]);

  const updateNode = useCallback((nodeId: string, updates: Partial<BehaviorNodeType>) => {
    const updatedRoot = updateNodeInTree(tree.root, nodeId, updates);
    if (updatedRoot) {
      updateTree({ ...tree, root: updatedRoot });

      // Update selected node if it's the one being updated
      if (selectedNode?.id === nodeId) {
        setSelectedNode({ ...selectedNode, ...updates });
      }
    }
  }, [tree, updateNodeInTree, updateTree, selectedNode]);

  const addChild = useCallback((parentId: string, childNode: BehaviorNodeType) => {
    const updatedRoot = updateNodeInTree(tree.root, parentId, {
      children: [...(selectedNode?.children || []), childNode],
    });
    if (updatedRoot) {
      updateTree({ ...tree, root: updatedRoot });
    }
  }, [tree, updateNodeInTree, updateTree, selectedNode]);

  // Selection operations
  const selectNode = useCallback((nodeId: string, addToSelection: boolean = false) => {
    // Find the node in the tree
    const findNode = (node: BehaviorNodeType | null): BehaviorNodeType | null => {
      if (!node) return null;
      if (node.id === nodeId) return node;
      for (const child of node.children) {
        const found = findNode(child);
        if (found) return found;
      }
      return null;
    };

    const node = findNode(tree.root);

    setSelectedNodes(prev => {
      const newSelection = addToSelection ? new Set<string>(prev) : new Set<string>();
      if (newSelection.has(nodeId)) {
        newSelection.delete(nodeId);
      } else {
        newSelection.add(nodeId);
      }
      return newSelection;
    });

    if (node && !addToSelection) {
      setSelectedNode(node);
    }
  }, [tree.root]);

  const clearSelection = useCallback(() => {
    setSelectedNodes(new Set<string>());
    setSelectedNode(null);
  }, []);

  // Delete selected nodes
  const deleteSelected = useCallback(() => {
    selectedNodes.forEach(nodeId => {
      removeNode(nodeId);
    });
    clearSelection();
  }, [selectedNodes, removeNode, clearSelection]);

  // Copy/Paste
  const copySelected = useCallback(() => {
    const nodesToCopy = Array.from(selectedNodes)
      .map(id => {
        const findNode = (node: BehaviorNodeType | null): BehaviorNodeType | null => {
          if (!node) return null;
          if (node.id === id) return node;
          for (const child of node.children) {
            const found = findNode(child);
            if (found) return found;
          }
          return null;
        };
        return findNode(tree.root);
      })
      .filter(Boolean) as BehaviorNodeType[];

    setCopiedNodes(nodesToCopy);
  }, [selectedNodes, tree.root]);

  const pasteNodes = useCallback(() => {
    // Implementation for paste - would need to handle reparenting
    console.log('Paste nodes:', copiedNodes);
  }, [copiedNodes]);

  // Blackboard operations
  const updateBlackboard = useCallback((blackboard: Blackboard) => {
    updateTree({ ...tree, blackboard });
  }, [tree, updateTree]);

  // Debug operations
  const handleDebugRun = useCallback(() => {
    setIsRunning(true);
    setIsPaused(false);
    // Simulate running
    console.log('Starting behavior tree execution');
  }, []);

  const handleDebugPause = useCallback(() => {
    setIsPaused(true);
  }, []);

  const handleDebugStop = useCallback(() => {
    setIsRunning(false);
    setIsPaused(false);
    setCurrentNodeId(null);
    setNodeStates({});
  }, []);

  const handleDebugStep = useCallback(() => {
    // Step implementation
    console.log('Step');
  }, []);

  const handleDebugStepOver = useCallback(() => {
    // Step over implementation
    console.log('Step over');
  }, []);

  const handleToggleBreakpoint = useCallback((nodeId: string) => {
    setBreakpoints(prev => {
      const newSet = new Set(prev);
      if (newSet.has(nodeId)) {
        newSet.delete(nodeId);
      } else {
        newSet.add(nodeId);
      }
      return newSet;
    });
  }, []);

  const handleClearBreakpoints = useCallback(() => {
    setBreakpoints(new Set());
  }, []);

  // Tree operations
  const handleNewTree = useCallback(() => {
    if (confirm('Create new behavior tree? Unsaved changes will be lost.')) {
      const newTree: BehaviorTree = {
        id: 'tree_' + Date.now(),
        name: 'New Behavior Tree',
        description: 'A new behavior tree',
        root: createDefaultRoot(),
        blackboard: { variables: {} },
        createdAt: Date.now(),
        updatedAt: Date.now(),
      };
      setTree(newTree);
      setSelectedNode(null);
      setSelectedNodes(new Set());
    }
  }, []);

  const handleSaveTree = useCallback(() => {
    // Save to local storage or backend
    const updatedTrees = [...savedTrees.filter(t => t.id !== tree.id), tree];
    setSavedTrees(updatedTrees);
    localStorage.setItem('behavior_trees', JSON.stringify(updatedTrees));
    console.log('Tree saved:', tree.name);
  }, [savedTrees, tree]);

  const handleLoadTree = useCallback((loadedTree: BehaviorTree) => {
    setTree(loadedTree);
    setIsManagerOpen(false);
  }, []);

  const handleValidateTree = useCallback(() => {
    // Basic validation
    const errors: string[] = [];
    const warnings: string[] = [];

    if (!tree.root) {
      errors.push('Behavior tree has no root node');
    }

    // Check for empty children in composite nodes
    const checkNode = (node: BehaviorNodeType, path: string[]) => {
      if (['sequence', 'selector', 'parallel'].includes(node.type)) {
        if (node.children.length === 0) {
          warnings.push(`Node ${node.name} at ${path.join('/ > ')} has no children`);
        }
      }

      node.children.forEach((child, idx) => {
        checkNode(child, [...path, `${node.name}[${idx}]`]);
      });
    };

    if (tree.root) {
      checkNode(tree.root, ['Root']);
    }

    console.log('Validation result:', { errors, warnings });
    alert(`Validation complete:\nErrors: ${errors.length}\nWarnings: ${warnings.length}`);
  }, [tree]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
        return;
      }

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

      // Ctrl+S: Save tree
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault();
        handleSaveTree();
      }

      // Ctrl+N: New tree
      if ((e.ctrlKey || e.metaKey) && e.key === 'n') {
        e.preventDefault();
        handleNewTree();
      }

      // Escape: Clear selection
      if (e.key === 'Escape') {
        clearSelection();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [selectedNodes, deleteSelected, copySelected, pasteNodes, clearSelection, handleSaveTree, handleNewTree]);

  // Load saved trees from localStorage
  useEffect(() => {
    const saved = localStorage.getItem('behavior_trees');
    if (saved) {
      try {
        setSavedTrees(JSON.parse(saved));
      } catch (e) {
        console.error('Failed to load saved trees:', e);
      }
    }
  }, []);

  return (
    <div className="behavior-editor">
      {/* Toolbar */}
      <Toolbar
        onNew={handleNewTree}
        onSave={handleSaveTree}
        onLoad={() => setIsManagerOpen(true)}
        onValidate={handleValidateTree}
        onZoomIn={() => console.log('Zoom in')}
        onZoomOut={() => console.log('Zoom out')}
        onResetView={() => console.log('Reset view')}
        onToggleDebug={() => setIsDebugMode(!isDebugMode)}
        isDebugMode={isDebugMode}
        canSave={true}
      />

      {/* Main Content */}
      <div className="editor-content">
        {/* Node Palette */}
        <NodePalette
          isOpen={isPaletteOpen}
          onToggle={() => setIsPaletteOpen(!isPaletteOpen)}
          onNodeTypeSelect={(nodeType) => {
            const newNode: BehaviorNodeType = {
              id: `node_${Date.now()}`,
              type: nodeType,
              name: nodeType.charAt(0).toUpperCase() + nodeType.slice(1),
              position: { x: 100 + Math.random() * 200, y: 100 + Math.random() * 200 },
              children: [],
              parameters: [],
              status: NodeStatus.Idle,
            };

            if (selectedNode && ['sequence', 'selector', 'parallel', 'inverter', 'repeater', 'cooldown'].includes(selectedNode.type)) {
              // Add as child of selected node
              addChild(selectedNode.id, newNode);
            } else {
              // Add as root or standalone
              if (!tree.root || confirm('Replace root node?')) {
                updateTree({ ...tree, root: newNode });
              }
            }
          }}
        />

        {/* Node Canvas */}
        <div className="editor-canvas">
          <NodeCanvas
            tree={tree}
            selectedNodes={selectedNodes}
            onNodeSelect={selectNode}
            onNodeUpdate={updateNode}
            onNodeDelete={removeNode}
            onAddChild={addChild}
            onRemoveChild={(parentId, childId) => {
              const parent = tree.root;
              const updateParent = updateNodeInTree(parent, parentId, {
                children: parent!.children.filter(c => c.id !== childId),
              });
              if (updateParent) {
                updateTree({ ...tree, root: updateParent });
              }
            }}
          />
        </div>

        {/* Properties Panel */}
        <PropertiesPanel
          node={selectedNode}
          onNodeUpdate={updateNode}
          isOpen={isPropertiesOpen}
          onToggle={() => setIsPropertiesOpen(!isPropertiesOpen)}
        />

        {/* Blackboard Editor */}
        <BlackboardEditor
          blackboard={tree.blackboard}
          onBlackboardUpdate={updateBlackboard}
          isOpen={isBlackboardOpen}
          onToggle={() => setIsBlackboardOpen(!isBlackboardOpen)}
        />

        {/* Debug Panel */}
        <DebugPanel
          isDebugMode={isDebugMode}
          isRunning={isRunning}
          isPaused={isPaused}
          currentNodeId={currentNodeId}
          nodeStates={nodeStates}
          blackboard={tree.blackboard}
          breakpoints={breakpoints}
          onPlay={handleDebugRun}
          onPause={handleDebugPause}
          onStop={handleDebugStop}
          onStep={handleDebugStep}
          onStepOver={handleDebugStepOver}
          onToggleBreakpoint={handleToggleBreakpoint}
          onClearBreakpoints={handleClearBreakpoints}
        />
      </div>

      {/* Tree Manager Modal */}
      {isManagerOpen && (
        <BehaviorTreeManager
          isOpen={isManagerOpen}
          trees={savedTrees}
          onLoad={handleLoadTree}
          onClose={() => setIsManagerOpen(false)}
          onDelete={(treeId) => {
            const updatedTrees = savedTrees.filter(t => t.id !== treeId);
            setSavedTrees(updatedTrees);
            localStorage.setItem('behavior_trees', JSON.stringify(updatedTrees));
          }}
          onDuplicate={(treeToDuplicate) => {
            const duplicated: BehaviorTree = {
              ...treeToDuplicate,
              id: 'tree_' + Date.now(),
              name: treeToDuplicate.name + ' (Copy)',
              createdAt: Date.now(),
              updatedAt: Date.now(),
            };
            const updatedTrees = [...savedTrees, duplicated];
            setSavedTrees(updatedTrees);
            localStorage.setItem('behavior_trees', JSON.stringify(updatedTrees));
          }}
        />
      )}
    </div>
  );
};

export default BehaviorEditor;
