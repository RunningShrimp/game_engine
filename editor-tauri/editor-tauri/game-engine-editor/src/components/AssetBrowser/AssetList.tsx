import React, { useState, useRef, useCallback } from 'react';
import { getAssetIcon, formatFileSize, formatDate } from './utils';
import type { AssetInfo } from './types';

interface AssetListProps {
  assets: AssetInfo[];
  selectedAssets: string[];
  onAssetSelect: (assetPaths: string[]) => void;
  onAssetDoubleClick: (assetPath: string) => void;
}

export function AssetList({
  assets,
  selectedAssets,
  onAssetSelect,
  onAssetDoubleClick,
}: AssetListProps) {
  const lastSelectedRef = useRef<string | null>(null);

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
    <div className="h-full overflow-y-auto">
      {/* Header */}
      <div className="flex items-center gap-4 px-4 py-2 bg-slate-800 border-b border-slate-700 text-xs font-semibold text-slate-400 uppercase tracking-wide sticky top-0">
        <div className="w-8" />
        <div className="flex-1">Name</div>
        <div className="w-24">Type</div>
        <div className="w-24">Size</div>
        <div className="w-40">Modified</div>
      </div>

      {/* List */}
      {assets.map((asset) => {
        const isSelected = selectedAssets.includes(asset.path);

        return (
          <div
            key={asset.path}
            className={`flex items-center gap-4 px-4 py-2 border-b border-slate-800 cursor-pointer hover:bg-slate-800 transition-colors ${
              isSelected ? 'bg-blue-600/20 border-l-4 border-l-blue-500' : ''
            }`}
            onClick={(e) => handleAssetClick(asset.path, e)}
            onDoubleClick={() => handleAssetDoubleClick(asset.path)}
          >
            {/* Icon */}
            <div className="w-8 text-2xl flex items-center justify-center">
              {getAssetIcon(asset.type)}
            </div>

            {/* Name */}
            <div className="flex-1 min-w-0">
              <div className="text-sm text-slate-200 truncate" title={asset.name}>
                {asset.name}
              </div>
              <div className="text-xs text-slate-500 truncate" title={asset.path}>
                {asset.path}
              </div>
            </div>

            {/* Type */}
            <div className="w-24 text-sm text-slate-400 capitalize">
              {asset.type}
            </div>

            {/* Size */}
            <div className="w-24 text-sm text-slate-400">
              {formatFileSize(asset.size)}
            </div>

            {/* Modified */}
            <div className="w-40 text-sm text-slate-400">
              {formatDate(asset.modified)}
            </div>
          </div>
        );
      })}
    </div>
  );
}
