/**
 * 单个快捷键编辑项
 */

import React, { useState, useEffect } from 'react';
import { Shortcut, KeySequence } from '../../types/shortcuts';
import { formatKeySequence } from '../ShortcutOverlay/ShortcutHelp';
import './ShortcutItem.css';

interface ShortcutItemProps {
  shortcut: Shortcut;
  isEditing: boolean;
  onEdit: () => void;
  onSave: (id: string, keys: KeySequence) => void;
  onReset: () => void;
  onCancel: () => void;
}

export function ShortcutItem({
  shortcut,
  isEditing,
  onEdit,
  onSave,
  onReset,
  onCancel,
}: ShortcutItemProps) {
  const [recording, setRecording] = useState(false);
  const [recordedKeys, setRecordedKeys] = useState<KeySequence>([]);

  useEffect(() => {
    if (isEditing) {
      startRecording();
    }
  }, [isEditing]);

  const startRecording = () => {
    setRecording(true);
    setRecordedKeys([]);
  };

  useEffect(() => {
    if (!recording) return;

    const handleKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();

      const combo = {
        key: event.key,
        ctrl: event.ctrlKey,
        shift: event.shiftKey,
        alt: event.altKey,
        meta: event.metaKey,
      };

      setRecordedKeys([...recordedKeys, combo]);
    };

    const handleKeyUp = () => {
      // 完成录制
      if (recordedKeys.length > 0) {
        setTimeout(() => {
          handleSave();
        }, 500);
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    document.addEventListener('keyup', handleKeyUp);

    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      document.removeEventListener('keyup', handleKeyUp);
    };
  }, [recording, recordedKeys]);

  const handleSave = () => {
    if (recordedKeys.length > 0) {
      onSave(shortcut.id, recordedKeys);
    }
    setRecording(false);
  };

  const handleReset = () => {
    onReset();
    setRecording(false);
  };

  const handleCancel = () => {
    onCancel();
    setRecording(false);
  };

  return (
    <div className={`shortcut-item ${isEditing ? 'editing' : ''}`}>
      <div className="item-info">
        <span className="item-description">{shortcut.description}</span>
        <span className="item-context">{getContextLabel(shortcut.context)}</span>
        <span className="item-category">{getCategoryLabel(shortcut.category)}</span>
      </div>

      <div className="item-keys">
        {recording ? (
          <div className="recording-indicator">
            <span className="recording-dot"></span>
            <span className="recording-text">按下快捷键...</span>
            {recordedKeys.length > 0 && (
              <span className="recorded-keys">{formatKeySequence(recordedKeys)}</span>
            )}
          </div>
        ) : (
          <span className="keys-display">{formatKeySequence(shortcut.keys)}</span>
        )}

        {shortcut.customized && <span className="customized-badge">自定义</span>}
      </div>

      <div className="item-actions">
        {isEditing ? (
          <>
            <button className="action-btn save" onClick={handleSave}>
              保存
            </button>
            <button className="action-btn cancel" onClick={handleCancel}>
              取消
            </button>
          </>
        ) : (
          <>
            <button className="action-btn edit" onClick={onEdit}>
              编辑
            </button>
            {shortcut.customized && (
              <button className="action-btn reset" onClick={handleReset}>
                重置
              </button>
            )}
          </>
        )}
      </div>
    </div>
  );
}

function getContextLabel(context?: string): string {
  if (!context) return '全局';
  const labels: Record<string, string> = {
    global: '全局',
    editor: '编辑器',
    viewport: '视口',
    'material-editor': '材质编辑器',
    'behavior-editor': '行为树编辑器',
    timeline: '时间轴',
    'asset-browser': '资源浏览器',
    console: '控制台',
  };
  return labels[context] || context;
}

function getCategoryLabel(category: string): string {
  const labels: Record<string, string> = {
    file: '文件',
    edit: '编辑',
    view: '视图',
    tools: '工具',
    transform: '变换',
    navigation: '导航',
    playback: '播放',
    window: '窗口',
    help: '帮助',
  };
  return labels[category] || category;
}
