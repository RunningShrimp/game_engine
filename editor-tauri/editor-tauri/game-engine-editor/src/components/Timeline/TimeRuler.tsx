/**
 * 时间标尺组件
 */

import React, { useRef, useEffect, useCallback, useState } from 'react';
import './TimeRuler.css';

export interface TimeRulerProps {
  duration: number;
  zoom: number;
  scrollOffset: number;
  currentTime: number;
  onTimeChange: (time: number) => void;
  onZoomChange: (zoom: number) => void;
}

export const TimeRuler: React.FC<TimeRulerProps> = ({
  duration,
  zoom,
  scrollOffset,
  currentTime,
  onTimeChange,
  onZoomChange,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [hoverTime, setHoverTime] = useState<number | null>(null);

  // 绘制时间标尺
  const drawRuler = useCallback(() => {
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

    // 计算时间刻度间隔
    let majorInterval = 1.0; // 主刻度间隔（秒）
    let minorInterval = 0.25; // 次刻度间隔（秒）

    // 根据zoom调整刻度密度
    if (zoom < 20) {
      majorInterval = 5.0;
      minorInterval = 1.0;
    } else if (zoom < 50) {
      majorInterval = 2.0;
      minorInterval = 0.5;
    } else if (zoom < 100) {
      majorInterval = 1.0;
      minorInterval = 0.25;
    } else if (zoom < 200) {
      majorInterval = 0.5;
      minorInterval = 0.1;
    } else {
      majorInterval = 0.25;
      minorInterval = 0.05;
    }

    // 计算可见范围
    const startTime = scrollOffset / zoom;
    const endTime = startTime + width / zoom;

    // 绘制刻度
    ctx.font = '11px Monaco, Menlo, Ubuntu Mono, monospace';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'top';

    // 绘制次刻度
    ctx.strokeStyle = '#334155';
    ctx.lineWidth = 1;

    for (let time = Math.floor(startTime / minorInterval) * minorInterval;
         time <= endTime;
         time += minorInterval) {
      const x = time * zoom - scrollOffset;
      if (x < 0 || x > width) continue;

      const isMajor = Math.abs(time % majorInterval) < 0.001;
      const tickHeight = isMajor ? 16 : 8;

      ctx.beginPath();
      ctx.moveTo(x, height - tickHeight);
      ctx.lineTo(x, height);
      ctx.stroke();
    }

    // 绘制主刻度和标签
    ctx.strokeStyle = '#475569';
    ctx.fillStyle = '#94a3b8';

    for (let time = Math.floor(startTime / majorInterval) * majorInterval;
         time <= endTime;
         time += majorInterval) {
      const x = time * zoom - scrollOffset;
      if (x < 0 || x > width) continue;

      // 主刻度
      ctx.beginPath();
      ctx.moveTo(x, height - 16);
      ctx.lineTo(x, height);
      ctx.stroke();

      // 时间标签
      const label = formatTime(time);
      ctx.fillText(label, x, height - 16);
    }

    // 绘制当前时间指示器
    const currentX = currentTime * zoom - scrollOffset;
    if (currentX >= 0 && currentX <= width) {
      ctx.strokeStyle = '#ef4444';
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(currentX, 0);
      ctx.lineTo(currentX, height);
      ctx.stroke();

      // 绘制三角形箭头
      ctx.fillStyle = '#ef4444';
      ctx.beginPath();
      ctx.moveTo(currentX - 6, 0);
      ctx.lineTo(currentX + 6, 0);
      ctx.lineTo(currentX, 8);
      ctx.closePath();
      ctx.fill();
    }

    // 绘制悬停时间指示器
    if (hoverTime !== null && hoverTime !== currentTime) {
      const hoverX = hoverTime * zoom - scrollOffset;
      if (hoverX >= 0 && hoverX <= width) {
        ctx.strokeStyle = '#f59e0b';
        ctx.lineWidth = 1;
        ctx.setLineDash([4, 4]);
        ctx.beginPath();
        ctx.moveTo(hoverX, 0);
        ctx.lineTo(hoverX, height);
        ctx.stroke();
        ctx.setLineDash([]);
      }
    }
  }, [duration, zoom, scrollOffset, currentTime, hoverTime]);

  // 格式化时间显示
  const formatTime = (time: number): string => {
    const minutes = Math.floor(time / 60);
    const seconds = Math.floor(time % 60);
    const milliseconds = Math.floor((time % 1) * 100);

    if (minutes > 0) {
      return `${minutes}:${seconds.toString().padStart(2, '0')}`;
    }
    return `${seconds}.${milliseconds.toString().padStart(2, '0')}s`;
  };

  // 计算从像素到时间的转换
  const pixelsToTime = useCallback((pixels: number): number => {
    const x = pixels + scrollOffset;
    return x / zoom;
  }, [scrollOffset, zoom]);

  // 处理鼠标事件
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    setIsDragging(true);
    const rect = containerRef.current?.getBoundingClientRect();
    if (!rect) return;

    const x = e.clientX - rect.left;
    const time = pixelsToTime(x);
    onTimeChange(Math.max(0, Math.min(time, duration)));
  }, [onTimeChange, duration, pixelsToTime]);

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    const rect = containerRef.current?.getBoundingClientRect();
    if (!rect) return;

    const x = e.clientX - rect.left;
    const time = pixelsToTime(x);

    if (isDragging) {
      onTimeChange(Math.max(0, Math.min(time, duration)));
    } else {
      setHoverTime(time);
    }
  }, [isDragging, onTimeChange, duration, pixelsToTime]);

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
  }, []);

  const handleMouseLeave = useCallback(() => {
    setIsDragging(false);
    setHoverTime(null);
  }, []);

  const handleDoubleClick = useCallback((e: React.MouseEvent) => {
    const rect = containerRef.current?.getBoundingClientRect();
    if (!rect) return;

    const x = e.clientX - rect.left;
    const time = pixelsToTime(x);
    onTimeChange(Math.max(0, Math.min(time, duration)));
  }, [onTimeChange, duration, pixelsToTime]);

  // 处理滚轮缩放
  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault();

    const rect = containerRef.current?.getBoundingClientRect();
    if (!rect) return;

    const mouseX = e.clientX - rect.left;
    const mouseTime = pixelsToTime(mouseX);

    // 计算新的缩放级别
    const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;
    const newZoom = Math.max(10, Math.min(500, zoom * zoomFactor));

    // 调整scrollOffset以保持鼠标位置的时间不变
    const newScrollOffset = mouseX - mouseTime * newZoom;
    onZoomChange(newZoom);

    // 注意：这里需要同时更新scrollOffset，但当前的props结构不支持
    // 实际使用时应该在父组件中处理
  }, [zoom, pixelsToTime, onZoomChange]);

  // 绘制和更新
  useEffect(() => {
    drawRuler();
  }, [drawRuler]);

  // 监听鼠标释放
  useEffect(() => {
    const handleMouseUpGlobal = () => {
      setIsDragging(false);
    };

    if (isDragging) {
      window.addEventListener('mouseup', handleMouseUpGlobal);
      return () => window.removeEventListener('mouseup', handleMouseUpGlobal);
    }
  }, [isDragging]);

  return (
    <div
      ref={containerRef}
      className="timeline-ruler"
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseLeave}
      onDoubleClick={handleDoubleClick}
      onWheel={handleWheel}
      title="单击跳转，拖拽移动，滚轮缩放"
    >
      <canvas
        ref={canvasRef}
        className="timeline-ruler-canvas"
      />
    </div>
  );
};

export default TimeRuler;
