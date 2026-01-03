/**
 * Plugin Marketplace Type Definitions
 */

export interface Plugin {
  id: string;
  name: string;
  slug: string;
  description: string;
  author: Author;
  version: string;
  latestVersion: string;
  categories: string[];
  tags: string[];
  license: string;
  homepage?: string;
  repository?: string;
  documentation?: string;
  screenshots: string[];
  videos: Video[];
  rating: Rating;
  downloads: number;
  pricing: Pricing;
  compatibility: Compatibility;
  manifest: PluginManifest;
  status: string;
  createdAt: string;
  updatedAt: string;
}

export interface Author {
  id: string;
  name: string;
  email?: string;
  avatar?: string;
  website?: string;
}

export interface Rating {
  average: number;
  count: number;
  distribution: Record<number, number>;
}

export interface Video {
  url: string;
  thumbnail: string;
  title: string;
  duration?: number;
}

export interface Pricing {
  pricingType: 'free' | 'paid' | 'freemium' | 'subscription';
  price?: number;
  currency?: string;
  trialAvailable: boolean;
  subscription?: SubscriptionInfo;
}

export interface SubscriptionInfo {
  monthly?: number;
  yearly?: number;
  currency: string;
}

export interface Compatibility {
  engineVersionMin: string;
  engineVersionMax?: string;
  platforms: string[];
  features: string[];
}

export interface PluginManifest {
  name: string;
  displayName: string;
  version: string;
  description: string;
  entryPoint: string;
  permissions: string[];
  resources: Resource[];
  commands: Command[];
  settings: Setting[];
}

export interface Resource {
  path: string;
  resourceType: string;
  description: string;
}

export interface Command {
  id: string;
  title: string;
  category: string;
  icon?: string;
  keybinding?: string;
}

export interface Setting {
  key: string;
  title: string;
  description: string;
  settingType: string;
  defaultValue: any;
  options?: SettingOption[];
}

export interface SettingOption {
  label: string;
  value: any;
}

export interface PluginVersion {
  id: string;
  version: string;
  changelog: string;
  downloadUrl: string;
  fileSize: number;
  sha256: string;
  status: string;
  publishedAt: string;
}

export interface Review {
  id: string;
  pluginId: string;
  user: ReviewUser;
  rating: number;
  title: string;
  content: string;
  helpfulCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface ReviewUser {
  id: string;
  name: string;
  avatar?: string;
}

export interface Category {
  id: string;
  name: string;
  slug: string;
  description?: string;
  icon?: string;
  parentId?: string;
  pluginCount: number;
  displayOrder: number;
}

export interface User {
  id: string;
  email: string;
  username: string;
  avatar?: string;
  website?: string;
  bio?: string;
  role: 'user' | 'developer' | 'admin';
  createdAt: string;
}

export interface SearchFilters {
  query?: string;
  categories?: string[];
  tags?: string[];
  pricingType?: PricingType;
  minRating?: number;
  platforms?: string[];
  sortBy?: SortOption;
}

export type PricingType = 'free' | 'paid' | 'freemium' | 'subscription';
export type SortOption = 'relevance' | 'downloads' | 'rating' | 'updated' | 'name';

export interface SearchResults {
  plugins: Plugin[];
  total: number;
  page: number;
  limit: number;
}

export interface InstallResult {
  pluginId: string;
  version: string;
  path: string;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

export interface MarketplaceStats {
  totalPlugins: number;
  totalDownloads: number;
  activeDevelopers: number;
  categories: Record<string, number>;
}
