/**
 * 冲突对话框
 * 显示并解决快捷键冲突
 */

import React from 'react';
import { ShortcutConflict } from '../../types/shortcuts';
import { formatKeySequence } from '../ShortcutOverlay/ShortcutHelp';
import './ConflictDialog.css';

interface ConflictDialogProps {
  conflicts: ShortcutConflict[];
  onResolve: (conflict: ShortcutConflict, resolution: string) => void;
  onClose: () => void;
}

export function ConflictDialog({ conflicts, onResolve, onClose }: ConflictDialogProps) {
  return (
    <div className="conflict-dialog-overlay" onClick={onClose}>
      <div className="conflict-dialog" onClick={e => e.stopPropagation()}>
        <div className="dialog-header">
          <h2>快捷键冲突</h2>
          <button className="close-btn" onClick={onClose}>
            ×
          </button>
        </div>

        <div className="dialog-body">
          {conflicts.map((conflict, index) => (
            <ConflictItem
              key={index}
              conflict={conflict}
              onResolve={resolution => onResolve(conflict, resolution)}
            />
          ))}
        </div>
      </div>
    </div>
  );
}

interface ConflictItemProps {
  conflict: ShortcutConflict;
  onResolve: (resolution: string) => void;
}

function ConflictItem({ conflict, onResolve }: ConflictItemProps) {
  const isWarning = conflict.severity === 'warning';

  return (
    <div className={`conflict-item ${conflict.severity}`}>
      <div className="conflict-header">
        <span className={`severity-icon ${conflict.severity}`}>
          {isWarning ? '⚠️' : '❌'}
        </span>
        <span className="conflict-keys">{formatKeySequence(conflict.keys)}</span>
        <span className={`severity-badge ${conflict.severity}`}>
          {isWarning ? '警告' : '错误'}
        </span>
      </div>

      <div className="conflict-shortcuts">
        {conflict.shortcuts.map(shortcut => (
          <div key={shortcut.id} className="conflict-shortcut">
            <span className="shortcut-description">{shortcut.description}</span>
            <span className="shortcut-id">{shortcut.id}</span>
          </div>
        ))}
      </div>

      <div className="conflict-resolution">
        <span className="resolution-label">解决方案：</span>
        <div className="resolution-options">
          {isWarning ? (
            <button
              className="resolution-btn"
              onClick={() => onResolve('keep-both')}
            >
              保留所有（不同上下文）
            </button>
          ) : (
            <>
              <button
                className="resolution-btn"
                onClick={() => onResolve('disable-old')}
              >
                禁用旧快捷键
              </button>
              <button
                className="resolution-btn"
                onClick={() => onResolve('disable-new')}
              >
                禁用新快捷键
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
