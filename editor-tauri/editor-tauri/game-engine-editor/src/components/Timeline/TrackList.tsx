/**
 * 轨道列表组件
 */

import React from 'react';
import { AnimationTrack } from '../../types/animation';
import { Track } from './Track';
import './TrackList.css';

export interface TrackListProps {
  tracks: AnimationTrack[];
  zoom: number;
  scrollOffset: number;
  currentTime: number;
  selectedKeyframes: Set<string>;
  onAddKeyframe: (
    trackId: string,
    curveId: string,
    time: number,
    value: any
  ) => void;
  onUpdateKeyframe: (keyframeId: string, updates: any) => void;
  onDeleteKeyframe: (keyframeId: string) => void;
  onSelectKeyframe: (keyframeId: string, multiSelect: boolean) => void;
  onToggleVisibility: (trackId: string) => void;
  onToggleLock: (trackId: string) => void;
  onToggleMute: (trackId: string) => void;
  onToggleExpanded: (trackId: string) => void;
}

export const TrackList: React.FC<TrackListProps> = ({
  tracks,
  zoom,
  scrollOffset,
  currentTime,
  selectedKeyframes,
  onAddKeyframe,
  onUpdateKeyframe,
  onDeleteKeyframe,
  onSelectKeyframe,
  onToggleVisibility,
  onToggleLock,
  onToggleMute,
  onToggleExpanded,
}) => {
  return (
    <div className="timeline-track-list">
      {tracks.map((track) => (
        <Track
          key={track.id}
          track={track}
          zoom={zoom}
          scrollOffset={scrollOffset}
          currentTime={currentTime}
          selectedKeyframes={selectedKeyframes}
          onAddKeyframe={onAddKeyframe}
          onUpdateKeyframe={onUpdateKeyframe}
          onDeleteKeyframe={onDeleteKeyframe}
          onSelectKeyframe={onSelectKeyframe}
          onToggleVisibility={onToggleVisibility}
          onToggleLock={onToggleLock}
          onToggleMute={onToggleMute}
          onToggleExpanded={onToggleExpanded}
        />
      ))}

      {/* 空状态提示 */}
      {tracks.length === 0 && (
        <div className="timeline-tracks-empty">
          <div className="empty-icon">🎬</div>
          <div className="empty-title">暂无动画轨道</div>
          <div className="empty-description">
            选择一个实体并添加动画属性开始创作
          </div>
        </div>
      )}
    </div>
  );
};

export default TrackList;
