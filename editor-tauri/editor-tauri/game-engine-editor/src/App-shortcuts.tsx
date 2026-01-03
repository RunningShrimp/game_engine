/**
 * 快捷键系统集成示例
 * 展示如何在主应用中初始化和使用快捷键系统
 */

import React, { useEffect, useState } from 'react';
import { getShortcutManager } from './utils/ShortcutManager';
import { allShortcuts } from './shortcuts';
import { ShortcutOverlay } from './components/ShortcutOverlay';
import { ShortcutEditor } from './components/ShortcutEditor';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

/**
 * 快捷键系统提供者
 */
export function ShortcutProvider({ children }: { children: React.ReactNode }) {
  const [initialized, setInitialized] = useState(false);
  const [showHelp, setShowHelp] = useState(false);
  const [showEditor, setShowEditor] = useState(false);
  const [learningMode, setLearningMode] = useState(false);

  useEffect(() => {
    initializeShortcuts();
    setupEventListeners();
    loadSavedShortcuts();

    return () => {
      cleanup();
    };
  }, []);

  const initializeShortcuts = () => {
    const manager = getShortcutManager();

    // 设置命令执行器
    manager.setCommandExecutor((command: string) => {
      executeCommand(command);
    });

    // 注册所有快捷键
    manager.registerAll(allShortcuts);

    // 监听全局键盘事件
    document.addEventListener('keydown', handleGlobalKeydown);

    // 监听快捷键事件
    manager.on('shortcut-executed', handleShortcutExecuted);
    manager.on('conflict-detected', handleConflictDetected);
    manager.on('context-changed', handleContextChanged);

    setInitialized(true);
  };

  const loadSavedShortcuts = async () => {
    try {
      // 从 Tauri 后端加载保存的快捷键配置
      const config = await invoke<any>('load_shortcut_config');

      if (config) {
        const manager = getShortcutManager();
        manager.importShortcuts(config);
        console.log('Shortcuts loaded successfully');
      }
    } catch (error) {
      console.error('Failed to load shortcuts:', error);
    }
  };

  const saveShortcuts = async () => {
    try {
      const manager = getShortcutManager();
      const config = manager.exportShortcuts();

      // 保存到 Tauri 后端
      await invoke('save_shortcut_config', { config });
      console.log('Shortcuts saved successfully');
    } catch (error) {
      console.error('Failed to save shortcuts:', error);
    }
  };

  const handleGlobalKeydown = (event: KeyboardEvent) => {
    const manager = getShortcutManager();

    // 如果在输入框中，不处理快捷键
    if (
      event.target instanceof HTMLInputElement ||
      event.target instanceof HTMLTextAreaElement
    ) {
      return;
    }

    // 处理快捷键
    const handled = manager.handleKeyboardEvent(event);

    if (handled) {
      event.preventDefault();
      event.stopPropagation();
    }
  };

  const handleShortcutExecuted = (event: any) => {
    console.log('Shortcut executed:', event.shortcutId);

    // 可以在这里添加快捷键执行的反馈
    // 例如显示 toast 通知或播放音效
  };

  const handleConflictDetected = (conflict: any) => {
    console.warn('Shortcut conflict detected:', conflict);

    // 显示冲突通知
    showConflictNotification(conflict);
  };

  const handleContextChanged = (context: string) => {
    console.log('Shortcut context changed:', context);
  };

  const executeCommand = (command: string) => {
    console.log('Executing command:', command);

    // 根据命令名称执行相应操作
    switch (command) {
      case 'file.new':
        // TODO: 实现新建场景
        break;

      case 'file.open':
        // TODO: 实现打开场景
        break;

      case 'file.save':
        // TODO: 实现保存
        break;

      case 'edit.undo':
        // TODO: 实现撤销
        break;

      case 'edit.redo':
        // TODO: 实现重做
        break;

      case 'viewport.setTranslateMode':
        // TODO: 设置平移模式
        break;

      case 'viewport.setRotateMode':
        // TODO: 设置旋转模式
        break;

      case 'viewport.setScaleMode':
        // TODO: 设置缩放模式
        break;

      case 'tools.shortcuts':
        setShowEditor(true);
        break;

      case 'docs.shortcuts':
        setShowHelp(true);
        break;

      default:
        console.warn('Unknown command:', command);
    }
  };

  const setupEventListeners = () => {
    // 监听来自 Tauri 的事件（如果需要）
    const unlisten = listen('shortcut-change', (event) => {
      console.log('Shortcut changed:', event.payload);
    });

    return () => {
      unlisten.then(fn => fn());
    };
  };

  const cleanup = () => {
    const manager = getShortcutManager();

    document.removeEventListener('keydown', handleGlobalKeydown);
    manager.removeAllListeners();
  };

  const showConflictNotification = (conflict: any) => {
    // TODO: 显示冲突通知
    // 可以使用 toast 或其他通知系统
    alert(
      `快捷键冲突: ${conflict.keys.map((k: any) => k.key).join('+')}\n` +
      '请在快捷键设置中解决此冲突'
    );
  };

  if (!initialized) {
    return <div>Loading shortcuts...</div>;
  }

  return (
    <>
      {children}

      <ShortcutOverlay
        learningMode={learningMode}
        onLearningModeChange={setLearningMode}
      />

      {showHelp && (
        <ShortcutOverlay>
          <ShortcutHelp onClose={() => setShowHelp(false)} />
        </ShortcutOverlay>
      )}

      {showEditor && (
        <ShortcutEditor
          onClose={() => {
            setShowEditor(false);
            saveShortcuts();
          }}
        />
      )}
    </>
  );
}

/**
 * 快捷键帮助面板
 */
import { ShortcutHelp } from './components/ShortcutOverlay/ShortcutHelp';

function ShortcutHelpWrapper({ onClose }: { onClose: () => void }) {
  return <ShortcutHelp onClose={onClose} />;
}

/**
 * 使用快捷键的 Hook
 */
export function useShortcuts() {
  const manager = getShortcutManager();

  return {
    manager,

    setContext: (context: string) => manager.setContext(context as any),
    getContext: () => manager.getContext(),
    executeShortcut: (id: string) => manager.executeShortcutById(id),

    enableShortcut: (id: string) => manager.enableShortcut(id),
    disableShortcut: (id: string) => manager.disableShortcut(id),

    customizeShortcut: (id: string, keys: any[]) =>
      manager.customizeShortcut(id, keys),

    resetShortcut: (id: string) => manager.resetShortcut(id),

    getShortcut: (id: string) => manager.getShortcut(id),
    getAllShortcuts: () => manager.getAllShortcuts(),

    detectConflicts: () => manager.detectConflicts(),
  };
}

/**
 * 快捷键按钮 Hook
 * 为按钮添加快捷键支持
 */
export function useShortcutButton(shortcutId: string) {
  const manager = getShortcutManager();
  const shortcut = manager.getShortcut(shortcutId);

  return {
    shortcut,
    keys: shortcut?.keys || [],
    formattedKeys: shortcut ? manager.formatKeys(shortcut.keys) : '',
  };
}

/**
 * 快捷键上下文 Hook
 * 管理当前快捷键上下文
 */
export function useShortcutContext() {
  const manager = getShortcutManager();

  return {
    context: manager.getContext(),
    setContext: (context: string) => manager.setContext(context as any),
  };
}
