import React, { useState, useEffect } from 'react';
import { getAssetIcon, formatFileSize, formatDate, assetApi } from './utils';
import type { AssetInfo, AssetPreview } from './types';

interface AssetDetailsProps {
  asset: AssetInfo | null;
}

export function AssetDetails({ asset }: AssetDetailsProps) {
  const [preview, setPreview] = useState<AssetPreview | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (asset) {
      loadPreview();
    } else {
      setPreview(null);
    }
  }, [asset]);

  const loadPreview = async () => {
    if (!asset) return;

    try {
      setLoading(true);
      setError(null);
      const assetPreview = await assetApi.getAssetPreview(asset.path);
      setPreview(assetPreview);
    } catch (err) {
      console.error('Failed to load preview:', err);
      setError('Failed to load preview');
    } finally {
      setLoading(false);
    }
  };

  if (!asset) {
    return (
      <div className="flex items-center justify-center h-full text-slate-500">
        <div className="text-center">
          <div className="text-4xl mb-2">📦</div>
          <div>Select an asset to view details</div>
        </div>
      </div>
    );
  }

  const renderPreview = () => {
    if (loading) {
      return (
        <div className="flex items-center justify-center h-64 bg-slate-900 rounded-lg">
          <div className="text-slate-400">Loading preview...</div>
        </div>
      );
    }

    if (error) {
      return (
        <div className="flex items-center justify-center h-64 bg-slate-900 rounded-lg">
          <div className="text-red-400">{error}</div>
        </div>
      );
    }

    if (!preview) {
      return (
        <div className="flex items-center justify-center h-64 bg-slate-900 rounded-lg">
          <div className="text-6xl">{getAssetIcon(asset.type)}</div>
        </div>
      );
    }

    switch (preview.type) {
      case 'image':
        return (
          <div className="flex items-center justify-center h-64 bg-slate-900 rounded-lg overflow-hidden">
            <img
              src={preview.content}
              alt={asset.name}
              className="max-w-full max-h-full object-contain"
            />
          </div>
        );

      case 'model':
        return (
          <div className="flex items-center justify-center h-64 bg-slate-900 rounded-lg">
            <div className="text-center">
              <div className="text-6xl mb-2">📦</div>
              <div className="text-sm text-slate-400">3D Model Preview</div>
            </div>
          </div>
        );

      case 'audio':
        return (
          <div className="flex items-center justify-center h-64 bg-slate-900 rounded-lg">
            <div className="text-center">
              <div className="text-6xl mb-2">🎵</div>
              <div className="text-sm text-slate-400">Audio Preview</div>
            </div>
          </div>
        );

      case 'text':
        return (
          <div className="h-64 bg-slate-900 rounded-lg overflow-hidden">
            <pre className="p-4 text-xs text-slate-300 overflow-auto h-full">
              {preview.content.substring(0, 1000)}
              {preview.content.length > 1000 && '\n...'}
            </pre>
          </div>
        );

      default:
        return (
          <div className="flex items-center justify-center h-64 bg-slate-900 rounded-lg">
            <div className="text-slate-400">No preview available</div>
          </div>
        );
    }
  };

  const renderMetadata = () => {
    if (!preview?.metadata) return null;

    const metadata = preview.metadata;
    const items: { label: string; value: string | number | undefined }[] = [];

    // Texture metadata
    if (metadata.width !== undefined) {
      items.push({ label: 'Width', value: `${metadata.width}px` });
    }
    if (metadata.height !== undefined) {
      items.push({ label: 'Height', value: `${metadata.height}px` });
    }
    if (metadata.textureFormat) {
      items.push({ label: 'Format', value: metadata.textureFormat });
    }
    if (metadata.meshFormat) {
      items.push({ label: 'Format', value: metadata.meshFormat });
    }

    // Mesh metadata
    if (metadata.vertices !== undefined) {
      items.push({ label: 'Vertices', value: metadata.vertices.toLocaleString() });
    }
    if (metadata.triangles !== undefined) {
      items.push({ label: 'Triangles', value: metadata.triangles.toLocaleString() });
    }

    // Audio metadata
    if (metadata.duration !== undefined) {
      items.push({ label: 'Duration', value: `${metadata.duration.toFixed(2)}s` });
    }
    if (metadata.sampleRate !== undefined) {
      items.push({ label: 'Sample Rate', value: `${metadata.sampleRate}Hz` });
    }
    if (metadata.channels !== undefined) {
      items.push({ label: 'Channels', value: metadata.channels });
    }

    if (items.length === 0) return null;

    return (
      <div className="mt-4">
        <h4 className="text-xs font-semibold text-slate-400 uppercase tracking-wide mb-2">
          Metadata
        </h4>
        <div className="space-y-1">
          {items.map((item, index) => (
            <div key={index} className="flex justify-between text-sm">
              <span className="text-slate-500">{item.label}:</span>
              <span className="text-slate-200">{item.value}</span>
            </div>
          ))}
        </div>
      </div>
    );
  };

  return (
    <div className="h-full overflow-y-auto p-4">
      {/* Preview */}
      {renderPreview()}

      {/* Details */}
      <div className="mt-4 space-y-3">
        <div>
          <h4 className="text-xs font-semibold text-slate-400 uppercase tracking-wide mb-1">
            Name
          </h4>
          <div className="text-sm text-slate-200 break-all">{asset.name}</div>
        </div>

        <div>
          <h4 className="text-xs font-semibold text-slate-400 uppercase tracking-wide mb-1">
            Type
          </h4>
          <div className="flex items-center gap-2">
            <span className="text-2xl">{getAssetIcon(asset.type)}</span>
            <span className="text-sm text-slate-200 capitalize">{asset.type}</span>
          </div>
        </div>

        <div>
          <h4 className="text-xs font-semibold text-slate-400 uppercase tracking-wide mb-1">
            Path
          </h4>
          <div className="text-xs text-slate-400 break-all font-mono">{asset.path}</div>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div>
            <h4 className="text-xs font-semibold text-slate-400 uppercase tracking-wide mb-1">
              Size
            </h4>
            <div className="text-sm text-slate-200">{formatFileSize(asset.size)}</div>
          </div>

          <div>
            <h4 className="text-xs font-semibold text-slate-400 uppercase tracking-wide mb-1">
              Modified
            </h4>
            <div className="text-sm text-slate-200">{formatDate(asset.modified)}</div>
          </div>
        </div>

        {renderMetadata()}
      </div>
    </div>
  );
}
