// Asset Store Panel
// 资源商店主面板

import React, { useState, useEffect } from 'react';
import {
  Search,
  Filter,
  Grid3X3,
  List,
  Heart,
  Download,
  Star,
  Tag,
  X,
} from 'lucide-react';
import { AssetStoreAPI } from '../../api/assetStore';
import { AssetMetadata, AssetType, AssetCategory, SortField, SortOrder } from '../../types/assetStore';
import AssetCard from './AssetCard';
import AssetDetails from './AssetDetails';
import FilterPanel from './FilterPanel';
import LoadingSpinner from '../loading/LoadingSpinner';

interface AssetStorePanelProps {
  projectPath: string;
  userId: string;
}

type ViewMode = 'grid' | 'list';

const AssetStorePanel: React.FC<AssetStorePanelProps> = ({ projectPath, userId }) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [assets, setAssets] = useState<AssetMetadata[]>([]);
  const [filteredAssets, setFilteredAssets] = useState<AssetMetadata[]>([]);
  const [selectedAsset, setSelectedAsset] = useState<AssetMetadata | null>(null);
  const [loading, setLoading] = useState(false);
  const [viewMode, setViewMode] = useState<ViewMode>('grid');
  const [showFilters, setShowFilters] = useState(false);
  const [favorites, setFavorites] = useState<Set<string>>(new Set());
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [totalCount, setTotalCount] = useState(0);

  // 过滤器状态
  const [selectedType, setSelectedType] = useState<AssetType | null>(null);
  const [selectedCategory, setSelectedCategory] = useState<AssetCategory | null>(null);
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [sortBy, setSortBy] = useState<SortField>(SortField.CreatedAt);
  const [sortOrder, setSortOrder] = useState<SortOrder>(SortOrder.Desc);

  // 加载收藏列表
  useEffect(() => {
    loadFavorites();
  }, [userId]);

  // 搜索资源
  useEffect(() => {
    searchAssets();
  }, [selectedType, selectedCategory, sortBy, sortOrder, currentPage]);

  // 过滤资源
  useEffect(() => {
    filterAssets();
  }, [assets, searchQuery, selectedTags]);

  const loadFavorites = async () => {
    try {
      const favs = await AssetStoreAPI.getFavorites(userId);
      const favIds = new Set(favs.map((a) => a.id));
      setFavorites(favIds);
    } catch (error) {
      console.error('Failed to load favorites:', error);
    }
  };

  const searchAssets = async () => {
    setLoading(true);
    try {
      const result = await AssetStoreAPI.searchAssets({
        query: searchQuery || undefined,
        assetType: selectedType || undefined,
        category: selectedCategory || undefined,
        tags: selectedTags,
        page: currentPage,
        perPage: 20,
        sortBy,
        sortOrder,
      });

      setAssets(result.assets);
      setTotalPages(result.totalPages);
      setTotalCount(result.totalCount);
    } catch (error) {
      console.error('Search failed:', error);
    } finally {
      setLoading(false);
    }
  };

  const filterAssets = () => {
    let filtered = [...assets];

    // 搜索词过滤
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (asset) =>
          asset.name.toLowerCase().includes(query) ||
          asset.description.toLowerCase().includes(query) ||
          asset.tags.some((tag) => tag.toLowerCase().includes(query))
      );
    }

    // 标签过滤
    if (selectedTags.length > 0) {
      filtered = filtered.filter((asset) =>
        selectedTags.every((tag) => asset.tags.includes(tag))
      );
    }

    setFilteredAssets(filtered);
  };

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    setCurrentPage(1);
    searchAssets();
  };

  const handleAssetClick = (asset: AssetMetadata) => {
    setSelectedAsset(asset);
  };

  const handleDownload = async (assetId: string) => {
    try {
      const path = await AssetStoreAPI.importAsset(assetId, projectPath);
      alert(`Asset imported to: ${path}`);
    } catch (error) {
      console.error('Download failed:', error);
      alert('Failed to download asset');
    }
  };

  const handleToggleFavorite = async (assetId: string) => {
    try {
      if (favorites.has(assetId)) {
        await AssetStoreAPI.removeFavorite(userId, assetId);
        setFavorites((prev) => {
          const next = new Set(prev);
          next.delete(assetId);
          return next;
        });
      } else {
        await AssetStoreAPI.addFavorite(userId, assetId);
        setFavorites((prev) => new Set(prev).add(assetId));
      }
    } catch (error) {
      console.error('Failed to toggle favorite:', error);
    }
  };

  const handleClearFilters = () => {
    setSelectedType(null);
    setSelectedCategory(null);
    setSelectedTags([]);
    setSearchQuery('');
    setCurrentPage(1);
  };

  return (
    <div className="flex h-full bg-gray-900 text-white">
      {/* 主内容区域 */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* 顶部工具栏 */}
        <div className="p-4 border-b border-gray-700">
          <form onSubmit={handleSearch} className="flex gap-2 mb-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search assets..."
                className="w-full bg-gray-800 border border-gray-700 rounded-lg pl-10 pr-4 py-2 focus:outline-none focus:border-blue-500"
              />
            </div>
            <button
              type="button"
              onClick={() => setShowFilters(!showFilters)}
              className={`px-4 py-2 rounded-lg flex items-center gap-2 ${
                showFilters ? 'bg-blue-600' : 'bg-gray-700 hover:bg-gray-600'
              }`}
            >
              <Filter className="w-5 h-5" />
              Filters
            </button>
            <div className="flex bg-gray-700 rounded-lg">
              <button
                type="button"
                onClick={() => setViewMode('grid')}
                className={`px-3 py-2 ${
                  viewMode === 'grid' ? 'bg-blue-600' : 'hover:bg-gray-600'
                }`}
              >
                <Grid3X3 className="w-5 h-5" />
              </button>
              <button
                type="button"
                onClick={() => setViewMode('list')}
                className={`px-3 py-2 ${
                  viewMode === 'list' ? 'bg-blue-600' : 'hover:bg-gray-600'
                }`}
              >
                <List className="w-5 h-5" />
              </button>
            </div>
          </form>

          {/* 活跃过滤器标签 */}
          {(selectedType || selectedCategory || selectedTags.length > 0) && (
            <div className="flex items-center gap-2 flex-wrap">
              <span className="text-gray-400 text-sm">Active filters:</span>
              {selectedType && (
                <span className="bg-blue-600 px-2 py-1 rounded text-sm flex items-center gap-1">
                  {selectedType}
                  <X
                    className="w-4 h-4 cursor-pointer"
                    onClick={() => setSelectedType(null)}
                  />
                </span>
              )}
              {selectedCategory && (
                <span className="bg-green-600 px-2 py-1 rounded text-sm flex items-center gap-1">
                  {selectedCategory}
                  <X
                    className="w-4 h-4 cursor-pointer"
                    onClick={() => setSelectedCategory(null)}
                  />
                </span>
              )}
              {selectedTags.map((tag) => (
                <span key={tag} className="bg-purple-600 px-2 py-1 rounded text-sm flex items-center gap-1">
                  <Tag className="w-3 h-3" />
                  {tag}
                  <X
                    className="w-4 h-4 cursor-pointer"
                    onClick={() =>
                      setSelectedTags((prev) => prev.filter((t) => t !== tag))
                    }
                  />
                </span>
              ))}
              <button
                onClick={handleClearFilters}
                className="text-red-400 hover:text-red-300 text-sm"
              >
                Clear all
              </button>
            </div>
          )}

          {/* 结果统计 */}
          <div className="mt-2 text-sm text-gray-400">
            Showing {filteredAssets.length} of {totalCount} assets
          </div>
        </div>

        {/* 资源网格/列表 */}
        <div className="flex-1 overflow-y-auto p-4">
          {loading ? (
            <div className="flex items-center justify-center h-full">
              <LoadingSpinner />
            </div>
          ) : filteredAssets.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-gray-400">
              <Search className="w-16 h-16 mb-4" />
              <p className="text-xl mb-2">No assets found</p>
              <p className="text-sm">Try adjusting your search or filters</p>
            </div>
          ) : viewMode === 'grid' ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
              {filteredAssets.map((asset) => (
                <AssetCard
                  key={asset.id}
                  asset={asset}
                  isFavorite={favorites.has(asset.id)}
                  onClick={() => handleAssetClick(asset)}
                  onDownload={handleDownload}
                  onToggleFavorite={handleToggleFavorite}
                />
              ))}
            </div>
          ) : (
            <div className="space-y-2">
              {filteredAssets.map((asset) => (
                <AssetCard
                  key={asset.id}
                  asset={asset}
                  isFavorite={favorites.has(asset.id)}
                  viewMode="list"
                  onClick={() => handleAssetClick(asset)}
                  onDownload={handleDownload}
                  onToggleFavorite={handleToggleFavorite}
                />
              ))}
            </div>
          )}

          {/* 分页 */}
          {totalPages > 1 && (
            <div className="mt-4 flex justify-center items-center gap-2">
              <button
                onClick={() => setCurrentPage((prev) => Math.max(1, prev - 1))}
                disabled={currentPage === 1}
                className="px-4 py-2 bg-gray-700 rounded disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-600"
              >
                Previous
              </button>
              <span className="text-gray-400">
                Page {currentPage} of {totalPages}
              </span>
              <button
                onClick={() => setCurrentPage((prev) => Math.min(totalPages, prev + 1))}
                disabled={currentPage === totalPages}
                className="px-4 py-2 bg-gray-700 rounded disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-600"
              >
                Next
              </button>
            </div>
          )}
        </div>
      </div>

      {/* 侧边过滤器 */}
      {showFilters && (
        <FilterPanel
          selectedType={selectedType}
          selectedCategory={selectedCategory}
          selectedTags={selectedTags}
          sortBy={sortBy}
          sortOrder={sortOrder}
          onTypeChange={setSelectedType}
          onCategoryChange={setSelectedCategory}
          onTagsChange={setSelectedTags}
          onSortByChange={setSortBy}
          onSortOrderChange={setSortOrder}
          onClose={() => setShowFilters(false)}
        />
      )}

      {/* 资源详情面板 */}
      {selectedAsset && (
        <AssetDetails
          asset={selectedAsset}
          isFavorite={favorites.has(selectedAsset.id)}
          onClose={() => setSelectedAsset(null)}
          onDownload={handleDownload}
          onToggleFavorite={handleToggleFavorite}
        />
      )}
    </div>
  );
};

export default AssetStorePanel;
