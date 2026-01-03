import React, { useRef, useEffect, useState, useCallback } from 'react';
import { Entity, TransformMode, Space, Vector3 as Vector3Interface } from '../../types/engine';
import {
  GizmoController,
  GizmoRenderer,
  GizmoAxis
} from '../../gizmo';
import { Camera } from '../../utils/raycast';
import { Vector3 } from '../../utils/math3d';
import { WebGPURenderer, createWebGPURenderer } from '../../utils/webgpu';

interface ViewportProps {
  entities: Entity[];
  selectedEntities: string[];
  transformMode: TransformMode;
  space: Space;
  gridSize: number;
  snapEnabled: boolean;
  snapValue: number;
  showGrid: boolean;
  showStats: boolean;
  onEntityTransform?: (entityId: string, transform: Partial<Vector3Interface>) => void;
}

interface ViewportStats {
  fps: number;
  frameTime: number;
  drawCalls: number;
  triangles: number;
}

export const Viewport: React.FC<ViewportProps> = ({
  entities,
  selectedEntities,
  transformMode,
  space,
  snapEnabled,
  snapValue,
  showStats = true,
  onEntityTransform,
}) => {
  const webgpuCanvasRef = useRef<HTMLCanvasElement>(null);
  const gizmoCanvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const gizmoControllerRef = useRef<GizmoController | null>(null);
  const cameraRef = useRef<Camera | null>(null);
  const webgpuRendererRef = useRef<WebGPURenderer | null>(null);
  const animationFrameRef = useRef<number | undefined>(undefined);

  const [stats, setStats] = useState<ViewportStats>({
    fps: 60,
    frameTime: 16.67,
    drawCalls: 0,
    triangles: 0,
  });

  const [canvasSize, setCanvasSize] = useState({ width: 0, height: 0 });

  // Initialize WebGPU renderer and Gizmo controller
  useEffect(() => {
    if (!webgpuCanvasRef.current || !gizmoCanvasRef.current || !containerRef.current) return;

    const webgpuCanvas = webgpuCanvasRef.current;
    const gizmoCanvas = gizmoCanvasRef.current;
    const container = containerRef.current;

    // Set canvas size
    const updateCanvasSize = () => {
      const rect = container.getBoundingClientRect();
      webgpuCanvas.width = rect.width;
      webgpuCanvas.height = rect.height;
      gizmoCanvas.width = rect.width;
      gizmoCanvas.height = rect.height;
      setCanvasSize({ width: rect.width, height: rect.height });

      // Resize WebGPU renderer if initialized
      if (webgpuRendererRef.current) {
        webgpuRendererRef.current.resize(rect.width, rect.height);
      }
    };

    updateCanvasSize();

    // Initialize WebGPU renderer
    const initWebGPU = async () => {
      const renderer = await createWebGPURenderer(
        webgpuCanvas,
        webgpuCanvas.width,
        webgpuCanvas.height
      );

      if (renderer) {
        webgpuRendererRef.current = renderer;
        console.log('WebGPU renderer initialized successfully');
      } else {
        console.warn('WebGPU initialization failed, falling back to 2D only');
      }
    };

    initWebGPU();

    // Get 2D context for gizmo rendering
    const ctx = gizmoCanvas.getContext('2d');
    if (!ctx) {
      console.error('Failed to get 2D context');
      return;
    }

    // Initialize camera
    const cam = new Camera(
      new Vector3(5, 5, 5),
      new Vector3(0, 0, 0),
      45,
      gizmoCanvas.width / gizmoCanvas.height
    );
    cameraRef.current = cam;

    // Initialize gizmo controller
    const gizmoController = new GizmoController({
      snapEnabled,
      snapValue,
    });
    gizmoController.setMode(transformMode);
    gizmoController.setSpace(space);
    gizmoControllerRef.current = gizmoController;

    // Initialize gizmo renderer
    const gizmoRenderer = new GizmoRenderer(ctx);

    // Render loop
    let lastTime = performance.now();

    const render = (currentTime: number) => {
      // Render WebGPU 3D scene
      if (webgpuRendererRef.current) {
        const webgpuStats = webgpuRendererRef.current.render();

        // Update stats from WebGPU renderer
        setStats((prev) => ({
          ...prev,
          fps: webgpuStats.fps,
          frameTime: webgpuStats.frameTime,
          drawCalls: webgpuStats.drawCalls,
          triangles: webgpuStats.triangles,
        }));
      } else {
        // Fallback FPS calculation if WebGPU not initialized
        const deltaTime = currentTime - lastTime;
        if (deltaTime >= 1000) {
          const fps = Math.round(1000 / deltaTime);
          setStats((prev) => ({
            ...prev,
            fps,
            frameTime: deltaTime,
          }));
        }
      }

      lastTime = currentTime;

      // Clear gizmo canvas
      ctx.clearRect(0, 0, gizmoCanvas.width, gizmoCanvas.height);

      // Get selected entity
      const selectedEntity = entities.find((e) =>
        selectedEntities.includes(e.id)
      );

      // Render gizmo if entity is selected
      if (selectedEntity && gizmoControllerRef.current) {
        const position = new Vector3(
          selectedEntity.transform.position.x,
          selectedEntity.transform.position.y,
          selectedEntity.transform.position.z
        );

        const state = gizmoControllerRef.current.getState();
        gizmoRenderer.render(position, state, cam, gizmoCanvas.width, gizmoCanvas.height);
      }

      animationFrameRef.current = requestAnimationFrame(render);
    };

    render(performance.now());

    // Handle window resize
    window.addEventListener('resize', updateCanvasSize);

    return () => {
      window.removeEventListener('resize', updateCanvasSize);
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
      if (webgpuRendererRef.current) {
        webgpuRendererRef.current.cleanup();
      }
    };
  }, []);

  // Update gizmo config when props change
  useEffect(() => {
    if (gizmoControllerRef.current) {
      gizmoControllerRef.current.updateConfig({
        snapEnabled,
        snapValue,
      });
      gizmoControllerRef.current.setMode(transformMode);
      gizmoControllerRef.current.setSpace(space);
    }
  }, [snapEnabled, snapValue, transformMode, space]);

  // Handle mouse move
  const handleMouseMove = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      if (!gizmoControllerRef.current || !cameraRef.current) return;

      const rect = gizmoCanvasRef.current?.getBoundingClientRect();
      if (!rect) return;

      const mouseX = e.clientX - rect.left;
      const mouseY = e.clientY - rect.top;

      const selectedEntity = entities.find((e) =>
        selectedEntities.includes(e.id)
      );

      if (!selectedEntity) return;

      const position = new Vector3(
        selectedEntity.transform.position.x,
        selectedEntity.transform.position.y,
        selectedEntity.transform.position.z
      );

      const hoveredAxis = gizmoControllerRef.current.handleMouseMove(
        mouseX,
        mouseY,
        position,
        cameraRef.current,
        canvasSize.width,
        canvasSize.height
      );

      // Update cursor based on hover state
      if (hoveredAxis && hoveredAxis !== GizmoAxis.None) {
        gizmoCanvasRef.current?.style.setProperty('cursor', 'pointer');
      } else {
        gizmoCanvasRef.current?.style.setProperty('cursor', 'default');
      }
    },
    [entities, selectedEntities, canvasSize]
  );

  // Handle mouse down
  const handleMouseDown = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      if (!gizmoControllerRef.current || !cameraRef.current) return;

      const rect = gizmoCanvasRef.current?.getBoundingClientRect();
      if (!rect) return;

      const mouseX = e.clientX - rect.left;
      const mouseY = e.clientY - rect.top;

      const selectedEntity = entities.find((e) =>
        selectedEntities.includes(e.id)
      );

      if (!selectedEntity) return;

      const position = new Vector3(
        selectedEntity.transform.position.x,
        selectedEntity.transform.position.y,
        selectedEntity.transform.position.z
      );

      const isDragging = gizmoControllerRef.current.handleMouseDown(
        mouseX,
        mouseY,
        position,
        cameraRef.current,
        canvasSize.width,
        canvasSize.height
      );

      if (isDragging) {
        gizmoCanvasRef.current?.style.setProperty('cursor', 'grabbing');
      }
    },
    [entities, selectedEntities, canvasSize]
  );

  // Handle mouse up
  const handleMouseUp = useCallback(() => {
    if (!gizmoControllerRef.current) return;

    const result = gizmoControllerRef.current.handleMouseUp();

    if (result && selectedEntities.length > 0 && onEntityTransform) {
      const entityId = selectedEntities[0];

      switch (transformMode) {
        case TransformMode.Translate:
          if (result.newPosition) {
            onEntityTransform(entityId, {
              x: result.newPosition.x,
              y: result.newPosition.y,
              z: result.newPosition.z,
            });
          }
          break;
        case TransformMode.Rotate:
          if (result.newRotation) {
            onEntityTransform(entityId, {
              x: result.newRotation.x,
              y: result.newRotation.y,
              z: result.newRotation.z,
            });
          }
          break;
        case TransformMode.Scale:
          if (result.newScale) {
            onEntityTransform(entityId, {
              x: result.newScale.x,
              y: result.newScale.y,
              z: result.newScale.z,
            });
          }
          break;
      }
    }

    gizmoCanvasRef.current?.style.setProperty('cursor', 'default');
  }, [selectedEntities, transformMode, onEntityTransform]);

  // Handle keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Transform mode shortcuts
      switch (e.key.toLowerCase()) {
        case 'w':
          // Switch to translate mode
          break;
        case 'e':
          // Switch to rotate mode
          break;
        case 'r':
          // Switch to scale mode
          break;
        case 'delete':
        case 'backspace':
          // Delete selected entities
          break;
        case 'd':
          if (e.ctrlKey || e.metaKey) {
            e.preventDefault();
            // Duplicate selected entities
          }
          break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  return (
    <div ref={containerRef} className="relative w-full h-full bg-slate-950">
      {/* WebGPU 3D Canvas (Background Layer) */}
      <canvas
        ref={webgpuCanvasRef}
        className="absolute inset-0 w-full h-full"
        style={{ zIndex: 0 }}
      />

      {/* Gizmo 2D Canvas (Foreground Overlay) */}
      <canvas
        ref={gizmoCanvasRef}
        className="absolute inset-0 w-full h-full"
        style={{ zIndex: 1 }}
        onMouseMove={handleMouseMove}
        onMouseDown={handleMouseDown}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      />

      {/* Viewport Stats */}
      {showStats && (
        <div className="absolute top-4 left-4 bg-slate-900/90 backdrop-blur border border-slate-700 rounded-lg px-3 py-2 text-xs space-y-1" style={{ zIndex: 2 }}>
          <div className="flex items-center gap-2">
            <span className="text-slate-400">FPS:</span>
            <span className="text-green-400 font-mono">{stats.fps}</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-slate-400">Frame:</span>
            <span className="text-blue-400 font-mono">{stats.frameTime.toFixed(2)}ms</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-slate-400">Draw Calls:</span>
            <span className="text-yellow-400 font-mono">{stats.drawCalls}</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-slate-400">Triangles:</span>
            <span className="text-purple-400 font-mono">{stats.triangles}</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-slate-400">WebGPU:</span>
            <span className={`${webgpuRendererRef.current?.getIsInitialized() ? 'text-green-400' : 'text-red-400'} font-mono`}>
              {webgpuRendererRef.current?.getIsInitialized() ? 'Active' : 'Inactive'}
            </span>
          </div>
        </div>
      )}

      {/* Camera Controls Hint */}
      <div className="absolute bottom-4 left-4 bg-slate-900/90 backdrop-blur border border-slate-700 rounded-lg px-3 py-2 text-xs" style={{ zIndex: 2 }}>
        <div className="text-slate-400 space-y-0.5">
          <div><span className="text-slate-300">Right-click + Drag:</span> Rotate</div>
          <div><span className="text-slate-300">Middle-click + Drag:</span> Pan</div>
          <div><span className="text-slate-300">Scroll:</span> Zoom</div>
          <div><span className="text-slate-300">W/E/R:</span> Translate/Rotate/Scale</div>
        </div>
      </div>

      {/* Gizmo Mode Indicator */}
      <div className="absolute top-4 right-4 bg-slate-900/90 backdrop-blur border border-slate-700 rounded-lg px-3 py-2" style={{ zIndex: 2 }}>
        <div className="flex items-center gap-2 text-xs">
          <span className="text-slate-400">Mode:</span>
          <span className={`font-mono font-semibold ${
            transformMode === TransformMode.Translate ? 'text-blue-400' :
            transformMode === TransformMode.Rotate ? 'text-green-400' :
            'text-yellow-400'
          }`}>
            {transformMode.toUpperCase()}
          </span>
        </div>
        <div className="flex items-center gap-2 text-xs mt-1">
          <span className="text-slate-400">Space:</span>
          <span className={`font-mono ${
            space === Space.World ? 'text-purple-400' : 'text-orange-400'
          }`}>
            {space.toUpperCase()}
          </span>
        </div>
      </div>

      {/* Snap Indicator */}
      {snapEnabled && (
        <div className="absolute top-20 right-4 bg-green-900/90 backdrop-blur border border-green-700 rounded-lg px-2 py-1 text-xs" style={{ zIndex: 2 }}>
          <div className="flex items-center gap-1 text-green-400">
            <svg className="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
              <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
            </svg>
            <span className="font-mono">{snapValue}</span>
          </div>
        </div>
      )}

      {/* Selected Entity Info */}
      {selectedEntities.length > 0 && (
        <div className="absolute bottom-4 right-4 bg-blue-900/90 backdrop-blur border border-blue-700 rounded-lg px-3 py-2 text-xs" style={{ zIndex: 2 }}>
          <div className="text-blue-300">
            {selectedEntities.length} entity{selectedEntities.length > 1 ? 's' : ''} selected
          </div>
        </div>
      )}
    </div>
  );
};
