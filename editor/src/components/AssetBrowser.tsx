import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';

interface AssetData {
  path: string;
  name: string;
  asset_type: string;
  thumbnail?: string;
}

export const AssetBrowser: React.FC = () => {
  const [assets, setAssets] = useState<AssetData[]>([]);
  const [filter, setFilter] = useState('');
  const [selectedType, setSelectedType] = useState<string>('all');
  const [selectedAsset, setSelectedAsset] = useState<AssetData | null>(null);

  useEffect(() => {
    refreshAssets();
  }, [selectedType]);

  const refreshAssets = async () => {
    try {
      const assetType = selectedType === 'all' ? null : selectedType;
      const data = await invoke<AssetData[]>('get_assets', { assetType });
      setAssets(data);
    } catch (error) {
      console.error('Failed to load assets:', error);
    }
  };

  const handleImportAsset = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'Assets',
            extensions: ['glb', 'gltf', 'png', 'jpg', 'mat']
          }
        ]
      });

      if (selected && typeof selected === 'string') {
        const assetType = getAssetType(selected);
        await invoke('import_asset', {
          sourcePath: selected,
          assetType
        });
        await refreshAssets();
      }
    } catch (error) {
      console.error('Failed to import asset:', error);
    }
  };

  const getAssetType = (path: string): string => {
    const ext = path.split('.').pop()?.toLowerCase();
    if (ext === 'glb' || ext === 'gltf') return 'model';
    if (ext === 'png' || ext === 'jpg' || ext === 'jpeg') return 'texture';
    if (ext === 'mat') return 'material';
    return 'other';
  };

  const getAssetIcon = (type: string): string => {
    switch (type) {
      case 'model': return '🎲';
      case 'texture': return '🖼️';
      case 'material': return '🎨';
      case 'audio': return '🔊';
      case 'scene': return '🎬';
      default: return '📄';
    }
  };

  const filteredAssets = assets.filter(asset =>
    filter === '' || asset.name.toLowerCase().includes(filter.toLowerCase())
  );

  return (
    <div className="asset-browser">
      <div className="asset-browser-header">
        <h3>Assets</h3>
        <button onClick={handleImportAsset} title="Import Asset">
          Import
        </button>
      </div>

      <div className="asset-toolbar">
        <input
          type="text"
          placeholder="Search..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          className="asset-search"
        />
        <select
          value={selectedType}
          onChange={(e) => setSelectedType(e.target.value)}
          className="asset-type-filter"
        >
          <option value="all">All Types</option>
          <option value="model">Models</option>
          <option value="texture">Textures</option>
          <option value="material">Materials</option>
          <option value="audio">Audio</option>
          <option value="scene">Scenes</option>
        </select>
      </div>

      <div className="asset-list">
        {filteredAssets.map((asset, index) => (
          <div
            key={index}
            className={`asset-item ${selectedAsset?.path === asset.path ? 'selected' : ''}`}
            onClick={() => setSelectedAsset(asset)}
            onDoubleClick={() => {
              // TODO: 实现资源预览或打开功能
              console.log('Open asset:', asset);
            }}
          >
            <span className="asset-icon">{getAssetIcon(asset.asset_type)}</span>
            <span className="asset-name">{asset.name}</span>
            <span className="asset-type">{asset.asset_type}</span>
          </div>
        ))}
      </div>

      {selectedAsset && (
        <div className="asset-details">
          <h4>Asset Details</h4>
          <div className="detail-row">
            <label>Name:</label>
            <span>{selectedAsset.name}</span>
          </div>
          <div className="detail-row">
            <label>Type:</label>
            <span>{selectedAsset.asset_type}</span>
          </div>
          <div className="detail-row">
            <label>Path:</label>
            <span className="path">{selectedAsset.path}</span>
          </div>
        </div>
      )}
    </div>
  );
};
