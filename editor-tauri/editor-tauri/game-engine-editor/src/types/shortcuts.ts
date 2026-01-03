/**
 * 快捷键系统类型定义
 * 完整的快捷键类型系统，支持单键、组合键、序列键等
 */

/**
 * 快捷键定义
 */
export interface Shortcut {
  /** 唯一标识符 */
  id: string;
  /** 按键序列（支持单键和序列键） */
  keys: KeySequence;
  /** 动作：函数或命令名称 */
  action: ShortcutAction;
  /** 上下文 */
  context?: ShortcutContext;
  /** 描述 */
  description: string;
  /** 分类 */
  category: ShortcutCategory;
  /** 优先级（数字越大优先级越高） */
  priority?: number;
  /** 是否启用 */
  enabled?: boolean;
  /** 平台限制 */
  platform?: Platform;
  /** 是否为用户自定义 */
  customized?: boolean;
  /** 默认按键（用于重置） */
  defaultKeys?: KeySequence;
}

/**
 * 按键序列（支持多键序列，如 Vim 的 Ctrl+K, Ctrl+K）
 */
export type KeySequence = KeyCombo[];

/**
 * 按键组合
 */
export interface KeyCombo {
  /** 按键名称 */
  key: string;
  /** Ctrl 键 */
  ctrl?: boolean;
  /** Shift 键 */
  shift?: boolean;
  /** Alt 键 */
  alt?: boolean;
  /** Meta 键（Windows/Cmd） */
  meta?: boolean;
}

/**
 * 快捷键动作
 */
export type ShortcutAction = (() => void) | (() => Promise<void>) | string;

/**
 * 快捷键上下文
 */
export type ShortcutContext =
  | 'global'
  | 'editor'
  | 'viewport'
  | 'material-editor'
  | 'behavior-editor'
  | 'timeline'
  | 'asset-browser'
  | 'console';

/**
 * 快捷键分类
 */
export type ShortcutCategory =
  | 'file'
  | 'edit'
  | 'view'
  | 'tools'
  | 'window'
  | 'help'
  | 'playback'
  | 'transform'
  | 'navigation';

/**
 * 平台类型
 */
export type Platform = 'all' | 'windows' | 'macos' | 'linux';

/**
 * 快捷键冲突
 */
export interface ShortcutConflict {
  /** 冲突的按键序列 */
  keys: KeySequence;
  /** 冲突的快捷键列表 */
  shortcuts: Shortcut[];
  /** 严重程度 */
  severity: ConflictSeverity;
  /** 解决方案 */
  resolution?: ConflictResolution;
}

/**
 * 冲突严重程度
 */
export type ConflictSeverity = 'error' | 'warning';

/**
 * 冲突解决方案
 */
export type ConflictResolution = 'disable-old' | 'disable-new' | 'keep-both' | 'custom';

/**
 * 快捷键配置
 */
export interface ShortcutConfig {
  /** 版本号 */
  version: string;
  /** 快捷键映射 */
  shortcuts: Record<string, KeySequence>;
  /** 禁用的快捷键 */
  disabled?: string[];
  /** 元数据 */
  metadata?: {
    exportedAt: string;
    platform: Platform;
    preset?: ShortcutPreset;
  };
}

/**
 * 快捷键预设方案
 */
export type ShortcutPreset = 'default' | 'vscode' | 'unity' | 'unreal' | 'blender';

/**
 * 快捷键事件
 */
export interface ShortcutEvent {
  /** 快捷键ID */
  shortcutId: string;
  /** 按键序列 */
  keys: KeySequence;
  /** 时间戳 */
  timestamp: number;
  /** 是否被阻止默认行为 */
  prevented: boolean;
}

/**
 * 快捷键统计
 */
export interface ShortcutStatistics {
  /** 总快捷键数 */
  total: number;
  /** 已启用的快捷键数 */
  enabled: number;
  /** 用户自定义的快捷键数 */
  customized: number;
  /** 各上下文的快捷键数 */
  byContext: Record<ShortcutContext, number>;
  /** 各分类的快捷键数 */
  byCategory: Record<ShortcutCategory, number>;
}

/**
 * 快捷键帮助项
 */
export interface ShortcutHelpItem {
  /** 快捷键 */
  shortcut: Shortcut;
  /** 格式化的按键显示 */
  formattedKeys: string;
  /** 是否有冲突 */
  hasConflict: boolean;
}

/**
 * 快捷键编辑器状态
 */
export interface ShortcutEditorState {
  /** 正在编辑的快捷键ID */
  editingId: string | null;
  /** 正在录制的按键 */
  recordingKeys: KeySequence | null;
  /** 冲突列表 */
  conflicts: ShortcutConflict[];
  /** 搜索关键词 */
  searchQuery: string;
  /** 当前分类筛选 */
  categoryFilter: ShortcutCategory | 'all';
  /** 当前上下文筛选 */
  contextFilter: ShortcutContext | 'all';
}

/**
 * 快捷键覆盖层配置
 */
export interface ShortcutOverlayConfig {
  /** 是否启用工具提示 */
  showTooltips: boolean;
  /** 是否启用帮助面板 */
  showHelpPanel: boolean;
  /** 是否显示冲突提示 */
  showConflicts: boolean;
  /** 是否启用学习模式 */
  learningMode: boolean;
}

/**
 * 按键录制状态
 */
export interface KeyRecordingState {
  /** 是否正在录制 */
  isRecording: boolean;
  /** 已录制的按键序列 */
  recordedKeys: KeySequence;
  /** 当前按键组合 */
  currentCombo: KeyCombo | null;
  /** 录制开始时间 */
  startTime: number;
  /** 超时时间（毫秒） */
  timeout: number;
}
