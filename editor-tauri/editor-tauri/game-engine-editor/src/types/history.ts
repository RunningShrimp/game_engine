// Advanced History Types for Enhanced Undo/Redo System

import { Command } from './commands';

/**
 * Represents a complete history state with branching support
 */
export interface HistoryState {
  past: Command[];
  present: Command | null;
  future: Command[];
  bookmarks: HistoryBookmark[];
  branches: HistoryBranch[];
  currentBranch: string;
  metadata: HistoryMetadata;
}

/**
 * A bookmark in history that allows quick navigation to specific points
 */
export interface HistoryBookmark {
  id: string;
  name: string;
  description?: string;
  timestamp: Date;
  stateId: string;
  tags: string[];
  color?: string;
  author?: string;
}

/**
 * A branch in history allowing parallel timelines
 */
export interface HistoryBranch {
  id: string;
  name: string;
  parentId: string | null;
  createdAt: Date;
  stateId: string;
  isActive: boolean;
}

/**
 * Metadata about the history state
 */
export interface HistoryMetadata {
  createdAt: Date;
  lastModified: Date;
  totalCommands: number;
  estimatedSize: number;
  tags: string[];
  version: string;
}

/**
 * Difference between two history states
 */
export interface HistoryDiff {
  before: string;
  after: string;
  added: Command[];
  removed: Command[];
  modified: CommandDiff[];
  timestamp: Date;
}

/**
 * Individual command difference
 */
export interface CommandDiff {
  command: Command;
  changeType: 'added' | 'removed' | 'modified';
  previousState?: any;
  newState?: any;
}

/**
 * Statistics about history usage
 */
export interface HistoryStatistics {
  totalCommands: number;
  undoStackDepth: number;
  redoStackDepth: number;
  totalBookmarks: number;
  totalBranches: number;
  memoryUsage: number;
  averageCommandSize: number;
  mostUsedCommandTypes: CommandTypeStats[];
  oldestCommand?: Date;
  newestCommand?: Date;
}

/**
 * Statistics for command types
 */
export interface CommandTypeStats {
  type: string;
  count: number;
  percentage: number;
}

/**
 * Configuration for history persistence
 */
export interface HistoryPersistenceConfig {
  enabled: boolean;
  storageType: 'localStorage' | 'indexedDB' | 'custom';
  autoSave: boolean;
  autoSaveInterval: number; // milliseconds
  maxStorageSize: number; // bytes
  compress: boolean;
  encrypt: boolean;
  encryptionKey?: string;
}

/**
 * Result of a search operation
 */
export interface HistorySearchResult {
  commands: Command[];
  bookmarks: HistoryBookmark[];
  totalResults: number;
  searchTime: number;
}

/**
 * Options for filtering history
 */
export interface HistoryFilterOptions {
  dateRange?: {
    start: Date;
    end: Date;
  };
  commandTypes?: string[];
  tags?: string[];
  author?: string;
  textQuery?: string;
}

/**
 * Options for transaction execution
 */
export interface TransactionOptions {
  rollbackOnFailure: boolean;
  stopOnFirstError: boolean;
  timeout?: number; // milliseconds
  metadata?: Record<string, any>;
}

/**
 * Options for batch execution
 */
export interface BatchOptions {
  mode: 'sequential' | 'parallel';
  maxConcurrency?: number;
  continueOnError?: boolean;
}

/**
 * Options for history optimization
 */
export interface OptimizationOptions {
  mergeConsecutiveCommands: boolean;
  removeDuplicateCommands: boolean;
  compressOldCommands: boolean;
  keepBookmarks: boolean;
  ageThreshold?: number; // days
}

/**
 * Timeline event for visualization
 */
export interface TimelineEvent {
  id: string;
  timestamp: Date;
  type: 'command' | 'bookmark' | 'branch';
  data: Command | HistoryBookmark | HistoryBranch;
  metadata?: Record<string, any>;
}

/**
 * Export format options
 */
export type ExportFormat = 'json' | 'binary' | 'compressed';

/**
 * Import/Export data structure
 */
export interface HistoryExportData {
  version: string;
  exportedAt: Date;
  historyState: HistoryState;
  metadata: {
    totalSize: number;
    compressed: boolean;
    encrypted: boolean;
  };
}
