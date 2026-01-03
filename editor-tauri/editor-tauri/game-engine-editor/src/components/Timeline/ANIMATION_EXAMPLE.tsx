/**
 * Timeline组件使用示例
 *
 * 这个文件展示了如何使用Timeline组件创建和管理动画
 */

import React, { useState } from 'react';
import { Timeline } from './Timeline';
import {
  AnimationClip,
  AnimationTrack,
  AnimationCurve,
  Keyframe,
  TrackType,
  InterpolationType,
  EasingFunction,
  createEmptyAnimationClip,
  createEmptyTrack,
  createKeyframe,
} from '../../types/animation';

export const AnimationExample: React.FC = () => {
  // 创建一个示例动画剪辑
  const createExampleAnimation = (): AnimationClip => {
    const clip = createEmptyAnimationClip('Cube Animation');

    // 创建Position轨道
    const positionTrack: AnimationTrack = {
      ...createEmptyTrack('Cube', TrackType.Transform, 'cube_entity'),
      curves: [
        {
          id: 'curve_x',
          name: 'Position X',
          propertyPath: 'transform.position.x',
          keyframes: [
            createKeyframe(0, 0, InterpolationType.Linear, EasingFunction.EaseInOutQuad),
            createKeyframe(2, 5, InterpolationType.Cubic, EasingFunction.EaseOutBounce),
            createKeyframe(4, 0, InterpolationType.Linear, EasingFunction.EaseInOutQuad),
          ],
          color: '#ef4444',
          valueType: 'number',
        },
        {
          id: 'curve_y',
          name: 'Position Y',
          propertyPath: 'transform.position.y',
          keyframes: [
            createKeyframe(0, 0),
            createKeyframe(1, 3, InterpolationType.Cubic, EasingFunction.EaseOutCubic),
            createKeyframe(3, 3),
            createKeyframe(4, 0, InterpolationType.Cubic, EasingFunction.EaseInCubic),
          ],
          color: '#ef4444',
          valueType: 'number',
        },
      ],
    };

    // 创建Rotation轨道
    const rotationTrack: AnimationTrack = {
      ...createEmptyTrack('Cube', TrackType.Rotation, 'cube_entity'),
      curves: [
        {
          id: 'curve_rot_y',
          name: 'Rotation Y',
          propertyPath: 'transform.rotation.y',
          keyframes: [
            createKeyframe(0, 0),
            createKeyframe(4, Math.PI * 2, InterpolationType.Linear, EasingFunction.Linear),
          ],
          color: '#22c55e',
          valueType: 'number',
        },
      ],
    };

    // 创建Scale轨道
    const scaleTrack: AnimationTrack = {
      ...createEmptyTrack('Cube', TrackType.Scale, 'cube_entity'),
      curves: [
        {
          id: 'curve_scale',
          name: 'Scale',
          propertyPath: 'transform.scale.x',
          keyframes: [
            createKeyframe(0, 1),
            createKeyframe(2, 1.5, InterpolationType.Cubic, EasingFunction.EaseInOutElastic),
            createKeyframe(4, 1, InterpolationType.Cubic, EasingFunction.EaseInOutElastic),
          ],
          color: '#3b82f6',
          valueType: 'number',
        },
      ],
    };

    clip.tracks = [positionTrack, rotationTrack, scaleTrack];
    return clip;
  };

  const [clip, setClip] = useState<AnimationClip>(createExampleAnimation());
  const [currentTime, setCurrentTime] = useState(0);
  const [isPlaying, setIsPlaying] = useState(false);

  return (
    <div style={{ width: '100%', height: '100vh' }}>
      <Timeline
        clip={clip}
        onClipChange={setClip}
        currentTime={currentTime}
        onTimeChange={setCurrentTime}
        isPlaying={isPlaying}
        onPlayChange={setIsPlaying}
      />

      {/* 显示当前动画状态 */}
      <div style={{
        position: 'fixed',
        top: '20px',
        right: '20px',
        background: '#1e293b',
        padding: '16px',
        borderRadius: '8px',
        color: '#e2e8f0',
        fontFamily: 'monospace',
        fontSize: '12px',
      }}>
        <div><strong>Animation State:</strong></div>
        <div>Time: {currentTime.toFixed(2)}s</div>
        <div>Playing: {isPlaying ? 'Yes' : 'No'}</div>
        <div>Tracks: {clip.tracks.length}</div>
        <div>
          Keyframes: {clip.tracks.reduce(
            (sum, track) => sum + track.curves.reduce(
              (s, curve) => s + curve.keyframes.length,
              0
            ),
            0
          )}
        </div>
      </div>
    </div>
  );
};

export default AnimationExample;
