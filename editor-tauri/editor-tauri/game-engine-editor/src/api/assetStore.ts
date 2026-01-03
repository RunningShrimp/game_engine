// Asset Store API
// 资源商店 API 客户端

import { invoke } from '@tauri-apps/api/core';
import type {
  AssetData,
  AssetMetadata,
  DownloadHistory,
  PreviewData,
  SearchQuery,
  SearchResult,
  UserFavorite,
} from '../types/assetStore';

export class AssetStoreAPI {
  /**
   * 搜索资源
   */
  static async searchAssets(query: SearchQuery): Promise<SearchResult> {
    return await invoke<SearchResult>('search_assets', {
      query: query.query,
      assetType: query.assetType,
      category: query.category,
      tags: query.tags,
      page: query.page,
      perPage: query.perPage,
    });
  }

  /**
   * 下载资源
   */
  static async downloadAsset(id: string): Promise<AssetData> {
    return await invoke<AssetData>('download_asset', { id });
  }

  /**
   * 导入资源到项目
   */
  static async importAsset(assetId: string, projectPath: string): Promise<string> {
    return await invoke<string>('import_asset', {
      assetId,
      projectPath,
    });
  }

  /**
   * 获取预览
   */
  static async getPreview(id: string): Promise<PreviewData> {
    return await invoke<PreviewData>('get_preview', { id });
  }

  /**
   * 获取资源详情
   */
  static async getAssetDetails(id: string): Promise<AssetMetadata> {
    return await invoke<AssetMetadata>('get_asset_details', { id });
  }

  /**
   * 添加到收藏
   */
  static async addFavorite(userId: string, assetId: string): Promise<void> {
    await invoke('add_favorite', { userId, assetId });
  }

  /**
   * 移除收藏
   */
  static async removeFavorite(userId: string, assetId: string): Promise<void> {
    await invoke('remove_favorite', { userId, assetId });
  }

  /**
   * 获取收藏列表
   */
  static async getFavorites(userId: string): Promise<AssetMetadata[]> {
    return await invoke<AssetMetadata[]>('get_favorites', { userId });
  }

  /**
   * 获取下载历史
   */
  static async getDownloadHistory(userId: string): Promise<DownloadHistory[]> {
    return await invoke<DownloadHistory[]>('get_download_history', { userId });
  }

  /**
   * 获取类别列表
   */
  static async getCategories(): Promise<string[]> {
    return await invoke<string[]>('get_categories');
  }

  /**
   * 获取资源类型列表
   */
  static async getAssetTypes(): Promise<string[]> {
    return await invoke<string[]>('get_asset_types');
  }
}
