/**
 * 曲线编辑器组件
 */

import React, { useRef, useEffect, useCallback, useState } from 'react';
import { AnimationCurve, Keyframe, InterpolationType, EasingFunction } from '../../types/animation';
import './CurveEditor.css';

export interface CurveEditorProps {
  curves: AnimationCurve[];
  selectedCurves: Set<string>;
  onCurveSelect: (curveIds: Set<string>) => void;
  onKeyframeUpdate: (keyframeId: string, updates: Partial<Keyframe>) => void;
}

export const CurveEditor: React.FC<CurveEditorProps> = ({
  curves,
  selectedCurves,
  onCurveSelect,
  onKeyframeUpdate,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [viewState, setViewState] = useState({
    minTime: 0,
    maxTime: 5,
    minValue: -2,
    maxValue: 2,
    showTangents: true,
    showGrid: true,
  });
  const [draggingKeyframe, setDraggingKeyframe] = useState<{
    curveId: string;
    keyframeId: string;
    type: 'position' | 'inTangent' | 'outTangent';
  } | null>(null);
  const [hoverKeyframe, setHoverKeyframe] = useState<{
    curveId: string;
    keyframeId: string;
  } | null>(null);

  // 绘制曲线编辑器
  const drawEditor = useCallback(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const width = container.clientWidth;
    const height = container.clientHeight;

    // 设置canvas实际大小
    canvas.width = width * window.devicePixelRatio;
    canvas.height = height * window.devicePixelRatio;
    ctx.scale(window.devicePixelRatio, window.devicePixelRatio);

    // 清空画布
    ctx.clearRect(0, 0, width, height);

    // 绘制网格
    if (viewState.showGrid) {
      drawGrid(ctx, width, height);
    }

    // 绘制所有曲线
    curves.forEach((curve) => {
      if (selectedCurves.size > 0 && !selectedCurves.has(curve.id)) {
        // 如果有选中的曲线，未选中的曲线半透明显示
        ctx.globalAlpha = 0.3;
      } else {
        ctx.globalAlpha = 1.0;
      }

      drawCurve(ctx, curve, width, height);
    });

    // 绘制关键帧
    curves.forEach((curve) => {
      if (selectedCurves.size > 0 && !selectedCurves.has(curve.id)) {
        ctx.globalAlpha = 0.3;
      } else {
        ctx.globalAlpha = 1.0;
      }

      drawKeyframes(ctx, curve, width, height);
    });
  }, [curves, selectedCurves, viewState]);

  // 绘制网格
  const drawGrid = (
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number
  ) => {
    ctx.strokeStyle = '#1e293b';
    ctx.lineWidth = 1;

    const padding = { top: 20, right: 20, bottom: 30, left: 50 };
    const graphWidth = width - padding.left - padding.right;
    const graphHeight = height - padding.top - padding.bottom;

    // 垂直线（时间）
    const timeStep = calculateStep(viewState.maxTime - viewState.minTime);
    for (let time = Math.ceil(viewState.minTime / timeStep) * timeStep;
         time <= viewState.maxTime;
         time += timeStep) {
      const x = padding.left + ((time - viewState.minTime) / (viewState.maxTime - viewState.minTime)) * graphWidth;

      ctx.beginPath();
      ctx.moveTo(x, padding.top);
      ctx.lineTo(x, height - padding.bottom);
      ctx.stroke();

      // 时间标签
      ctx.fillStyle = '#64748b';
      ctx.font = '10px Monaco, Menlo, Ubuntu Mono, monospace';
      ctx.textAlign = 'center';
      ctx.fillText(time.toFixed(1) + 's', x, height - 10);
    }

    // 水平线（值）
    const valueStep = calculateStep(viewState.maxValue - viewState.minValue);
    for (let value = Math.ceil(viewState.minValue / valueStep) * valueStep;
         value <= viewState.maxValue;
         value += valueStep) {
      const y = height - padding.bottom - ((value - viewState.minValue) / (viewState.maxValue - viewState.minValue)) * graphHeight;

      ctx.beginPath();
      ctx.moveTo(padding.left, y);
      ctx.lineTo(width - padding.right, y);
      ctx.stroke();

      // 值标签
      ctx.fillStyle = '#64748b';
      ctx.font = '10px Monaco, Menlo, Ubuntu Mono, monospace';
      ctx.textAlign = 'right';
      ctx.fillText(value.toFixed(1), padding.left - 8, y + 3);
    }

    // 零线
    const zeroY = height - padding.bottom - ((0 - viewState.minValue) / (viewState.maxValue - viewState.minValue)) * graphHeight;
    if (zeroY >= padding.top && zeroY <= height - padding.bottom) {
      ctx.strokeStyle = '#475569';
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(padding.left, zeroY);
      ctx.lineTo(width - padding.right, zeroY);
      ctx.stroke();
    }
  };

  // 计算合适的步长
  const calculateStep = (range: number): number => {
    const roughStep = range / 10;
    const magnitude = Math.pow(10, Math.floor(Math.log10(roughStep)));
    const normalizedStep = roughStep / magnitude;

    if (normalizedStep < 2) return 1 * magnitude;
    if (normalizedStep < 5) return 2 * magnitude;
    return 5 * magnitude;
  };

  // 绘制曲线
  const drawCurve = (
    ctx: CanvasRenderingContext2D,
    curve: AnimationCurve,
    width: number,
    height: number
  ) => {
    const padding = { top: 20, right: 20, bottom: 30, left: 50 };
    const graphWidth = width - padding.left - padding.right;
    const graphHeight = height - padding.top - padding.bottom;

    if (curve.keyframes.length < 2) return;

    ctx.strokeStyle = curve.color;
    ctx.lineWidth = 2;
    ctx.beginPath();

    curve.keyframes.forEach((keyframe, index) => {
      const x = padding.left + ((keyframe.time - viewState.minTime) / (viewState.maxTime - viewState.minTime)) * graphWidth;
      let value: number;

      if (typeof keyframe.value === 'number') {
        value = keyframe.value as number;
      } else if ('x' in keyframe.value) {
        value = keyframe.value.x;
      } else {
        return;
      }

      const y = height - padding.bottom - ((value - viewState.minValue) / (viewState.maxValue - viewState.minValue)) * graphHeight;

      if (index === 0) {
        ctx.moveTo(x, y);
      } else {
        // 根据插值类型绘制曲线
        if (keyframe.interpolation === InterpolationType.Linear) {
          ctx.lineTo(x, y);
        } else if (keyframe.interpolation === InterpolationType.Cubic) {
          // 简化的三次样条
          ctx.lineTo(x, y);
        } else {
          // Constant
          const prevKf = curve.keyframes[index - 1];
          const prevX = padding.left + ((prevKf.time - viewState.minTime) / (viewState.maxTime - viewState.minTime)) * graphWidth;
          ctx.lineTo(x, y - 0.01);
          ctx.moveTo(x, y);
        }
      }
    });

    ctx.stroke();
  };

  // 绘制关键帧
  const drawKeyframes = (
    ctx: CanvasRenderingContext2D,
    curve: AnimationCurve,
    width: number,
    height: number
  ) => {
    const padding = { top: 20, right: 20, bottom: 30, left: 50 };
    const graphWidth = width - padding.left - padding.right;
    const graphHeight = height - padding.top - padding.bottom;

    curve.keyframes.forEach((keyframe) => {
      const x = padding.left + ((keyframe.time - viewState.minTime) / (viewState.maxTime - viewState.minTime)) * graphWidth;
      let value: number;

      if (typeof keyframe.value === 'number') {
        value = keyframe.value as number;
      } else if ('x' in keyframe.value) {
        value = keyframe.value.x;
      } else {
        return;
      }

      const y = height - padding.bottom - ((value - viewState.minValue) / (viewState.maxValue - viewState.minValue)) * graphHeight;

      const isHovered = hoverKeyframe?.keyframeId === keyframe.id;

      // 绘制关键帧点
      ctx.fillStyle = curve.color;
      ctx.strokeStyle = '#ffffff';
      ctx.lineWidth = 2;

      const size = isHovered ? 8 : 6;

      ctx.beginPath();
      ctx.arc(x, y, size, 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();

      // 绘制切线手柄
      if (viewState.showTangents && isHovered && keyframe.interpolation === InterpolationType.Cubic) {
        ctx.strokeStyle = '#f59e0b';
        ctx.lineWidth = 1;

        // 入切线
        if (keyframe.inTangent) {
          ctx.beginPath();
          ctx.moveTo(x, y);
          ctx.lineTo(x - 20, y - 20);
          ctx.stroke();

          ctx.beginPath();
          ctx.arc(x - 20, y - 20, 3, 0, Math.PI * 2);
          ctx.fill();
        }

        // 出切线
        if (keyframe.outTangent) {
          ctx.beginPath();
          ctx.moveTo(x, y);
          ctx.lineTo(x + 20, y + 20);
          ctx.stroke();

          ctx.beginPath();
          ctx.arc(x + 20, y + 20, 3, 0, Math.PI * 2);
          ctx.fill();
        }
      }
    });
  };

  // 处理鼠标事件
  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    const rect = containerRef.current?.getBoundingClientRect();
    if (!rect) return;

    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    // 检查是否悬停在关键帧上
    // TODO: 实现关键帧检测逻辑

    if (draggingKeyframe) {
      // 拖拽关键帧
      // TODO: 实现拖拽逻辑
    }
  }, [draggingKeyframe, viewState]);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    // TODO: 实现点击检测
  }, []);

  const handleMouseUp = useCallback(() => {
    setDraggingKeyframe(null);
  }, []);

  // 绘制和更新
  useEffect(() => {
    drawEditor();
  }, [drawEditor]);

  return (
    <div className="curve-editor" ref={containerRef}>
      <canvas
        ref={canvasRef}
        className="curve-editor-canvas"
        onMouseMove={handleMouseMove}
        onMouseDown={handleMouseDown}
        onMouseUp={handleMouseUp}
      />

      {/* 工具栏 */}
      <div className="curve-editor-toolbar">
        <button
          className={`toolbar-btn ${viewState.showGrid ? 'active' : ''}`}
          onClick={() => setViewState(prev => ({ ...prev, showGrid: !prev.showGrid }))}
          title="显示网格"
        >
          📐
        </button>
        <button
          className={`toolbar-btn ${viewState.showTangents ? 'active' : ''}`}
          onClick={() => setViewState(prev => ({ ...prev, showTangents: !prev.showTangents }))}
          title="显示切线"
        >
          〰️
        </button>
        <div className="toolbar-separator" />
        <button
          className="toolbar-btn"
          onClick={() => {
            // 自动适配视图
          }}
          title="自动适配"
        >
          🔍
        </button>
      </div>
    </div>
  );
};

export default CurveEditor;
