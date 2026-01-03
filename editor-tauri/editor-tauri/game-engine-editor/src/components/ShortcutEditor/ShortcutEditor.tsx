/**
 * 快捷键编辑器
 * 允许用户自定义快捷键
 */

import React, { useState, useEffect, useMemo } from 'react';
import { Shortcut, ShortcutConflict, ShortcutPreset, ShortcutCategory } from '../../types/shortcuts';
import { getShortcutManager } from '../../utils/ShortcutManager';
import { ShortcutItem } from './ShortcutItem';
import { ConflictDialog } from './ConflictDialog';
import { ShortcutPresets } from './ShortcutPresets';
import { formatKeySequence } from '../ShortcutOverlay/ShortcutHelp';
import './ShortcutEditor.css';

interface ShortcutEditorProps {
  onClose?: () => void;
}

export function ShortcutEditor({ onClose }: ShortcutEditorProps) {
  const shortcutManager = getShortcutManager();
  const [shortcuts, setShortcuts] = useState<Shortcut[]>([]);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [conflicts, setConflicts] = useState<ShortcutConflict[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [categoryFilter, setCategoryFilter] = useState<ShortcutCategory | 'all'>('all');
  const [contextFilter, setContextFilter] = useState<string>('all');
  const [showResetConfirm, setShowResetConfirm] = useState(false);
  const [showPresets, setShowPresets] = useState(false);

  useEffect(() => {
    loadShortcuts();
    detectConflicts();
  }, []);

  const loadShortcuts = () => {
    const all = shortcutManager.getAllShortcuts();
    setShortcuts(all);
  };

  const detectConflicts = () => {
    const detected = shortcutManager.detectConflicts();
    setConflicts(detected);
  };

  const filteredShortcuts = useMemo(() => {
    return shortcuts.filter(shortcut => {
      // 搜索过滤
      if (searchQuery) {
        const query = searchQuery.toLowerCase();
        if (
          !shortcut.description.toLowerCase().includes(query) &&
          !shortcut.id.toLowerCase().includes(query) &&
          !formatKeySequence(shortcut.keys).toLowerCase().includes(query)
        ) {
          return false;
        }
      }

      // 分类过滤
      if (categoryFilter !== 'all' && shortcut.category !== categoryFilter) {
        return false;
      }

      // 上下文过滤
      if (contextFilter !== 'all' && shortcut.context !== contextFilter) {
        return false;
      }

      return true;
    });
  }, [shortcuts, searchQuery, categoryFilter, contextFilter]);

  const handleSaveShortcut = (id: string, newKeys: any[]) => {
    const success = shortcutManager.customizeShortcut(id, newKeys);

    if (success) {
      loadShortcuts();
      detectConflicts();
      setEditingId(null);
    } else {
      alert('快捷键冲突，请选择其他按键');
    }
  };

  const handleResetShortcut = (id: string) => {
    shortcutManager.resetShortcut(id);
    loadShortcuts();
    detectConflicts();
  };

  const handleResetAll = () => {
    if (window.confirm('确定要重置所有快捷键吗？')) {
      shortcutManager.resetAllShortcuts();
      loadShortcuts();
      detectConflicts();
      setShowResetConfirm(false);
    }
  };

  const handleExport = () => {
    const config = shortcutManager.exportShortcuts();
    const blob = new Blob([JSON.stringify(config, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `shortcuts-${new Date().toISOString().split('T')[0]}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleImport = () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = e => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) {
        const reader = new FileReader();
        reader.onload = event => {
          try {
            const config = JSON.parse(event.target?.result as string);
            shortcutManager.importShortcuts(config);
            loadShortcuts();
            detectConflicts();
            alert('快捷键导入成功');
          } catch (error) {
            alert('导入失败：无效的配置文件');
          }
        };
        reader.readAsText(file);
      }
    };
    input.click();
  };

  const handleApplyPreset = (preset: ShortcutPreset) => {
    if (window.confirm(`应用 "${preset}" 预设将覆盖当前快捷键，确定吗？`)) {
      shortcutManager.applyPreset(preset);
      loadShortcuts();
      detectConflicts();
      setShowPresets(false);
    }
  };

  const handleResolveConflict = (conflict: ShortcutConflict, resolution: string) => {
    shortcutManager.resolveConflict(conflict, resolution);
    detectConflicts();
    loadShortcuts();
  };

  const categories: Array<{ value: ShortcutCategory | 'all'; label: string }> = [
    { value: 'all', label: '全部' },
    { value: 'file', label: '文件' },
    { value: 'edit', label: '编辑' },
    { value: 'view', label: '视图' },
    { value: 'tools', label: '工具' },
    { value: 'transform', label: '变换' },
    { value: 'navigation', label: '导航' },
    { value: 'playback', label: '播放' },
    { value: 'window', label: '窗口' },
    { value: 'help', label: '帮助' },
  ];

  const contexts = [
    { value: 'all', label: '全部' },
    { value: 'global', label: '全局' },
    { value: 'editor', label: '编辑器' },
    { value: 'viewport', label: '视口' },
    { value: 'material-editor', label: '材质编辑器' },
    { value: 'behavior-editor', label: '行为树编辑器' },
    { value: 'timeline', label: '时间轴' },
    { value: 'asset-browser', label: '资源浏览器' },
    { value: 'console', label: '控制台' },
  ];

  return (
    <div className="shortcut-editor">
      <div className="editor-header">
        <h2>快捷键设置</h2>
        <div className="header-actions">
          <button onClick={() => setShowResetConfirm(true)}>重置所有</button>
          <button onClick={handleExport}>导出</button>
          <button onClick={handleImport}>导入</button>
          <button onClick={() => setShowPresets(true)}>应用预设</button>
          {onClose && (
            <button className="close-button" onClick={onClose}>
              ×
            </button>
          )}
        </div>
      </div>

      <div className="editor-controls">
        <input
          type="text"
          className="search-input"
          placeholder="搜索快捷键..."
          value={searchQuery}
          onChange={e => setSearchQuery(e.target.value)}
        />

        <select
          className="filter-select"
          value={categoryFilter}
          onChange={e => setCategoryFilter(e.target.value as ShortcutCategory | 'all')}
        >
          {categories.map(cat => (
            <option key={cat.value} value={cat.value}>
              {cat.label}
            </option>
          ))}
        </select>

        <select
          className="filter-select"
          value={contextFilter}
          onChange={e => setContextFilter(e.target.value)}
        >
          {contexts.map(ctx => (
            <option key={ctx.value} value={ctx.value}>
              {ctx.label}
            </option>
          ))}
        </select>
      </div>

      <div className="conflict-warning">
        {conflicts.length > 0 && (
          <div className="conflict-alert">
            <span className="alert-icon">⚠️</span>
            <span>
              检测到 {conflicts.length} 个快捷键冲突，
              <button onClick={() => {/* TODO: 显示冲突详情 */}}>
                查看详情
              </button>
            </span>
          </div>
        )}
      </div>

      <div className="shortcut-list">
        {filteredShortcuts.map(shortcut => (
          <ShortcutItem
            key={shortcut.id}
            shortcut={shortcut}
            isEditing={editingId === shortcut.id}
            onEdit={() => setEditingId(shortcut.id)}
            onSave={handleSaveShortcut}
            onReset={handleResetShortcut}
            onCancel={() => setEditingId(null)}
          />
        ))}
      </div>

      {showResetConfirm && (
        <div className="confirm-dialog">
          <div className="dialog-content">
            <h3>确认重置</h3>
            <p>确定要重置所有快捷键为默认值吗？此操作不可撤销。</p>
            <div className="dialog-actions">
              <button onClick={() => setShowResetConfirm(false)}>取消</button>
              <button className="danger" onClick={handleResetAll}>
                确认重置
              </button>
            </div>
          </div>
        </div>
      )}

      {showPresets && (
        <ShortcutPresets
          onSelect={handleApplyPreset}
          onClose={() => setShowPresets(false)}
        />
      )}

      {conflicts.length > 0 && (
        <ConflictDialog
          conflicts={conflicts}
          onResolve={handleResolveConflict}
          onClose={() => setConflicts([])}
        />
      )}
    </div>
  );
}
