/**
 * 动画管理器组件
 */

import React, { useState } from 'react';
import { AnimationClip, createEmptyAnimationClip } from '../../types/animation';
import './AnimationManager.css';

export interface AnimationManagerProps {
  clips: AnimationClip[];
  currentClipId?: string;
  onClipSelect: (clipId: string) => void;
  onClipCreate: (clip: AnimationClip) => void;
  onClipUpdate: (clip: AnimationClip) => void;
  onClipDelete: (clipId: string) => void;
}

export const AnimationManager: React.FC<AnimationManagerProps> = ({
  clips,
  currentClipId,
  onClipSelect,
  onClipCreate,
  onClipUpdate,
  onClipDelete,
}) => {
  const [showNewDialog, setShowNewDialog] = useState(false);
  const [newClipName, setNewClipName] = useState('');
  const [showMenu, setShowMenu] = useState(false);
  const [menuClipId, setMenuClipId] = useState<string | null>(null);

  const handleCreateClip = async () => {
    if (!newClipName.trim()) return;

    const newClip = createEmptyAnimationClip(newClipName.trim());

    // TODO: 调用Tauri命令保存动画剪辑
    // await invoke('save_animation_clip', { clip: newClip });

    onClipCreate(newClip);
    setNewClipName('');
    setShowNewDialog(false);
  };

  const handleDuplicateClip = async (clip: AnimationClip) => {
    const duplicatedClip: AnimationClip = {
      ...JSON.parse(JSON.stringify(clip)),
      id: `clip_${Date.now()}`,
      name: `${clip.name} (Copy)`,
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };

    // TODO: 调用Tauri命令保存动画剪辑
    // await invoke('save_animation_clip', { clip: duplicatedClip });

    onClipCreate(duplicatedClip);
  };

  const handleDeleteClip = async (clipId: string) => {
    if (clips.length <= 1) {
      alert('不能删除最后一个动画剪辑');
      return;
    }

    if (confirm('确定要删除这个动画剪辑吗？')) {
      // TODO: 调用Tauri命令删除动画剪辑
      // await invoke('delete_animation_clip', { clipId });

      onClipDelete(clipId);
      setShowMenu(false);
    }
  };

  const handleRenameClip = async (clipId: string, newName: string) => {
    const clip = clips.find(c => c.id === clipId);
    if (!clip) return;

    const updatedClip = { ...clip, name: newName, updatedAt: Date.now() };

    // TODO: 调用Tauri命令保存动画剪辑
    // await invoke('save_animation_clip', { clip: updatedClip });

    onClipUpdate(updatedClip);
  };

  const handleExportClip = async (clip: AnimationClip) => {
    try {
      const data = JSON.stringify(clip, null, 2);
      const blob = new Blob([data], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${clip.name}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Failed to export animation clip:', error);
    }
  };

  const handleImportClip = async () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      try {
        const text = await file.text();
        const clip = JSON.parse(text) as AnimationClip;

        // 生成新的ID
        clip.id = `clip_${Date.now()}`;
        clip.createdAt = Date.now();
        clip.updatedAt = Date.now();

        // TODO: 调用Tauri命令保存动画剪辑
        // await invoke('save_animation_clip', { clip });
        onClipCreate(clip);
      } catch (error) {
        console.error('Failed to import animation clip:', error);
        alert('导入失败：无效的动画文件');
      }
    };
    input.click();
  };

  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    const frames = Math.floor((seconds % 1) * 60);
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}:${frames.toString().padStart(2, '0')}`;
  };

  return (
    <div className="animation-manager">
      <div className="animation-manager-header">
        <span className="animation-manager-title">动画剪辑</span>
        <div className="animation-manager-actions">
          <button
            className="action-btn"
            onClick={() => setShowNewDialog(true)}
            title="新建动画"
          >
            ➕
          </button>
          <button
            className="action-btn"
            onClick={handleImportClip}
            title="导入动画"
          >
            📥
          </button>
        </div>
      </div>

      <div className="animation-manager-list">
        {clips.map((clip) => (
          <div
            key={clip.id}
            className={`animation-clip-item ${clip.id === currentClipId ? 'selected' : ''}`}
            onClick={() => onClipSelect(clip.id)}
            onContextMenu={(e) => {
              e.preventDefault();
              setMenuClipId(clip.id);
              setShowMenu(true);
            }}
          >
            <div className="clip-icon">🎬</div>
            <div className="clip-info">
              <div className="clip-name">{clip.name}</div>
              <div className="clip-details">
                {formatDuration(clip.duration)} • {clip.frameRate}fps • {clip.tracks.length} tracks
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* 新建动画对话框 */}
      {showNewDialog && (
        <div className="dialog-overlay" onClick={() => setShowNewDialog(false)}>
          <div className="dialog" onClick={(e) => e.stopPropagation()}>
            <div className="dialog-header">
              <span className="dialog-title">新建动画剪辑</span>
              <button
                className="dialog-close"
                onClick={() => setShowNewDialog(false)}
              >
                ×
              </button>
            </div>
            <div className="dialog-content">
              <input
                type="text"
                className="dialog-input"
                placeholder="动画名称"
                value={newClipName}
                onChange={(e) => setNewClipName(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleCreateClip();
                  if (e.key === 'Escape') setShowNewDialog(false);
                }}
                autoFocus
              />
            </div>
            <div className="dialog-footer">
              <button
                className="dialog-btn dialog-btn-secondary"
                onClick={() => setShowNewDialog(false)}
              >
                取消
              </button>
              <button
                className="dialog-btn dialog-btn-primary"
                onClick={handleCreateClip}
                disabled={!newClipName.trim()}
              >
                创建
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 右键菜单 */}
      {showMenu && menuClipId && (
        <div
          className="context-menu"
          onClick={() => setShowMenu(false)}
        >
          <div className="context-menu-content">
            <button
              className="context-menu-item"
              onClick={() => {
                const clip = clips.find(c => c.id === menuClipId);
                if (clip) handleDuplicateClip(clip);
              }}
            >
              📋 复制
            </button>
            <button
              className="context-menu-item"
              onClick={() => {
                const clip = clips.find(c => c.id === menuClipId);
                if (clip) handleExportClip(clip);
              }}
            >
              📤 导出
            </button>
            <button
              className="context-menu-item"
              onClick={() => {
                const newName = prompt('输入新名称：');
                if (newName) handleRenameClip(menuClipId, newName);
              }}
            >
              ✏️ 重命名
            </button>
            <div className="context-menu-separator" />
            <button
              className="context-menu-item context-menu-item-danger"
              onClick={() => handleDeleteClip(menuClipId)}
            >
              🗑️ 删除
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

export default AnimationManager;
