/**
 * 播放控制组件
 */

import React from 'react';
import './PlaybackControls.css';

export interface PlaybackControlsProps {
  isPlaying: boolean;
  isLooping: boolean;
  playbackSpeed: number;
  currentTime: number;
  duration: number;
  onPlayPause: () => void;
  onStop: () => void;
  onSkipToStart: () => void;
  onSkipToEnd: () => void;
  onStepForward: () => void;
  onStepBackward: () => void;
  onSpeedChange: (speed: number) => void;
  onLoopToggle: () => void;
}

export const PlaybackControls: React.FC<PlaybackControlsProps> = ({
  isPlaying,
  isLooping,
  playbackSpeed,
  currentTime,
  duration,
  onPlayPause,
  onStop,
  onSkipToStart,
  onSkipToEnd,
  onStepForward,
  onStepBackward,
  onSpeedChange,
  onLoopToggle,
}) => {
  const formatTime = (time: number): string => {
    const minutes = Math.floor(time / 60);
    const seconds = Math.floor(time % 60);
    const frames = Math.floor((time % 1) * 60);
    return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}:${frames.toString().padStart(2, '0')}`;
  };

  return (
    <div className="timeline-playback-controls">
      {/* 播放控制按钮 */}
      <button
        className="playback-btn"
        onClick={onSkipToStart}
        title="跳到开始 (Home)"
      >
        ⏮️
      </button>

      <button
        className="playback-btn"
        onClick={onStepBackward}
        title="前一帧 (←)"
      >
        ⏪
      </button>

      <button
        className={`playback-btn primary ${isPlaying ? 'active' : ''}`}
        onClick={onPlayPause}
        title={isPlaying ? '暂停 (Space)' : '播放 (Space)'}
      >
        {isPlaying ? '⏸️' : '▶️'}
      </button>

      <button
        className="playback-btn"
        onClick={onStop}
        title="停止"
      >
        ⏹️
      </button>

      <button
        className="playback-btn"
        onClick={onStepForward}
        title="后一帧 (→)"
      >
        ⏩
      </button>

      <button
        className="playback-btn"
        onClick={onSkipToEnd}
        title="跳到结束 (End)"
      >
        ⏭️
      </button>

      {/* 循环按钮 */}
      <button
        className={`playback-btn ${isLooping ? 'active' : ''}`}
        onClick={onLoopToggle}
        title={isLooping ? '循环播放: 开' : '循环播放: 关'}
      >
        🔂
      </button>

      {/* 播放速度选择 */}
      <div className="playback-speed-selector">
        <span className="playback-speed-label">速度:</span>
        <select
          className="playback-speed-select"
          value={playbackSpeed}
          onChange={(e) => onSpeedChange(parseFloat(e.target.value))}
        >
          <option value={0.1}>0.1x</option>
          <option value={0.25}>0.25x</option>
          <option value={0.5}>0.5x</option>
          <option value={1.0}>1.0x</option>
          <option value={1.5}>1.5x</option>
          <option value={2.0}>2.0x</option>
        </select>
      </div>

      {/* 时间显示 */}
      <div className="playback-time-display">
        <span className="playback-current-time">{formatTime(currentTime)}</span>
        <span>/</span>
        <span className="playback-total-time">{formatTime(duration)}</span>
      </div>
    </div>
  );
};

export default PlaybackControls;
