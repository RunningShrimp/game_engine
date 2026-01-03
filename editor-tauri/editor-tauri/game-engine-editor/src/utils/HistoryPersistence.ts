// History Persistence Implementation

import { Command } from '../types/commands';
import {
  HistoryState,
  HistoryPersistenceConfig,
  HistoryExportData,
  ExportFormat,
} from '../types/history';

/**
 * Compressed history data for efficient storage
 */
interface CompressedHistoryData {
  v: string; // version
  ts: number; // timestamp
  cmds: string; // compressed commands JSON
  bkmarks?: string; // compressed bookmarks
  brnches?: string; // compressed branches
  meta: string; // metadata
}

/**
 * Storage adapter interface
 */
interface StorageAdapter {
  get(key: string): Promise<string | null>;
  set(key: string, value: string): Promise<void>;
  delete(key: string): Promise<void>;
  clear(): Promise<void>;
}

/**
 * LocalStorage adapter
 */
class LocalStorageAdapter implements StorageAdapter {
  async get(key: string): Promise<string | null> {
    return localStorage.getItem(key);
  }

  async set(key: string, value: string): Promise<void> {
    localStorage.setItem(key, value);
  }

  async delete(key: string): Promise<void> {
    localStorage.removeItem(key);
  }

  async clear(): Promise<void> {
    localStorage.clear();
  }
}

/**
 * IndexedDB adapter for larger data
 */
class IndexedDBAdapter implements StorageAdapter {
  private dbName = 'HistoryDB';
  private storeName = 'history';
  private db: IDBDatabase | null = null;

  async init(): Promise<void> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(this.dbName, 1);

      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        this.db = request.result;
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;
        if (!db.objectStoreNames.contains(this.storeName)) {
          db.createObjectStore(this.storeName);
        }
      };
    });
  }

  async get(key: string): Promise<string | null> {
    if (!this.db) await this.init();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction(this.storeName, 'readonly');
      const store = transaction.objectStore(this.storeName);
      const request = store.get(key);

      request.onsuccess = () => resolve(request.result || null);
      request.onerror = () => reject(request.error);
    });
  }

  async set(key: string, value: string): Promise<void> {
    if (!this.db) await this.init();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction(this.storeName, 'readwrite');
      const store = transaction.objectStore(this.storeName);
      const request = store.put(value, key);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  async delete(key: string): Promise<void> {
    if (!this.db) await this.init();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction(this.storeName, 'readwrite');
      const store = transaction.objectStore(this.storeName);
      const request = store.delete(key);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  async clear(): Promise<void> {
    if (!this.db) await this.init();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction(this.storeName, 'readwrite');
      const store = transaction.objectStore(this.storeName);
      const request = store.clear();

      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }
}

/**
 * History Persistence Manager
 */
export class HistoryPersistence {
  private config: HistoryPersistenceConfig;
  private storage: StorageAdapter;
  private storageKey: string;
  private autoSaveTimer?: NodeJS.Timeout;

  constructor(
    config: Partial<HistoryPersistenceConfig> = {},
    storageKey: string = 'editor_history'
  ) {
    this.config = {
      enabled: config.enabled ?? true,
      storageType: config.storageType ?? 'localStorage',
      autoSave: config.autoSave ?? false,
      autoSaveInterval: config.autoSaveInterval ?? 60000, // 1 minute
      maxStorageSize: config.maxStorageSize ?? 10 * 1024 * 1024, // 10MB
      compress: config.compress ?? true,
      encrypt: config.encrypt ?? false,
      encryptionKey: config.encryptionKey,
    };

    this.storageKey = storageKey;
    this.storage = this.createStorageAdapter();
  }

  /**
   * Save history state to storage
   */
  async save(state: HistoryState): Promise<void> {
    if (!this.config.enabled) {
      return;
    }

    try {
      const data = await this.serialize(state);
      const serialized = JSON.stringify(data);

      // Check size limit
      if (serialized.length > this.config.maxStorageSize) {
        console.warn('History data exceeds storage limit, truncating old history');
        await this.saveTruncated(state);
        return;
      }

      await this.storage.set(this.storageKey, serialized);
    } catch (error) {
      console.error('Error saving history:', error);
      throw error;
    }
  }

  /**
   * Load history state from storage
   */
  async load(): Promise<HistoryState | null> {
    if (!this.config.enabled) {
      return null;
    }

    try {
      const serialized = await this.storage.get(this.storageKey);
      if (!serialized) {
        return null;
      }

      const data = JSON.parse(serialized);
      return this.deserialize(data);
    } catch (error) {
      console.error('Error loading history:', error);
      return null;
    }
  }

  /**
   * Export history to downloadable format
   */
  async export(state: HistoryState, format: ExportFormat = 'json'): Promise<Blob> {
    const exportData: HistoryExportData = {
      version: '1.0.0',
      exportedAt: new Date(),
      historyState: state,
      metadata: {
        totalSize: 0,
        compressed: format !== 'json',
        encrypted: this.config.encrypt,
      },
    };

    let data: string;

    switch (format) {
      case 'json':
        data = JSON.stringify(exportData, null, 2);
        break;

      case 'binary':
        // Simple binary encoding (in production, use proper compression)
        data = btoa(JSON.stringify(exportData));
        break;

      case 'compressed':
        data = await this.compressData(JSON.stringify(exportData));
        break;

      default:
        throw new Error(`Unsupported export format: ${format}`);
    }

    exportData.metadata.totalSize = data.length;

    const mimeType = format === 'json' ? 'application/json' : 'application/octet-stream';
    return new Blob([data], { type: mimeType });
  }

  /**
   * Import history from file
   */
  async import(blob: Blob): Promise<HistoryState> {
    const data = await this.readFileAsText(blob);

    try {
      const exportData: HistoryExportData = JSON.parse(data);
      return exportData.historyState;
    } catch (error) {
      // Try to decode as base64 binary
      try {
        const decoded = atob(data);
        const exportData: HistoryExportData = JSON.parse(decoded);
        return exportData.historyState;
      } catch {
        throw new Error('Invalid import file format');
      }
    }
  }

  /**
   * Clear persisted history
   */
  async clear(): Promise<void> {
    await this.storage.delete(this.storageKey);
  }

  /**
   * Enable auto-save
   */
  enableAutoSave(saveCallback: () => Promise<HistoryState>): void {
    if (this.autoSaveTimer) {
      clearInterval(this.autoSaveTimer);
    }

    this.autoSaveTimer = setInterval(async () => {
      try {
        const state = await saveCallback();
        await this.save(state);
      } catch (error) {
        console.error('Auto-save failed:', error);
      }
    }, this.config.autoSaveInterval);
  }

  /**
   * Disable auto-save
   */
  disableAutoSave(): void {
    if (this.autoSaveTimer) {
      clearInterval(this.autoSaveTimer);
      this.autoSaveTimer = undefined;
    }
  }

  /**
   * Get storage size in bytes
   */
  async getStorageSize(): Promise<number> {
    const serialized = await this.storage.get(this.storageKey);
    return serialized ? serialized.length : 0;
  }

  /**
   * Check if storage is available
   */
  async isAvailable(): Promise<boolean> {
    try {
      const testKey = '__storage_test__';
      await this.storage.set(testKey, 'test');
      await this.storage.delete(testKey);
      return true;
    } catch {
      return false;
    }
  }

  private createStorageAdapter(): StorageAdapter {
    switch (this.config.storageType) {
      case 'localStorage':
        return new LocalStorageAdapter();

      case 'indexedDB':
        return new IndexedDBAdapter();

      default:
        throw new Error(`Unsupported storage type: ${this.config.storageType}`);
    }
  }

  private async serialize(state: HistoryState): Promise<any> {
    if (this.config.compress) {
      return this.compressState(state);
    }

    return this.serializeState(state);
  }

  private deserialize(data: any): HistoryState {
    if (data.cmds) {
      return this.decompressState(data);
    }

    return data as HistoryState;
  }

  private serializeState(state: HistoryState): any {
    return {
      v: '1.0',
      ts: Date.now(),
      past: state.past.map(cmd => this.serializeCommand(cmd)),
      present: state.present ? this.serializeCommand(state.present) : null,
      future: state.future.map(cmd => this.serializeCommand(cmd)),
      bookmarks: state.bookmarks,
      branches: state.branches,
      currentBranch: state.currentBranch,
      metadata: state.metadata,
    };
  }

  private compressState(state: HistoryState): CompressedHistoryData {
    const serialized = this.serializeState(state);

    return {
      v: serialized.v,
      ts: serialized.ts,
      cmds: JSON.stringify(serialized.past),
      bkmarks: JSON.stringify(serialized.bookmarks),
      brnches: JSON.stringify(serialized.branches),
      meta: JSON.stringify(serialized.metadata),
    };
  }

  private decompressState(data: CompressedHistoryData): HistoryState {
    return {
      past: JSON.parse(data.cmds).map((cmd: any) => this.deserializeCommand(cmd)),
      present: null, // Will be restored separately
      future: [],
      bookmarks: data.bkmarks ? JSON.parse(data.bkmarks) : [],
      branches: data.brnches ? JSON.parse(data.brnches) : [],
      currentBranch: 'main',
      metadata: JSON.parse(data.meta),
    };
  }

  private serializeCommand(command: Command): any {
    return {
      type: command.constructor.name,
      description: command.description,
      id: command.id,
      timestamp: command.timestamp,
      metadata: command.metadata,
    };
  }

  private deserializeCommand(data: any): Command {
    // Commands should be deserialized by the history manager
    // which has access to the command registry
    return data as any;
  }

  private async compressData(data: string): Promise<string> {
    // Simple compression using base64 (in production, use proper compression library)
    return btoa(data);
  }

  private async saveTruncated(state: HistoryState): Promise<void> {
    // Keep only recent history (last 50 commands)
    const truncated: HistoryState = {
      ...state,
      past: state.past.slice(-50),
      future: state.future.slice(-50),
    };

    await this.save(truncated);
  }

  private readFileAsText(blob: Blob): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(reader.result as string);
      reader.onerror = () => reject(reader.error);
      reader.readAsText(blob);
    });
  }
}

/**
 * Create a history persistence instance
 */
export function createHistoryPersistence(
  config?: Partial<HistoryPersistenceConfig>,
  storageKey?: string
): HistoryPersistence {
  return new HistoryPersistence(config, storageKey);
}
