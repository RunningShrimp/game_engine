/**
 * 行为树编辑器快捷键
 */

import { Shortcut } from '../types/shortcuts';

export const behaviorShortcuts: Shortcut[] = [
  // 文件操作
  {
    id: 'behavior.new',
    keys: [{ key: 'n', ctrl: true, shift: true }],
    action: 'behavior.new',
    description: '新建行为树',
    category: 'file',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.save',
    keys: [{ key: 's', ctrl: true }],
    action: 'behavior.save',
    description: '保存行为树',
    category: 'file',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.saveAs',
    keys: [{ key: 's', ctrl: true, shift: true }],
    action: 'behavior.saveAs',
    description: '行为树另存为',
    category: 'file',
    context: 'behavior-editor',
  },

  // 节点操作
  {
    id: 'behavior.addNode',
    keys: [{ key: 'a' }],
    action: 'behavior.addNode',
    description: '添加节点',
    category: 'edit',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.deleteNode',
    keys: [{ key: 'Delete' }],
    action: 'behavior.deleteNode',
    description: '删除节点',
    category: 'edit',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.duplicateNode',
    keys: [{ key: 'd', ctrl: true }],
    action: 'behavior.duplicateNode',
    description: '复制节点',
    category: 'edit',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.renameNode',
    keys: [{ key: 'F2' }],
    action: 'behavior.renameNode',
    description: '重命名节点',
    category: 'edit',
    context: 'behavior-editor',
  },

  // 视图操作
  {
    id: 'behavior.frameAll',
    keys: [{ key: 'a' }],
    action: 'behavior.frameAll',
    description: '适应窗口',
    category: 'view',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.frameSelected',
    keys: [{ key: 'f' }],
    action: 'behavior.frameSelected',
    description: '聚焦选中',
    category: 'view',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.zoomIn',
    keys: [{ key: '=' }],
    action: 'behavior.zoomIn',
    description: '放大',
    category: 'view',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.zoomOut',
    keys: [{ key: '-' }],
    action: 'behavior.zoomOut',
    description: '缩小',
    category: 'view',
    context: 'behavior-editor',
  },

  // 连接操作
  {
    id: 'behavior.connectNodes',
    keys: [{ key: 'c' }],
    action: 'behavior.connectNodes',
    description: '连接节点',
    category: 'edit',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.disconnectNodes',
    keys: [{ key: 'x' }],
    action: 'behavior.disconnectNodes',
    description: '断开连接',
    category: 'edit',
    context: 'behavior-editor',
  },

  // 常用节点
  {
    id: 'behavior.addSequence',
    keys: [{ key: 's' }],
    action: 'behavior.addSequence',
    description: '添加序列节点',
    category: 'tools',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.addSelector',
    keys: [{ key: 'e' }],
    action: 'behavior.addSelector',
    description: '添加选择节点',
    category: 'tools',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.addCondition',
    keys: [{ key: 'c' }],
    action: 'behavior.addCondition',
    description: '添加条件节点',
    category: 'tools',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.addAction',
    keys: [{ key: 't' }],
    action: 'behavior.addAction',
    description: '添加动作节点',
    category: 'tools',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.addDecorator',
    keys: [{ key: 'd' }],
    action: 'behavior.addDecorator',
    description: '添加装饰节点',
    category: 'tools',
    context: 'behavior-editor',
  },

  // 调试
  {
    id: 'behavior.toggleBreakpoint',
    keys: [{ key: 'F9' }],
    action: 'behavior.toggleBreakpoint',
    description: '切换断点',
    category: 'tools',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.stepOver',
    keys: [{ key: 'F10' }],
    action: 'behavior.stepOver',
    description: '单步跳过',
    category: 'tools',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.stepInto',
    keys: [{ key: 'F11' }],
    action: 'behavior.stepInto',
    description: '单步进入',
    category: 'tools',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.stepOut',
    keys: [{ key: 'F11', shift: true }],
    action: 'behavior.stepOut',
    description: '单步跳出',
    category: 'tools',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.continue',
    keys: [{ key: 'F5' }],
    action: 'behavior.continue',
    description: '继续执行',
    category: 'tools',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.toggleDebug',
    keys: [{ key: 'd', ctrl: true, shift: true }],
    action: 'behavior.toggleDebug',
    description: '切换调试模式',
    category: 'tools',
    context: 'behavior-editor',
  },

  // 搜索
  {
    id: 'behavior.searchNode',
    keys: [{ key: 'f', ctrl: true }],
    action: 'behavior.searchNode',
    description: '搜索节点',
    category: 'edit',
    context: 'behavior-editor',
  },

  // 选择
  {
    id: 'behavior.selectAll',
    keys: [{ key: 'a', ctrl: true }],
    action: 'behavior.selectAll',
    description: '全选',
    category: 'edit',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.invertSelection',
    keys: [{ key: 'i', ctrl: true, shift: true }],
    action: 'behavior.invertSelection',
    description: '反选',
    category: 'edit',
    context: 'behavior-editor',
  },

  // 布局
  {
    id: 'behavior.autoLayout',
    keys: [{ key: 'l', ctrl: true }],
    action: 'behavior.autoLayout',
    description: '自动布局',
    category: 'view',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.toggleGrid',
    keys: [{ key: 'g' }],
    action: 'behavior.toggleGrid',
    description: '切换网格',
    category: 'view',
    context: 'behavior-editor',
  },

  // 黑板变量
  {
    id: 'behavior.addBlackboardKey',
    keys: [{ key: 'k' }],
    action: 'behavior.addBlackboardKey',
    description: '添加黑板变量',
    category: 'edit',
    context: 'behavior-editor',
  },
  {
    id: 'behavior.editBlackboard',
    keys: [{ key: 'b', ctrl: true }],
    action: 'behavior.editBlackboard',
    description: '编辑黑板',
    category: 'edit',
    context: 'behavior-editor',
  },
];
