/**
 * 快捷键工具提示
 * 在UI元素上显示快捷键提示
 */

import React from 'react';
import { Shortcut, KeySequence } from '../../types/shortcuts';
import { getShortcutManager } from '../../utils/ShortcutManager';
import './ShortcutTooltip.css';

interface ShortcutTooltipProps {
  shortcutId?: string;
  shortcut?: Shortcut;
  keys?: KeySequence;
  children: React.ReactNode;
  position?: 'top' | 'bottom' | 'left' | 'right';
  showKeys?: boolean;
}

export function ShortcutTooltip({
  shortcutId,
  shortcut: shortcutProp,
  keys: keysProp,
  children,
  position = 'bottom',
  showKeys = true,
}: ShortcutTooltipProps) {
  const [isVisible, setIsVisible] = React.useState(false);
  const timeoutRef = React.useRef<number | null>(null);

  // 获取快捷键
  const shortcut = React.useMemo(() => {
    if (shortcutProp) return shortcutProp;
    if (shortcutId) {
      return getShortcutManager().getShortcut(shortcutId);
    }
    return null;
  }, [shortcutId, shortcutProp]);

  // 获取按键
  const keys = React.useMemo(() => {
    if (keysProp) return keysProp;
    if (shortcut) return shortcut.keys;
    return [];
  }, [keysProp, shortcut]);

  const handleMouseEnter = () => {
    timeoutRef.current = window.setTimeout(() => {
      setIsVisible(true);
    }, 500);
  };

  const handleMouseLeave = () => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    setIsVisible(false);
  };

  if (!showKeys || keys.length === 0) {
    return <>{children}</>;
  }

  return (
    <div
      className="shortcut-tooltip-wrapper"
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      {children}
      {isVisible && (
        <div className={`shortcut-tooltip tooltip-${position}`}>
          <div className="tooltip-content">
            {shortcut && (
              <div className="tooltip-description">{shortcut.description}</div>
            )}
            <div className="tooltip-keys">
              {formatKeySequence(keys)}
            </div>
          </div>
          <div className="tooltip-arrow" />
        </div>
      )}
    </div>
  );
}

/**
 * 快捷键徽章
 * 直接在按钮上显示快捷键
 */
interface ShortcutBadgeProps {
  shortcutId?: string;
  shortcut?: Shortcut;
  keys?: KeySequence;
}

export function ShortcutBadge({
  shortcutId,
  shortcut: shortcutProp,
  keys: keysProp,
}: ShortcutBadgeProps) {
  const shortcut = React.useMemo(() => {
    if (shortcutProp) return shortcutProp;
    if (shortcutId) {
      return getShortcutManager().getShortcut(shortcutId);
    }
    return null;
  }, [shortcutId, shortcutProp]);

  const keys = React.useMemo(() => {
    if (keysProp) return keysProp;
    if (shortcut) return shortcut.keys;
    return [];
  }, [keysProp, shortcut]);

  if (keys.length === 0) {
    return null;
  }

  return (
    <span className="shortcut-badge">
      {formatKeySequence(keys, true)}
    </span>
  );
}

/**
 * 格式化按键序列
 */
export function formatKeySequence(keys: KeySequence, compact = false): string {
  return keys
    .map(combo => {
      const parts: string[] = [];
      if (combo.ctrl) parts.push(isMac() ? '⌘' : 'Ctrl');
      if (combo.shift) parts.push('⇧');
      if (combo.alt) parts.push(isMac() ? '⌥' : 'Alt');
      if (combo.meta) parts.push(isMac() ? '⌘' : '⊞');
      parts.push(combo.key.toUpperCase());

      if (compact && isMac()) {
        return parts.join('');
      }
      return parts.join(compact ? '' : '+');
    })
    .join(', ');
}

/**
 * 检查是否为 macOS
 */
function isMac(): boolean {
  return navigator.userAgent.includes('Mac OS X');
}

/**
 * 为按钮添加快捷键提示的高阶组件
 */
export function withShortcut<P extends { shortcut?: string }>(
  WrappedComponent: React.ComponentType<P>
) {
  return function WithShortcutComponent(props: P) {
    const { shortcut, ...rest } = props;

    if (!shortcut) {
      return <WrappedComponent {...(rest as P)} />;
    }

    return (
      <div className="shortcut-button-wrapper">
        <WrappedComponent {...(rest as P)} />
        <ShortcutBadge shortcutId={shortcut} />
      </div>
    );
  };
}
