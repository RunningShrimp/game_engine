/**
 * Toolbar Component
 * Top toolbar for behavior tree editor with common actions
 */

import React from 'react';
import './Toolbar.css';

interface ToolbarProps {
  onNew: () => void;
  onSave: () => void;
  onLoad: () => void;
  onValidate: () => void;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onResetView: () => void;
  onToggleDebug: () => void;
  isDebugMode: boolean;
  canSave: boolean;
}

export const Toolbar: React.FC<ToolbarProps> = ({
  onNew,
  onSave,
  onLoad,
  onValidate,
  onZoomIn,
  onZoomOut,
  onResetView,
  onToggleDebug,
  isDebugMode,
  canSave,
}) => {
  return (
    <div className="behavior-toolbar">
      <div className="toolbar-section">
        <button
          className="toolbar-button"
          onClick={onNew}
          title="New Behavior Tree (Ctrl+N)"
        >
          <span className="button-icon">📄</span>
          <span className="button-text">New</span>
        </button>
        <button
          className="toolbar-button"
          onClick={onLoad}
          title="Load Behavior Tree (Ctrl+O)"
        >
          <span className="button-icon">📂</span>
          <span className="button-text">Open</span>
        </button>
        <button
          className="toolbar-button"
          onClick={onSave}
          disabled={!canSave}
          title="Save Behavior Tree (Ctrl+S)"
        >
          <span className="button-icon">💾</span>
          <span className="button-text">Save</span>
        </button>
      </div>

      <div className="toolbar-section">
        <button
          className="toolbar-button"
          onClick={onValidate}
          title="Validate Behavior Tree"
        >
          <span className="button-icon">✓</span>
          <span className="button-text">Validate</span>
        </button>
      </div>

      <div className="toolbar-section toolbar-spacer">
        <h1 className="toolbar-title">Behavior Tree Editor</h1>
      </div>

      <div className="toolbar-section">
        <button
          className={`toolbar-button ${isDebugMode ? 'active' : ''}`}
          onClick={onToggleDebug}
          title="Toggle Debug Mode"
        >
          <span className="button-icon">{isDebugMode ? '🐛' : '🔧'}</span>
          <span className="button-text">{isDebugMode ? 'Debug' : 'Edit'}</span>
        </button>
      </div>

      <div className="toolbar-section">
        <button
          className="toolbar-button"
          onClick={onZoomOut}
          title="Zoom Out (Ctrl+-)"
        >
          <span className="button-icon">➖</span>
        </button>
        <button
          className="toolbar-button"
          onClick={onResetView}
          title="Reset View (Ctrl+0)"
        >
          <span className="button-icon">🎯</span>
        </button>
        <button
          className="toolbar-button"
          onClick={onZoomIn}
          title="Zoom In (Ctrl++)"
        >
          <span className="button-icon">➕</span>
        </button>
      </div>
    </div>
  );
};

export default Toolbar;
