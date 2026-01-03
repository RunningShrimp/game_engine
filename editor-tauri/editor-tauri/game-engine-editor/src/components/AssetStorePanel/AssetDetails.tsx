// Asset Details Panel
// 资源详情面板

import React, { useState, useEffect } from 'react';
import { X, Heart, Download, ExternalLink, FileText, Package } from 'lucide-react';
import { AssetMetadata, AssetData } from '../../types/assetStore';
import { AssetStoreAPI } from '../../api/assetStore';

interface AssetDetailsProps {
  asset: AssetMetadata;
  isFavorite: boolean;
  onClose: () => void;
  onDownload: (assetId: string) => void;
  onToggleFavorite: (assetId: string) => void;
}

const AssetDetails: React.FC<AssetDetailsProps> = ({
  asset,
  isFavorite,
  onClose,
  onDownload,
  onToggleFavorite,
}) => {
  const [assetData, setAssetData] = useState<AssetData | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadAssetDetails();
  }, [asset.id]);

  const loadAssetDetails = async () => {
    setLoading(true);
    try {
      const data = await AssetStoreAPI.downloadAsset(asset.id);
      setAssetData(data);
    } catch (error) {
      console.error('Failed to load asset details:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleDownload = () => {
    onDownload(asset.id);
  };

  const handleToggleFavorite = () => {
    onToggleFavorite(asset.id);
  };

  return (
    <div className="fixed inset-y-0 right-0 w-96 bg-gray-800 border-l border-gray-700 overflow-y-auto shadow-xl">
      {/* 头部 */}
      <div className="sticky top-0 bg-gray-800 border-b border-gray-700 p-4 z-10">
        <div className="flex justify-between items-start">
          <h2 className="text-xl font-bold">{asset.name}</h2>
          <button
            onClick={onClose}
            className="p-1 hover:bg-gray-700 rounded"
          >
            <X className="w-6 h-6" />
          </button>
        </div>
        <p className="text-gray-400 text-sm mt-1">by {asset.author}</p>
      </div>

      {/* 预览图 */}
      {asset.previewUrls.length > 0 && (
        <div className="p-4">
          <img
            src={asset.previewUrls[0]}
            alt={asset.name}
            className="w-full rounded-lg"
          />
        </div>
      )}

      {/* 内容 */}
      <div className="p-4 space-y-4">
        {/* 描述 */}
        <div>
          <h3 className="font-semibold mb-2">Description</h3>
          <p className="text-gray-300 text-sm">{asset.description}</p>
        </div>

        {/* 信息 */}
        <div className="grid grid-cols-2 gap-2 text-sm">
          <div>
            <span className="text-gray-400">Version:</span>{' '}
            <span className="text-white">{asset.version}</span>
          </div>
          <div>
            <span className="text-gray-400">Type:</span>{' '}
            <span className="text-white capitalize">{asset.assetType}</span>
          </div>
          <div>
            <span className="text-gray-400">Category:</span>{' '}
            <span className="text-white capitalize">{asset.category}</span>
          </div>
          <div>
            <span className="text-gray-400">Size:</span>{' '}
            <span className="text-white">
              {(asset.fileSizeBytes / 1024 / 1024).toFixed(2)} MB
            </span>
          </div>
          <div>
            <span className="text-gray-400">Rating:</span>{' '}
            <span className="text-yellow-500">
              ★ {asset.rating.toFixed(1)}
            </span>
          </div>
          <div>
            <span className="text-gray-400">Downloads:</span>{' '}
            <span className="text-white">{asset.downloadCount}</span>
          </div>
          <div className="col-span-2">
            <span className="text-gray-400">License:</span>{' '}
            <span className="text-white capitalize">{asset.license}</span>
          </div>
        </div>

        {/* 标签 */}
        {asset.tags.length > 0 && (
          <div>
            <h3 className="font-semibold mb-2">Tags</h3>
            <div className="flex gap-2 flex-wrap">
              {asset.tags.map((tag) => (
                <span
                  key={tag}
                  className="bg-gray-700 px-2 py-1 rounded text-sm"
                >
                  {tag}
                </span>
              ))}
            </div>
          </div>
        )}

        {/* 兼容性 */}
        {asset.compatibility.length > 0 && (
          <div>
            <h3 className="font-semibold mb-2">Compatibility</h3>
            <div className="space-y-1 text-sm">
              {asset.compatibility.map((version) => (
                <div key={version} className="text-gray-300">
                  Engine {version}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* 依赖 */}
        {asset.dependencies.length > 0 && (
          <div>
            <h3 className="font-semibold mb-2">Dependencies</h3>
            <div className="space-y-1 text-sm text-gray-300">
              {asset.dependencies.map((dep) => (
                <div key={dep} className="flex items-center gap-2">
                  <Package className="w-4 h-4" />
                  {dep}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* 文件列表 */}
        {assetData && assetData.files.length > 0 && (
          <div>
            <h3 className="font-semibold mb-2">Files</h3>
            <div className="space-y-2">
              {assetData.files.map((file, index) => (
                <div
                  key={index}
                  className="bg-gray-700 p-3 rounded flex items-center justify-between text-sm"
                >
                  <div className="flex items-center gap-2 flex-1 min-w-0">
                    <FileText className="w-4 h-4 text-gray-400 flex-shrink-0" />
                    <span className="truncate">{file.filename}</span>
                  </div>
                  <span className="text-gray-400 ml-2">
                    {(file.sizeBytes / 1024).toFixed(0)} KB
                  </span>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* 价格 */}
        <div className="bg-gray-700 p-4 rounded-lg">
          {asset.pricing.type === 'free' ? (
            <div className="text-center">
              <div className="text-green-500 text-2xl font-bold mb-2">Free</div>
              <p className="text-gray-400 text-sm">Use in any project</p>
            </div>
          ) : asset.pricing.type === 'paid' ? (
            <div className="text-center">
              <div className="text-blue-500 text-2xl font-bold mb-2">
                ${asset.pricing.priceUsd}
              </div>
              {asset.pricing.discountPercent && (
                <p className="text-gray-400 text-sm">
                  Save {asset.pricing.discountPercent}%
                </p>
              )}
            </div>
          ) : (
            <div className="text-center">
              <div className="text-purple-500 text-lg font-bold mb-2">
                Subscription
              </div>
              <div className="text-sm space-y-1">
                <p className="text-gray-300">
                  ${asset.pricing.monthlyUsd}/month
                </p>
                <p className="text-gray-300">
                  ${asset.pricing.yearlyUsd}/year
                </p>
              </div>
            </div>
          )}
        </div>

        {/* 操作按钮 */}
        <div className="flex gap-2">
          <button
            onClick={handleDownload}
            className="flex-1 bg-blue-600 hover:bg-blue-700 py-3 rounded-lg font-semibold flex items-center justify-center gap-2"
          >
            <Download className="w-5 h-5" />
            Download
          </button>
          <button
            onClick={handleToggleFavorite}
            className={`px-4 py-3 rounded-lg ${
              isFavorite ? 'bg-red-600 hover:bg-red-700' : 'bg-gray-700 hover:bg-gray-600'
            }`}
          >
            <Heart
              className={`w-5 h-5 ${isFavorite ? 'fill-current' : ''}`}
            />
          </button>
        </div>

        {/* 许可证信息 */}
        <div className="text-xs text-gray-400 border-t border-gray-700 pt-4">
          <p>
            By downloading, you agree to the {asset.license} license terms.
          </p>
        </div>
      </div>
    </div>
  );
};

export default AssetDetails;
