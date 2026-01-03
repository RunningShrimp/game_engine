/**
 * 快捷键帮助面板
 * 显示所有可用的快捷键
 */

import React, { useState, useMemo } from 'react';
import { Shortcut, ShortcutCategory, KeySequence } from '../../types/shortcuts';
import { getShortcutManager } from '../../utils/ShortcutManager';
import './ShortcutHelp.css';

interface ShortcutHelpProps {
  onClose?: () => void;
}

export function ShortcutHelp({ onClose }: ShortcutHelpProps) {
  const [filter, setFilter] = useState('');
  const [categoryFilter, setCategoryFilter] = useState<ShortcutCategory | 'all'>('all');
  const shortcutManager = getShortcutManager();

  const allShortcuts = useMemo(() => {
    return shortcutManager.getAllShortcuts();
  }, [shortcutManager]);

  const filteredShortcuts = useMemo(() => {
    return allShortcuts
      .filter(shortcut => {
        // 启用状态筛选
        if (shortcut.enabled === false) return false;

        // 分类筛选
        if (categoryFilter !== 'all' && shortcut.category !== categoryFilter) {
          return false;
        }

        // 关键词筛选
        if (filter) {
          const lowerFilter = filter.toLowerCase();
          return (
            shortcut.description.toLowerCase().includes(lowerFilter) ||
            shortcut.id.toLowerCase().includes(lowerFilter) ||
            formatKeySequence(shortcut.keys).toLowerCase().includes(lowerFilter)
          );
        }

        return true;
      })
      .sort((a, b) => {
        // 按分类和优先级排序
        if (a.category !== b.category) {
          return a.category.localeCompare(b.category);
        }
        return (b.priority || 0) - (a.priority || 0);
      });
  }, [allShortcuts, filter, categoryFilter]);

  const groupedShortcuts = useMemo(() => {
    return filteredShortcuts.reduce((acc, shortcut) => {
      const category = shortcut.category;
      if (!acc[category]) {
        acc[category] = [];
      }
      acc[category].push(shortcut);
      return acc;
    }, {} as Record<string, Shortcut[]>);
  }, [filteredShortcuts]);

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

  return (
    <div className="shortcut-help">
      <div className="shortcut-help-header">
        <h2>快捷键参考</h2>
        {onClose && (
          <button className="close-button" onClick={onClose}>
            ×
          </button>
        )}
      </div>

      <div className="shortcut-help-controls">
        <input
          type="text"
          className="shortcut-search"
          placeholder="搜索快捷键..."
          value={filter}
          onChange={e => setFilter(e.target.value)}
          autoFocus
        />

        <select
          className="category-filter"
          value={categoryFilter}
          onChange={e => setCategoryFilter(e.target.value as ShortcutCategory | 'all')}
        >
          {categories.map(cat => (
            <option key={cat.value} value={cat.value}>
              {cat.label}
            </option>
          ))}
        </select>
      </div>

      <div className="shortcut-help-content">
        {Object.entries(groupedShortcuts).map(([category, shortcuts]) => (
          <div key={category} className="shortcut-category">
            <h3 className="category-title">{getCategoryLabel(category)}</h3>
            <div className="shortcut-list">
              {shortcuts.map(shortcut => (
                <ShortcutItem key={shortcut.id} shortcut={shortcut} />
              ))}
            </div>
          </div>
        ))}

        {filteredShortcuts.length === 0 && (
          <div className="no-results">
            <p>未找到匹配的快捷键</p>
          </div>
        )}
      </div>

      <div className="shortcut-help-footer">
        <span className="shortcut-count">
          共 {filteredShortcuts.length} 个快捷键
        </span>
      </div>
    </div>
  );
}

interface ShortcutItemProps {
  shortcut: Shortcut;
}

function ShortcutItem({ shortcut }: ShortcutItemProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    const text = `${shortcut.description}: ${formatKeySequence(shortcut.keys)}`;
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="shortcut-item" title={shortcut.id}>
      <span className="shortcut-keys">{formatKeySequence(shortcut.keys)}</span>
      <span className="shortcut-description">{shortcut.description}</span>
      {shortcut.context && (
        <span className="shortcut-context">{getContextLabel(shortcut.context)}</span>
      )}
      <button
        className="copy-button"
        onClick={handleCopy}
        title="复制快捷键"
      >
        {copied ? '✓' : '📋'}
      </button>
    </div>
  );
}

/**
 * 格式化按键序列
 */
export function formatKeySequence(keys: KeySequence): string {
  return keys
    .map(combo => {
      const parts: string[] = [];
      if (combo.ctrl) parts.push(isMac() ? '⌘' : 'Ctrl');
      if (combo.shift) parts.push('Shift');
      if (combo.alt) parts.push(isMac() ? 'Option' : 'Alt');
      if (combo.meta) parts.push(isMac() ? '⌘' : 'Win');
      parts.push(combo.key.toUpperCase());
      return parts.join(isMac() ? '' : '+');
    })
    .join(', ');
}

/**
 * 获取分类标签
 */
function getCategoryLabel(category: string): string {
  const labels: Record<string, string> = {
    file: '文件',
    edit: '编辑',
    view: '视图',
    tools: '工具',
    transform: '变换',
    navigation: '导航',
    playback: '播放控制',
    window: '窗口',
    help: '帮助',
  };
  return labels[category] || category;
}

/**
 * 获取上下文标签
 */
function getContextLabel(context: string): string {
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

/**
 * 检查是否为 macOS
 */
function isMac(): boolean {
  return navigator.userAgent.includes('Mac OS X');
}
