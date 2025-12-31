import React, { useRef, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface SceneViewProps {
  onEntitySelect?: (entityId: number) => void;
  selectedEntity?: number | null;
}

export const SceneView: React.FC<SceneViewProps> = ({
  onEntitySelect,
  selectedEntity
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [isInitialized, setIsInitialized] = useState(false);

  useEffect(() => {
    // 初始化WebGL渲染上下文
    const canvas = canvasRef.current;
    if (canvas && !isInitialized) {
      const gl = canvas.getContext('webgl2') || canvas.getContext('webgl');
      if (gl) {
        // TODO: 初始化WebGL渲染管线
        // 这里需要集成游戏引擎的WebGL渲染器
        console.log('WebGL context initialized');

        // 设置清空颜色为深灰色
        gl.clearColor(0.1, 0.1, 0.1, 1.0);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

        setIsInitialized(true);
      } else {
        console.error('Failed to initialize WebGL context');
      }
    }
  }, [isInitialized]);

  const handleClick = async (event: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    try {
      // 调用Rust后端进行射线拾取
      const entityId = await invoke<number | null>('raycast', {
        x: x / canvas.width,
        y: y / canvas.height
      });

      if (entityId !== null && onEntitySelect) {
        onEntitySelect(entityId);
      }
    } catch (error) {
      console.error('Raycast failed:', error);
    }
  };

  return (
    <div className="scene-view">
      <div className="scene-toolbar">
        <button title="Translate">G</button>
        <button title="Rotate">R</button>
        <button title="Scale">S</button>
        <div className="separator"></div>
        <button title="Local Space">Local</button>
        <button title="World Space">World</button>
        <div className="separator"></div>
        <button title="Toggle Grid">#</button>
      </div>
      <canvas
        ref={canvasRef}
        width={800}
        height={600}
        onClick={handleClick}
        className={selectedEntity ? 'has-selection' : ''}
      />
      <div className="scene-info">
        {selectedEntity !== null && selectedEntity !== undefined ? (
          <span>Selected: Entity {selectedEntity}</span>
        ) : (
          <span>No selection</span>
        )}
      </div>
    </div>
  );
};
