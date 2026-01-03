/**
 * 时间轴快捷键
 */

import { Shortcut } from '../types/shortcuts';

export const timelineShortcuts: Shortcut[] = [
  // 播放控制
  {
    id: 'timeline.play',
    keys: [{ key: ' ' }],
    action: 'timeline.play',
    description: '播放/暂停',
    category: 'playback',
    context: 'timeline',
    priority: 100,
  },
  {
    id: 'timeline.stop',
    keys: [{ key: 'Escape' }],
    action: 'timeline.stop',
    description: '停止',
    category: 'playback',
    context: 'timeline',
  },
  {
    id: 'timeline.gotoStart',
    keys: [{ key: 'Home' }],
    action: 'timeline.gotoStart',
    description: '跳转到开始',
    category: 'playback',
    context: 'timeline',
  },
  {
    id: 'timeline.gotoEnd',
    keys: [{ key: 'End' }],
    action: 'timeline.gotoEnd',
    description: '跳转到结束',
    category: 'playback',
    context: 'timeline',
  },
  {
    id: 'timeline.previousFrame',
    keys: [{ key: ',' }],
    action: 'timeline.previousFrame',
    description: '上一帧',
    category: 'playback',
    context: 'timeline',
  },
  {
    id: 'timeline.nextFrame',
    keys: [{ key: '.' }],
    action: 'timeline.nextFrame',
    description: '下一帧',
    category: 'playback',
    context: 'timeline',
  },
  {
    id: 'timeline.previousKey',
    keys: [{ key: 'ArrowLeft' }],
    action: 'timeline.previousKey',
    description: '上一关键帧',
    category: 'playback',
    context: 'timeline',
  },
  {
    id: 'timeline.nextKey',
    keys: [{ key: 'ArrowRight' }],
    action: 'timeline.nextKey',
    description: '下一关键帧',
    category: 'playback',
    context: 'timeline',
  },

  // 关键帧操作
  {
    id: 'timeline.addKey',
    keys: [{ key: 'k' }],
    action: 'timeline.addKey',
    description: '添加关键帧',
    category: 'edit',
    context: 'timeline',
  },
  {
    id: 'timeline.deleteKey',
    keys: [{ key: 'Delete' }],
    action: 'timeline.deleteKey',
    description: '删除关键帧',
    category: 'edit',
    context: 'timeline',
  },
  {
    id: 'timeline.copyKeys',
    keys: [{ key: 'c', ctrl: true }],
    action: 'timeline.copyKeys',
    description: '复制关键帧',
    category: 'edit',
    context: 'timeline',
  },
  {
    id: 'timeline.pasteKeys',
    keys: [{ key: 'v', ctrl: true }],
    action: 'timeline.pasteKeys',
    description: '粘贴关键帧',
    category: 'edit',
    context: 'timeline',
  },
  {
    id: 'timeline.cutKeys',
    keys: [{ key: 'x', ctrl: true }],
    action: 'timeline.cutKeys',
    description: '剪切关键帧',
    category: 'edit',
    context: 'timeline',
  },

  // 视图操作
  {
    id: 'timeline.zoomIn',
    keys: [{ key: '=' }],
    action: 'timeline.zoomIn',
    description: '放大时间轴',
    category: 'view',
    context: 'timeline',
  },
  {
    id: 'timeline.zoomOut',
    keys: [{ key: '-' }],
    action: 'timeline.zoomOut',
    description: '缩小时间轴',
    category: 'view',
    context: 'timeline',
  },
  {
    id: 'timeline.fitToWindow',
    keys: [{ key: 'f' }],
    action: 'timeline.fitToWindow',
    description: '适应窗口',
    category: 'view',
    context: 'timeline',
  },

  // 轨道操作
  {
    id: 'timeline.addTrack',
    keys: [{ key: 't' }],
    action: 'timeline.addTrack',
    description: '添加轨道',
    category: 'edit',
    context: 'timeline',
  },
  {
    id: 'timeline.deleteTrack',
    keys: [{ key: 'Delete', shift: true }],
    action: 'timeline.deleteTrack',
    description: '删除轨道',
    category: 'edit',
    context: 'timeline',
  },
  {
    id: 'timeline.renameTrack',
    keys: [{ key: 'F2' }],
    action: 'timeline.renameTrack',
    description: '重命名轨道',
    category: 'edit',
    context: 'timeline',
  },
  {
    id: 'timeline.toggleTrack',
    keys: [{ key: 'm' }],
    action: 'timeline.toggleTrack',
    description: '切换轨道可见性',
    category: 'view',
    context: 'timeline',
  },
  {
    id: 'timeline.lockTrack',
    keys: [{ key: 'l' }],
    action: 'timeline.lockTrack',
    description: '锁定轨道',
    category: 'edit',
    context: 'timeline',
  },

  // 选择操作
  {
    id: 'timeline.selectAll',
    keys: [{ key: 'a', ctrl: true }],
    action: 'timeline.selectAll',
    description: '全选',
    category: 'edit',
    context: 'timeline',
  },
  {
    id: 'timeline.selectNone',
    keys: [{ key: 'd', ctrl: true, shift: true }],
    action: 'timeline.selectNone',
    description: '取消选择',
    category: 'edit',
    context: 'timeline',
  },
  {
    id: 'timeline.invertSelection',
    keys: [{ key: 'i', ctrl: true, shift: true }],
    action: 'timeline.invertSelection',
    description: '反选',
    category: 'edit',
    context: 'timeline',
  },

  // 时间控制
  {
    id: 'timeline.setTime',
    keys: [{ key: 'g' }],
    action: 'timeline.setTime',
    description: '设置时间',
    category: 'edit',
    context: 'timeline',
  },
  {
    id: 'timeline.setStartTime',
    keys: [{ key: 's', shift: true }],
    action: 'timeline.setStartTime',
    description: '设置开始时间',
    category: 'edit',
    context: 'timeline',
  },
  {
    id: 'timeline.setEndTime',
    keys: [{ key: 'e', shift: true }],
    action: 'timeline.setEndTime',
    description: '设置结束时间',
    category: 'edit',
    context: 'timeline',
  },

  // 关键帧类型
  {
    id: 'timeline.setLinear',
    keys: [{ key: '1' }],
    action: 'timeline.setLinear',
    description: '线性插值',
    category: 'edit',
    context: 'timeline',
  },
  {
    id: 'timeline.setStep',
    keys: [{ key: '2' }],
    action: 'timeline.setStep',
    description: '阶梯插值',
    category: 'edit',
    context: 'timeline',
  },
  {
    id: 'timeline.setCubic',
    keys: [{ key: '3' }],
    action: 'timeline.setCubic',
    description: '三次样条插值',
    category: 'edit',
    context: 'timeline',
  },

  // 播放选项
  {
    id: 'timeline.toggleLoop',
    keys: [{ key: 'l', shift: true }],
    action: 'timeline.toggleLoop',
    description: '切换循环',
    category: 'playback',
    context: 'timeline',
  },
  {
    id: 'timeline.togglePingPong',
    keys: [{ key: 'p' }],
    action: 'timeline.togglePingPong',
    description: '切换往复',
    category: 'playback',
    context: 'timeline',
  },

  // 预览
  {
    id: 'timeline.toggleRealtime',
    keys: [{ key: 'r' }],
    action: 'timeline.toggleRealtime',
    description: '切换实时预览',
    category: 'view',
    context: 'timeline',
  },
  {
    id: 'timeline.toggleSound',
    keys: [{ key: 's', alt: true }],
    action: 'timeline.toggleSound',
    description: '切换声音',
    category: 'view',
    context: 'timeline',
  },

  // 剪辑操作
  {
    id: 'timeline.splitClip',
    keys: [{ key: 's', ctrl: true }],
    action: 'timeline.splitClip',
    description: '分割剪辑',
    category: 'edit',
    context: 'timeline',
  },
  {
    id: 'timeline.mergeClips',
    keys: [{ key: 'm', ctrl: true }],
    action: 'timeline.mergeClips',
    description: '合并剪辑',
    category: 'edit',
    context: 'timeline',
  },

  // 标记
  {
    id: 'timeline.addMarker',
    keys: [{ key: 'm' }],
    action: 'timeline.addMarker',
    description: '添加标记',
    category: 'edit',
    context: 'timeline',
  },
  {
    id: 'timeline.deleteMarker',
    keys: [{ key: 'Delete', alt: true }],
    action: 'timeline.deleteMarker',
    description: '删除标记',
    category: 'edit',
    context: 'timeline',
  },
  {
    id: 'timeline.gotoMarker',
    keys: [{ key: 'g', ctrl: true }],
    action: 'timeline.gotoMarker',
    description: '跳转到标记',
    category: 'view',
    context: 'timeline',
  },
];
