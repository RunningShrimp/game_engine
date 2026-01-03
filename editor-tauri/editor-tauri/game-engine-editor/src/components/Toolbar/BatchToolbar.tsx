// Batch Operation Toolbar

import React, { useState } from 'react';
import { SelectionManager } from '../../utils/SelectionManager';
import { BatchOperationManager } from '../../utils/BatchOperation';
import { AlignmentUtils } from '../../utils/AlignmentUtils';
import { AlignmentOptions, DistributionOptions } from '../../types/selection';

interface BatchToolbarProps {
  selectionManager: SelectionManager;
  batchOperationManager: BatchOperationManager;
  entities: Map<string, Entity>;
}

export function BatchToolbar({
  selectionManager,
  batchOperationManager,
  entities,
}: BatchToolbarProps) {
  const [showAlignMenu, setShowAlignMenu] = useState(false);
  const [showDistributeMenu, setShowDistributeMenu] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);

  const selectedCount = selectionManager.getSelectedCount();

  const handleAlign = async (axis: 'x' | 'y' | 'z', mode: 'min' | 'max' | 'center') => {
    const selectedEntities = selectionManager.getSelectedEntities();
    if (selectedEntities.length === 0) return;

    const options: AlignmentOptions = { axis, mode };
    const updates = AlignmentUtils.alignEntities(selectedEntities, options);

    await applyUpdates(updates);
    setShowAlignMenu(false);
  };

  const handleDistribute = async (axis: 'x' | 'y' | 'z') => {
    const selectedEntities = selectionManager.getSelectedEntities();
    if (selectedEntities.length === 0) return;

    const options: DistributionOptions = { axis, mode: 'equal' };
    const updates = AlignmentUtils.distributeEntities(selectedEntities, options);

    await applyUpdates(updates);
    setShowDistributeMenu(false);
  };

  const handleAlignToGrid = async () => {
    const selectedEntities = selectionManager.getSelectedEntities();
    if (selectedEntities.length === 0) return;

    const gridSize = 1.0; // Get from editor settings
    const updates = AlignmentUtils.alignToGrid(selectedEntities, gridSize);

    await applyUpdates(updates);
  };

  const handleArrangeInGrid = async () => {
    const selectedEntities = selectionManager.getSelectedEntities();
    if (selectedEntities.length === 0) return;

    const columns = Math.ceil(Math.sqrt(selectedEntities.length));
    const spacing = { x: 2, y: 0, z: 2 };
    const updates = AlignmentUtils.arrangeInGrid(selectedEntities, columns, spacing);

    await applyUpdates(updates);
  };

  const handleArrangeInCircle = async () => {
    const selectedEntities = selectionManager.getSelectedEntities();
    if (selectedEntities.length === 0) return;

    const radius = 5.0;
    const updates = AlignmentUtils.arrangeInCircle(selectedEntities, radius, 'y');

    await applyUpdates(updates);
  };

  const handleMatchRotation = async () => {
    const selectedEntities = selectionManager.getSelectedEntities();
    const primaryId = selectionManager.getPrimarySelection();

    if (selectedEntities.length === 0 || !primaryId) return;

    const updates = AlignmentUtils.matchRotation(selectedEntities, primaryId);
    await applyUpdates(updates);
  };

  const handleMatchScale = async () => {
    const selectedEntities = selectionManager.getSelectedEntities();
    const primaryId = selectionManager.getPrimarySelection();

    if (selectedEntities.length === 0 || !primaryId) return;

    const updates = AlignmentUtils.matchScale(selectedEntities, primaryId);
    await applyUpdates(updates);
  };

  async function applyUpdates(updates: Record<string, Partial<Entity>>) {
    setIsProcessing(true);

    try {
      for (const [id, update] of Object.entries(updates)) {
        const entity = entities.get(id);
        if (entity && !entity.locked) {
          // Apply the update
          // This would integrate with your entity update system
          console.log(`Updating entity ${id}:`, update);
        }
      }
    } finally {
      setIsProcessing(false);
    }
  }

  if (selectedCount <= 1) {
    return null;
  }

  return (
    <div className="batch-toolbar">
      <div className="toolbar-section">
        <span className="toolbar-label">{selectedCount} selected</span>
      </div>

      <div className="toolbar-section">
        <button
          className="toolbar-btn"
          onClick={() => setShowAlignMenu(!showAlignMenu)}
          title="Align"
        >
          <svg width="16" height="16" viewBox="0 0 16 16">
            <path d="M2 2h12v2H2zM2 6h8v2H2zM2 10h12v2H2zM2 14h8v2H2z" fill="currentColor" />
          </svg>
          Align
        </button>

        {showAlignMenu && (
          <div className="toolbar-dropdown">
            <div className="dropdown-section">
              <h5>Align X</h5>
              <button onClick={() => handleAlign('x', 'min')}>Min X</button>
              <button onClick={() => handleAlign('x', 'center')}>Center X</button>
              <button onClick={() => handleAlign('x', 'max')}>Max X</button>
            </div>

            <div className="dropdown-section">
              <h5>Align Y</h5>
              <button onClick={() => handleAlign('y', 'min')}>Min Y</button>
              <button onClick={() => handleAlign('y', 'center')}>Center Y</button>
              <button onClick={() => handleAlign('y', 'max')}>Max Y</button>
            </div>

            <div className="dropdown-section">
              <h5>Align Z</h5>
              <button onClick={() => handleAlign('z', 'min')}>Min Z</button>
              <button onClick={() => handleAlign('z', 'center')}>Center Z</button>
              <button onClick={() => handleAlign('z', 'max')}>Max Z</button>
            </div>

            <div className="dropdown-section">
              <h5>Grid</h5>
              <button onClick={handleAlignToGrid}>Align to Grid</button>
            </div>
          </div>
        )}
      </div>

      <div className="toolbar-section">
        <button
          className="toolbar-btn"
          onClick={() => setShowDistributeMenu(!showDistributeMenu)}
          title="Distribute"
        >
          <svg width="16" height="16" viewBox="0 0 16 16">
            <path d="M1 3h3v2H1zM6 3h3v2H6zM11 3h3v2h-3zM1 7h3v2H1zM6 7h3v2H6zM11 7h3v2h-3zM1 11h3v2H1zM6 11h3v2H6zM11 11h3v2h-3z" fill="currentColor" />
          </svg>
          Distribute
        </button>

        {showDistributeMenu && (
          <div className="toolbar-dropdown">
            <div className="dropdown-section">
              <h5>Distribute</h5>
              <button onClick={() => handleDistribute('x')}>Distribute X</button>
              <button onClick={() => handleDistribute('y')}>Distribute Y</button>
              <button onClick={() => handleDistribute('z')}>Distribute Z</button>
            </div>

            <div className="dropdown-section">
              <h5>Arrange</h5>
              <button onClick={handleArrangeInGrid}>Grid Layout</button>
              <button onClick={handleArrangeInCircle}>Circle Layout</button>
            </div>
          </div>
        )}
      </div>

      <div className="toolbar-section">
        <button
          className="toolbar-btn"
          onClick={handleMatchRotation}
          title="Match Rotation"
          disabled={isProcessing}
        >
          <svg width="16" height="16" viewBox="0 0 16 16">
            <path d="M8 2A6 6 0 1 0 8 14 6 6 0 0 0 8 2zM8 12A4 4 0 1 1 8 4 4 4 0 0 1 8 12z" fill="currentColor" />
          </svg>
          Match Rotation
        </button>

        <button
          className="toolbar-btn"
          onClick={handleMatchScale}
          title="Match Scale"
          disabled={isProcessing}
        >
          <svg width="16" height="16" viewBox="0 0 16 16">
            <path d="M2 2h4v4H2zM6 6h4v4H6zM10 10h4v4h-4z" fill="currentColor" />
          </svg>
          Match Scale
        </button>
      </div>

      <div className="toolbar-section">
        <button
          className="toolbar-btn"
          onClick={() => selectionManager.selectAll()}
          title="Select All (Ctrl+A)"
        >
          Select All
        </button>

        <button
          className="toolbar-btn"
          onClick={() => selectionManager.deselectAll()}
          title="Deselect All (Ctrl+D)"
        >
          Deselect All
        </button>

        <button
          className="toolbar-btn"
          onClick={() => selectionManager.invertSelection()}
          title="Invert Selection (Ctrl+I)"
        >
          Invert
        </button>
      </div>

      {isProcessing && (
        <div className="toolbar-status">
          <span className="spinner"></span>
          Processing...
        </div>
      )}
    </div>
  );
}

/**
 * Batch operation context menu
 */
export function BatchContextMenu({
  visible,
  x,
  y,
  onClose,
  onAction,
}: {
  visible: boolean;
  x: number;
  y: number;
  onClose: () => void;
  onAction: (action: string) => void;
}) {
  if (!visible) return null;

  const menuItems = [
    { label: 'Duplicate', action: 'duplicate', icon: '📋' },
    { label: 'Delete', action: 'delete', icon: '🗑️' },
    { separator: true },
    { label: 'Group', action: 'group', icon: '📦' },
    { label: 'Ungroup', action: 'ungroup', icon: '📂' },
    { separator: true },
    { label: 'Lock', action: 'lock', icon: '🔒' },
    { label: 'Unlock', action: 'unlock', icon: '🔓' },
    { separator: true },
    { label: 'Show', action: 'show', icon: '👁️' },
    { label: 'Hide', action: 'hide', icon: '👁️‍🗨️' },
  ];

  return (
    <div
      className="batch-context-menu"
      style={{
        position: 'fixed',
        left: x,
        top: y,
        zIndex: 10000,
      }}
      onContextMenu={(e) => e.preventDefault()}
    >
      {menuItems.map((item, index) => {
        if (item.separator) {
          return <div key={index} className="menu-separator" />;
        }

        return (
          <div
            key={index}
            className="menu-item"
            onClick={() => {
              onAction(item.action);
              onClose();
            }}
          >
            <span className="menu-icon">{item.icon}</span>
            <span className="menu-label">{item.label}</span>
          </div>
        );
      })}
    </div>
  );
}

/**
 * Keyboard shortcuts for batch operations
 */
export function useBatchShortcuts(
  selectionManager: SelectionManager,
  batchOperationManager: BatchOperationManager
) {
  React.useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      const ctrl = e.ctrlKey || e.metaKey;
      const shift = e.shiftKey;

      // Ctrl+A: Select All
      if (ctrl && e.key === 'a' && !shift) {
        e.preventDefault();
        selectionManager.selectAll();
        return;
      }

      // Ctrl+D: Deselect All
      if (ctrl && e.key === 'd' && !shift) {
        e.preventDefault();
        selectionManager.deselectAll();
        return;
      }

      // Ctrl+I: Invert Selection
      if (ctrl && e.key === 'i' && !shift) {
        e.preventDefault();
        selectionManager.invertSelection();
        return;
      }

      // Delete: Delete Selected
      if (e.key === 'Delete' && !ctrl && !shift) {
        e.preventDefault();
        await batchOperationManager.batchDelete();
        return;
      }

      // F2: Rename
      if (e.key === 'F2' && !ctrl && !shift) {
        e.preventDefault();
        // Show rename dialog
        return;
      }

      // Ctrl+G: Group
      if (ctrl && e.key === 'g' && !shift) {
        e.preventDefault();
        // Implement grouping
        return;
      }

      // Ctrl+Shift+G: Ungroup
      if (ctrl && shift && e.key === 'g') {
        e.preventDefault();
        // Implement ungrouping
        return;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [selectionManager, batchOperationManager]);
}
