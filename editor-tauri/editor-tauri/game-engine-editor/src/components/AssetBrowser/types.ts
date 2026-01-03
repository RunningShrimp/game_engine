// Asset Browser Types

export type AssetType =
  | 'mesh'
  | 'texture'
  | 'audio'
  | 'scene'
  | 'material'
  | 'script'
  | 'shader'
  | 'unknown';

export type ViewMode = 'grid' | 'list' | 'large-icons' | 'small-icons';
export type SortBy = 'name' | 'type' | 'date' | 'size';
export type SortOrder = 'asc' | 'desc';

export interface AssetInfo {
  path: string;
  name: string;
  type: AssetType;
  size: number;
  created: string;
  modified: string;
  thumbnail?: string;
  metadata?: AssetMetadata;
}

export interface AssetMetadata {
  // Texture metadata
  width?: number;
  height?: number;
  textureFormat?: string;
  mipmaps?: number;

  // Mesh metadata
  vertices?: number;
  triangles?: number;
  meshFormat?: string;

  // Audio metadata
  duration?: number;
  sampleRate?: number;
  channels?: number;

  // Scene metadata
  entities?: number;
}

export interface AssetPreview {
  type: 'image' | 'model' | 'audio' | 'text' | 'binary';
  content: string; // Base64 or URL
  metadata?: AssetMetadata;
}

export interface FolderNode {
  path: string;
  name: string;
  children: FolderNode[];
  expanded: boolean;
  assetCount: number;
}

export interface AssetFilter {
  types: AssetType[];
  search: string;
  tags: string[];
}

export interface AssetSelection {
  assets: string[];
  lastSelected: string;
}

export interface ImportOptions {
  compressTextures: boolean;
  generateThumbnails: boolean;
  format?: string;
  quality?: number;
}

export interface ImportResult {
  source: string;
  destination: string;
  success: boolean;
  error?: string;
}
