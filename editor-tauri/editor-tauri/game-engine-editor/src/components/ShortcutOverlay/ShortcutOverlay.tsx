/**
 * 快捷键覆盖层
 * 提供全局快捷键提示和学习模式
 */

import React, { useEffect, useState } from 'react';
import { ShortcutHelp } from './ShortcutHelp';
import { getShortcutManager } from '../../utils/ShortcutManager';
import './ShortcutOverlay.css';

interface ShortcutOverlayProps {
  learningMode?: boolean;
  onLearningModeChange?: (enabled: boolean) => void;
}

export function ShortcutOverlay({
  learningMode = false,
  onLearningModeChange,
}: ShortcutOverlayProps) {
  const [showHelp, setShowHelp] = useState(false);
  const [learningActive, setLearningActive] = useState(learningMode);

  useEffect(() => {
    setLearningActive(learningMode);
  }, [learningMode]);

  useEffect(() => {
    const shortcutManager = getShortcutManager();

    // 监听快捷键事件
    const handleShortcutExecuted = () => {
      // 可以在这里显示快捷键执行的反馈
    };

    const showHelpPanel = () => {
      setShowHelp(true);
    };

    shortcutManager.on('shortcut-executed', handleShortcutExecuted);

    // 监听快捷键帮助触发
    const handleKeyDown = (event: KeyboardEvent) => {
      // Ctrl+/ 或 Cmd+/ 显示帮助
      if ((event.ctrlKey || event.metaKey) && event.key === '/') {
        event.preventDefault();
        setShowHelp(prev => !prev);
      }
    };

    document.addEventListener('keydown', handleKeyDown);

    return () => {
      shortcutManager.off('shortcut-executed', handleShortcutExecuted);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, []);

  const toggleLearningMode = () => {
    const newState = !learningActive;
    setLearningActive(newState);
    onLearningModeChange?.(newState);
  };

  return (
    <>
      {showHelp && (
        <div className="shortcut-overlay-backdrop" onClick={() => setShowHelp(false)}>
          <div onClick={e => e.stopPropagation()}>
            <ShortcutHelp onClose={() => setShowHelp(false)} />
          </div>
        </div>
      )}

      {learningActive && (
        <div className="learning-mode-indicator">
          <span>学习模式</span>
          <button
            className="close-learning-mode"
            onClick={toggleLearningMode}
            title="退出学习模式"
          >
            ×
          </button>
        </div>
      )}
    </>
  );
}
