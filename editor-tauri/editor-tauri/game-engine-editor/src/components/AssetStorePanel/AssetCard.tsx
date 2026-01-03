// Asset Card
// 资源卡片组件

import React from 'react';
import { Heart, Download, Star, Eye } from 'lucide-react';
import { AssetMetadata } from '../../types/assetStore';

interface AssetCardProps {
  asset: AssetMetadata;
  isFavorite: boolean;
  viewMode?: 'grid' | 'list';
  onClick: () => void;
  onDownload: (assetId: string) => void;
  onToggleFavorite: (assetId: string) => void;
}

const AssetCard: React.FC<AssetCardProps> = ({
  asset,
  isFavorite,
  viewMode = 'grid',
  onClick,
  onDownload,
  onToggleFavorite,
}) => {
  const handleDownload = (e: React.MouseEvent) => {
    e.stopPropagation();
    onDownload(asset.id);
  };

  const handleFavorite = (e: React.MouseEvent) => {
    e.stopPropagation();
    onToggleFavorite(asset.id);
  };

  if (viewMode === 'list') {
    return (
      <div
        onClick={onClick}
        className="bg-gray-800 rounded-lg p-4 hover:bg-gray-750 cursor-pointer transition-colors border border-gray-700 hover:border-blue-500"
      >
        <div className="flex gap-4">
          {/* 缩略图 */}
          <div className="w-32 h-32 bg-gray-900 rounded flex-shrink-0 flex items-center justify-center">
            {asset.previewUrls.length > 0 ? (
              <img
                src={asset.previewUrls[0]}
                alt={asset.name}
                className="w-full h-full object-cover rounded"
              />
            ) : (
              <Eye className="w-8 h-8 text-gray-600" />
            )}
          </div>

          {/* 信息 */}
          <div className="flex-1 min-w-0">
            <h3 className="text-lg font-semibold mb-1 truncate">{asset.name}</h3>
            <p className="text-gray-400 text-sm mb-2 line-clamp-2">
              {asset.description}
            </p>
            <div className="flex items-center gap-4 text-sm text-gray-400">
              <span className="flex items-center gap-1">
                <Star className="w-4 h-4 text-yellow-500" />
                {asset.rating.toFixed(1)}
              </span>
              <span className="flex items-center gap-1">
                <Download className="w-4 h-4" />
                {asset.downloadCount}
              </span>
              <span>by {asset.author}</span>
              <span className="ml-auto">
                {asset.pricing.type === 'free' ? (
                  <span className="text-green-500 font-semibold">Free</span>
                ) : (
                  <span className="text-blue-500 font-semibold">
                    ${asset.pricing.priceUsd}
                  </span>
                )}
              </span>
            </div>
            <div className="mt-2 flex gap-2 flex-wrap">
              {asset.tags.slice(0, 3).map((tag) => (
                <span
                  key={tag}
                  className="bg-gray-700 px-2 py-1 rounded text-xs"
                >
                  {tag}
                </span>
              ))}
            </div>
          </div>

          {/* 操作按钮 */}
          <div className="flex flex-col gap-2">
            <button
              onClick={handleFavorite}
              className={`p-2 rounded ${
                isFavorite ? 'bg-red-600 hover:bg-red-700' : 'bg-gray-700 hover:bg-gray-600'
              }`}
            >
              <Heart
                className={`w-5 h-5 ${isFavorite ? 'fill-current' : ''}`}
              />
            </button>
            <button
              onClick={handleDownload}
              className="bg-blue-600 hover:bg-blue-700 p-2 rounded"
            >
              <Download className="w-5 h-5" />
            </button>
          </div>
        </div>
      </div>
    );
  }

  // Grid view
  return (
    <div
      onClick={onClick}
      className="bg-gray-800 rounded-lg overflow-hidden hover:shadow-xl transition-all cursor-pointer border border-gray-700 hover:border-blue-500 group"
    >
      {/* 预览图 */}
      <div className="aspect-square bg-gray-900 relative">
        {asset.previewUrls.length > 0 ? (
          <img
            src={asset.previewUrls[0]}
            alt={asset.name}
            className="w-full h-full object-cover"
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center">
            <Eye className="w-16 h-16 text-gray-600" />
          </div>
        )}

        {/* 悬浮操作按钮 */}
        <div className="absolute top-2 right-2 flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
          <button
            onClick={handleFavorite}
            className={`p-2 rounded-lg ${
              isFavorite ? 'bg-red-600' : 'bg-gray-700'
            } hover:scale-110 transition-transform`}
          >
            <Heart
              className={`w-5 h-5 ${isFavorite ? 'fill-current' : ''}`}
            />
          </button>
          <button
            onClick={handleDownload}
            className="bg-blue-600 p-2 rounded-lg hover:bg-blue-700 hover:scale-110 transition-transform"
          >
            <Download className="w-5 h-5" />
          </button>
        </div>

        {/* 价格标签 */}
        <div className="absolute bottom-2 left-2">
          {asset.pricing.type === 'free' ? (
            <span className="bg-green-600 px-2 py-1 rounded text-sm font-semibold">
              Free
            </span>
          ) : (
            <span className="bg-blue-600 px-2 py-1 rounded text-sm font-semibold">
              ${asset.pricing.priceUsd}
            </span>
          )}
        </div>
      </div>

      {/* 信息 */}
      <div className="p-4">
        <h3 className="font-semibold mb-1 truncate">{asset.name}</h3>
        <p className="text-gray-400 text-sm mb-2 line-clamp-2">
          {asset.description}
        </p>

        <div className="flex items-center justify-between text-sm">
          <div className="flex items-center gap-2">
            <span className="flex items-center gap-1 text-yellow-500">
              <Star className="w-4 h-4 fill-current" />
              {asset.rating.toFixed(1)}
            </span>
            <span className="text-gray-400">
              {asset.downloadCount} downloads
            </span>
          </div>
          <span className="text-gray-400 text-xs">
            {asset.assetType}
          </span>
        </div>

        {/* 标签 */}
        <div className="mt-2 flex gap-1 flex-wrap">
          {asset.tags.slice(0, 3).map((tag) => (
            <span
              key={tag}
              className="bg-gray-700 px-2 py-0.5 rounded text-xs"
            >
              {tag}
            </span>
          ))}
        </div>
      </div>
    </div>
  );
};

export default AssetCard;
