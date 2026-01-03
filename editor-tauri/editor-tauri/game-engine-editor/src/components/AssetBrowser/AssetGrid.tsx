import React, { useState, useRef, useCallback } from 'react';
import { getAssetIcon, formatFileSize } from './utils';
import type { AssetInfo, ViewMode } from './types';

interface AssetGridProps {
  assets: AssetInfo[];
  selectedAssets: string[];
  onAssetSelect: (assetPaths: string[]) => void;
  onAssetDoubleClick: (assetPath: string) => void;
  viewMode: ViewMode;
}

export function AssetGrid({
  assets,
  selectedAssets,
  onAssetSelect,
  onAssetDoubleClick,
  viewMode,
}: AssetGridProps) {
  const [draggedAsset, setDraggedAsset] = useState<string | null>(null);
  const lastSelectedRef = useRef<string | null>(null);

  // Calculate grid size based on view mode
  const getItemSize = () => {
    switch (viewMode) {
      case 'large-icons':
        return 'w-48 h-48';
      case 'small-icons':
        return 'w-24 h-24';
      case 'grid':
      default:
        return 'w-32 h-32';
    }
  };

  const getThumbnailSize = () => {
    switch (viewMode) {
      case 'large-icons':
        return 'w-40 h-40';
      case 'small-icons':
        return 'w-16 h-16';
      case 'grid':
      default:
        return 'w-24 h-24';
    }
  };

  const handleAssetClick = useCallback(
    (assetPath: string, event: React.MouseEvent) => {
      let newSelection: string[];

      if (event.ctrlKey || event.metaKey) {
        // Toggle selection (Ctrl/Cmd + Click)
        newSelection = selectedAssets.includes(assetPath)
          ? selectedAssets.filter((a) => a !== assetPath)
          : [...selectedAssets, assetPath];
      } else if (event.shiftKey && lastSelectedRef.current) {
        // Range selection (Shift + Click)
        const lastIdx = assets.findIndex((a) => a.path === lastSelectedRef.current);
        const currentIdx = assets.findIndex((a) => a.path === assetPath);

        if (lastIdx !== -1 && currentIdx !== -1) {
          const start = Math.min(lastIdx, currentIdx);
          const end = Math.max(lastIdx, currentIdx);
          newSelection = assets.slice(start, end + 1).map((a) => a.path);
        } else {
          newSelection = [assetPath];
        }
      } else {
        // Single selection
        newSelection = [assetPath];
      }

      lastSelectedRef.current = assetPath;
      onAssetSelect(newSelection);
    },
    [assets, selectedAssets, onAssetSelect]
  );

  const handleAssetDoubleClick = (assetPath: string) => {
    onAssetDoubleClick(assetPath);
  };

  const handleDragStart = (assetPath: string) => {
    setDraggedAsset(assetPath);
  };

  const handleDragEnd = () => {
    setDraggedAsset(null);
  };

  const getThumbnail = (asset: AssetInfo) => {
    if (asset.thumbnail) {
      return <img src={asset.thumbnail} alt={asset.name} className="object-cover w-full h-full" />;
    }

    const icon = getAssetIcon(asset.type);
    return (
      <div className="flex items-center justify-center text-4xl">
        {icon}
      </div>
    );
  };

  if (assets.length === 0) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center text-slate-500">
          <div className="text-4xl mb-2">📦</div>
          <div>No assets found</div>
        </div>
      </div>
    );
  }

  return (
    <div
      className={`p-4 gap-4 overflow-y-auto`}
      style={{
        display: 'grid',
        gridTemplateColumns: `repeat(auto-fill, minmax(${viewMode === 'small-icons' ? '100px' : viewMode === 'large-icons' ? '200px' : '140px'}, 1fr))`,
      }}
    >
      {assets.map((asset) => {
        const isSelected = selectedAssets.includes(asset.path);

        return (
          <div
            key={asset.path}
            className={`${getItemSize()} bg-slate-800 border rounded-lg cursor-pointer transition-all hover:border-blue-500 hover:shadow-lg ${
              isSelected ? 'border-blue-500 ring-2 ring-blue-500' : 'border-slate-700'
            } ${draggedAsset === asset.path ? 'opacity-50' : ''}`}
            draggable
            onDragStart={() => handleDragStart(asset.path)}
            onDragEnd={handleDragEnd}
            onClick={(e) => handleAssetClick(asset.path, e)}
            onDoubleClick={() => handleAssetDoubleClick(asset.path)}
          >
            <div className="flex flex-col h-full p-2">
              {/* Thumbnail */}
              <div className={`${getThumbnailSize()} flex items-center justify-center bg-slate-900 rounded mb-2 overflow-hidden`}>
                {getThumbnail(asset)}
              </div>

              {/* Name */}
              <div className="flex-1 min-h-0">
                <div className="text-xs text-slate-200 truncate" title={asset.name}>
                  {asset.name}
                </div>
              </div>

              {/* Info (only show in larger views) */}
              {viewMode !== 'small-icons' && (
                <div className="mt-auto pt-2">
                  <div className="flex items-center justify-between text-xs text-slate-500">
                    <span className="capitalize">{asset.type}</span>
                    <span>{formatFileSize(asset.size)}</span>
                  </div>
                </div>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}
