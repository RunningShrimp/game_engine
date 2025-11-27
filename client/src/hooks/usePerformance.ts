import { useEffect, useState } from 'react';

export interface PerformanceStats {
  fps: number;
  frameTime: number;
  memory: number;
  timestamp: number;
}

export function usePerformance(interval: number = 1000) {
  const [stats, setStats] = useState<PerformanceStats>({
    fps: 0,
    frameTime: 0,
    memory: 0,
    timestamp: Date.now(),
  });

  useEffect(() => {
    let frameCount = 0;
    let lastTime = performance.now();
    let animationFrameId: number;

    const countFrame = () => {
      frameCount++;
      animationFrameId = requestAnimationFrame(countFrame);
    };

    const updateStats = () => {
      const currentTime = performance.now();
      const deltaTime = currentTime - lastTime;
      const fps = Math.round((frameCount * 1000) / deltaTime);
      const frameTime = deltaTime / frameCount;

      // 获取内存使用情况（如果支持）
      let memory = 0;
      if ('memory' in performance) {
        const perfMemory = (performance as any).memory;
        memory = Math.round(perfMemory.usedJSHeapSize / 1048576); // 转换为MB
      }

      setStats({
        fps,
        frameTime: Math.round(frameTime * 100) / 100,
        memory,
        timestamp: Date.now(),
      });

      frameCount = 0;
      lastTime = currentTime;
    };

    countFrame();
    const intervalId = setInterval(updateStats, interval);

    return () => {
      cancelAnimationFrame(animationFrameId);
      clearInterval(intervalId);
    };
  }, [interval]);

  return stats;
}
