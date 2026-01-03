/**
 * 动画时间轴编辑器主组件
 */

import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Playhead } from './Playhead';
import { TimeRuler } from './TimeRuler';
import { TrackList } from './TrackList';
import { PlaybackControls } from './PlaybackControls';
import {
  AnimationClip,
  TimelineState,
  TrackType,
  createEmptyAnimationClip,
  createEmptyTrack,
  createKeyframe,
  InterpolationType,
  EasingFunction,
  Keyframe,
} from '../../types/animation';
import './Timeline.css';

export interface TimelineProps {
  clip?: AnimationClip;
  onClipChange?: (clip: AnimationClip) => void;
  currentTime?: number;
  onTimeChange?: (time: number) => void;
  isPlaying?: boolean;
  onPlayChange?: (isPlaying: boolean) => void;
}

export const Timeline: React.FC<TimelineProps> = ({
  clip: initialClip,
  onClipChange,
  currentTime: externalCurrentTime,
  onTimeChange,
  isPlaying: externalIsPlaying,
  onPlayChange,
}) => {
  // ==================== 状态管理 ====================

  // 动画剪辑
  const [clip, setClip] = useState<AnimationClip>(() =>
    initialClip || createEmptyAnimationClip('Animation 1')
  );

  // 时间轴状态
  const [timelineState, setTimelineState] = useState<TimelineState>({
    currentTime: 0,
    playbackSpeed: 1.0,
    isPlaying: false,
    isLooping: false,
    selection: {
      selectedTracks: new Set(),
      selectedKeyframes: new Set(),
      selectedCurves: new Set(),
    },
    zoom: 100, // 像素/秒
    scrollOffset: 0,
  });

  // 播放相关状态
  const [lastFrameTime, setLastFrameTime] = useState<number>(0);
  const animationFrameRef = useRef<number>();

  // DOM引用
  const timelineRef = useRef<HTMLDivElement>(null);
  const keyframeAreaRef = useRef<HTMLDivElement>(null);

  // ==================== 播放控制 ====================

  useEffect(() => {
    if (timelineState.isPlaying) {
      setLastFrameTime(performance.now());
      animationFrameRef.current = requestAnimationFrame(tick);
    } else {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    }

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [timelineState.isPlaying]);

  const tick = useCallback((currentTime: number) => {
    if (!timelineState.isPlaying) return;

    const deltaTime = (currentTime - lastFrameTime) / 1000; // 转换为秒
    setLastFrameTime(currentTime);

    setTimelineState((prev) => {
      let newTime = prev.currentTime + deltaTime * prev.playbackSpeed;

      // 循环播放
      if (prev.isLooping && newTime >= clip.duration) {
        newTime = 0;
      } else if (!prev.isLooping && newTime >= clip.duration) {
        // 播放结束
        onPlayChange?.(false);
        return { ...prev, isPlaying: false, currentTime: clip.duration };
      }

      onTimeChange?.(newTime);
      return { ...prev, currentTime: newTime };
    });

    animationFrameRef.current = requestAnimationFrame(tick);
  }, [timelineState.isPlaying, lastFrameTime, timelineState.playbackSpeed, clip.duration, timelineState.isLooping]);

  // ==================== 播放控制函数 ====================

  const handlePlayPause = useCallback(() => {
    const newState = !timelineState.isPlaying;
    setTimelineState((prev) => ({ ...prev, isPlaying: newState }));
    onPlayChange?.(newState);
  }, [timelineState.isPlaying, onPlayChange]);

  const handleStop = useCallback(() => {
    setTimelineState((prev) => ({
      ...prev,
      isPlaying: false,
      currentTime: 0,
    }));
    onPlayChange?.(false);
    onTimeChange?.(0);
  }, [onPlayChange, onTimeChange]);

  const handleSkipToStart = useCallback(() => {
    setTimelineState((prev) => ({ ...prev, currentTime: 0 }));
    onTimeChange?.(0);
  }, [onTimeChange]);

  const handleSkipToEnd = useCallback(() => {
    setTimelineState((prev) => ({ ...prev, currentTime: clip.duration }));
    onTimeChange?.(clip.duration);
  }, [clip.duration, onTimeChange]);

  const handleStepForward = useCallback(() => {
    const frameTime = 1 / clip.frameRate;
    setTimelineState((prev) => {
      const newTime = Math.min(prev.currentTime + frameTime, clip.duration);
      onTimeChange?.(newTime);
      return { ...prev, currentTime: newTime };
    });
  }, [clip.frameRate, clip.duration, onTimeChange]);

  const handleStepBackward = useCallback(() => {
    const frameTime = 1 / clip.frameRate;
    setTimelineState((prev) => {
      const newTime = Math.max(prev.currentTime - frameTime, 0);
      onTimeChange?.(newTime);
      return { ...prev, currentTime: newTime };
    });
  }, [clip.frameRate, onTimeChange]);

  const handleSpeedChange = useCallback((speed: number) => {
    setTimelineState((prev) => ({ ...prev, playbackSpeed: speed }));
  }, []);

  const handleLoopToggle = useCallback(() => {
    setTimelineState((prev) => ({ ...prev, isLooping: !prev.isLooping }));
  }, []);

  // ==================== 时间控制 ====================

  const handleTimeChange = useCallback((time: number) => {
    const clampedTime = Math.max(0, Math.min(time, clip.duration));
    setTimelineState((prev) => ({ ...prev, currentTime: clampedTime }));
    onTimeChange?.(clampedTime);
  }, [clip.duration, onTimeChange]);

  const handleZoomChange = useCallback((zoom: number) => {
    setTimelineState((prev) => ({ ...prev, zoom }));
  }, []);

  // ==================== 关键帧操作 ====================

  const handleAddKeyframe = useCallback((
    trackId: string,
    curveId: string,
    time: number,
    value: any
  ) => {
    setClip((prevClip) => {
      const newClip = { ...prevClip };
      const track = newClip.tracks.find((t) => t.id === trackId);
      if (!track) return prevClip;

      const curve = track.curves.find((c) => c.id === curveId);
      if (!curve) return prevClip;

      const newKeyframe = createKeyframe(time, value);
      curve.keyframes.push(newKeyframe);
      curve.keyframes.sort((a, b) => a.time - b.time);

      newClip.updatedAt = Date.now();
      onClipChange?.(newClip);
      return newClip;
    });
  }, [onClipChange]);

  const handleUpdateKeyframe = useCallback((
    keyframeId: string,
    updates: Partial<Keyframe>
  ) => {
    setClip((prevClip) => {
      const newClip = { ...prevClip };

      for (const track of newClip.tracks) {
        for (const curve of track.curves) {
          const keyframe = curve.keyframes.find((kf) => kf.id === keyframeId);
          if (keyframe) {
            Object.assign(keyframe, updates);
            curve.keyframes.sort((a, b) => a.time - b.time);
            newClip.updatedAt = Date.now();
            onClipChange?.(newClip);
            return newClip;
          }
        }
      }

      return prevClip;
    });
  }, [onClipChange]);

  const handleDeleteKeyframe = useCallback((keyframeId: string) => {
    setClip((prevClip) => {
      const newClip = { ...prevClip };

      for (const track of newClip.tracks) {
        for (const curve of track.curves) {
          const index = curve.keyframes.findIndex((kf) => kf.id === keyframeId);
          if (index !== -1) {
            curve.keyframes.splice(index, 1);
            newClip.updatedAt = Date.now();
            onClipChange?.(newClip);
            return newClip;
          }
        }
      }

      return prevClip;
    });
  }, [onClipChange]);

  // ==================== 选择操作 ====================

  const handleSelectKeyframe = useCallback((keyframeId: string, multiSelect: boolean = false) => {
    setTimelineState((prev) => {
      const newSelection = { ...prev.selection };

      if (multiSelect) {
        if (newSelection.selectedKeyframes.has(keyframeId)) {
          newSelection.selectedKeyframes.delete(keyframeId);
        } else {
          newSelection.selectedKeyframes.add(keyframeId);
        }
      } else {
        newSelection.selectedKeyframes.clear();
        newSelection.selectedKeyframes.add(keyframeId);
      }

      return { ...prev, selection: newSelection };
    });
  }, []);

  const handleClearSelection = useCallback(() => {
    setTimelineState((prev) => ({
      ...prev,
      selection: {
        selectedTracks: new Set(),
        selectedKeyframes: new Set(),
        selectedCurves: new Set(),
      },
    }));
  }, []);

  // ==================== 轨道操作 ====================

  const handleToggleTrackVisibility = useCallback((trackId: string) => {
    setClip((prevClip) => {
      const newClip = { ...prevClip };
      const track = newClip.tracks.find((t) => t.id === trackId);
      if (track) {
        track.visible = !track.visible;
        newClip.updatedAt = Date.now();
        onClipChange?.(newClip);
      }
      return newClip;
    });
  }, [onClipChange]);

  const handleToggleTrackLock = useCallback((trackId: string) => {
    setClip((prevClip) => {
      const newClip = { ...prevClip };
      const track = newClip.tracks.find((t) => t.id === trackId);
      if (track) {
        track.locked = !track.locked;
        newClip.updatedAt = Date.now();
        onClipChange?.(newClip);
      }
      return newClip;
    });
  }, [onClipChange]);

  const handleToggleTrackMute = useCallback((trackId: string) => {
    setClip((prevClip) => {
      const newClip = { ...prevClip };
      const track = newClip.tracks.find((t) => t.id === trackId);
      if (track) {
        track.muted = !track.muted;
        newClip.updatedAt = Date.now();
        onClipChange?.(newClip);
      }
      return newClip;
    });
  }, [onClipChange]);

  const handleToggleTrackExpanded = useCallback((trackId: string) => {
    setClip((prevClip) => {
      const newClip = { ...prevClip };
      const track = newClip.tracks.find((t) => t.id === trackId);
      if (track) {
        track.expanded = !track.expanded;
        newClip.updatedAt = Date.now();
        onClipChange?.(newClip);
      }
      return newClip;
    });
  }, [onClipChange]);

  // ==================== 键盘快捷键 ====================

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // 如果焦点在输入框中，不处理快捷键
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
        return;
      }

      switch (e.key) {
        case ' ':
          e.preventDefault();
          handlePlayPause();
          break;
        case 'Home':
          e.preventDefault();
          handleSkipToStart();
          break;
        case 'End':
          e.preventDefault();
          handleSkipToEnd();
          break;
        case 'ArrowLeft':
          e.preventDefault();
          if (e.shiftKey) {
            // Shift+Left: 前一秒
            handleTimeChange(timelineState.currentTime - 1);
          } else {
            // Left: 前一帧
            handleStepBackward();
          }
          break;
        case 'ArrowRight':
          e.preventDefault();
          if (e.shiftKey) {
            // Shift+Right: 后一秒
            handleTimeChange(timelineState.currentTime + 1);
          } else {
            // Right: 后一帧
            handleStepForward();
          }
          break;
        case 'Delete':
        case 'Backspace':
          e.preventDefault();
          // 删除选中的关键帧
          timelineState.selection.selectedKeyframes.forEach((keyframeId) => {
            handleDeleteKeyframe(keyframeId);
          });
          handleClearSelection();
          break;
        case 'k':
        case 'K':
          e.preventDefault();
          // K键添加关键帧（需要在选中轨道上）
          break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [
    timelineState.currentTime,
    timelineState.selection.selectedKeyframes,
    handlePlayPause,
    handleSkipToStart,
    handleSkipToEnd,
    handleStepForward,
    handleStepBackward,
    handleTimeChange,
    handleDeleteKeyframe,
    handleClearSelection,
  ]);

  // ==================== 渲染 ====================

  return (
    <div className="timeline-container" ref={timelineRef}>
      {/* 顶部工具栏 */}
      <div className="timeline-header">
        <div className="timeline-title">
          <span className="timeline-icon">🎬</span>
          <span className="timeline-name">{clip.name}</span>
        </div>
        <button
          className="timeline-close-btn"
          onClick={() => {
            // TODO: 关闭时间轴
          }}
        >
          ×
        </button>
      </div>

      {/* 播放控制 */}
      <PlaybackControls
        isPlaying={timelineState.isPlaying}
        isLooping={timelineState.isLooping}
        playbackSpeed={timelineState.playbackSpeed}
        currentTime={timelineState.currentTime}
        duration={clip.duration}
        onPlayPause={handlePlayPause}
        onStop={handleStop}
        onSkipToStart={handleSkipToStart}
        onSkipToEnd={handleSkipToEnd}
        onStepForward={handleStepForward}
        onStepBackward={handleStepBackward}
        onSpeedChange={handleSpeedChange}
        onLoopToggle={handleLoopToggle}
      />

      {/* 时间标尺 */}
      <TimeRuler
        duration={clip.duration}
        zoom={timelineState.zoom}
        scrollOffset={timelineState.scrollOffset}
        currentTime={timelineState.currentTime}
        onTimeChange={handleTimeChange}
        onZoomChange={handleZoomChange}
      />

      {/* 主要内容区域 */}
      <div className="timeline-content">
        {/* 轨道列表 */}
        <TrackList
          tracks={clip.tracks}
          zoom={timelineState.zoom}
          scrollOffset={timelineState.scrollOffset}
          currentTime={timelineState.currentTime}
          selectedKeyframes={timelineState.selection.selectedKeyframes}
          onAddKeyframe={handleAddKeyframe}
          onUpdateKeyframe={handleUpdateKeyframe}
          onDeleteKeyframe={handleDeleteKeyframe}
          onSelectKeyframe={handleSelectKeyframe}
          onToggleVisibility={handleToggleTrackVisibility}
          onToggleLock={handleToggleTrackLock}
          onToggleMute={handleToggleTrackMute}
          onToggleExpanded={handleToggleTrackExpanded}
        />

        {/* 关键帧区域 */}
        <div
          className="keyframe-area"
          ref={keyframeAreaRef}
          onClick={handleClearSelection}
        >
          {clip.tracks.map((track) =>
            track.expanded && track.visible && !track.locked ? (
              <React.Fragment key={track.id}>
                {track.curves.map((curve) => (
                  <div
                    key={curve.id}
                    className="curve-row"
                    style={{
                      borderColor: curve.color,
                    }}
                  >
                    {/* 关键帧将由CurveRow子组件渲染 */}
                    {curve.keyframes.map((keyframe) => (
                      <div
                        key={keyframe.id}
                        className={`keyframe ${
                          timelineState.selection.selectedKeyframes.has(keyframe.id)
                            ? 'selected'
                            : ''
                        }`}
                        style={{
                          left: `${keyframe.time * timelineState.zoom}px`,
                          backgroundColor: curve.color,
                        }}
                        onClick={(e) => {
                          e.stopPropagation();
                          handleSelectKeyframe(keyframe.id, e.shiftKey);
                        }}
                      >
                        ◆
                      </div>
                    ))}
                  </div>
                ))}
              </React.Fragment>
            ) : null
          )}

          {/* 播放头 */}
          <Playhead
            currentTime={timelineState.currentTime}
            zoom={timelineState.zoom}
          />
        </div>
      </div>

      {/* 底部状态栏 */}
      <div className="timeline-footer">
        <div className="timeline-status">
          <span>Time: {timelineState.currentTime.toFixed(2)}s / {clip.duration.toFixed(2)}s</span>
          <span> | </span>
          <span>Speed: {timelineState.playbackSpeed}x</span>
          <span> | </span>
          <span>Zoom: {timelineState.zoom}px/s</span>
        </div>
        <div className="timeline-info">
          <span>{clip.tracks.length} tracks</span>
          <span> | </span>
          <span>
            {clip.tracks.reduce((sum, track) =>
              sum + track.curves.reduce((s, curve) => s + curve.keyframes.length, 0), 0
            )} keyframes
          </span>
        </div>
      </div>
    </div>
  );
};

export default Timeline;
