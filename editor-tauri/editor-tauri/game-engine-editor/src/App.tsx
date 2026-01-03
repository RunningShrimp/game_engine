import React, { useState, useCallback, useEffect, useRef } from 'react';
import { Toolbar } from './components/Toolbar/Toolbar';
import { EntityTree } from './components/EntityTree/EntityTree';
import { Viewport } from './components/Viewport/Viewport';
import { PropertyInspector } from './components/PropertyInspector/PropertyInspector';
import {
  LazyPerformanceDashboard,
  LazyAssetBrowser,
  LazyTimeline,
  preloadAllEditors,
} from './components/lazyComponents';
import { initPreloadStrategies } from './utils/preload';
import './utils/lazyLoad'; // 确保懒加载工具被包含在构建中
import { Entity, TransformMode, Space, Transform } from './types/engine';
import { AnimationClip } from './types/animation';
import {
  CreateEntityCommand,
  DeleteEntityCommand,
  RenameEntityCommand,
  TransformEntityCommand,
  DuplicateEntityCommand,
  ToggleVisibilityCommand,
  ToggleLockCommand,
} from './types/commands';
import { HistoryManager } from './utils/HistoryManager';
import './App.css';

// Sample data for demonstration
const sampleEntities: Entity[] = [
  {
    id: '1',
    name: 'Main Camera',
    transform: {
      position: { x: 0, y: 2, z: -10 },
      rotation: { x: 0, y: 0, z: 0, w: 1 },
      scale: { x: 1, y: 1, z: 1 },
    },
    components: [],
    children: [],
    visible: true,
    locked: false,
  },
  {
    id: '2',
    name: 'Directional Light',
    transform: {
      position: { x: 0, y: 10, z: 0 },
      rotation: { x: 0.5, y: 0, z: 0, w: 1 },
      scale: { x: 1, y: 1, z: 1 },
    },
    components: [],
    children: [],
    visible: true,
    locked: false,
  },
  {
    id: '3',
    name: 'Cube',
    transform: {
      position: { x: 0, y: 0, z: 0 },
      rotation: { x: 0, y: 0, z: 0, w: 1 },
      scale: { x: 1, y: 1, z: 1 },
    },
    components: [],
    children: [],
    visible: true,
    locked: false,
  },
];

function App() {
  // Editor state
  const [entities, setEntities] = useState<Entity[]>(sampleEntities);
  const [selectedEntities, setSelectedEntities] = useState<string[]>([]);
  const [transformMode, setTransformMode] = useState<TransformMode>(TransformMode.Translate);
  const [space, setSpace] = useState<Space>(Space.World);
  const [isPlaying, setIsPlaying] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const [snapEnabled, setSnapEnabled] = useState(false);
  const [copiedEntity, setCopiedEntity] = useState<Entity | null>(null);
  const [showPerformanceDashboard, setShowPerformanceDashboard] = useState(false);
  const [showAssetBrowser, setShowAssetBrowser] = useState(false);
  const [showTimeline, setShowTimeline] = useState(false);
  const [animationClips, setAnimationClips] = useState<AnimationClip[]>([]);
  const [currentClip, setCurrentClip] = useState<AnimationClip | undefined>();
  const [animationCurrentTime, setAnimationCurrentTime] = useState(0);
  const [isAnimationPlaying, setIsAnimationPlaying] = useState(false);
  const gridSize = 1;
  const snapValue = 0.1;

  // History manager
  const historyManagerRef = useRef<HistoryManager>(new HistoryManager(100));

  // Track history state
  const [canUndo, setCanUndo] = useState(false);
  const [canRedo, setCanRedo] = useState(false);

  useEffect(() => {
    const unsubscribe = historyManagerRef.current.subscribe((state) => {
      setCanUndo(state.canUndo);
      setCanRedo(state.canRedo);
    });
    return unsubscribe;
  }, []);

  // 初始化预加载策略（在应用启动后延迟执行）
  useEffect(() => {
    const timer = setTimeout(() => {
      initPreloadStrategies();

      // 在空闲时预加载所有编辑器组件
      if ('requestIdleCallback' in window) {
        (window as any).requestIdleCallback(
          () => {
            preloadAllEditors();
          },
          { timeout: 5000 }
        );
      } else {
        setTimeout(() => {
          preloadAllEditors();
        }, 5000);
      }
    }, 2000);

    return () => clearTimeout(timer);
  }, []);

  // Helper function to deep clone an entity
  const cloneEntity = (entity: Entity): Entity => {
    return JSON.parse(JSON.stringify(entity));
  };

  // Helper function to find entity by ID (recursive for children)
  const findEntityById = useCallback((entityId: string, entityList: Entity[] = entities): Entity | null => {
    for (const entity of entityList) {
      if (entity.id === entityId) return entity;
      if (entity.children.length > 0) {
        const found = findEntityById(entityId, entity.children);
        if (found) return found;
      }
    }
    return null;
  }, [entities]);

  // Entity CRUD operations
  const addEntity = useCallback((entity: Entity) => {
    setEntities((prev) => [...prev, entity]);
  }, []);

  const removeEntity = useCallback((entityId: string) => {
    setEntities((prev) => prev.filter((entity) => entity.id !== entityId));
    setSelectedEntities((prev) => prev.filter((id) => id !== entityId));
  }, []);

  const updateEntityName = useCallback((entityId: string, name: string) => {
    setEntities((prev) =>
      prev.map((entity) =>
        entity.id === entityId ? { ...entity, name } : entity
      )
    );
  }, []);

  const updateTransform = useCallback((entityId: string, transform: Transform) => {
    setEntities((prev) =>
      prev.map((entity) =>
        entity.id === entityId ? { ...entity, transform } : entity
      )
    );
  }, []);

  // Event handlers with undo/redo support
  const handleEntityCreate = useCallback(async () => {
    const newEntity: Entity = {
      id: Date.now().toString(),
      name: `Entity ${entities.length + 1}`,
      transform: {
        position: { x: 0, y: 0, z: 0 },
        rotation: { x: 0, y: 0, z: 0, w: 1 },
        scale: { x: 1, y: 1, z: 1 },
      },
      components: [],
      children: [],
      visible: true,
      locked: false,
    };

    const command = new CreateEntityCommand(newEntity.id, newEntity, addEntity, removeEntity);
    await historyManagerRef.current.executeCommand(command);
  }, [entities.length, addEntity, removeEntity]);

  const handleEntityDelete = useCallback(async (entityId: string) => {
    const entity = findEntityById(entityId);
    if (!entity) return;

    const command = new DeleteEntityCommand(entity, removeEntity, addEntity);
    await historyManagerRef.current.executeCommand(command);
  }, [findEntityById, removeEntity, addEntity]);

  const handleEntityRename = useCallback(async (entityId: string, newName: string) => {
    const entity = findEntityById(entityId);
    if (!entity) return;

    const command = new RenameEntityCommand(entityId, entity.name, newName, updateEntityName);
    await historyManagerRef.current.executeCommand(command);
  }, [findEntityById, updateEntityName]);

  const handleTransformChange = useCallback(async (entityId: string, transform: Transform) => {
    const entity = findEntityById(entityId);
    if (!entity) return;

    const command = new TransformEntityCommand(
      entityId,
      entity.transform,
      transform,
      updateTransform
    );
    await historyManagerRef.current.executeCommand(command);
  }, [findEntityById, updateTransform]);

  const handleEntityToggleVisibility = useCallback(async (entityId: string) => {
    const entity = findEntityById(entityId);
    if (!entity) return;

    const oldVisibility = entity.visible;
    const newVisibility = !oldVisibility;

    const command = new ToggleVisibilityCommand(
      entityId,
      oldVisibility,
      newVisibility,
      (id) => {
        setEntities((prev) =>
          prev.map((entity) =>
            entity.id === id ? { ...entity, visible: !entity.visible } : entity
          )
        );
      }
    );
    await historyManagerRef.current.executeCommand(command);
  }, [findEntityById]);

  const handleEntityToggleLock = useCallback(async (entityId: string) => {
    const entity = findEntityById(entityId);
    if (!entity) return;

    const oldLock = entity.locked;
    const newLock = !oldLock;

    const command = new ToggleLockCommand(
      entityId,
      oldLock,
      newLock,
      (id) => {
        setEntities((prev) =>
          prev.map((entity) =>
            entity.id === id ? { ...entity, locked: !entity.locked } : entity
          )
        );
      }
    );
    await historyManagerRef.current.executeCommand(command);
  }, [findEntityById]);

  // Copy/Paste functionality
  const handleEntityCopy = useCallback(() => {
    if (selectedEntities.length !== 1) return;
    const entity = findEntityById(selectedEntities[0]);
    if (entity) {
      setCopiedEntity(entity);
    }
  }, [selectedEntities, findEntityById]);

  const handleEntityPaste = useCallback(async () => {
    if (!copiedEntity) return;

    const newEntity: Entity = {
      ...cloneEntity(copiedEntity),
      id: Date.now().toString(),
      name: `${copiedEntity.name} (Copy)`,
      transform: {
        ...copiedEntity.transform,
        position: {
          x: copiedEntity.transform.position.x + 1,
          y: copiedEntity.transform.position.y,
          z: copiedEntity.transform.position.z,
        },
      },
    };

    const command = new DuplicateEntityCommand(newEntity, addEntity, removeEntity);
    await historyManagerRef.current.executeCommand(command);
  }, [copiedEntity, addEntity, removeEntity]);

  // Undo/Redo handlers
  const handleUndo = useCallback(async () => {
    await historyManagerRef.current.undo();
  }, []);

  const handleRedo = useCallback(async () => {
    await historyManagerRef.current.redo();
  }, []);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      // Ctrl/Cmd + Z: Undo
      if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
        e.preventDefault();
        await handleUndo();
      }
      // Ctrl/Cmd + Shift + Z or Ctrl/Cmd + Y: Redo
      if ((e.ctrlKey || e.metaKey) && (e.key === 'y' || (e.key === 'z' && e.shiftKey))) {
        e.preventDefault();
        await handleRedo();
      }
      // Ctrl/Cmd + C: Copy
      if ((e.ctrlKey || e.metaKey) && e.key === 'c') {
        handleEntityCopy();
      }
      // Ctrl/Cmd + V: Paste
      if ((e.ctrlKey || e.metaKey) && e.key === 'v') {
        await handleEntityPaste();
      }
      // Delete: Delete selected entity
      if (e.key === 'Delete' && selectedEntities.length > 0) {
        await handleEntityDelete(selectedEntities[0]);
      }
      // W: Translate mode
      if (e.key === 'w' || e.key === 'W') {
        setTransformMode(TransformMode.Translate);
      }
      // E: Rotate mode
      if (e.key === 'e' || e.key === 'E') {
        setTransformMode(TransformMode.Rotate);
      }
      // R: Scale mode
      if (e.key === 'r' || e.key === 'R') {
        setTransformMode(TransformMode.Scale);
      }
      // F12: Toggle performance dashboard
      if (e.key === 'F12') {
        setShowPerformanceDashboard((prev) => !prev);
      }
      // Ctrl/Cmd + O: Open Asset Browser
      if ((e.ctrlKey || e.metaKey) && e.key === 'o') {
        e.preventDefault();
        setShowAssetBrowser(true);
      }
      // Ctrl/Cmd + T: Toggle Timeline
      if ((e.ctrlKey || e.metaKey) && e.key === 't') {
        e.preventDefault();
        setShowTimeline(prev => !prev);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleUndo, handleRedo, handleEntityCopy, handleEntityPaste, handleEntityDelete, selectedEntities]);

  // Selection handler
  const handleEntitySelect = useCallback((entityIds: string[]) => {
    setSelectedEntities(entityIds);
  }, []);

  // Playback controls
  const handlePlay = () => {
    setIsPlaying(true);
    setIsPaused(false);
  };

  const handlePause = () => {
    setIsPaused(true);
  };

  const handleStop = () => {
    setIsPlaying(false);
    setIsPaused(false);
  };

  return (
    <div className="h-screen w-screen flex flex-col bg-slate-900 text-slate-200">
      {/* Top Toolbar */}
      <Toolbar
        transformMode={transformMode}
        space={space}
        isPlaying={isPlaying}
        isPaused={isPaused}
        snapEnabled={snapEnabled}
        canUndo={canUndo}
        canRedo={canRedo}
        onTransformModeChange={setTransformMode}
        onSpaceChange={setSpace}
        onPlay={handlePlay}
        onPause={handlePause}
        onStop={handleStop}
        onSnapToggle={() => setSnapEnabled(!snapEnabled)}
        onUndo={handleUndo}
        onRedo={handleRedo}
        onCopy={handleEntityCopy}
        onPaste={handleEntityPaste}
        copiedEntity={copiedEntity}
      />

      {/* Main Content Area */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Panel - Entity Tree */}
        <div className="w-64 border-r border-slate-700">
          <EntityTree
            entities={entities}
            selectedEntities={selectedEntities}
            onEntitySelect={handleEntitySelect}
            onEntityRename={handleEntityRename}
            onEntityDelete={handleEntityDelete}
            onEntityCreate={handleEntityCreate}
            onEntityToggleVisibility={handleEntityToggleVisibility}
            onEntityToggleLock={handleEntityToggleLock}
          />
        </div>

        {/* Center - 3D Viewport */}
        <div className="flex-1">
          <Viewport
            entities={entities}
            selectedEntities={selectedEntities}
            transformMode={transformMode}
            space={space}
            gridSize={gridSize}
            snapEnabled={snapEnabled}
            snapValue={snapValue}
            showGrid={true}
            showStats={true}
          />
        </div>

        {/* Right Panel - Property Inspector */}
        <div className="w-80 border-l border-slate-700">
          <PropertyInspector
            entities={entities}
            selectedEntities={selectedEntities}
            onTransformChange={handleTransformChange}
          />
        </div>
      </div>

      {/* Bottom Status Bar */}
      <div className="h-8 bg-slate-800 border-t border-slate-700 px-4 flex items-center justify-between text-xs text-slate-400">
        <div className="flex items-center gap-4">
          <span>Game Engine Editor v0.1.0</span>
          <span>|</span>
          <span>{entities.length} entities</span>
          <span>|</span>
          <span>{selectedEntities.length} selected</span>
        </div>
        <div className="flex items-center gap-4">
          <span className={isPlaying ? 'text-green-400' : 'text-slate-400'}>
            {isPlaying ? (isPaused ? '● Paused' : '● Playing') : '○ Stopped'}
          </span>
          <span>|</span>
          <span>Ready</span>
          <span>|</span>
          <button
            onClick={() => setShowPerformanceDashboard(true)}
            className="hover:text-slate-200 transition-colors"
            title="Open Performance Monitor (F12)"
          >
            📊 Performance
          </button>
          <span>|</span>
          <button
            onClick={() => setShowAssetBrowser(true)}
            className="hover:text-slate-200 transition-colors"
            title="Open Asset Browser (Ctrl+O)"
          >
            📁 Assets
          </button>
          <span>|</span>
          <button
            onClick={() => setShowTimeline(prev => !prev)}
            className="hover:text-slate-200 transition-colors"
            title="Toggle Timeline (Ctrl+T)"
          >
            🎬 Timeline
          </button>
        </div>
      </div>

      {/* Performance Dashboard - Lazy Loaded */}
      {showPerformanceDashboard && (
        <LazyPerformanceDashboard onClose={() => setShowPerformanceDashboard(false)} />
      )}

      {/* Asset Browser - Lazy Loaded */}
      {showAssetBrowser && (
        <LazyAssetBrowser isOpen={showAssetBrowser} onClose={() => setShowAssetBrowser(false)} />
      )}

      {/* Timeline - Lazy Loaded */}
      {showTimeline && (
        <div className="fixed bottom-0 left-0 right-0 z-50">
          <LazyTimeline
            clip={currentClip}
            onClipChange={(clip) => {
              setAnimationClips(prev => {
                const index = prev.findIndex(c => c.id === clip.id);
                if (index !== -1) {
                  const newClips = [...prev];
                  newClips[index] = clip;
                  return newClips;
                }
                return [...prev, clip];
              });
              setCurrentClip(clip);
            }}
            currentTime={animationCurrentTime}
            onTimeChange={setAnimationCurrentTime}
            isPlaying={isAnimationPlaying}
            onPlayChange={setIsAnimationPlaying}
          />
        </div>
      )}
    </div>
  );
}

export default App;
