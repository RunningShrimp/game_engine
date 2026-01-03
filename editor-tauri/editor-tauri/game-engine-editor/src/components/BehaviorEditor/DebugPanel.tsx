/**
 * Debug Panel Component
 * Provides runtime debugging capabilities for behavior trees
 */

import React, { useState, useEffect } from 'react';
import { NodeStatus, Blackboard } from '../../types/behavior';
import './DebugPanel.css';

interface DebugPanelProps {
  isDebugMode: boolean;
  isRunning: boolean;
  isPaused: boolean;
  currentNodeId: string | null;
  nodeStates: Record<string, NodeStatus>;
  blackboard: Blackboard;
  breakpoints: Set<string>;
  onPlay: () => void;
  onPause: () => void;
  onStop: () => void;
  onStep: () => void;
  onStepOver: () => void;
  onToggleBreakpoint: (nodeId: string) => void;
  onClearBreakpoints: () => void;
}

export const DebugPanel: React.FC<DebugPanelProps> = ({
  isDebugMode,
  isRunning,
  isPaused,
  currentNodeId,
  nodeStates,
  blackboard,
  breakpoints,
  onPlay,
  onPause,
  onStop,
  onStep,
  onStepOver,
  onToggleBreakpoint,
  onClearBreakpoints,
}) => {
  const [activeTab, setActiveTab] = useState<'control' | 'blackboard' | 'breakpoints'>('control');

  if (!isDebugMode) {
    return null;
  }

  const getStatusCount = (status: NodeStatus) => {
    return Object.values(nodeStates).filter(s => s === status).length;
  };

  return (
    <div className="debug-panel">
      {/* Debug Mode Indicator */}
      <div className="debug-mode-indicator">
        <span className="debug-icon">🐛</span>
        <span className="debug-label">Debug Mode</span>
      </div>

      {/* Control Buttons */}
      <div className="debug-controls">
        <button
          className={`debug-button ${isRunning && !isPaused ? 'active' : ''}`}
          onClick={isRunning && isPaused ? onPlay : onPlay}
          disabled={isRunning && !isPaused}
          title={isRunning && isPaused ? 'Resume (F5)' : 'Start (F5)'}
        >
          <span className="button-icon">{isRunning && isPaused ? '▶️' : '▶️'}</span>
          <span className="button-text">{isRunning && isPaused ? 'Resume' : 'Run'}</span>
        </button>

        <button
          className={`debug-button ${isPaused ? 'active' : ''}`}
          onClick={onPause}
          disabled={!isRunning || isPaused}
          title="Pause (F6)"
        >
          <span className="button-icon">⏸️</span>
          <span className="button-text">Pause</span>
        </button>

        <button
          className="debug-button"
          onClick={onStop}
          disabled={!isRunning}
          title="Stop (Shift+F5)"
        >
          <span className="button-icon">⏹️</span>
          <span className="button-text">Stop</span>
        </button>

        <div className="debug-divider" />

        <button
          className="debug-button"
          onClick={onStep}
          disabled={!isRunning || !isPaused}
          title="Step Into (F11)"
        >
          <span className="button-icon">🔍</span>
          <span className="button-text">Step Into</span>
        </button>

        <button
          className="debug-button"
          onClick={onStepOver}
          disabled={!isRunning || !isPaused}
          title="Step Over (F10)"
        >
          <span className="button-icon">⏭️</span>
          <span className="button-text">Step Over</span>
        </button>
      </div>

      {/* Status Summary */}
      <div className="debug-status-summary">
        <div className="status-item">
          <span className="status-icon">🟡</span>
          <span className="status-count">{getStatusCount(NodeStatus.Running)}</span>
          <span className="status-label">Running</span>
        </div>
        <div className="status-item">
          <span className="status-icon">🟢</span>
          <span className="status-count">{getStatusCount(NodeStatus.Success)}</span>
          <span className="status-label">Success</span>
        </div>
        <div className="status-item">
          <span className="status-icon">🔴</span>
          <span className="status-count">{getStatusCount(NodeStatus.Failure)}</span>
          <span className="status-label">Failure</span>
        </div>
        <div className="status-item">
          <span className="status-icon">⚪</span>
          <span className="status-count">{getStatusCount(NodeStatus.Idle)}</span>
          <span className="status-label">Idle</span>
        </div>
      </div>

      {/* Tabs */}
      <div className="debug-tabs">
        <button
          className={`tab-button ${activeTab === 'control' ? 'active' : ''}`}
          onClick={() => setActiveTab('control')}
        >
          Control
        </button>
        <button
          className={`tab-button ${activeTab === 'blackboard' ? 'active' : ''}`}
          onClick={() => setActiveTab('blackboard')}
        >
          Blackboard ({Object.keys(blackboard.variables).length})
        </button>
        <button
          className={`tab-button ${activeTab === 'breakpoints' ? 'active' : ''}`}
          onClick={() => setActiveTab('breakpoints')}
        >
          Breakpoints ({breakpoints.size})
        </button>
      </div>

      {/* Tab Content */}
      <div className="debug-tab-content">
        {activeTab === 'control' && (
          <div className="control-tab">
            {currentNodeId && (
              <div className="current-node">
                <div className="current-node-label">Current Node:</div>
                <div className="current-node-id">{currentNodeId}</div>
              </div>
            )}
            {!currentNodeId && (
              <div className="no-execution">
                <p>Not running</p>
                <p className="hint">Press Run to start debugging</p>
              </div>
            )}
          </div>
        )}

        {activeTab === 'blackboard' && (
          <div className="blackboard-tab">
            {Object.keys(blackboard.variables).length === 0 ? (
              <div className="empty-state">
                <p>No variables</p>
              </div>
            ) : (
              <div className="blackboard-variables">
                {Object.entries(blackboard.variables).map(([name, variable]) => (
                  <div key={name} className="blackboard-variable">
                    <span className="variable-name">{name}:</span>
                    <span className={`variable-value type-${variable.type}`}>
                      {formatValue(variable.value, variable.type)}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {activeTab === 'breakpoints' && (
          <div className="breakpoints-tab">
            {breakpoints.size === 0 ? (
              <div className="empty-state">
                <p>No breakpoints</p>
                <p className="hint">Click on nodes to add breakpoints</p>
              </div>
            ) : (
              <div className="breakpoints-list">
                {Array.from(breakpoints).map(nodeId => (
                  <div key={nodeId} className="breakpoint-item">
                    <span className="breakpoint-id">{nodeId}</span>
                    <button
                      className="breakpoint-remove"
                      onClick={() => onToggleBreakpoint(nodeId)}
                    >
                      Remove
                    </button>
                  </div>
                ))}
              </div>
            )}
            <button
              className="clear-breakpoints-button"
              onClick={onClearBreakpoints}
              disabled={breakpoints.size === 0}
            >
              Clear All Breakpoints
            </button>
          </div>
        )}
      </div>
    </div>
  );
};

// Helper function to format values
function formatValue(value: any, type: string): string {
  switch (type) {
    case 'object':
    case 'vector3':
      return JSON.stringify(value);
    default:
      return String(value);
  }
}

export default DebugPanel;
