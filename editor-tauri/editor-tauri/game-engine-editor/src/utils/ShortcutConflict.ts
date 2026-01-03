/**
 * 快捷键冲突检测和解决
 */

import {
  Shortcut,
  KeySequence,
  ShortcutConflict as Conflict,
  ConflictSeverity,
  ConflictResolution,
  ShortcutContext,
} from '../types/shortcuts';
import { ShortcutRegistry } from './ShortcutRegistry';

export class ShortcutConflictDetector {
  private registry: ShortcutRegistry;

  constructor(registry: ShortcutRegistry) {
    this.registry = registry;
  }

  /**
   * 检测所有冲突
   */
  detectAll(): Conflict[] {
    const conflicts: Conflict[] = [];
    const conflictMap = this.registry.findConflicts();

    for (const [keySig, shortcuts] of conflictMap.entries()) {
      const conflict = this.analyzeConflict(shortcuts);
      if (conflict) {
        conflicts.push(conflict);
      }
    }

    return conflicts.sort((a, b) => {
      // 优先显示严重错误
      if (a.severity === 'error' && b.severity === 'warning') return -1;
      if (a.severity === 'warning' && b.severity === 'error') return 1;
      return 0;
    });
  }

  /**
   * 检测单个快捷键的冲突
   */
  detectForShortcut(shortcut: Shortcut): Conflict[] {
    const conflicts: Conflict[] = [];
    const allShortcuts = this.registry.findByKeys(shortcut.keys);

    if (allShortcuts.length <= 1) {
      return conflicts;
    }

    const conflict = this.analyzeConflict(allShortcuts);
    if (conflict) {
      conflicts.push(conflict);
    }

    return conflicts;
  }

  /**
   * 分析冲突严重程度
   */
  private analyzeConflict(shortcuts: Shortcut[]): Conflict | null {
    if (shortcuts.length < 2) {
      return null;
    }

    const keys = shortcuts[0].keys;
    const severity = this.determineSeverity(shortcuts);
    const suggestedResolution = this.suggestResolution(shortcuts, severity);

    return {
      keys,
      shortcuts,
      severity,
      resolution: suggestedResolution,
    };
  }

  /**
   * 确定冲突严重程度
   */
  private determineSeverity(shortcuts: Shortcut[]): ConflictSeverity {
    const contexts = new Set(shortcuts.map(s => s.context || 'global'));

    // 不同上下文的快捷键共享按键是警告（优先级解决）
    if (contexts.size > 1) {
      return 'warning';
    }

    // 相同上下文的冲突是错误
    return 'error';
  }

  /**
   * 建议解决方案
   */
  private suggestResolution(
    shortcuts: Shortcut[],
    severity: ConflictSeverity
  ): ConflictResolution {
    if (severity === 'warning') {
      // 不同上下文，可以共存
      return 'keep-both';
    }

    // 相同上下文，保留优先级高的或用户自定义的
    const customShortcut = shortcuts.find(s => s.customized);
    if (customShortcut) {
      // 保留自定义的，禁用其他的
      return 'disable-old';
    }

    // 比较优先级
    const sorted = [...shortcuts].sort((a, b) => (b.priority || 0) - (a.priority || 0));
    if (sorted[0].priority !== undefined && sorted[0].priority > 0) {
      return 'disable-new';
    }

    return 'custom';
  }

  /**
   * 检测快捷键是否与其他快捷键冲突
   */
  willConflict(shortcut: Shortcut, excludeId?: string): boolean {
    const existing = this.registry.findByKeys(shortcut.keys);
    const filtered = excludeId
      ? existing.filter(s => s.id !== excludeId)
      : existing;

    // 检查是否在同一上下文
    for (const other of filtered) {
      if (other.context === shortcut.context) {
        return true;
      }
    }

    return false;
  }

  /**
   * 获取冲突详情
   */
  getConflictDetails(conflict: Conflict): string {
    const { shortcuts, severity } = conflict;
    const names = shortcuts.map(s => s.description || s.id).join('", "');

    if (severity === 'error') {
      return `严重冲突：快捷键 "${this.formatKeys(conflict.keys)}" 被多个快捷键使用：${names}`;
    } else {
      return `警告：快捷键 "${this.formatKeys(conflict.keys)}" 在不同上下文被使用：${names}`;
    }
  }

  /**
   * 格式化按键显示
   */
  private formatKeys(keys: KeySequence): string {
    return keys
      .map(combo => {
        const parts: string[] = [];
        if (combo.ctrl) parts.push('Ctrl');
        if (combo.shift) parts.push('Shift');
        if (combo.alt) parts.push('Alt');
        if (combo.meta) parts.push(this.isMac() ? 'Cmd' : 'Win');
        parts.push(combo.key.toUpperCase());
        return parts.join('+');
      })
      .join(', ');
  }

  private isMac(): boolean {
    return navigator.userAgent.includes('Mac OS X');
  }
}

export class ShortcutConflictResolver {
  private registry: ShortcutRegistry;

  constructor(registry: ShortcutRegistry) {
    this.registry = registry;
  }

  /**
   * 解决冲突
   */
  resolve(conflict: Conflict, resolution: ConflictResolution): boolean {
    const { shortcuts } = conflict;

    switch (resolution) {
      case 'disable-old':
        // 禁用除第一个外的所有快捷键
        for (let i = 1; i < shortcuts.length; i++) {
          this.registry.disable(shortcuts[i].id);
        }
        return true;

      case 'disable-new':
        // 禁用最后一个快捷键
        const last = shortcuts[shortcuts.length - 1];
        this.registry.disable(last.id);
        return true;

      case 'keep-both':
        // 保留所有（适用于不同上下文）
        return true;

      case 'custom':
        // 需要用户手动解决
        return false;

      default:
        return false;
    }
  }

  /**
   * 自动解决所有冲突
   */
  autoResolve(conflicts: Conflict[]): { resolved: number; failed: number } {
    let resolved = 0;
    let failed = 0;

    for (const conflict of conflicts) {
      if (conflict.resolution && conflict.resolution !== 'custom') {
        if (this.resolve(conflict, conflict.resolution)) {
          resolved++;
        } else {
          failed++;
        }
      } else {
        failed++;
      }
    }

    return { resolved, failed };
  }

  /**
   * 通过优先级解决冲突
   */
  resolveByPriority(conflict: Conflict): boolean {
    const { shortcuts } = conflict;

    // 按优先级排序
    const sorted = [...shortcuts].sort((a, b) => {
      // 用户自定义优先
      if (a.customized && !b.customized) return -1;
      if (!a.customized && b.customized) return 1;

      // 然后按优先级值
      return (b.priority || 0) - (a.priority || 0);
    });

    // 禁用除最高优先级外的所有快捷键
    for (let i = 1; i < sorted.length; i++) {
      this.registry.disable(sorted[i].id);
    }

    return true;
  }

  /**
   * 通过上下文分离解决冲突
   */
  resolveByContext(conflict: Conflict): boolean {
    const { shortcuts } = conflict;

    // 如果在不同上下文，可以共存
    const contexts = new Set(shortcuts.map(s => s.context || 'global'));
    if (contexts.size > 1) {
      return true; // 已经通过上下文分离
    }

    // 相同上下文，使用优先级
    return this.resolveByPriority(conflict);
  }

  /**
   * 重命名快捷键以避免冲突
   */
  resolveByRenaming(
    conflict: Conflict,
    newKeys: KeySequence,
    shortcutId: string
  ): boolean {
    const shortcut = conflict.shortcuts.find(s => s.id === shortcutId);
    if (!shortcut) {
      return false;
    }

    return this.registry.update(shortcutId, {
      keys: newKeys,
      customized: true,
    });
  }
}
