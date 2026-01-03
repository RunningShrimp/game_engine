/**
 * 材质编辑器快捷键
 */

import { Shortcut } from '../types/shortcuts';

export const materialShortcuts: Shortcut[] = [
  // 文件操作
  {
    id: 'material.new',
    keys: [{ key: 'n', ctrl: true, shift: true }],
    action: 'material.new',
    description: '新建材质',
    category: 'file',
    context: 'material-editor',
  },
  {
    id: 'material.save',
    keys: [{ key: 's', ctrl: true }],
    action: 'material.save',
    description: '保存材质',
    category: 'file',
    context: 'material-editor',
  },
  {
    id: 'material.saveAs',
    keys: [{ key: 's', ctrl: true, shift: true }],
    action: 'material.saveAs',
    description: '材质另存为',
    category: 'file',
    context: 'material-editor',
  },

  // 节点操作
  {
    id: 'material.addNode',
    keys: [{ key: 'a' }],
    action: 'material.addNode',
    description: '添加节点',
    category: 'edit',
    context: 'material-editor',
  },
  {
    id: 'material.deleteNode',
    keys: [{ key: 'Delete' }],
    action: 'material.deleteNode',
    description: '删除节点',
    category: 'edit',
    context: 'material-editor',
  },
  {
    id: 'material.duplicateNode',
    keys: [{ key: 'd', ctrl: true }],
    action: 'material.duplicateNode',
    description: '复制节点',
    category: 'edit',
    context: 'material-editor',
  },
  {
    id: 'material.renameNode',
    keys: [{ key: 'F2' }],
    action: 'material.renameNode',
    description: '重命名节点',
    category: 'edit',
    context: 'material-editor',
  },

  // 视图操作
  {
    id: 'material.frameAll',
    keys: [{ key: 'a' }],
    action: 'material.frameAll',
    description: '适应窗口',
    category: 'view',
    context: 'material-editor',
  },
  {
    id: 'material.frameSelected',
    keys: [{ key: 'f' }],
    action: 'material.frameSelected',
    description: '聚焦选中',
    category: 'view',
    context: 'material-editor',
  },
  {
    id: 'material.zoomIn',
    keys: [{ key: '=' }],
    action: 'material.zoomIn',
    description: '放大',
    category: 'view',
    context: 'material-editor',
  },
  {
    id: 'material.zoomOut',
    keys: [{ key: '-' }],
    action: 'material.zoomOut',
    description: '缩小',
    category: 'view',
    context: 'material-editor',
  },

  // 连接操作
  {
    id: 'material.connectNodes',
    keys: [{ key: 'c' }],
    action: 'material.connectNodes',
    description: '连接节点',
    category: 'edit',
    context: 'material-editor',
  },
  {
    id: 'material.disconnectNodes',
    keys: [{ key: 'x' }],
    action: 'material.disconnectNodes',
    description: '断开连接',
    category: 'edit',
    context: 'material-editor',
  },

  // 常用节点
  {
    id: 'material.addTextureNode',
    keys: [{ key: 't' }],
    action: 'material.addTextureNode',
    description: '添加纹理节点',
    category: 'tools',
    context: 'material-editor',
  },
  {
    id: 'material.addColorNode',
    keys: [{ key: 'c', shift: true }],
    action: 'material.addColorNode',
    description: '添加颜色节点',
    category: 'tools',
    context: 'material-editor',
  },
  {
    id: 'material.addMathNode',
    keys: [{ key: 'm' }],
    action: 'material.addMathNode',
    description: '添加数学节点',
    category: 'tools',
    context: 'material-editor',
  },
  {
    id: 'material.addVectorNode',
    keys: [{ key: 'v' }],
    action: 'material.addVectorNode',
    description: '添加向量节点',
    category: 'tools',
    context: 'material-editor',
  },

  // 预览
  {
    id: 'material.togglePreview',
    keys: [{ key: 'p' }],
    action: 'material.togglePreview',
    description: '切换预览',
    category: 'view',
    context: 'material-editor',
  },
  {
    id: 'material.refreshPreview',
    keys: [{ key: 'r' }],
    action: 'material.refreshPreview',
    description: '刷新预览',
    category: 'view',
    context: 'material-editor',
  },

  // 网格
  {
    id: 'material.toggleGrid',
    keys: [{ key: 'g' }],
    action: 'material.toggleGrid',
    description: '切换网格',
    category: 'view',
    context: 'material-editor',
  },

  // 搜索
  {
    id: 'material.searchNode',
    keys: [{ key: 's', ctrl: true }],
    action: 'material.searchNode',
    description: '搜索节点',
    category: 'edit',
    context: 'material-editor',
  },

  // 选择
  {
    id: 'material.selectAll',
    keys: [{ key: 'a', ctrl: true }],
    action: 'material.selectAll',
    description: '全选',
    category: 'edit',
    context: 'material-editor',
  },
  {
    id: 'material.invertSelection',
    keys: [{ key: 'i', ctrl: true, shift: true }],
    action: 'material.invertSelection',
    description: '反选',
    category: 'edit',
    context: 'material-editor',
  },

  // 对齐
  {
    id: 'material.alignLeft',
    keys: [{ key: 'ArrowLeft', ctrl: true }],
    action: 'material.alignLeft',
    description: '左对齐',
    category: 'edit',
    context: 'material-editor',
  },
  {
    id: 'material.alignRight',
    keys: [{ key: 'ArrowRight', ctrl: true }],
    action: 'material.alignRight',
    description: '右对齐',
    category: 'edit',
    context: 'material-editor',
  },
  {
    id: 'material.alignTop',
    keys: [{ key: 'ArrowUp', ctrl: true }],
    action: 'material.alignTop',
    description: '顶对齐',
    category: 'edit',
    context: 'material-editor',
  },
  {
    id: 'material.alignBottom',
    keys: [{ key: 'ArrowDown', ctrl: true }],
    action: 'material.alignBottom',
    description: '底对齐',
    category: 'edit',
    context: 'material-editor',
  },
];
