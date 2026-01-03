import React, { useState, useEffect, useCallback, useRef } from 'react';
import { X, ArrowLeft, ArrowRight, RefreshCw, FolderPlus, Upload } from 'lucide-react';
import { FolderTree } from './FolderTree';
import { SearchBar } from './SearchBar';
import { FilterBar } from './FilterBar';
import { AssetGrid } from './AssetGrid';
import { AssetList } from './AssetList';
import { AssetDetails } from './AssetDetails';
import { ImportDialog } from './ImportDialog';
import { ContextMenu, createAssetContextMenuItems } from './ContextMenu';
import { assetApi, filterAssets, sortAssets } from './utils';
import type { AssetInfo, AssetType, ViewMode, SortBy, SortOrder } from './types';

interface AssetBrowserProps {
  isOpen: boolean;
  onClose: () => void;
}

export function AssetBrowser({ isOpen, onClose }: AssetBrowserProps) {
  // State
  const [currentPath, setCurrentPath] = useState('/');
  const [assets, setAssets] = useState<AssetInfo[]>([]);
  const [filteredAssets, setFilteredAssets] = useState<AssetInfo[]>([]);
  const [selectedAssets, setSelectedAssets] = useState<string[]>([]);
  const [selectedAsset, setSelectedAsset] = useState<AssetInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedTypes, setSelectedTypes] = useState<AssetType[]>([]);
  const [viewMode, setViewMode] = useState<ViewMode>('grid');
  const [sortBy, setSortBy] = useState<SortBy>('name');
  const [sortOrder, setSortOrder] = useState<SortOrder>('asc');
  const [showImportDialog, setShowImportDialog] = useState(false);
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    items: any[];
  } | null>(null);

  // Navigation history
  const [history, setHistory] = useState<string[]>(['/']);
  const [historyIndex, setHistoryIndex] = useState(0);

  // Refs
  const containerRef = useRef<HTMLDivElement>(null);

  // Load assets
  const loadAssets = useCallback(async () => {
    try {
      setLoading(true);
      const assetList = await assetApi.listAssets(currentPath);
      setAssets(assetList);
    } catch (error) {
      console.error('Failed to load assets:', error);
    } finally {
      setLoading(false);
    }
  }, [currentPath]);

  // Filter and sort assets
  useEffect(() => {
    let filtered = filterAssets(assets, {
      types: selectedTypes,
      search: searchQuery,
      tags: [],
    });

    filtered = sortAssets(filtered, sortBy, sortOrder);
    setFilteredAssets(filtered);
  }, [assets, selectedTypes, searchQuery, sortBy, sortOrder]);

  // Load assets when path changes
  useEffect(() => {
    if (isOpen) {
      loadAssets();
    }
  }, [currentPath, isOpen]);

  // Clear selection when changing filters
  useEffect(() => {
    setSelectedAssets([]);
    setSelectedAsset(null);
  }, [currentPath, selectedTypes, searchQuery]);

  // Handle folder selection
  const handleFolderSelect = useCallback((path: string) => {
    setCurrentPath(path);
    setSelectedAssets([]);
    setSelectedAsset(null);

    // Update history
    const newHistory = history.slice(0, historyIndex + 1);
    newHistory.push(path);
    setHistory(newHistory);
    setHistoryIndex(newHistory.length - 1);
  }, [history, historyIndex]);

  // Handle navigation
  const handleBack = useCallback(() => {
    if (historyIndex > 0) {
      const newIndex = historyIndex - 1;
      setHistoryIndex(newIndex);
      setCurrentPath(history[newIndex]);
    }
  }, [history, historyIndex]);

  const handleForward = useCallback(() => {
    if (historyIndex < history.length - 1) {
      const newIndex = historyIndex + 1;
      setHistoryIndex(newIndex);
      setCurrentPath(history[newIndex]);
    }
  }, [history, historyIndex]);

  // Handle asset selection
  const handleAssetSelect = useCallback((assetPaths: string[]) => {
    setSelectedAssets(assetPaths);

    if (assetPaths.length === 1) {
      const asset = assets.find((a) => a.path === assetPaths[0]);
      setSelectedAsset(asset || null);
    } else {
      setSelectedAsset(null);
    }
  }, [assets]);

  // Handle asset double-click
  const handleAssetDoubleClick = useCallback((assetPath: string) => {
    const asset = assets.find((a) => a.path === assetPath);
    if (asset?.type === 'scene') {
      // Open scene
      console.log('Opening scene:', assetPath);
    } else {
      // Open in default application
      console.log('Opening asset:', assetPath);
    }
  }, [assets]);

  // Handle refresh
  const handleRefresh = useCallback(() => {
    loadAssets();
  }, [loadAssets]);

  // Handle new folder
  const handleNewFolder = useCallback(async () => {
    const name = prompt('Enter folder name:', 'New Folder');
    if (name) {
      try {
        await assetApi.createFolder(currentPath, name);
        loadAssets();
      } catch (error) {
        console.error('Failed to create folder:', error);
        alert('Failed to create folder');
      }
    }
  }, [currentPath, loadAssets]);

  // Handle context menu
  const handleContextMenu = useCallback((e: React.MouseEvent, asset: AssetInfo) => {
    e.preventDefault();
    e.stopPropagation();

    const items = createAssetContextMenuItems(
      asset.path,
      asset.name,
      () => handleRename(asset),
      () => handleDelete(asset),
      () => handleCopyPath(asset),
      () => handleShowInFinder(asset),
      () => handleViewDependencies(asset)
    );

    setContextMenu({
      x: e.clientX,
      y: e.clientY,
      items,
    });
  }, []);

  // Handle rename
  const handleRename = useCallback(async (asset: AssetInfo) => {
    const newName = prompt('Enter new name:', asset.name);
    if (newName && newName !== asset.name) {
      try {
        await assetApi.renameAsset(asset.path, newName);
        loadAssets();
      } catch (error) {
        console.error('Failed to rename asset:', error);
        alert('Failed to rename asset');
      }
    }
  }, [loadAssets]);

  // Handle delete
  const handleDelete = useCallback(async (asset: AssetInfo) => {
    if (confirm(`Are you sure you want to delete "${asset.name}"?`)) {
      try {
        await assetApi.deleteAsset(asset.path);
        loadAssets();
        setSelectedAssets([]);
        setSelectedAsset(null);
      } catch (error) {
        console.error('Failed to delete asset:', error);
        alert('Failed to delete asset');
      }
    }
  }, [loadAssets]);

  // Handle copy path
  const handleCopyPath = useCallback(async (asset: AssetInfo) => {
    try {
      await navigator.clipboard.writeText(asset.path);
      console.log('Copied to clipboard:', asset.path);
    } catch (error) {
      console.error('Failed to copy path:', error);
    }
  }, []);

  // Handle show in finder
  const handleShowInFinder = useCallback((asset: AssetInfo) => {
    // This would be implemented via Tauri command
    console.log('Show in finder:', asset.path);
  }, []);

  // Handle view dependencies
  const handleViewDependencies = useCallback(async (asset: AssetInfo) => {
    try {
      const dependencies = await assetApi.getAssetDependencies(asset.path);
      console.log('Dependencies:', dependencies);
      alert(`Dependencies:\n${dependencies.join('\n') || 'None'}`);
    } catch (error) {
      console.error('Failed to get dependencies:', error);
    }
  }, []);

  // Handle import complete
  const handleImportComplete = useCallback((importedFiles: string[]) => {
    console.log('Imported files:', importedFiles);
    loadAssets();
  }, [loadAssets]);

  // Handle drag and drop for import
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    const files = Array.from(e.dataTransfer.files);
    if (files.length > 0) {
      setShowImportDialog(true);
    }
  }, []);

  if (!isOpen) return null;

  return (
    <>
      <div
        ref={containerRef}
        className="fixed inset-0 bg-slate-900 z-40 flex"
        onDragOver={handleDragOver}
        onDrop={handleDrop}
      >
        {/* Left Panel - Folder Tree */}
        <div className="w-64 border-r border-slate-700 flex flex-col">
          <div className="flex items-center justify-between px-4 py-3 border-b border-slate-700">
            <h2 className="font-semibold">Asset Browser</h2>
            <button
              onClick={onClose}
              className="p-1 hover:bg-slate-700 rounded transition-colors"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
          <FolderTree
            onFolderSelect={handleFolderSelect}
            selectedPath={currentPath}
          />
        </div>

        {/* Center Panel - Asset View */}
        <div className="flex-1 flex flex-col">
          {/* Toolbar */}
          <div className="border-b border-slate-700">
            {/* Navigation */}
            <div className="flex items-center gap-2 px-4 py-2 border-b border-slate-700">
              <button
                onClick={handleBack}
                disabled={historyIndex === 0}
                className="p-2 hover:bg-slate-700 rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                title="Back"
              >
                <ArrowLeft className="w-4 h-4" />
              </button>
              <button
                onClick={handleForward}
                disabled={historyIndex === history.length - 1}
                className="p-2 hover:bg-slate-700 rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                title="Forward"
              >
                <ArrowRight className="w-4 h-4" />
              </button>

              {/* Breadcrumb */}
              <div className="flex-1 flex items-center gap-1 px-4 py-1.5 bg-slate-800 rounded-lg text-sm">
                {currentPath.split('/').filter(Boolean).map((part, index, parts) => (
                  <React.Fragment key={index}>
                    <button
                      onClick={() => handleFolderSelect('/' + parts.slice(0, index + 1).join('/'))}
                      className="hover:text-blue-400 transition-colors"
                    >
                      {part || 'Assets'}
                    </button>
                    {index < parts.length - 1 && <span className="text-slate-500">/</span>}
                  </React.Fragment>
                ))}
              </div>

              {/* Actions */}
              <button
                onClick={handleRefresh}
                className="p-2 hover:bg-slate-700 rounded transition-colors"
                title="Refresh"
              >
                <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
              </button>
              <button
                onClick={handleNewFolder}
                className="p-2 hover:bg-slate-700 rounded transition-colors"
                title="New Folder"
              >
                <FolderPlus className="w-4 h-4" />
              </button>
              <button
                onClick={() => setShowImportDialog(true)}
                className="p-2 hover:bg-slate-700 rounded transition-colors"
                title="Import Assets"
              >
                <Upload className="w-4 h-4" />
              </button>
            </div>

            {/* Search and Filter */}
            <div className="px-4 py-2">
              <SearchBar value={searchQuery} onSearchChange={setSearchQuery} />
            </div>

            {/* Filter Bar */}
            <FilterBar
              selectedTypes={selectedTypes}
              onTypeFilterChange={setSelectedTypes}
              viewMode={viewMode}
              onViewModeChange={setViewMode}
              sortBy={sortBy}
              onSortByChange={setSortBy}
              sortOrder={sortOrder}
              onSortOrderChange={setSortOrder}
            />
          </div>

          {/* Asset List */}
          <div className="flex-1 overflow-hidden">
            {viewMode === 'list' ? (
              <AssetList
                assets={filteredAssets}
                selectedAssets={selectedAssets}
                onAssetSelect={handleAssetSelect}
                onAssetDoubleClick={handleAssetDoubleClick}
              />
            ) : (
              <AssetGrid
                assets={filteredAssets}
                selectedAssets={selectedAssets}
                onAssetSelect={handleAssetSelect}
                onAssetDoubleClick={handleAssetDoubleClick}
                viewMode={viewMode}
              />
            )}
          </div>

          {/* Status Bar */}
          <div className="flex items-center justify-between px-4 py-2 border-t border-slate-700 text-xs text-slate-400">
            <div>
              {filteredAssets.length} asset{filteredAssets.length !== 1 ? 's' : ''}
              {selectedAssets.length > 0 && ` · ${selectedAssets.length} selected`}
            </div>
            <div>{currentPath}</div>
          </div>
        </div>

        {/* Right Panel - Asset Details */}
        <div className="w-80 border-l border-slate-700">
          <div className="px-4 py-3 border-b border-slate-700">
            <h2 className="font-semibold">Details</h2>
          </div>
          <AssetDetails asset={selectedAsset} />
        </div>
      </div>

      {/* Import Dialog */}
      <ImportDialog
        isOpen={showImportDialog}
        onClose={() => setShowImportDialog(false)}
        currentPath={currentPath}
        onComplete={handleImportComplete}
      />

      {/* Context Menu */}
      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          items={contextMenu.items}
          onClose={() => setContextMenu(null)}
        />
      )}
    </>
  );
}
