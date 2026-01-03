import React from 'react';
import {
  Filter,
  Grid3x3,
  List,
  ZoomIn,
  ZoomOut,
  ChevronDown,
} from 'lucide-react';
import type { AssetType, ViewMode, SortBy, SortOrder } from './types';

interface FilterBarProps {
  selectedTypes: AssetType[];
  onTypeFilterChange: (types: AssetType[]) => void;
  viewMode: ViewMode;
  onViewModeChange: (mode: ViewMode) => void;
  sortBy: SortBy;
  onSortByChange: (sortBy: SortBy) => void;
  sortOrder: SortOrder;
  onSortOrderChange: (order: SortOrder) => void;
}

const ASSET_TYPES: { value: AssetType; label: string; icon: string }[] = [
  { value: 'mesh', label: 'Meshes', icon: '📦' },
  { value: 'texture', label: 'Textures', icon: '🎨' },
  { value: 'audio', label: 'Audio', icon: '🎵' },
  { value: 'scene', label: 'Scenes', icon: '🎬' },
  { value: 'material', label: 'Materials', icon: '✨' },
  { value: 'script', label: 'Scripts', icon: '📜' },
  { value: 'shader', label: 'Shaders', icon: '🔧' },
];

const SORT_OPTIONS: { value: SortBy; label: string }[] = [
  { value: 'name', label: 'Name' },
  { value: 'type', label: 'Type' },
  { value: 'date', label: 'Date Modified' },
  { value: 'size', label: 'Size' },
];

export function FilterBar({
  selectedTypes,
  onTypeFilterChange,
  viewMode,
  onViewModeChange,
  sortBy,
  onSortByChange,
  sortOrder,
  onSortOrderChange,
}: FilterBarProps) {
  const toggleType = (type: AssetType) => {
    if (selectedTypes.includes(type)) {
      onTypeFilterChange(selectedTypes.filter((t) => t !== type));
    } else {
      onTypeFilterChange([...selectedTypes, type]);
    }
  };

  const toggleSortOrder = () => {
    onSortOrderChange(sortOrder === 'asc' ? 'desc' : 'asc');
  };

  return (
    <div className="flex items-center gap-2 px-4 py-2 border-b border-slate-700">
      {/* Type Filter */}
      <div className="relative">
        <button className="flex items-center gap-2 px-3 py-1.5 bg-slate-800 border border-slate-700 rounded-lg text-sm hover:bg-slate-700 transition-colors">
          <Filter className="w-4 h-4" />
          <span>Filter</span>
          {selectedTypes.length > 0 && (
            <span className="px-1.5 py-0.5 text-xs bg-blue-500 rounded">
              {selectedTypes.length}
            </span>
          )}
          <ChevronDown className="w-4 h-4" />
        </button>

        {/* Type Filter Dropdown */}
        <div className="absolute top-full left-0 mt-1 w-48 bg-slate-800 border border-slate-700 rounded-lg shadow-xl z-50">
          <div className="p-2">
            <button
              className={`w-full text-left px-3 py-2 rounded text-sm hover:bg-slate-700 transition-colors ${
                selectedTypes.length === 0 ? 'bg-blue-600' : ''
              }`}
              onClick={() => onTypeFilterChange([])}
            >
              All Types
            </button>
            {ASSET_TYPES.map((type) => (
              <button
                key={type.value}
                className={`w-full flex items-center gap-2 px-3 py-2 rounded text-sm hover:bg-slate-700 transition-colors ${
                  selectedTypes.includes(type.value) ? 'bg-blue-600' : ''
                }`}
                onClick={() => toggleType(type.value)}
              >
                <span>{type.icon}</span>
                <span>{type.label}</span>
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Sort */}
      <div className="relative">
        <select
          value={sortBy}
          onChange={(e) => onSortByChange(e.target.value as SortBy)}
          className="appearance-none flex items-center gap-2 px-3 py-1.5 bg-slate-800 border border-slate-700 rounded-lg text-sm hover:bg-slate-700 transition-colors cursor-pointer"
        >
          {SORT_OPTIONS.map((option) => (
            <option key={option.value} value={option.value}>
              Sort: {option.label}
            </option>
          ))}
        </select>
      </div>

      {/* Sort Order Toggle */}
      <button
        onClick={toggleSortOrder}
        className="p-1.5 bg-slate-800 border border-slate-700 rounded-lg hover:bg-slate-700 transition-colors"
        title={sortOrder === 'asc' ? 'Ascending' : 'Descending'}
      >
        {sortOrder === 'asc' ? '↑' : '↓'}
      </button>

      <div className="flex-1" />

      {/* View Mode */}
      <div className="flex items-center gap-1 bg-slate-800 border border-slate-700 rounded-lg p-1">
        <button
          className={`p-1.5 rounded transition-colors ${
            viewMode === 'grid' ? 'bg-blue-600' : 'hover:bg-slate-700'
          }`}
          onClick={() => onViewModeChange('grid')}
          title="Grid View"
        >
          <Grid3x3 className="w-4 h-4" />
        </button>
        <button
          className={`p-1.5 rounded transition-colors ${
            viewMode === 'list' ? 'bg-blue-600' : 'hover:bg-slate-700'
          }`}
          onClick={() => onViewModeChange('list')}
          title="List View"
        >
          <List className="w-4 h-4" />
        </button>
        <button
          className={`p-1.5 rounded transition-colors ${
            viewMode === 'large-icons' ? 'bg-blue-600' : 'hover:bg-slate-700'
          }`}
          onClick={() => onViewModeChange('large-icons')}
          title="Large Icons"
        >
          <ZoomIn className="w-4 h-4" />
        </button>
        <button
          className={`p-1.5 rounded transition-colors ${
            viewMode === 'small-icons' ? 'bg-blue-600' : 'hover:bg-slate-700'
          }`}
          onClick={() => onViewModeChange('small-icons')}
          title="Small Icons"
        >
          <ZoomOut className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}
