/**
 * 编辑器快捷键
 * 场景编辑器上下文中的快捷键
 */

import { Shortcut } from '../types/shortcuts';

export const editorShortcuts: Shortcut[] = [
  // 查找
  {
    id: 'editor.find',
    keys: [{ key: 'f', ctrl: true }],
    action: 'editor.find',
    description: '查找',
    category: 'edit',
    context: 'editor',
  },
  {
    id: 'editor.replace',
    keys: [{ key: 'h', ctrl: true }],
    action: 'editor.replace',
    description: '替换',
    category: 'edit',
    context: 'editor',
  },
  {
    id: 'editor.findInFiles',
    keys: [{ key: 'f', ctrl: true, shift: true }],
    action: 'editor.findInFiles',
    description: '在文件中查找',
    category: 'edit',
    context: 'editor',
  },

  // 对齐
  {
    id: 'editor.alignLeft',
    keys: [{ key: 'a', ctrl: true, alt: true }],
    action: 'editor.alignLeft',
    description: '左对齐',
    category: 'edit',
    context: 'editor',
  },
  {
    id: 'editor.alignCenter',
    keys: [{ key: 'c', ctrl: true, alt: true }],
    action: 'editor.alignCenter',
    description: '居中对齐',
    category: 'edit',
    context: 'editor',
  },
  {
    id: 'editor.alignRight',
    keys: [{ key: 'r', ctrl: true, alt: true }],
    action: 'editor.alignRight',
    description: '右对齐',
    category: 'edit',
    context: 'editor',
  },
  {
    id: 'editor.alignTop',
    keys: [{ key: 't', ctrl: true, alt: true }],
    action: 'editor.alignTop',
    description: '顶对齐',
    category: 'edit',
    context: 'editor',
  },
  {
    id: 'editor.alignMiddle',
    keys: [{ key: 'm', ctrl: true, alt: true }],
    action: 'editor.alignMiddle',
    description: '垂直居中',
    category: 'edit',
    context: 'editor',
  },
  {
    id: 'editor.alignBottom',
    keys: [{ key: 'b', ctrl: true, alt: true }],
    action: 'editor.alignBottom',
    description: '底对齐',
    category: 'edit',
    context: 'editor',
  },

  // 分布
  {
    id: 'editor.distributeHorizontally',
    keys: [{ key: 'd', ctrl: true, shift: true, alt: true }],
    action: 'editor.distributeHorizontally',
    description: '水平分布',
    category: 'edit',
    context: 'editor',
  },
  {
    id: 'editor.distributeVertically',
    keys: [{ key: 'v', ctrl: true, shift: true, alt: true }],
    action: 'editor.distributeVertically',
    description: '垂直分布',
    category: 'edit',
    context: 'editor',
  },

  // 分组
  {
    id: 'editor.group',
    keys: [{ key: 'g', ctrl: true }],
    action: 'editor.group',
    description: '编组',
    category: 'edit',
    context: 'editor',
  },
  {
    id: 'editor.ungroup',
    keys: [{ key: 'g', ctrl: true, shift: true }],
    action: 'editor.ungroup',
    description: '取消编组',
    category: 'edit',
    context: 'editor',
  },
  {
    id: 'editor.lock',
    keys: [{ key: 'l', ctrl: true }],
    action: 'editor.lock',
    description: '锁定',
    category: 'edit',
    context: 'editor',
  },
  {
    id: 'editor.unlock',
    keys: [{ key: 'l', ctrl: true, shift: true }],
    action: 'editor.unlock',
    description: '解锁',
    category: 'edit',
    context: 'editor',
  },

  // 视图
  {
    id: 'editor.zoomIn',
    keys: [{ key: '=' }],
    action: 'editor.zoomIn',
    description: '放大',
    category: 'view',
    context: 'editor',
  },
  {
    id: 'editor.zoomOut',
    keys: [{ key: '-' }],
    action: 'editor.zoomOut',
    description: '缩小',
    category: 'view',
    context: 'editor',
  },
  {
    id: 'editor.resetZoom',
    keys: [{ key: '0' }],
    action: 'editor.resetZoom',
    description: '重置缩放',
    category: 'view',
    context: 'editor',
  },
  {
    id: 'editor.fitToScreen',
    keys: [{ key: 'f' }],
    action: 'editor.fitToScreen',
    description: '适应屏幕',
    category: 'view',
    context: 'editor',
  },

  // 网格和吸附
  {
    id: 'editor.toggleGrid',
    keys: [{ key: 'g', ctrl: true }],
    action: 'editor.toggleGrid',
    description: '切换网格',
    category: 'view',
    context: 'editor',
  },
  {
    id: 'editor.toggleSnap',
    keys: [{ key: 's', ctrl: true, shift: true }],
    action: 'editor.toggleSnap',
    description: '切换吸附',
    category: 'view',
    context: 'editor',
  },

  // 选择
  {
    id: 'editor.selectInverse',
    keys: [{ key: 'i', ctrl: true, shift: true }],
    action: 'editor.selectInverse',
    description: '反选',
    category: 'edit',
    context: 'editor',
  },
  {
    id: 'editor.deselect',
    keys: [{ key: 'd', ctrl: true, shift: true }],
    action: 'editor.deselect',
    description: '取消选择',
    category: 'edit',
    context: 'editor',
  },

  // 层级
  {
    id: 'editor.bringToFront',
    keys: [{ key: ']' }],
    action: 'editor.bringToFront',
    description: '移到最前',
    category: 'edit',
    context: 'editor',
  },
  {
    id: 'editor.sendToBack',
    keys: [{ key: '[' }],
    action: 'editor.sendToBack',
    description: '移到最后',
    category: 'edit',
    context: 'editor',
  },
  {
    id: 'editor.bringForward',
    keys: [{ key: ']', ctrl: true }],
    action: 'editor.bringForward',
    description: '上移一层',
    category: 'edit',
    context: 'editor',
  },
  {
    id: 'editor.sendBackward',
    keys: [{ key: '[', ctrl: true }],
    action: 'editor.sendBackward',
    description: '下移一层',
    category: 'edit',
    context: 'editor',
  },

  // 历史记录
  {
    id: 'editor.history',
    keys: [{ key: 'h', alt: true }],
    action: 'editor.history',
    description: '历史记录',
    category: 'edit',
    context: 'editor',
  },
];
