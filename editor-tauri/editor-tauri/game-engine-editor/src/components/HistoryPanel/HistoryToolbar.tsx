// History Toolbar Component

import React from 'react';
import './HistoryToolbar.css';

interface HistoryToolbarProps {
  canUndo: boolean;
  canRedo: boolean;
  onUndo: () => void;
  onRedo: () => void;
  onSave: () => void;
  onClear: () => void;
  onExport: () => void;
  onImport: (event: React.ChangeEvent<HTMLInputElement>) => void;
  onOptimize: () => void;
}

export function HistoryToolbar({
  canUndo,
  canRedo,
  onUndo,
  onRedo,
  onSave,
  onClear,
  onExport,
  onImport,
  onOptimize,
}: HistoryToolbarProps) {
  return (
    <div className="history-toolbar">
      <div className="history-toolbar-group">
        <button
          className="history-toolbar-btn"
          onClick={onUndo}
          disabled={!canUndo}
          title="Undo (Ctrl+Z)"
        >
          <span className="icon">↶</span>
          Undo
        </button>
        <button
          className="history-toolbar-btn"
          onClick={onRedo}
          disabled={!canRedo}
          title="Redo (Ctrl+Y)"
        >
          <span className="icon">↷</span>
          Redo
        </button>
      </div>

      <div className="history-toolbar-group">
        <button
          className="history-toolbar-btn"
          onClick={onSave}
          title="Save History"
        >
          <span className="icon">💾</span>
          Save
        </button>
        <button
          className="history-toolbar-btn"
          onClick={onExport}
          title="Export History"
        >
          <span className="icon">📤</span>
          Export
        </button>
        <label className="history-toolbar-btn" title="Import History">
          <span className="icon">📥</span>
          Import
          <input
            type="file"
            accept=".json"
            onChange={onImport}
            style={{ display: 'none' }}
          />
        </label>
      </div>

      <div className="history-toolbar-group">
        <button
          className="history-toolbar-btn"
          onClick={onOptimize}
          title="Optimize History"
        >
          <span className="icon">⚡</span>
          Optimize
        </button>
        <button
          className="history-toolbar-btn danger"
          onClick={onClear}
          title="Clear History"
        >
          <span className="icon">🗑️</span>
          Clear
        </button>
      </div>
    </div>
  );
}
