// Enhanced History Manager for Advanced Undo/Redo functionality

import { Command } from '../types/commands';
import {
  HistoryState as EnhancedHistoryState,
  HistoryBookmark,
  HistoryBranch,
  HistoryDiff,
  HistoryStatistics,
  HistoryFilterOptions,
  HistorySearchResult,
  TransactionOptions,
  BatchOptions,
  OptimizationOptions,
  TimelineEvent,
} from '../types/history';
import { HistoryPersistence } from './HistoryPersistence';
import { BatchCommand, TransactionCommand, MacroCommand } from '../commands';
import { getGlobalRegistry } from './CommandRegistry';

/**
 * Backward compatible HistoryState interface
 */
export interface HistoryState {
  canUndo: boolean;
  canRedo: boolean;
  undoStack: Command[];
  redoStack: Command[];
}

/**
 * Enhanced History Manager with advanced features
 */
export class HistoryManager {
  private past: Command[] = [];
  private future: Command[] = [];
  private present: Command | null = null;

  private bookmarks: Map<string, HistoryBookmark> = new Map();
  private branches: Map<string, HistoryBranch> = new Map();
  private currentBranchId: string = 'main';

  private maxHistorySize: number = 100;
  private listeners: Set<(state: EnhancedHistoryState) => void> = new Set();

  private persistence?: HistoryPersistence;
  private registry = getGlobalRegistry();

  private commandIdCounter: number = 0;

  constructor(
    maxHistorySize: number = 100,
    persistence?: HistoryPersistence
  ) {
    this.maxHistorySize = maxHistorySize;
    this.persistence = persistence;

    // Initialize main branch
    this.createBranch('main', null);
  }

  // ==================== Core Operations ====================

  /**
   * Execute a command and add it to history
   */
  async executeCommand(command: Command): Promise<void> {
    const startTime = Date.now();
    let success = false;

    try {
      // Ensure command has ID and timestamp
      if (!command.id) {
        command.id = this.generateCommandId();
      }
      if (!command.timestamp) {
        command.timestamp = new Date();
      }

      await command.execute();
      success = true;

      // Move present to past
      if (this.present) {
        this.past.push(this.present);
      }

      // Set new present
      this.present = command;

      // Clear future
      this.future = [];

      // Limit history size
      await this.enforceHistoryLimit();

      // Auto-save if enabled
      if (this.persistence) {
        await this.autoSave();
      }

      this.notifyListeners();
    } catch (error) {
      console.error('Error executing command:', error);
      throw error;
    } finally {
      // Track execution in registry
      const executionTime = Date.now() - startTime;
      const type = command.constructor.name;
      this.registry.trackExecution(type, success, executionTime);
    }
  }

  /**
   * Undo the last command
   */
  async undo(): Promise<void> {
    if (!this.present) {
      return;
    }

    try {
      await this.present.undo();

      // Move present to future
      this.future.unshift(this.present);

      // Get new present from past
      this.present = this.past.pop() || null;

      this.notifyListeners();
    } catch (error) {
      console.error('Error undoing command:', error);
      throw error;
    }
  }

  /**
   * Redo the last undone command
   */
  async redo(): Promise<void> {
    if (this.future.length === 0) {
      return;
    }

    const command = this.future.shift()!;

    try {
      await command.redo();

      // Move present to past
      if (this.present) {
        this.past.push(this.present);
      }

      // Set new present
      this.present = command;

      this.notifyListeners();
    } catch (error) {
      console.error('Error redoing command:', error);
      throw error;
    }
  }

  // ==================== Batch Operations ====================

  /**
   * Execute multiple commands as a batch
   */
  async executeBatch(
    commands: Command[],
    mode: 'sequential' | 'parallel' = 'sequential'
  ): Promise<void> {
    const batch = new BatchCommand(commands, { mode });
    await this.executeCommand(batch);
  }

  /**
   * Execute commands as a transaction with rollback
   */
  async executeTransaction(
    commands: Command[],
    options?: TransactionOptions
  ): Promise<void> {
    const transaction = new TransactionCommand(commands, options);
    await this.executeCommand(transaction);
  }

  /**
   * Create and execute a macro command
   */
  async executeMacro(
    name: string,
    commands: Command[],
    parameters?: Record<string, any>
  ): Promise<void> {
    const macro = new MacroCommand(name, commands, parameters);
    await this.executeCommand(macro);
  }

  // ==================== Bookmarks ====================

  /**
   * Create a bookmark at current history state
   */
  createBookmark(
    name: string,
    description?: string,
    tags: string[] = []
  ): HistoryBookmark {
    const bookmark: HistoryBookmark = {
      id: this.generateBookmarkId(),
      name,
      description,
      timestamp: new Date(),
      stateId: this.present?.id || 'root',
      tags,
    };

    this.bookmarks.set(bookmark.id, bookmark);
    this.notifyListeners();

    return bookmark;
  }

  /**
   * Jump to a bookmark
   */
  async jumpToBookmark(bookmarkId: string): Promise<void> {
    const bookmark = this.bookmarks.get(bookmarkId);
    if (!bookmark) {
      throw new Error(`Bookmark not found: ${bookmarkId}`);
    }

    await this.jumpToState(bookmark.stateId);
  }

  /**
   * Delete a bookmark
   */
  deleteBookmark(bookmarkId: string): void {
    this.bookmarks.delete(bookmarkId);
    this.notifyListeners();
  }

  /**
   * Get all bookmarks
   */
  getBookmarks(): HistoryBookmark[] {
    return Array.from(this.bookmarks.values());
  }

  // ==================== Branches ====================

  /**
   * Create a new branch
   */
  createBranch(name: string, parentId: string | null = null): HistoryBranch {
    const branch: HistoryBranch = {
      id: this.generateBranchId(),
      name,
      parentId,
      createdAt: new Date(),
      stateId: this.present?.id || 'root',
      isActive: false,
    };

    this.branches.set(branch.id, branch);
    this.notifyListeners();

    return branch;
  }

  /**
   * Switch to a different branch
   */
  async switchBranch(branchId: string): Promise<void> {
    const branch = this.branches.get(branchId);
    if (!branch) {
      throw new Error(`Branch not found: ${branchId}`);
    }

    // Save current state
    const currentBranch = this.branches.get(this.currentBranchId);
    if (currentBranch) {
      currentBranch.stateId = this.present?.id || 'root';
      currentBranch.isActive = false;
    }

    // Switch to new branch
    this.currentBranchId = branchId;
    branch.isActive = true;

    // Jump to branch state
    await this.jumpToState(branch.stateId);

    this.notifyListeners();
  }

  /**
   * Get all branches
   */
  getBranches(): HistoryBranch[] {
    return Array.from(this.branches.values());
  }

  /**
   * Delete a branch
   */
  deleteBranch(branchId: string): void {
    if (branchId === this.currentBranchId) {
      throw new Error('Cannot delete active branch');
    }

    this.branches.delete(branchId);
    this.notifyListeners();
  }

  // ==================== Search and Filter ====================

  /**
   * Search commands by query
   */
  search(query: string): HistorySearchResult {
    const startTime = Date.now();
    const allCommands = [...this.past, this.present, ...this.future].filter(
      (cmd): cmd is Command => cmd !== null
    );

    const lowerQuery = query.toLowerCase();
    const commands = allCommands.filter(
      (cmd) =>
        cmd.description.toLowerCase().includes(lowerQuery) ||
        cmd.id?.toLowerCase().includes(lowerQuery) ||
        cmd.metadata?.tags?.some((tag: string) => tag.toLowerCase().includes(lowerQuery))
    );

    const matchingBookmarks = Array.from(this.bookmarks.values()).filter(
      (bm) =>
        bm.name.toLowerCase().includes(lowerQuery) ||
        bm.description?.toLowerCase().includes(lowerQuery) ||
        bm.tags.some((tag) => tag.toLowerCase().includes(lowerQuery))
    );

    return {
      commands,
      bookmarks: matchingBookmarks,
      totalResults: commands.length + matchingBookmarks.length,
      searchTime: Date.now() - startTime,
    };
  }

  /**
   * Filter commands by criteria
   */
  filter(options: HistoryFilterOptions): Command[] {
    let commands = [...this.past, this.present, ...this.future].filter(
      (cmd): cmd is Command => cmd !== null
    );

    if (options.dateRange) {
      commands = commands.filter((cmd) => {
        const timestamp = cmd.timestamp || new Date();
        return timestamp >= options.dateRange!.start && timestamp <= options.dateRange!.end;
      });
    }

    if (options.commandTypes && options.commandTypes.length > 0) {
      commands = commands.filter((cmd) =>
        options.commandTypes!.includes(cmd.constructor.name)
      );
    }

    if (options.tags && options.tags.length > 0) {
      commands = commands.filter((cmd) =>
        cmd.metadata?.tags?.some((tag: string) => options.tags!.includes(tag))
      );
    }

    if (options.textQuery) {
      const query = options.textQuery.toLowerCase();
      commands = commands.filter((cmd) => cmd.description.toLowerCase().includes(query));
    }

    return commands;
  }

  // ==================== Comparison ====================

  /**
   * Compare two history states
   */
  compare(stateId1: string, stateId2: string): HistoryDiff {
    const state1 = this.findStateById(stateId1);
    const state2 = this.findStateById(stateId2);

    const commands1 = this.getCommandsUpTo(state1);
    const commands2 = this.getCommandsUpTo(state2);

    const added = commands2.filter((cmd) => !commands1.includes(cmd));
    const removed = commands1.filter((cmd) => !commands2.includes(cmd));

    return {
      before: stateId1,
      after: stateId2,
      added,
      removed,
      modified: [],
      timestamp: new Date(),
    };
  }

  // ==================== Utilities ====================

  /**
   * Clear all history
   */
  clearHistory(keepRecent: number = 0): void {
    if (keepRecent > 0) {
      this.past = this.past.slice(-keepRecent);
    } else {
      this.past = [];
      this.future = [];
      this.present = null;
    }

    this.notifyListeners();
  }

  /**
   * Optimize history storage
   */
  async optimize(options: OptimizationOptions = {}): Promise<void> {
    const {
      mergeConsecutiveCommands = true,
      removeDuplicateCommands = true,
      compressOldCommands = false,
      keepBookmarks = true,
    } = options;

    if (mergeConsecutiveCommands) {
      await this.mergeConsecutiveCommands();
    }

    if (removeDuplicateCommands) {
      await this.removeDuplicateCommands();
    }

    if (!keepBookmarks) {
      this.bookmarks.clear();
    }

    this.notifyListeners();
  }

  /**
   * Get history statistics
   */
  getStatistics(): HistoryStatistics {
    const allCommands = [...this.past, this.present, ...this.future].filter(
      (cmd): cmd is Command => cmd !== null
    );

    const commandTypes = new Map<string, number>();
    allCommands.forEach((cmd) => {
      const type = cmd.constructor.name;
      commandTypes.set(type, (commandTypes.get(type) || 0) + 1);
    });

    const total = allCommands.length;
    const mostUsedCommandTypes: Array<{ type: string; count: number; percentage: number }> =
      Array.from(commandTypes.entries())
        .map(([type, count]) => ({
          type,
          count,
          percentage: total > 0 ? (count / total) * 100 : 0,
        }))
        .sort((a, b) => b.count - a.count)
        .slice(0, 10);

    return {
      totalCommands: total,
      undoStackDepth: this.past.length,
      redoStackDepth: this.future.length,
      totalBookmarks: this.bookmarks.size,
      totalBranches: this.branches.size,
      memoryUsage: this.estimateMemoryUsage(),
      averageCommandSize: total > 0 ? this.estimateMemoryUsage() / total : 0,
      mostUsedCommandTypes,
      oldestCommand: this.past[0]?.timestamp,
      newestCommand: this.present?.timestamp,
    };
  }

  /**
   * Get timeline events for visualization
   */
  getTimelineEvents(): TimelineEvent[] {
    const events: TimelineEvent[] = [];

    this.past.forEach((cmd) => {
      events.push({
        id: cmd.id || '',
        timestamp: cmd.timestamp || new Date(),
        type: 'command',
        data: cmd,
      });
    });

    if (this.present) {
      events.push({
        id: this.present.id || '',
        timestamp: this.present.timestamp || new Date(),
        type: 'command',
        data: this.present,
      });
    }

    this.bookmarks.forEach((bm) => {
      events.push({
        id: bm.id,
        timestamp: bm.timestamp,
        type: 'bookmark',
        data: bm,
      });
    });

    this.branches.forEach((branch) => {
      events.push({
        id: branch.id,
        timestamp: branch.createdAt,
        type: 'branch',
        data: branch,
      });
    });

    return events.sort((a, b) => a.timestamp.getTime() - b.timestamp.getTime());
  }

  // ==================== Persistence ====================

  /**
   * Save history to storage
   */
  async save(): Promise<void> {
    if (!this.persistence) {
      throw new Error('Persistence not enabled');
    }

    const state = this.getEnhancedState();
    await this.persistence.save(state);
  }

  /**
   * Load history from storage
   */
  async load(): Promise<void> {
    if (!this.persistence) {
      throw new Error('Persistence not enabled');
    }

    const state = await this.persistence.load();
    if (state) {
      this.restoreFromState(state);
    }
  }

  /**
   * Export history
   */
  async export(format: 'json' | 'binary' | 'compressed' = 'json'): Promise<Blob> {
    if (!this.persistence) {
      throw new Error('Persistence not enabled');
    }

    const state = this.getEnhancedState();
    return await this.persistence.export(state, format);
  }

  /**
   * Import history
   */
  async import(blob: Blob): Promise<void> {
    if (!this.persistence) {
      throw new Error('Persistence not enabled');
    }

    const state = await this.persistence.import(blob);
    this.restoreFromState(state);
    this.notifyListeners();
  }

  // ==================== Backward Compatible API ====================

  canUndo(): boolean {
    return this.past.length > 0;
  }

  canRedo(): boolean {
    return this.future.length > 0;
  }

  getUndoDescription(): string | null {
    return this.present?.description || null;
  }

  getRedoDescription(): string | null {
    return this.future[0]?.description || null;
  }

  clear(): void {
    this.clearHistory();
  }

  getState(): HistoryState {
    return {
      canUndo: this.canUndo(),
      canRedo: this.canRedo(),
      undoStack: [...this.past],
      redoStack: [...this.future],
    };
  }

  subscribe(listener: (state: EnhancedHistoryState) => void): () => void {
    this.listeners.add(listener);
    listener(this.getEnhancedState());

    return () => {
      this.listeners.delete(listener);
    };
  }

  getHistorySize(): { undo: number; redo: number } {
    return {
      undo: this.past.length,
      redo: this.future.length,
    };
  }

  // ==================== Private Methods ====================

  private generateCommandId(): string {
    return `cmd_${Date.now()}_${++this.commandIdCounter}`;
  }

  private generateBookmarkId(): string {
    return `bmk_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private generateBranchId(): string {
    return `br_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private async enforceHistoryLimit(): Promise<void> {
    const totalCommands = this.past.length + 1 + this.future.length;

    if (totalCommands > this.maxHistorySize) {
      const excess = totalCommands - this.maxHistorySize;
      this.past.splice(0, excess);
    }
  }

  private async autoSave(): Promise<void> {
    // Defer save to next tick
    Promise.resolve().then(async () => {
      try {
        await this.save();
      } catch (error) {
        console.error('Auto-save failed:', error);
      }
    });
  }

  private notifyListeners(): void {
    const state = this.getEnhancedState();
    this.listeners.forEach((listener) => {
      try {
        listener(state);
      } catch (error) {
        console.error('Error notifying history listener:', error);
      }
    });
  }

  private getEnhancedState(): EnhancedHistoryState {
    return {
      past: [...this.past],
      present: this.present,
      future: [...this.future],
      bookmarks: Array.from(this.bookmarks.values()),
      branches: Array.from(this.branches.values()),
      currentBranch: this.currentBranchId,
      metadata: {
        createdAt: this.past[0]?.timestamp || new Date(),
        lastModified: new Date(),
        totalCommands: this.past.length + (this.present ? 1 : 0) + this.future.length,
        estimatedSize: this.estimateMemoryUsage(),
        tags: [],
        version: '1.0.0',
      },
    };
  }

  private restoreFromState(state: EnhancedHistoryState): void {
    this.past = [...state.past];
    this.present = state.present;
    this.future = [...state.future];

    this.bookmarks = new Map(state.bookmarks.map((bm) => [bm.id, bm]));
    this.branches = new Map(state.branches.map((br) => [br.id, br]));
    this.currentBranchId = state.currentBranch;
  }

  private async jumpToState(stateId: string): Promise<void> {
    // Find the state in history
    const allCommands = [...this.past, this.present, ...this.future].filter(
      (cmd): cmd is Command => cmd !== null
    );

    const targetIndex = allCommands.findIndex((cmd) => cmd.id === stateId);
    if (targetIndex === -1) {
      throw new Error(`State not found: ${stateId}`);
    }

    // Undo to target
    while (this.present && this.present.id !== stateId) {
      await this.undo();
    }

    // Redo to target if needed
    if (this.past.length > 0 && targetIndex > this.past.length - 1) {
      const stepsToRedo = targetIndex - this.past.length + 1;
      for (let i = 0; i < stepsToRedo && this.future.length > 0; i++) {
        await this.redo();
      }
    }
  }

  private findStateById(stateId: string): Command | null {
    const allCommands = [...this.past, this.present, ...this.future].filter(
      (cmd): cmd is Command => cmd !== null
    );

    return allCommands.find((cmd) => cmd.id === stateId) || null;
  }

  private getCommandsUpTo(targetCommand: Command | null): Command[] {
    if (!targetCommand) return [];

    const allCommands = [...this.past, this.present, ...this.future].filter(
      (cmd): cmd is Command => cmd !== null
    );

    const targetIndex = allCommands.findIndex((cmd) => cmd.id === targetCommand.id);
    if (targetIndex === -1) return [];

    return allCommands.slice(0, targetIndex + 1);
  }

  private estimateMemoryUsage(): number {
    // Rough estimation in bytes
    const allCommands = [...this.past, this.present, ...this.future].filter(
      (cmd): cmd is Command => cmd !== null
    );

    return (
      allCommands.reduce((sum, cmd) => {
        return sum + JSON.stringify(cmd).length * 2; // UTF-16
      }, 0) +
      this.bookmarks.size * 500 +
      this.branches.size * 300
    );
  }

  private async mergeConsecutiveCommands(): Promise<void> {
    // Merge consecutive commands of the same type
    const merged: Command[] = [];
    let previousType = '';
    let batch: Command[] = [];

    const processBatch = () => {
      if (batch.length > 1) {
        merged.push(new BatchCommand([...batch], { mode: 'sequential' }));
      } else if (batch.length === 1) {
        merged.push(batch[0]);
      }
      batch = [];
    };

    for (const cmd of this.past) {
      const type = cmd.constructor.name;
      if (type === previousType && batch.length > 0) {
        batch.push(cmd);
      } else {
        processBatch();
        batch.push(cmd);
        previousType = type;
      }
    }

    processBatch();
    this.past = merged;
  }

  private async removeDuplicateCommands(): Promise<void> {
    // Remove duplicate commands based on description and timestamp
    const seen = new Set<string>();
    const filtered: Command[] = [];

    for (const cmd of this.past) {
      const key = `${cmd.description}_${cmd.timestamp?.getTime() || 0}`;
      if (!seen.has(key)) {
        seen.add(key);
        filtered.push(cmd);
      }
    }

    this.past = filtered;
  }
}
