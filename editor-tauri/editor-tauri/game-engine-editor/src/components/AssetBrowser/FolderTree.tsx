import React, { useState, useEffect } from 'react';
import { ChevronRight, ChevronDown, Folder, FolderOpen, Star, Clock } from 'lucide-react';
import type { FolderNode } from './types';
import { assetApi } from './utils';

interface FolderTreeProps {
  onFolderSelect: (path: string) => void;
  selectedPath: string;
}

export function FolderTree({ onFolderSelect, selectedPath }: FolderTreeProps) {
  const [tree, setTree] = useState<FolderNode[]>([]);
  const [loading, setLoading] = useState(true);
  const [favorites, setFavorites] = useState<string[]>([]);
  const [recent, setRecent] = useState<string[]>([]);

  useEffect(() => {
    loadFolderTree();
    loadFavorites();
    loadRecent();
  }, []);

  const loadFolderTree = async () => {
    try {
      setLoading(true);
      const folderTree = await assetApi.getFolderTree('/');
      setTree(folderTree);
    } catch (error) {
      console.error('Failed to load folder tree:', error);
    } finally {
      setLoading(false);
    }
  };

  const loadFavorites = () => {
    const saved = localStorage.getItem('asset-browser-favorites');
    if (saved) {
      try {
        setFavorites(JSON.parse(saved));
      } catch (e) {
        console.error('Failed to load favorites:', e);
      }
    }
  };

  const loadRecent = () => {
    const saved = localStorage.getItem('asset-browser-recent');
    if (saved) {
      try {
        setRecent(JSON.parse(saved));
      } catch (e) {
        console.error('Failed to load recent:', e);
      }
    }
  };

  const toggleExpand = (node: FolderNode) => {
    const updateNodes = (nodes: FolderNode[]): FolderNode[] => {
      return nodes.map((n) => {
        if (n.path === node.path) {
          return { ...n, expanded: !n.expanded };
        }
        if (n.children.length > 0) {
          return { ...n, children: updateNodes(n.children) };
        }
        return n;
      });
    };

    setTree(updateNodes(tree));
  };

  const toggleFavorite = (path: string) => {
    const newFavorites = favorites.includes(path)
      ? favorites.filter((f) => f !== path)
      : [...favorites, path];

    setFavorites(newFavorites);
    localStorage.setItem('asset-browser-favorites', JSON.stringify(newFavorites));
  };

  const addToRecent = (path: string) => {
    const newRecent = [path, ...recent.filter((r) => r !== path)].slice(0, 10);
    setRecent(newRecent);
    localStorage.setItem('asset-browser-recent', JSON.stringify(newRecent));
  };

  const handleFolderClick = (path: string) => {
    onFolderSelect(path);
    addToRecent(path);
  };

  const renderNode = (node: FolderNode, level: number = 0): React.ReactNode => {
    const isSelected = node.path === selectedPath;
    const isFavorite = favorites.includes(node.path);

    return (
      <div key={node.path}>
        <div
          className={`flex items-center gap-1 px-2 py-1 cursor-pointer hover:bg-slate-700 transition-colors ${
            isSelected ? 'bg-slate-700' : ''
          }`}
          style={{ paddingLeft: `${level * 16 + 8}px` }}
          onClick={() => handleFolderClick(node.path)}
          onDoubleClick={() => toggleExpand(node)}
        >
          <button
            className="p-0.5 hover:bg-slate-600 rounded"
            onClick={(e) => {
              e.stopPropagation();
              toggleExpand(node);
            }}
          >
            {node.expanded ? (
              <ChevronDown className="w-3 h-3 text-slate-400" />
            ) : (
              <ChevronRight className="w-3 h-3 text-slate-400" />
            )}
          </button>

          {node.expanded ? (
            <FolderOpen className="w-4 h-4 text-yellow-500" />
          ) : (
            <Folder className="w-4 h-4 text-yellow-500" />
          )}

          <span className="flex-1 text-sm truncate">{node.name}</span>

          <span className="text-xs text-slate-500">{node.assetCount}</span>

          <button
            className={`p-0.5 hover:bg-slate-600 rounded ${
              isFavorite ? 'text-yellow-400' : 'text-slate-500'
            }`}
            onClick={(e) => {
              e.stopPropagation();
              toggleFavorite(node.path);
            }}
          >
            <Star className={`w-3 h-3 ${isFavorite ? 'fill-current' : ''}`} />
          </button>
        </div>

        {node.expanded && node.children.map((child) => renderNode(child, level + 1))}
      </div>
    );
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-slate-400">Loading...</div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto">
      {/* Quick Access Section */}
      <div className="border-b border-slate-700">
        <div className="px-3 py-2 text-xs font-semibold text-slate-400 uppercase tracking-wide">
          Quick Access
        </div>

        {/* Favorites */}
        {favorites.length > 0 && (
          <div className="mb-2">
            <div className="flex items-center gap-2 px-3 py-1 text-xs text-slate-400">
              <Star className="w-3 h-3 fill-yellow-400 text-yellow-400" />
              <span>Favorites</span>
            </div>
            {favorites.map((path) => (
              <div
                key={path}
                className={`flex items-center gap-2 px-3 py-1 cursor-pointer hover:bg-slate-700 transition-colors ${
                  selectedPath === path ? 'bg-slate-700' : ''
                }`}
                onClick={() => handleFolderClick(path)}
              >
                <Folder className="w-4 h-4 text-yellow-500" />
                <span className="flex-1 text-sm truncate">{path.split('/').pop()}</span>
              </div>
            ))}
          </div>
        )}

        {/* Recent */}
        {recent.length > 0 && (
          <div>
            <div className="flex items-center gap-2 px-3 py-1 text-xs text-slate-400">
              <Clock className="w-3 h-3" />
              <span>Recent</span>
            </div>
            {recent.map((path) => (
              <div
                key={path}
                className={`flex items-center gap-2 px-3 py-1 cursor-pointer hover:bg-slate-700 transition-colors ${
                  selectedPath === path ? 'bg-slate-700' : ''
                }`}
                onClick={() => handleFolderClick(path)}
              >
                <Folder className="w-4 h-4 text-slate-500" />
                <span className="flex-1 text-sm truncate">{path.split('/').pop()}</span>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Folder Tree */}
      <div>
        <div className="px-3 py-2 text-xs font-semibold text-slate-400 uppercase tracking-wide border-b border-slate-700">
          Assets
        </div>
        {tree.map((node) => renderNode(node))}
      </div>
    </div>
  );
}
