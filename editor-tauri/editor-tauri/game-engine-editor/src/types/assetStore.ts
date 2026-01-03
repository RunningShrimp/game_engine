// Asset Store Types
// 资源商店类型定义

export enum AssetType {
  Model3D = 'model_3d',
  Material = 'material',
  Texture = 'texture',
  Audio = 'audio',
  Script = 'script',
  Shader = 'shader',
  Scene = 'scene',
  Template = 'template',
  Plugin = 'plugin',
}

export enum AssetCategory {
  Characters = 'characters',
  Environments = 'environments',
  Props = 'props',
  Vehicles = 'vehicles',
  Weapons = 'weapons',
  Effects = 'effects',
  UI = 'ui',
  Tools = 'tools',
  Architecture = 'architecture',
  Nature = 'nature',
}

export enum LicenseType {
  MIT = 'mit',
  Apache2 = 'apache2',
  GPL = 'gpl',
  CC0 = 'cc0',
  CC_BY = 'cc_by',
  CC_BY_SA = 'cc_by_sa',
  CC_BY_NC = 'cc_by_nc',
  Proprietary = 'proprietary',
  Custom = 'custom',
}

export enum PricingType {
  Free = 'free',
  Paid = 'paid',
  Subscription = 'subscription',
}

export enum SortField {
  CreatedAt = 'created_at',
  UpdatedAt = 'updated_at',
  Name = 'name',
  Rating = 'rating',
  Downloads = 'downloads',
  Price = 'price',
}

export enum SortOrder {
  Asc = 'asc',
  Desc = 'desc',
}

export interface AssetPricing {
  type: PricingType;
  priceUsd?: number;
  discountPercent?: number;
  monthlyUsd?: number;
  yearlyUsd?: number;
}

export interface AssetMetadata {
  id: string;
  name: string;
  description: string;
  assetType: AssetType;
  category: AssetCategory;
  version: string;
  author: string;
  tags: string[];
  license: LicenseType;
  pricing: AssetPricing;
  fileSizeBytes: number;
  createdAt: string;
  updatedAt: string;
  rating: number;
  downloadCount: number;
  previewUrls: string[];
  dependencies: string[];
  compatibility: string[];
  minimumEngineVersion: string;
}

export interface AssetFile {
  filename: string;
  fileType: string;
  sizeBytes: number;
  url: string;
  hash: string;
}

export interface PreviewData {
  previewType: string;
  url: string;
  thumbnailUrl: string;
  width: number;
  height: number;
}

export interface AssetData {
  metadata: AssetMetadata;
  files: AssetFile[];
  previews: PreviewData[];
}

export interface SearchQuery {
  query?: string;
  assetType?: AssetType;
  category?: AssetCategory;
  tags: string[];
  page?: number;
  perPage?: number;
  sortBy?: SortField;
  sortOrder?: SortOrder;
}

export interface SearchResult {
  assets: AssetMetadata[];
  totalCount: number;
  page: number;
  perPage: number;
  totalPages: number;
}

export interface UserFavorite {
  userId: string;
  assetId: string;
  createdAt: string;
}

export interface DownloadHistory {
  userId: string;
  assetId: string;
  assetVersion: string;
  downloadedAt: string;
  importPath?: string;
}
