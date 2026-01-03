/**
 * 单个轨道组件
 */

import React from 'react';
import { AnimationTrack, TrackType } from '../../types/animation';
import './Track.css';

export interface TrackProps {
  track: AnimationTrack;
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

export const Track: React.FC<TrackProps> = ({
  track,
  selectedKeyframes,
  onToggleVisibility,
  onToggleLock,
  onToggleMute,
  onToggleExpanded,
}) => {
  const getTrackIcon = (type: TrackType): string => {
    switch (type) {
      case TrackType.Transform:
        return '🔄';
      case TrackType.Rotation:
        return '🔃';
      case TrackType.Scale:
        return '⏱️';
      case TrackType.Property:
        return '📊';
      case TrackType.Event:
        return '⚡';
      default:
        return '🎬';
    }
  };

  return (
    <div
      className={`timeline-track-item ${track.locked ? 'locked' : ''} ${track.muted ? 'muted' : ''}`}
    >
      {/* 轨道头部 */}
      <div
        className={`timeline-track-header ${track.muted ? 'muted' : ''}`}
        style={{ borderLeftColor: track.color }}
      >
        {/* 展开/收起图标 */}
        <div
          className={`timeline-track-expand-icon ${track.expanded ? 'expanded' : ''}`}
          onClick={() => onToggleExpanded(track.id)}
        >
          ▶
        </div>

        {/* 可见性切换 */}
        <div
          className="timeline-track-icon"
          onClick={() => onToggleVisibility(track.id)}
          title={track.visible ? '隐藏轨道' : '显示轨道'}
          style={{ opacity: track.visible ? 1 : 0.3 }}
        >
          {track.visible ? '👁️' : '🚫'}
        </div>

        {/* 锁定切换 */}
        <div
          className="timeline-track-icon"
          onClick={() => onToggleLock(track.id)}
          title={track.locked ? '解锁轨道' : '锁定轨道'}
          style={{ opacity: track.locked ? 1 : 0.3 }}
        >
          {track.locked ? '🔒' : '🔓'}
        </div>

        {/* 静音切换 */}
        <div
          className="timeline-track-icon"
          onClick={() => onToggleMute(track.id)}
          title={track.muted ? '取消静音' : '静音轨道'}
          style={{ opacity: track.muted ? 1 : 0.3 }}
        >
          {track.muted ? '🔇' : '🔊'}
        </div>

        {/* 轨道图标 */}
        <div className="timeline-track-icon" style={{ color: track.color }}>
          {getTrackIcon(track.type)}
        </div>

        {/* 轨道名称 */}
        <div className="timeline-track-name" title={track.name}>
          {track.name}
        </div>

        {/* 关键帧数量 */}
        <div className="timeline-track-keyframes-count">
          {track.curves.reduce((sum, curve) => sum + curve.keyframes.length, 0)}
        </div>
      </div>

      {/* 曲线列表 */}
      {track.expanded && (
        <div className="timeline-track-curves">
          {track.curves.map((curve) => (
            <div
              key={curve.id}
              className="timeline-track-curve-item"
              style={{ borderLeftColor: curve.color }}
              title={curve.propertyPath}
            >
              <div
                className="curve-color-indicator"
                style={{ backgroundColor: curve.color }}
              />
              <span className="curve-name">{curve.name}</span>
              <span className="curve-keyframes-count">
                {curve.keyframes.length}
              </span>
            </div>
          ))}

          {track.curves.length === 0 && (
            <div className="curve-empty">
              <span>暂无属性曲线</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default Track;
