/**
 * 快捷键注册表
 * 中心化的快捷键存储和管理
 */

import { Shortcut, KeySequence, ShortcutContext, ShortcutCategory, Platform } from '../types/shortcuts';

export class ShortcutRegistry {
  private shortcuts: Map<string, Shortcut> = new Map();
  private keyIndex: Map<string, Set<string>> = new Map(); // keySequence -> shortcutIds
  private contextIndex: Map<ShortcutContext, Set<string>> = new Map();
  private categoryIndex: Map<ShortcutCategory, Set<string>> = new Map();

  /**
   * 注册快捷键
   */
  register(shortcut: Shortcut): boolean {
    const { id, keys, context, category } = shortcut;

    // 检查是否已存在
    if (this.shortcuts.has(id)) {
      console.warn(`Shortcut ${id} already registered`);
      return false;
    }

    // 保存默认按键
    if (!shortcut.defaultKeys) {
      shortcut.defaultKeys = [...keys];
    }

    // 存储快捷键
    this.shortcuts.set(id, shortcut);

    // 更新索引
    const keySig = this.signature(keys);
    if (!this.keyIndex.has(keySig)) {
      this.keyIndex.set(keySig, new Set());
    }
    this.keyIndex.get(keySig)!.add(id);

    if (context) {
      if (!this.contextIndex.has(context)) {
        this.contextIndex.set(context, new Set());
      }
      this.contextIndex.get(context)!.add(id);
    }

    if (!this.categoryIndex.has(category)) {
      this.categoryIndex.set(category, new Set());
    }
    this.categoryIndex.get(category)!.add(id);

    return true;
  }

  /**
   * 注销快捷键
   */
  unregister(id: string): boolean {
    const shortcut = this.shortcuts.get(id);
    if (!shortcut) {
      return false;
    }

    // 从索引中移除
    const keySig = this.signature(shortcut.keys);
    const ids = this.keyIndex.get(keySig);
    if (ids) {
      ids.delete(id);
      if (ids.size === 0) {
        this.keyIndex.delete(keySig);
      }
    }

    if (shortcut.context) {
      const ctxIds = this.contextIndex.get(shortcut.context);
      if (ctxIds) {
        ctxIds.delete(id);
        if (ctxIds.size === 0) {
          this.contextIndex.delete(shortcut.context);
        }
      }
    }

    const catIds = this.categoryIndex.get(shortcut.category);
    if (catIds) {
      catIds.delete(id);
      if (catIds.size === 0) {
        this.categoryIndex.delete(shortcut.category);
      }
    }

    // 移除快捷键
    this.shortcuts.delete(id);
    return true;
  }

  /**
   * 批量注册
   */
  registerAll(shortcuts: Shortcut[]): number {
    let count = 0;
    for (const shortcut of shortcuts) {
      if (this.register(shortcut)) {
        count++;
      }
    }
    return count;
  }

  /**
   * 获取快捷键
   */
  get(id: string): Shortcut | undefined {
    return this.shortcuts.get(id);
  }

  /**
   * 获取所有快捷键
   */
  getAll(): Shortcut[] {
    return Array.from(this.shortcuts.values());
  }

  /**
   * 根据按键序列查找快捷键
   */
  findByKeys(keys: KeySequence): Shortcut[] {
    const keySig = this.signature(keys);
    const ids = this.keyIndex.get(keySig);

    if (!ids || ids.size === 0) {
      return [];
    }

    return Array.from(ids)
      .map(id => this.shortcuts.get(id)!)
      .filter(s => s && this.isEnabled(s));
  }

  /**
   * 根据上下文查找快捷键
   */
  findByContext(context: ShortcutContext): Shortcut[] {
    const ids = this.contextIndex.get(context);
    if (!ids) {
      return [];
    }
    return Array.from(ids)
      .map(id => this.shortcuts.get(id)!)
      .filter(s => s && this.isEnabled(s));
  }

  /**
   * 根据分类查找快捷键
   */
  findByCategory(category: ShortcutCategory): Shortcut[] {
    const ids = this.categoryIndex.get(category);
    if (!ids) {
      return [];
    }
    return Array.from(ids)
      .map(id => this.shortcuts.get(id)!)
      .filter(s => s && this.isEnabled(s));
  }

  /**
   * 更新快捷键
   */
  update(id: string, updates: Partial<Shortcut>): boolean {
    const shortcut = this.shortcuts.get(id);
    if (!shortcut) {
      return false;
    }

    // 如果更新按键，需要重建索引
    if (updates.keys && this.signature(updates.keys) !== this.signature(shortcut.keys)) {
      this.unregister(id);
      this.register({ ...shortcut, ...updates });
    } else {
      // 直接更新
      Object.assign(shortcut, updates);
    }

    return true;
  }

  /**
   * 启用快捷键
   */
  enable(id: string): boolean {
    const shortcut = this.shortcuts.get(id);
    if (!shortcut) {
      return false;
    }
    shortcut.enabled = true;
    return true;
  }

  /**
   * 禁用快捷键
   */
  disable(id: string): boolean {
    const shortcut = this.shortcuts.get(id);
    if (!shortcut) {
      return false;
    }
    shortcut.enabled = false;
    return true;
  }

  /**
   * 重置快捷键为默认
   */
  reset(id: string): boolean {
    const shortcut = this.shortcuts.get(id);
    if (!shortcut || !shortcut.defaultKeys) {
      return false;
    }
    return this.update(id, { keys: [...shortcut.defaultKeys], customized: false });
  }

  /**
   * 重置所有快捷键
   */
  resetAll(): number {
    let count = 0;
    for (const shortcut of this.shortcuts.values()) {
      if (shortcut.defaultKeys && this.reset(shortcut.id)) {
        count++;
      }
    }
    return count;
  }

  /**
   * 清空所有快捷键
   */
  clear(): void {
    this.shortcuts.clear();
    this.keyIndex.clear();
    this.contextIndex.clear();
    this.categoryIndex.clear();
  }

  /**
   * 获取统计信息
   */
  getStats() {
    const stats = {
      total: this.shortcuts.size,
      enabled: 0,
      customized: 0,
      byContext: {} as Record<string, number>,
      byCategory: {} as Record<string, number>,
    };

    for (const shortcut of this.shortcuts.values()) {
      if (this.isEnabled(shortcut)) {
        stats.enabled++;
      }
      if (shortcut.customized) {
        stats.customized++;
      }
      if (shortcut.context) {
        stats.byContext[shortcut.context] = (stats.byContext[shortcut.context] || 0) + 1;
      }
      stats.byCategory[shortcut.category] = (stats.byCategory[shortcut.category] || 0) + 1;
    }

    return stats;
  }

  /**
   * 导出配置
   */
  export(): Record<string, KeySequence> {
    const config: Record<string, KeySequence> = {};
    for (const [id, shortcut] of this.shortcuts.entries()) {
      if (shortcut.customized) {
        config[id] = [...shortcut.keys];
      }
    }
    return config;
  }

  /**
   * 导入配置
   */
  import(config: Record<string, KeySequence>): number {
    let count = 0;
    for (const [id, keys] of Object.entries(config)) {
      const shortcut = this.shortcuts.get(id);
      if (shortcut) {
        this.update(id, { keys, customized: true });
        count++;
      }
    }
    return count;
  }

  /**
   * 检查快捷键是否启用
   */
  private isEnabled(shortcut: Shortcut): boolean {
    // 检查 enabled 标志
    if (shortcut.enabled === false) {
      return false;
    }

    // 检查平台
    if (shortcut.platform && shortcut.platform !== 'all') {
      const currentPlatform = this.getCurrentPlatform();
      if (shortcut.platform !== currentPlatform) {
        return false;
      }
    }

    return true;
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
   * 生成按键序列的唯一签名
   */
  private signature(keys: KeySequence): string {
    return keys.map(combo => {
      const parts: string[] = [];
      if (combo.ctrl) parts.push('Ctrl');
      if (combo.shift) parts.push('Shift');
      if (combo.alt) parts.push('Alt');
      if (combo.meta) parts.push('Meta');
      parts.push(combo.key);
      return parts.join('+');
    }).join(',');
  }

  /**
   * 查找冲突的快捷键
   */
  findConflicts(): Map<string, Shortcut[]> {
    const conflicts = new Map<string, Shortcut[]>();

    // 反向索引：keySignature -> shortcuts
    const keyMap = new Map<string, Shortcut[]>();
    for (const shortcut of this.shortcuts.values()) {
      if (!this.isEnabled(shortcut)) {
        continue;
      }

      const sig = this.signature(shortcut.keys);
      if (!keyMap.has(sig)) {
        keyMap.set(sig, []);
      }
      keyMap.get(sig)!.push(shortcut);

      // 如果有多个快捷键使用相同的按键，记录冲突
      if (keyMap.get(sig)!.length > 1) {
        conflicts.set(sig, keyMap.get(sig)!);
      }
    }

    return conflicts;
  }
}
