/**
 * 关键帧属性面板组件
 */

import React, { useState, useEffect } from 'react';
import { Keyframe, InterpolationType, EasingFunction } from '../../types/animation';
import './PropertiesPanel.css';

export interface PropertiesPanelProps {
  keyframes: Keyframe[];
  selectedKeyframeIds: Set<string>;
  onUpdateKeyframe: (keyframeId: string, updates: Partial<Keyframe>) => void;
}

export const PropertiesPanel: React.FC<PropertiesPanelProps> = ({
  keyframes,
  selectedKeyframeIds,
  onUpdateKeyframe,
}) => {
  const [editingValue, setEditingValue] = useState<string>('');

  const selectedKeyframes = keyframes.filter(kf => selectedKeyframeIds.has(kf.id));

  if (selectedKeyframes.length === 0) {
    return (
      <div className="properties-panel">
        <div className="properties-empty">
          <div className="empty-icon">🔧</div>
          <div className="empty-title">未选择关键帧</div>
          <div className="empty-description">点击关键帧以编辑其属性</div>
        </div>
      </div>
    );
  }

  const keyframe = selectedKeyframes[0]; // 编辑第一个选中的关键帧
  const isMultiple = selectedKeyframes.length > 1;

  const formatValue = (value: any): string => {
    if (typeof value === 'number') {
      return value.toFixed(3);
    } else if (typeof value === 'object') {
      if ('w' in value) {
        // Quaternion
        return `(${value.x.toFixed(3)}, ${value.y.toFixed(3)}, ${value.z.toFixed(3)}, ${value.w.toFixed(3)})`;
      } else {
        // Vector3
        return `(${value.x.toFixed(3)}, ${value.y.toFixed(3)}, ${value.z.toFixed(3)})`;
      }
    }
    return String(value);
  };

  const parseValue = (valueStr: string): any => {
    const trimmed = valueStr.trim();

    // 尝试解析为数字
    const num = parseFloat(trimmed);
    if (!isNaN(num)) {
      return num;
    }

    // 尝试解析为Vector3或Quaternion
    const match = trimmed.match(/\(([^,]+),([^,]+),([^,]+)(?:,([^,]+))?\)/);
    if (match) {
      const values = match.slice(1).map(v => parseFloat(v.trim()));
      if (values.length === 4 && !isNaN(values[3])) {
        // Quaternion
        return { x: values[0], y: values[1], z: values[2], w: values[3] };
      } else if (values.length >= 3) {
        // Vector3
        return { x: values[0], y: values[1], z: values[2] };
      }
    }

    return trimmed;
  };

  return (
    <div className="properties-panel">
      <div className="properties-header">
        <span className="properties-title">关键帧属性</span>
        {isMultiple && (
          <span className="properties-count">
            ({selectedKeyframes.length} 个已选择)
          </span>
        )}
      </div>

      <div className="properties-content">
        {/* 时间 */}
        <div className="property-row">
          <label className="property-label">时间 (秒)</label>
          <input
            type="number"
            className="property-input"
            value={keyframe.time.toFixed(3)}
            onChange={(e) => {
              const time = parseFloat(e.target.value);
              if (!isNaN(time)) {
                onUpdateKeyframe(keyframe.id, { time });
              }
            }}
            step="0.001"
            min="0"
          />
        </div>

        {/* 值 */}
        <div className="property-row">
          <label className="property-label">值</label>
          <input
            type="text"
            className="property-input"
            value={editingValue || formatValue(keyframe.value)}
            onChange={(e) => setEditingValue(e.target.value)}
            onBlur={(e) => {
              const value = parseValue(e.target.value);
              onUpdateKeyframe(keyframe.id, { value });
              setEditingValue('');
            }}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                const value = parseValue(editingValue || formatValue(keyframe.value));
                onUpdateKeyframe(keyframe.id, { value });
                setEditingValue('');
              } else if (e.key === 'Escape') {
                setEditingValue('');
              }
            }}
          />
        </div>

        {/* 插值类型 */}
        <div className="property-row">
          <label className="property-label">插值</label>
          <select
            className="property-select"
            value={keyframe.interpolation}
            onChange={(e) => {
              onUpdateKeyframe(keyframe.id, {
                interpolation: e.target.value as InterpolationType,
              });
            }}
          >
            <option value={InterpolationType.Constant}>常量</option>
            <option value={InterpolationType.Linear}>线性</option>
            <option value={InterpolationType.Cubic}>三次样条</option>
            <option value={InterpolationType.Hermite}>Hermite</option>
          </select>
        </div>

        {/* Easing函数 */}
        <div className="property-row">
          <label className="property-label">缓动</label>
          <select
            className="property-select"
            value={keyframe.easing}
            onChange={(e) => {
              onUpdateKeyframe(keyframe.id, {
                easing: e.target.value as EasingFunction,
              });
            }}
          >
            <option value={EasingFunction.Linear}>Linear</option>
            <option value={EasingFunction.EaseInQuad}>Ease In Quad</option>
            <option value={EasingFunction.EaseOutQuad}>Ease Out Quad</option>
            <option value={EasingFunction.EaseInOutQuad}>Ease In Out Quad</option>
            <option value={EasingFunction.EaseInCubic}>Ease In Cubic</option>
            <option value={EasingFunction.EaseOutCubic}>Ease Out Cubic</option>
            <option value={EasingFunction.EaseInOutCubic}>Ease In Out Cubic</option>
            <option value={EasingFunction.EaseInElastic}>Ease In Elastic</option>
            <option value={EasingFunction.EaseOutElastic}>Ease Out Elastic</option>
            <option value={EasingFunction.EaseOutBounce}>Ease Out Bounce</option>
          </select>
        </div>

        {/* ID（只读） */}
        <div className="property-row">
          <label className="property-label">ID</label>
          <input
            type="text"
            className="property-input"
            value={keyframe.id}
            disabled
            title="关键帧唯一标识符"
          />
        </div>
      </div>
    </div>
  );
};

export default PropertiesPanel;
