/**
 * 快捷键管理器
 * 中心化的快捷键管理系统
 */

import { EventEmitter } from 'events';
import {
  Shortcut,
  KeySequence,
  KeyCombo,
  ShortcutContext,
  ShortcutCategory,
  ShortcutConflict,
  ShortcutConfig,
  ShortcutPreset,
  ShortcutEvent,
  ShortcutStatistics,
  KeyRecordingState,
  Platform,
} from '../types/shortcuts';
import { ShortcutRegistry } from './ShortcutRegistry';
import { ShortcutConflictDetector, ShortcutConflictResolver } from './ShortcutConflict';

export class ShortcutManager extends EventEmitter {
  private registry: ShortcutRegistry;
  private detector: ShortcutConflictDetector;
  private resolver: ShortcutConflictResolver;
  private currentContext: ShortcutContext = 'global';
  private recordingState: KeyRecordingState | null = null;
  private sequenceBuffer: KeyCombo[] = [];
  private sequenceTimer: number | null = null;
  private commandExecutor?: (command: string) => void;

  constructor() {
    super();
    this.registry = new ShortcutRegistry();
    this.detector = new ShortcutConflictDetector(this.registry);
    this.resolver = new ShortcutConflictResolver(this.registry);

    // 设置序列按键超时（1秒）
    this.SEQUENCE_TIMEOUT = 1000;
  }

  private SEQUENCE_TIMEOUT: number;

  /**
   * 设置命令执行器
   */
  setCommandExecutor(executor: (command: string) => void): void {
    this.commandExecutor = executor;
  }

  // ==================== 注册管理 ====================

  /**
   * 注册单个快捷键
   */
  register(shortcut: Shortcut): boolean {
    // 检测冲突
    const conflicts = this.detector.detectForShortcut(shortcut);
    if (conflicts.length > 0) {
      this.emit('conflict-detected', conflicts[0]);
    }

    return this.registry.register(shortcut);
  }

  /**
   * 批量注册快捷键
   */
  registerAll(shortcuts: Shortcut[]): number {
    const count = this.registry.registerAll(shortcuts);

    // 检测所有冲突
    const conflicts = this.detector.detectAll();
    if (conflicts.length > 0) {
      this.emit('conflicts-detected', conflicts);
    }

    return count;
  }

  /**
   * 注销快捷键
   */
  unregister(id: string): boolean {
    return this.registry.unregister(id);
  }

  // ==================== 查询 ====================

  /**
   * 获取快捷键
   */
  getShortcut(id: string): Shortcut | undefined {
    return this.registry.get(id);
  }

  /**
   * 获取所有快捷键
   */
  getAllShortcuts(): Shortcut[] {
    return this.registry.getAll();
  }

  /**
   * 根据上下文获取快捷键
   */
  getShortcutsByContext(context: ShortcutContext): Shortcut[] {
    return this.registry.findByContext(context);
  }

  /**
   * 根据分类获取快捷键
   */
  getShortcutsByCategory(category: ShortcutCategory): Shortcut[] {
    return this.registry.findByCategory(category);
  }

  /**
   * 根据按键序列查找快捷键
   */
  findShortcutByKeySequence(keys: KeySequence): Shortcut[] {
    return this.registry.findByKeys(keys);
  }

  /**
   * 搜索快捷键
   */
  searchShortcuts(query: string): Shortcut[] {
    const lowerQuery = query.toLowerCase();
    return this.registry.getAll().filter(
      s =>
        s.description.toLowerCase().includes(lowerQuery) ||
        s.id.toLowerCase().includes(lowerQuery) ||
        this.formatKeys(s.keys).toLowerCase().includes(lowerQuery)
    );
  }

  // ==================== 执行 ====================

  /**
   * 处理键盘事件
   */
  handleKeyboardEvent(event: KeyboardEvent): boolean {
    // 如果正在录制，不执行快捷键
    if (this.recordingState?.isRecording) {
      this.recordKeyPress(event);
      return true;
    }

    // 将键盘事件转换为按键组合
    const combo = this.eventToCombo(event);

    // 添加到序列缓冲区
    this.sequenceBuffer.push(combo);

    // 清除之前的定时器
    if (this.sequenceTimer) {
      clearTimeout(this.sequenceTimer);
    }

    // 检查是否有匹配的快捷键
    const matched = this.findMatchingShortcut();

    if (matched) {
      // 执行快捷键
      this.executeShortcut(matched);
      this.sequenceBuffer = [];
      this.sequenceTimer = null;
      return true;
    } else {
      // 设置超时，如果超时后没有匹配，清空缓冲区
      this.sequenceTimer = window.setTimeout(() => {
        this.sequenceBuffer = [];
        this.sequenceTimer = null;
      }, this.SEQUENCE_TIMEOUT);
    }

    return false;
  }

  /**
   * 查找匹配的快捷键
   */
  private findMatchingShortcut(): Shortcut | null {
    // 先尝试完全匹配
    const exactMatches = this.registry.findByKeys(this.sequenceBuffer as KeySequence);

    // 过滤当前上下文
    const contextMatches = exactMatches.filter(s => {
      const context = s.context || 'global';
      return context === this.currentContext || context === 'global';
    });

    if (contextMatches.length > 0) {
      // 返回优先级最高的
      return contextMatches.sort((a, b) => (b.priority || 0) - (a.priority || 0))[0];
    }

    // 检查是否可能是序列的一部分（前缀匹配）
    const prefixMatches = this.registry.getAll().filter(s => {
      const keys = s.keys;
      if (keys.length <= this.sequenceBuffer.length) {
        return false;
      }

      // 检查缓冲区是否是快捷键的前缀
      for (let i = 0; i < this.sequenceBuffer.length; i++) {
        if (!this.compareCombo(this.sequenceBuffer[i], keys[i])) {
          return false;
        }
      }

      return true;
    });

    // 如果有前缀匹配，等待更多输入
    if (prefixMatches.length > 0) {
      return null;
    }

    // 没有匹配，返回 null
    return null;
  }

  /**
   * 比较两个按键组合是否相同
   */
  private compareCombo(a: KeyCombo, b: KeyCombo): boolean {
    return (
      a.key === b.key &&
      a.ctrl === b.ctrl &&
      a.shift === b.shift &&
      a.alt === b.alt &&
      a.meta === b.meta
    );
  }

  /**
   * 执行快捷键
   */
  executeShortcut(shortcut: Shortcut): boolean {
    try {
      const { action, id } = shortcut;

      // 触发事件
      const event: ShortcutEvent = {
        shortcutId: id,
        keys: shortcut.keys,
        timestamp: Date.now(),
        prevented: false,
      };

      if (typeof action === 'string') {
        // 命令名称
        if (this.commandExecutor) {
          this.commandExecutor(action);
        } else {
          console.warn(`No command executor set for action: ${action}`);
        }
      } else if (typeof action === 'function') {
        // 函数
        const result = action();
        if (result instanceof Promise) {
          result.catch(err => {
            console.error(`Error executing shortcut ${id}:`, err);
          });
        }
      }

      this.emit('shortcut-executed', event);
      return true;
    } catch (error) {
      console.error(`Error executing shortcut ${shortcut.id}:`, error);
      return false;
    }
  }

  /**
   * 通过ID执行快捷键
   */
  executeShortcutById(id: string): boolean {
    const shortcut = this.registry.get(id);
    if (!shortcut) {
      return false;
    }
    return this.executeShortcut(shortcut);
  }

  // ==================== 上下文管理 ====================

  /**
   * 设置当前上下文
   */
  setContext(context: ShortcutContext): void {
    this.currentContext = context;
    this.emit('context-changed', context);
  }

  /**
   * 获取当前上下文
   */
  getContext(): ShortcutContext {
    return this.currentContext;
  }

  // ==================== 冲突检测 ====================

  /**
   * 检测所有冲突
   */
  detectConflicts(): ShortcutConflict[] {
    return this.detector.detectAll();
  }

  /**
   * 解决冲突
   */
  resolveConflict(conflict: ShortcutConflict, resolution: string): boolean {
    return this.resolver.resolve(conflict, resolution as any);
  }

  /**
   * 自动解决所有冲突
   */
  autoResolveConflicts(): { resolved: number; failed: number } {
    const conflicts = this.detector.detectAll();
    return this.resolver.autoResolve(conflicts);
  }

  // ==================== 启用/禁用 ====================

  /**
   * 启用快捷键
   */
  enableShortcut(id: string): boolean {
    return this.registry.enable(id);
  }

  /**
   * 禁用快捷键
   */
  disableShortcut(id: string): boolean {
    return this.registry.disable(id);
  }

  /**
   * 检查快捷键是否启用
   */
  isShortcutEnabled(id: string): boolean {
    const shortcut = this.registry.get(id);
    return shortcut ? shortcut.enabled !== false : false;
  }

  // ==================== 自定义 ====================

  /**
   * 自定义快捷键
   */
  customizeShortcut(id: string, newKeys: KeySequence): boolean {
    // 检测冲突
    const shortcut = this.registry.get(id);
    if (!shortcut) {
      return false;
    }

    const tempShortcut = { ...shortcut, keys: newKeys };
    const hasConflict = this.detector.willConflict(tempShortcut, id);

    if (hasConflict) {
      this.emit('conflict-detected', {
        keys: newKeys,
        shortcuts: [tempShortcut],
        severity: 'error',
      });
      return false;
    }

    return this.registry.update(id, {
      keys: newKeys,
      customized: true,
    });
  }

  /**
   * 重置快捷键
   */
  resetShortcut(id: string): boolean {
    return this.registry.reset(id);
  }

  /**
   * 重置所有快捷键
   */
  resetAllShortcuts(): number {
    return this.registry.resetAll();
  }

  // ==================== 导入/导出 ====================

  /**
   * 导出快捷键配置
   */
  exportShortcuts(): ShortcutConfig {
    return {
      version: '1.0.0',
      shortcuts: this.registry.export(),
      disabled: this.getAllShortcuts()
        .filter(s => s.enabled === false)
        .map(s => s.id),
      metadata: {
        exportedAt: new Date().toISOString(),
        platform: this.getCurrentPlatform(),
      },
    };
  }

  /**
   * 导入快捷键配置
   */
  importShortcuts(config: ShortcutConfig): number {
    // 导入自定义按键
    const count = this.registry.import(config.shortcuts);

    // 应用禁用状态
    if (config.disabled) {
      for (const id of config.disabled) {
        this.registry.disable(id);
      }
    }

    // 检测冲突
    const conflicts = this.detector.detectAll();
    if (conflicts.length > 0) {
      this.emit('conflicts-detected', conflicts);
    }

    return count;
  }

  /**
   * 应用预设方案
   */
  applyPreset(preset: ShortcutPreset): void {
    // 重置所有快捷键
    this.resetAllShortcuts();

    // 根据预设应用不同的快捷键映射
    // 这里需要根据预设加载不同的快捷键配置
    this.emit('preset-applied', preset);
  }

  // ==================== 按键录制 ====================

  /**
   * 开始录制按键
   */
  startRecording(): void {
    this.recordingState = {
      isRecording: true,
      recordedKeys: [],
      currentCombo: null,
      startTime: Date.now(),
      timeout: 5000, // 5秒超时
    };
    this.emit('recording-started');
  }

  /**
   * 停止录制按键
   */
  stopRecording(): KeySequence | null {
    if (!this.recordingState) {
      return null;
    }

    const keys = [...this.recordingState.recordedKeys];
    this.recordingState = null;
    this.emit('recording-stopped', keys);
    return keys;
  }

  /**
   * 取消录制
   */
  cancelRecording(): void {
    this.recordingState = null;
    this.emit('recording-cancelled');
  }

  /**
   * 记录按键
   */
  private recordKeyPress(event: KeyboardEvent): void {
    if (!this.recordingState) {
      return;
    }

    const combo = this.eventToCombo(event);
    this.recordingState.recordedKeys.push(combo);
    this.emit('key-recorded', combo);

    // 检查超时
    if (Date.now() - this.recordingState.startTime > this.recordingState.timeout) {
      this.stopRecording();
    }
  }

  // ==================== 统计 ====================

  /**
   * 获取统计信息
   */
  getStatistics(): ShortcutStatistics {
    const stats = this.registry.getStats();
    return {
      total: stats.total,
      enabled: stats.enabled,
      customized: stats.customized,
      byContext: stats.byContext as any,
      byCategory: stats.byCategory as any,
    };
  }

  // ==================== 工具方法 ====================

  /**
   * 将键盘事件转换为按键组合
   */
  private eventToCombo(event: KeyboardEvent): KeyCombo {
    return {
      key: event.key,
      ctrl: event.ctrlKey,
      shift: event.shiftKey,
      alt: event.altKey,
      meta: event.metaKey,
    };
  }

  /**
   * 格式化按键显示
   */
  formatKeys(keys: KeySequence): string {
    return keys
      .map(combo => {
        const parts: string[] = [];
        if (combo.ctrl) parts.push(this.isMac() ? '⌘' : 'Ctrl');
        if (combo.shift) parts.push('Shift');
        if (combo.alt) parts.push(this.isMac() ? 'Option' : 'Alt');
        if (combo.meta) parts.push(this.isMac() ? '⌘' : 'Win');
        parts.push(combo.key.toUpperCase());
        return parts.join(this.isMac() ? '' : '+');
      })
      .join(', ');
  }

  /**
   * 获取当前平台
   */
  private getCurrentPlatform(): Platform {
    const userAgent = navigator.userAgent;
    if (userAgent.includes('Mac OS X')) {
      return 'macos';
    } else if (userAgent.includes('Windows')) {
      return 'windows';
    } else if (userAgent.includes('Linux')) {
      return 'linux';
    }
    return 'all';
  }

  /**
   * 检查是否为 macOS
   */
  private isMac(): boolean {
    return navigator.userAgent.includes('Mac OS X');
  }

  /**
   * 清理资源
   */
  dispose(): void {
    if (this.sequenceTimer) {
      clearTimeout(this.sequenceTimer);
    }
    this.removeAllListeners();
  }
}

// 单例
let shortcutManager: ShortcutManager | null = null;

export function getShortcutManager(): ShortcutManager {
  if (!shortcutManager) {
    shortcutManager = new ShortcutManager();
  }
  return shortcutManager;
}
