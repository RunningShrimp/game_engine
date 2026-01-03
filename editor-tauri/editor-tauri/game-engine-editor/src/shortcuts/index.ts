/**
 * 快捷键定义导出
 */

export { globalShortcuts } from './global';
export { editorShortcuts } from './editor';
export { viewportShortcuts } from './viewport';
export { materialShortcuts } from './material';
export { behaviorShortcuts } from './behavior';
export { timelineShortcuts } from './timeline';

import { Shortcut } from '../types/shortcuts';
import { globalShortcuts } from './global';
import { editorShortcuts } from './editor';
import { viewportShortcuts } from './viewport';
import { materialShortcuts } from './material';
import { behaviorShortcuts } from './behavior';
import { timelineShortcuts } from './timeline';

/**
 * 所有快捷键
 */
export const allShortcuts: Shortcut[] = [
  ...globalShortcuts,
  ...editorShortcuts,
  ...viewportShortcuts,
  ...materialShortcuts,
  ...behaviorShortcuts,
  ...timelineShortcuts,
];

/**
 * 快捷键预设方案
 */
export const shortcutPresets = {
  default: allShortcuts,
  // 其他预设可以通过修改按键来生成
};
