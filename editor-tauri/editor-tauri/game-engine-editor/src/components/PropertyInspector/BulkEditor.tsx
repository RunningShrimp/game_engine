// Bulk Property Editor for Multiple Entities

import React, { useState, useEffect } from 'react';
import { Entity, Vector3 } from '../../types/engine';
import { SelectionManager } from '../../utils/SelectionManager';
import { BatchOperationManager } from '../../utils/BatchOperation';
import { AlignmentUtils } from '../../utils/AlignmentUtils';
import { RenamePattern, MaterialBatchOperation } from '../../types/selection';

interface BulkEditorProps {
  selectionManager: SelectionManager;
  batchOperationManager: BatchOperationManager;
  entities: Map<string, Entity>;
  onEntityChange?: (id: string, changes: Partial<Entity>) => void;
}

export function BulkEditor({
  selectionManager,
  batchOperationManager,
  entities,
  onEntityChange,
}: BulkEditorProps) {
  const [selectedEntities, setSelectedEntities] = useState<Entity[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);
  const [progress, setProgress] = useState({ current: 0, total: 0 });
  const [activeTab, setActiveTab] = useState<'transform' | 'components' | 'materials' | 'actions'>('transform');

  useEffect(() => {
    const unsubscribe = selectionManager.onSelectionChange((state) => {
      const entities_list = state.selectedIds
        .map((id) => entities.get(id))
        .filter((e): e is Entity => e !== undefined);
      setSelectedEntities(entities_list);
    });

    return () => {
      unsubscribe();
    };
  }, [selectionManager, entities]);

  if (selectedEntities.length === 0) {
    return (
      <div className="bulk-editor-empty">
        <p>No entities selected</p>
        <p className="hint">Select multiple entities to edit their properties in bulk</p>
      </div>
    );
  }

  if (selectedEntities.length === 1) {
    return (
      <div className="bulk-editor-single">
        <p>Single entity selected</p>
        <p className="hint">Use the Property Inspector to edit this entity</p>
      </div>
    );
  }

  return (
    <div className="bulk-editor">
      <BulkHeader
        count={selectedEntities.length}
        onDeselectAll={() => selectionManager.deselectAll()}
        onInvertSelection={() => selectionManager.invertSelection()}
      />

      {isProcessing && (
        <div className="bulk-progress">
          <div className="progress-bar">
            <div
              className="progress-fill"
              style={{
                width: `${(progress.current / progress.total) * 100}%`,
              }}
            />
          </div>
          <span className="progress-text">
            Processing {progress.current} / {progress.total}...
          </span>
        </div>
      )}

      <div className="bulk-tabs">
        <button
          className={`tab ${activeTab === 'transform' ? 'active' : ''}`}
          onClick={() => setActiveTab('transform')}
        >
          Transform
        </button>
        <button
          className={`tab ${activeTab === 'components' ? 'active' : ''}`}
          onClick={() => setActiveTab('components')}
        >
          Components
        </button>
        <button
          className={`tab ${activeTab === 'materials' ? 'active' : ''}`}
          onClick={() => setActiveTab('materials')}
        >
          Materials
        </button>
        <button
          className={`tab ${activeTab === 'actions' ? 'active' : ''}`}
          onClick={() => setActiveTab('actions')}
        >
          Actions
        </button>
      </div>

      {activeTab === 'transform' && (
        <BulkTransformEditor
          entities={selectedEntities}
          onTransformChange={handleTransformChange}
        />
      )}

      {activeTab === 'components' && (
        <BulkComponentEditor
          entities={selectedEntities}
          onComponentOperation={handleComponentOperation}
        />
      )}

      {activeTab === 'materials' && (
        <BulkMaterialEditor
          entities={selectedEntities}
          onMaterialOperation={handleMaterialOperation}
        />
      )}

      {activeTab === 'actions' && (
        <BulkActionButtons
          entities={selectedEntities}
          onDelete={handleBatchDelete}
          onRename={handleBatchRename}
          onDuplicate={handleBatchDuplicate}
          onToggleVisibility={handleToggleVisibility}
          onToggleLocked={handleToggleLocked}
        />
      )}
    </div>
  );

  // Handlers

  async function handleTransformChange(transform: {
    position?: Vector3;
    rotation?: Vector3;
    scale?: Vector3;
  }) {
    setIsProcessing(true);
    setProgress({ current: 0, total: selectedEntities.length });

    try {
      if (transform.position) {
        await batchOperationManager.batchMove(transform.position, {
          progressCallback: (current, total) => {
            setProgress({ current, total });
          },
        });
      }

      if (transform.rotation) {
        await batchOperationManager.batchRotate(transform.rotation, 'local', {
          progressCallback: (current, total) => {
            setProgress({ current, total });
          },
        });
      }

      if (transform.scale) {
        await batchOperationManager.batchScale(transform.scale, {
          progressCallback: (current, total) => {
            setProgress({ current, total });
          },
        });
      }
    } finally {
      setIsProcessing(false);
    }
  }

  async function handleComponentOperation(operation: any) {
    setIsProcessing(true);
    setProgress({ current: 0, total: selectedEntities.length });

    try {
      await batchOperationManager.batchComponentOperation(operation, {
        progressCallback: (current, total) => {
          setProgress({ current, total });
        },
      });
    } finally {
      setIsProcessing(false);
    }
  }

  async function handleMaterialOperation(operation: MaterialBatchOperation) {
    setIsProcessing(true);
    setProgress({ current: 0, total: selectedEntities.length });

    try {
      await batchOperationManager.batchApplyMaterial(operation, {
        progressCallback: (current, total) => {
          setProgress({ current, total });
        },
      });
    } finally {
      setIsProcessing(false);
    }
  }

  async function handleBatchDelete() {
    const confirmed = window.confirm(
      `Delete ${selectedEntities.length} entities? This action cannot be undone.`
    );

    if (!confirmed) return;

    setIsProcessing(true);
    setProgress({ current: 0, total: selectedEntities.length });

    try {
      await batchOperationManager.batchDelete({
        progressCallback: (current, total) => {
          setProgress({ current, total });
        },
      });
    } finally {
      setIsProcessing(false);
    }
  }

  async function handleBatchRename(pattern: RenamePattern) {
    setIsProcessing(true);
    setProgress({ current: 0, total: selectedEntities.length });

    try {
      await batchOperationManager.batchRename(pattern, {
        progressCallback: (current, total) => {
          setProgress({ current, total });
        },
      });
    } finally {
      setIsProcessing(false);
    }
  }

  async function handleBatchDuplicate() {
    // Implement duplication logic
    console.log('Batch duplicate not yet implemented');
  }

  async function handleToggleVisibility(visible: boolean) {
    setIsProcessing(true);
    setProgress({ current: 0, total: selectedEntities.length });

    try {
      await batchOperationManager.batchToggleVisibility(visible, {
        progressCallback: (current, total) => {
          setProgress({ current, total });
        },
      });
    } finally {
      setIsProcessing(false);
    }
  }

  async function handleToggleLocked(locked: boolean) {
    setIsProcessing(true);
    setProgress({ current: 0, total: selectedEntities.length });

    try {
      await batchOperationManager.batchToggleLocked(locked, {
        progressCallback: (current, total) => {
          setProgress({ current, total });
        },
      });
    } finally {
      setIsProcessing(false);
    }
  }
}

// Header Component
function BulkHeader({
  count,
  onDeselectAll,
  onInvertSelection,
}: {
  count: number;
  onDeselectAll: () => void;
  onInvertSelection: () => void;
}) {
  return (
    <div className="bulk-header">
      <h3>Bulk Editor</h3>
      <div className="selection-info">
        <span className="count">{count} entities selected</span>
      </div>
      <div className="header-actions">
        <button onClick={onDeselectAll} className="btn-secondary">
          Deselect All
        </button>
        <button onClick={onInvertSelection} className="btn-secondary">
          Invert Selection
        </button>
      </div>
    </div>
  );
}

// Transform Editor Component
function BulkTransformEditor({
  entities,
  onTransformChange,
}: {
  entities: Entity[];
  onTransformChange: (transform: {
    position?: Vector3;
    rotation?: Vector3;
    scale?: Vector3;
  }) => void;
}) {
  const [offset, setOffset] = useState({ x: 0, y: 0, z: 0 });
  const [rotation, setRotation] = useState({ x: 0, y: 0, z: 0 });
  const [scale, setScale] = useState({ x: 1, y: 1, z: 1 });

  const handleApplyPosition = () => {
    onTransformChange({
      position: { x: offset.x, y: offset.y, z: offset.z },
    });
    setOffset({ x: 0, y: 0, z: 0 });
  };

  const handleApplyRotation = () => {
    onTransformChange({
      rotation: { x: rotation.x, y: rotation.y, z: rotation.z },
    });
    setRotation({ x: 0, y: 0, z: 0 });
  };

  const handleApplyScale = () => {
    onTransformChange({
      scale: { x: scale.x, y: scale.y, z: scale.z },
    });
    setScale({ x: 1, y: 1, z: 1 });
  };

  return (
    <div className="bulk-transform-editor">
      <h4>Position Offset</h4>
      <div className="transform-inputs">
        <label>
          X:
          <input
            type="number"
            value={offset.x}
            onChange={(e) => setOffset({ ...offset, x: parseFloat(e.target.value) || 0 })}
            step="0.1"
          />
        </label>
        <label>
          Y:
          <input
            type="number"
            value={offset.y}
            onChange={(e) => setOffset({ ...offset, y: parseFloat(e.target.value) || 0 })}
            step="0.1"
          />
        </label>
        <label>
          Z:
          <input
            type="number"
            value={offset.z}
            onChange={(e) => setOffset({ ...offset, z: parseFloat(e.target.value) || 0 })}
            step="0.1"
          />
        </label>
        <button onClick={handleApplyPosition} className="btn-primary">
          Apply
        </button>
      </div>

      <h4>Rotation (radians)</h4>
      <div className="transform-inputs">
        <label>
          X:
          <input
            type="number"
            value={rotation.x}
            onChange={(e) => setRotation({ ...rotation, x: parseFloat(e.target.value) || 0 })}
            step="0.1"
          />
        </label>
        <label>
          Y:
          <input
            type="number"
            value={rotation.y}
            onChange={(e) => setRotation({ ...rotation, y: parseFloat(e.target.value) || 0 })}
            step="0.1"
          />
        </label>
        <label>
          Z:
          <input
            type="number"
            value={rotation.z}
            onChange={(e) => setRotation({ ...rotation, z: parseFloat(e.target.value) || 0 })}
            step="0.1"
          />
        </label>
        <button onClick={handleApplyRotation} className="btn-primary">
          Apply
        </button>
      </div>

      <h4>Scale Multiplier</h4>
      <div className="transform-inputs">
        <label>
          X:
          <input
            type="number"
            value={scale.x}
            onChange={(e) => setScale({ ...scale, x: parseFloat(e.target.value) || 1 })}
            step="0.1"
          />
        </label>
        <label>
          Y:
          <input
            type="number"
            value={scale.y}
            onChange={(e) => setScale({ ...scale, y: parseFloat(e.target.value) || 1 })}
            step="0.1"
          />
        </label>
        <label>
          Z:
          <input
            type="number"
            value={scale.z}
            onChange={(e) => setScale({ ...scale, z: parseFloat(e.target.value) || 1 })}
            step="0.1"
          />
        </label>
        <button onClick={handleApplyScale} className="btn-primary">
          Apply
        </button>
      </div>
    </div>
  );
}

// Component Editor Component
function BulkComponentEditor({
  entities,
  onComponentOperation,
}: {
  entities: Entity[];
  onComponentOperation: (operation: any) => void;
}) {
  const [componentType, setComponentType] = useState('');
  const [operation, setOperation] = useState<'add' | 'remove' | 'toggle'>('add');

  const handleApply = () => {
    if (!componentType) return;

    onComponentOperation({
      mode: operation,
      componentType,
    });

    setComponentType('');
  };

  return (
    <div className="bulk-component-editor">
      <h4>Component Operations</h4>
      <div className="component-inputs">
        <select
          value={operation}
          onChange={(e) => setOperation(e.target.value as any)}
        >
          <option value="add">Add Component</option>
          <option value="remove">Remove Component</option>
          <option value="toggle">Toggle Component</option>
        </select>

        <input
          type="text"
          value={componentType}
          onChange={(e) => setComponentType(e.target.value)}
          placeholder="Component type (e.g., MeshRenderer)"
        />

        <button onClick={handleApply} className="btn-primary">
          Apply
        </button>
      </div>

      <div className="common-components">
        <h5>Common Components:</h5>
        <button onClick={() => setComponentType('MeshRenderer')}>
          MeshRenderer
        </button>
        <button onClick={() => setComponentType('Collider')}>
          Collider
        </button>
        <button onClick={() => setComponentType('Rigidbody')}>
          Rigidbody
        </button>
        <button onClick={() => setComponentType('Light')}>
          Light
        </button>
        <button onClick={() => setComponentType('AudioSource')}>
          AudioSource
        </button>
      </div>
    </div>
  );
}

// Material Editor Component
function BulkMaterialEditor({
  entities,
  onMaterialOperation,
}: {
  entities: Entity[];
  onMaterialOperation: (operation: MaterialBatchOperation) => void;
}) {
  const [materialId, setMaterialId] = useState('');
  const [operation, setOperation] = useState<'apply' | 'replace'>('apply');

  const handleApply = () => {
    if (!materialId) return;

    onMaterialOperation({
      mode: operation,
      materialId,
    });

    setMaterialId('');
  };

  return (
    <div className="bulk-material-editor">
      <h4>Material Operations</h4>
      <div className="material-inputs">
        <select
          value={operation}
          onChange={(e) => setOperation(e.target.value as any)}
        >
          <option value="apply">Apply Material</option>
          <option value="replace">Replace Material</option>
        </select>

        <input
          type="text"
          value={materialId}
          onChange={(e) => setMaterialId(e.target.value)}
          placeholder="Material ID"
        />

        <button onClick={handleApply} className="btn-primary">
          Apply
        </button>
      </div>
    </div>
  );
}

// Action Buttons Component
function BulkActionButtons({
  entities,
  onDelete,
  onRename,
  onDuplicate,
  onToggleVisibility,
  onToggleLocked,
}: {
  entities: Entity[];
  onDelete: () => void;
  onRename: (pattern: RenamePattern) => void;
  onDuplicate: () => void;
  onToggleVisibility: (visible: boolean) => void;
  onToggleLocked: (locked: boolean) => void;
}) {
  const [renameMode, setRenameMode] = useState<'prefix' | 'suffix' | 'number'>('prefix');
  const [renameValue, setRenameValue] = useState('');

  const handleRename = () => {
    if (!renameValue) return;

    onRename({
      mode: renameMode,
      value: renameValue,
      startNumber: 1,
      padding: 3,
    });

    setRenameValue('');
  };

  return (
    <div className="bulk-action-buttons">
      <h4>Bulk Actions</h4>

      <div className="action-group">
        <button onClick={onDelete} className="btn-danger">
          Delete All ({entities.length})
        </button>
        <button onClick={onDuplicate} className="btn-primary">
          Duplicate All
        </button>
      </div>

      <div className="action-group">
        <button onClick={() => onToggleVisibility(true)} className="btn-secondary">
          Show All
        </button>
        <button onClick={() => onToggleVisibility(false)} className="btn-secondary">
          Hide All
        </button>
      </div>

      <div className="action-group">
        <button onClick={() => onToggleLocked(true)} className="btn-secondary">
          Lock All
        </button>
        <button onClick={() => onToggleLocked(false)} className="btn-secondary">
          Unlock All
        </button>
      </div>

      <div className="rename-section">
        <h5>Bulk Rename</h5>
        <div className="rename-inputs">
          <select
            value={renameMode}
            onChange={(e) => setRenameMode(e.target.value as any)}
          >
            <option value="prefix">Add Prefix</option>
            <option value="suffix">Add Suffix</option>
            <option value="number">Number</option>
          </select>
          <input
            type="text"
            value={renameValue}
            onChange={(e) => setRenameValue(e.target.value)}
            placeholder="Enter value..."
          />
          <button onClick={handleRename} className="btn-primary">
            Rename
          </button>
        </div>
      </div>
    </div>
  );
}
