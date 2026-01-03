// Tauri API imports
import { invoke } from '@tauri-apps/api/core';
import type {
  AssetInfo,
  AssetPreview,
  AssetFilter,
  ViewMode,
  SortBy,
  SortOrder,
  ImportResult,
  FolderNode,
} from './types';

// Asset API functions
export const assetApi = {
  // List assets in a directory
  async listAssets(path: string): Promise<AssetInfo[]> {
    try {
      return await invoke<AssetInfo[]>('list_assets', { path });
    } catch (error) {
      console.error('Failed to list assets:', error);
      throw error;
    }
  },

  // Get asset preview
  async getAssetPreview(path: string): Promise<AssetPreview> {
    try {
      return await invoke<AssetPreview>('get_asset_preview', { path });
    } catch (error) {
      console.error('Failed to get asset preview:', error);
      throw error;
    }
  },

  // Import assets
  async importAssets(files: string[], dest: string): Promise<ImportResult[]> {
    try {
      return await invoke<ImportResult[]>('import_assets', { files, dest });
    } catch (error) {
      console.error('Failed to import assets:', error);
      throw error;
    }
  },

  // Delete asset
  async deleteAsset(path: string): Promise<void> {
    try {
      await invoke('delete_asset', { path });
    } catch (error) {
      console.error('Failed to delete asset:', error);
      throw error;
    }
  },

  // Rename asset
  async renameAsset(path: string, newName: string): Promise<void> {
    try {
      await invoke('rename_asset', { path, newName });
    } catch (error) {
      console.error('Failed to rename asset:', error);
      throw error;
    }
  },

  // Get asset dependencies
  async getAssetDependencies(path: string): Promise<string[]> {
    try {
      return await invoke<string[]>('get_asset_dependencies', { path });
    } catch (error) {
      console.error('Failed to get asset dependencies:', error);
      throw error;
    }
  },

  // Create folder
  async createFolder(path: string, name: string): Promise<string> {
    try {
      return await invoke<string>('create_folder', { path, name });
    } catch (error) {
      console.error('Failed to create folder:', error);
      throw error;
    }
  },

  // Get folder tree
  async getFolderTree(path: string): Promise<FolderNode[]> {
    try {
      return await invoke<FolderNode[]>('get_folder_tree', { path });
    } catch (error) {
      console.error('Failed to get folder tree:', error);
      throw error;
    }
  },
};

// Utility functions

export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';

  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

export function formatDate(timestamp: string): string {
  try {
    const date = new Date(parseInt(timestamp) * 1000);
    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  } catch {
    return timestamp;
  }
}

export function getAssetIcon(type: string): string {
  const icons: Record<string, string> = {
    mesh: '📦',
    texture: '🎨',
    audio: '🎵',
    scene: '🎬',
    material: '✨',
    script: '📜',
    shader: '🔧',
    unknown: '📄',
  };

  return icons[type] || icons.unknown;
}

export function filterAssets(
  assets: AssetInfo[],
  filter: AssetFilter
): AssetInfo[] {
  let filtered = [...assets];

  // Filter by type
  if (filter.types.length > 0 && !filter.types.includes('unknown' as any)) {
    filtered = filtered.filter((asset) =>
      filter.types.includes(asset.type as any)
    );
  }

  // Filter by search
  if (filter.search.trim()) {
    const searchLower = filter.search.toLowerCase();
    filtered = filtered.filter(
      (asset) =>
        asset.name.toLowerCase().includes(searchLower) ||
        asset.path.toLowerCase().includes(searchLower)
    );
  }

  // Filter by tags (if implemented)
  if (filter.tags.length > 0) {
    // TODO: Implement tag filtering
  }

  return filtered;
}

export function sortAssets(
  assets: AssetInfo[],
  sortBy: SortBy,
  sortOrder: SortOrder
): AssetInfo[] {
  const sorted = [...assets];

  sorted.sort((a, b) => {
    let comparison = 0;

    switch (sortBy) {
      case 'name':
        comparison = a.name.localeCompare(b.name);
        break;
      case 'type':
        comparison = a.type.localeCompare(b.type);
        break;
      case 'date':
        comparison = parseInt(a.modified) - parseInt(b.modified);
        break;
      case 'size':
        comparison = a.size - b.size;
        break;
    }

    return sortOrder === 'asc' ? comparison : -comparison;
  });

  return sorted;
}

export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: ReturnType<typeof setTimeout> | null = null;

  return function executedFunction(...args: Parameters<T>) {
    const later = () => {
      timeout = null;
      func(...args);
    };

    if (timeout) {
      clearTimeout(timeout);
    }
    timeout = setTimeout(later, wait);
  };
}

export function extractFileExtension(filename: string): string {
  const parts = filename.split('.');
  return parts.length > 1 ? parts[parts.length - 1].toLowerCase() : '';
}

export function isImageFile(filename: string): boolean {
  const ext = extractFileExtension(filename);
  return ['png', 'jpg', 'jpeg', 'gif', 'bmp', 'webp', 'tga'].includes(ext);
}

export function isMeshFile(filename: string): boolean {
  const ext = extractFileExtension(filename);
  return ['fbx', 'obj', 'gltf', 'glb', 'ply'].includes(ext);
}

export function isAudioFile(filename: string): boolean {
  const ext = extractFileExtension(filename);
  return ['mp3', 'wav', 'ogg', 'flac', 'aac', 'm4a'].includes(ext);
}
