import { useEffect } from 'react';

export type HotkeyHandler = (event: KeyboardEvent) => void;

export interface HotkeyConfig {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  handler: HotkeyHandler;
  description?: string;
}

export function useHotkeys(hotkeys: HotkeyConfig[]) {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      for (const hotkey of hotkeys) {
        const keyMatch = event.key.toLowerCase() === hotkey.key.toLowerCase();
        const ctrlMatch = hotkey.ctrl ? event.ctrlKey || event.metaKey : !event.ctrlKey && !event.metaKey;
        const shiftMatch = hotkey.shift ? event.shiftKey : !event.shiftKey;
        const altMatch = hotkey.alt ? event.altKey : !event.altKey;

        if (keyMatch && ctrlMatch && shiftMatch && altMatch) {
          event.preventDefault();
          hotkey.handler(event);
          break;
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [hotkeys]);
}

// 预定义的快捷键
export const HOTKEYS = {
  SAVE: { key: 's', ctrl: true, description: '保存' },
  UNDO: { key: 'z', ctrl: true, description: '撤销' },
  REDO: { key: 'y', ctrl: true, description: '重做' },
  DELETE: { key: 'Delete', description: '删除' },
  DUPLICATE: { key: 'd', ctrl: true, description: '复制' },
  SELECT_ALL: { key: 'a', ctrl: true, description: '全选' },
  PLAY: { key: 'F5', description: '运行' },
  STOP: { key: 'F5', shift: true, description: '停止' },
};
