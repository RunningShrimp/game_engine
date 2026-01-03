// Filter Panel
// 过滤器侧边栏

import React, { useState, useEffect } from 'react';
import { X, ArrowUpDown } from 'lucide-react';
import { AssetType, AssetCategory, SortField, SortOrder } from '../../types/assetStore';
import { AssetStoreAPI } from '../../api/assetStore';

interface FilterPanelProps {
  selectedType: AssetType | null;
  selectedCategory: AssetCategory | null;
  selectedTags: string[];
  sortBy: SortField;
  sortOrder: SortOrder;
  onTypeChange: (type: AssetType | null) => void;
  onCategoryChange: (category: AssetCategory | null) => void;
  onTagsChange: (tags: string[]) => void;
  onSortByChange: (field: SortField) => void;
  onSortOrderChange: (order: SortOrder) => void;
  onClose: () => void;
}

const FilterPanel: React.FC<FilterPanelProps> = ({
  selectedType,
  selectedCategory,
  selectedTags,
  sortBy,
  sortOrder,
  onTypeChange,
  onCategoryChange,
  onTagsChange,
  onSortByChange,
  onSortOrderChange,
  onClose,
}) => {
  const [availableTypes, setAvailableTypes] = useState<string[]>([]);
  const [availableCategories, setAvailableCategories] = useState<string[]>([]);
  const [popularTags, setPopularTags] = useState<string[]>([]);
  const [tagInput, setTagInput] = useState('');

  useEffect(() => {
    loadFilters();
  }, []);

  const loadFilters = async () => {
    try {
      const [types, categories] = await Promise.all([
        AssetStoreAPI.getAssetTypes(),
        AssetStoreAPI.getCategories(),
      ]);
      setAvailableTypes(types);
      setAvailableCategories(categories);

      // 模拟热门标签
      setPopularTags([
        'low-poly',
        'pbr',
        'animated',
        'sci-fi',
        'medieval',
        'modern',
        'fantasy',
        'realistic',
        'stylized',
        'hand-painted',
      ]);
    } catch (error) {
      console.error('Failed to load filters:', error);
    }
  };

  const handleAddTag = () => {
    if (tagInput && !selectedTags.includes(tagInput)) {
      onTagsChange([...selectedTags, tagInput]);
      setTagInput('');
    }
  };

  const handleRemoveTag = (tag: string) => {
    onTagsChange(selectedTags.filter((t) => t !== tag));
  };

  return (
    <div className="w-80 bg-gray-800 border-l border-gray-700 overflow-y-auto">
      {/* 头部 */}
      <div className="sticky top-0 bg-gray-800 border-b border-gray-700 p-4 flex justify-between items-center">
        <h2 className="text-lg font-bold">Filters</h2>
        <button
          onClick={onClose}
          className="p-1 hover:bg-gray-700 rounded"
        >
          <X className="w-5 h-5" />
        </button>
      </div>

      {/* 内容 */}
      <div className="p-4 space-y-6">
        {/* 资源类型 */}
        <div>
          <h3 className="font-semibold mb-3">Asset Type</h3>
          <div className="space-y-2">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="type"
                checked={selectedType === null}
                onChange={() => onTypeChange(null)}
                className="w-4 h-4"
              />
              <span>All Types</span>
            </label>
            {Object.values(AssetType).map((type) => (
              <label key={type} className="flex items-center gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="type"
                  checked={selectedType === type}
                  onChange={() => onTypeChange(type)}
                  className="w-4 h-4"
                />
                <span className="capitalize">{type.replace('_', ' ')}</span>
              </label>
            ))}
          </div>
        </div>

        {/* 类别 */}
        <div>
          <h3 className="font-semibold mb-3">Category</h3>
          <div className="space-y-2">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="category"
                checked={selectedCategory === null}
                onChange={() => onCategoryChange(null)}
                className="w-4 h-4"
              />
              <span>All Categories</span>
            </label>
            {Object.values(AssetCategory).map((category) => (
              <label key={category} className="flex items-center gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="category"
                  checked={selectedCategory === category}
                  onChange={() => onCategoryChange(category)}
                  className="w-4 h-4"
                />
                <span className="capitalize">{category}</span>
              </label>
            ))}
          </div>
        </div>

        {/* 标签 */}
        <div>
          <h3 className="font-semibold mb-3">Tags</h3>
          <div className="flex gap-2 mb-2">
            <input
              type="text"
              value={tagInput}
              onChange={(e) => setTagInput(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && handleAddTag()}
              placeholder="Add tag..."
              className="flex-1 bg-gray-700 border border-gray-600 rounded px-3 py-2 text-sm focus:outline-none focus:border-blue-500"
            />
            <button
              onClick={handleAddTag}
              className="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded text-sm"
            >
              Add
            </button>
          </div>

          {/* 已选标签 */}
          {selectedTags.length > 0 && (
            <div className="flex flex-wrap gap-2 mb-3">
              {selectedTags.map((tag) => (
                <span
                  key={tag}
                  className="bg-purple-600 px-2 py-1 rounded text-sm flex items-center gap-1"
                >
                  {tag}
                  <X
                    className="w-3 h-3 cursor-pointer"
                    onClick={() => handleRemoveTag(tag)}
                  />
                </span>
              ))}
            </div>
          )}

          {/* 热门标签 */}
          <div>
            <p className="text-gray-400 text-sm mb-2">Popular tags:</p>
            <div className="flex flex-wrap gap-2">
              {popularTags.map((tag) => (
                <button
                  key={tag}
                  onClick={() => {
                    if (!selectedTags.includes(tag)) {
                      onTagsChange([...selectedTags, tag]);
                    }
                  }}
                  className={`px-2 py-1 rounded text-sm ${
                    selectedTags.includes(tag)
                      ? 'bg-purple-600'
                      : 'bg-gray-700 hover:bg-gray-600'
                  }`}
                >
                  {tag}
                </button>
              ))}
            </div>
          </div>
        </div>

        {/* 排序 */}
        <div>
          <h3 className="font-semibold mb-3 flex items-center gap-2">
            <ArrowUpDown className="w-4 h-4" />
            Sort By
          </h3>
          <div className="space-y-2">
            <select
              value={sortBy}
              onChange={(e) => onSortByChange(e.target.value as SortField)}
              className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 focus:outline-none focus:border-blue-500"
            >
              <option value={SortField.CreatedAt}>Date Created</option>
              <option value={SortField.UpdatedAt}>Date Updated</option>
              <option value={SortField.Name}>Name</option>
              <option value={SortField.Rating}>Rating</option>
              <option value={SortField.Downloads}>Downloads</option>
            </select>
            <select
              value={sortOrder}
              onChange={(e) => onSortOrderChange(e.target.value as SortOrder)}
              className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 focus:outline-none focus:border-blue-500"
            >
              <option value={SortOrder.Desc}>Descending</option>
              <option value={SortOrder.Asc}>Ascending</option>
            </select>
          </div>
        </div>
      </div>
    </div>
  );
};

export default FilterPanel;
