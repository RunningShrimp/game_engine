import React, { useState, useCallback, useEffect, useRef } from 'react';
import { Activity, Folder, Film } from 'lucide-react';
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
import { announceToScreenReader, setFocus } from './utils/accessibility';
import './App.css';
import './styles/animations.css';

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

  // Refs for focus management
  const appContainerRef = useRef<HTMLDivElement>(null);
  const lastFocusedElementRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    const unsubscribe = historyManagerRef.current.subscribe((state) => {
      setCanUndo(state.canUndo);
      setCanRedo(state.canRedo);

      // Announce undo/redo state changes
      if (state.canUndo !== canUndo || state.canRedo !== canRedo) {
        announceToScreenReader(
          `Undo ${state.canUndo ? 'available' : 'unavailable'}, Redo ${state.canRedo ? 'available' : 'unavailable'}`,
          'polite'
        );
      }
    });
    return unsubscribe;
  }, [canUndo, canRedo]);

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

  // Announce entity count changes
  useEffect(() => {
    announceToScreenReader(`${entities.length} entities in scene`, 'polite');
  }, [entities.length]);

  // Announce selection changes
  useEffect(() => {
    if (selectedEntities.length > 0) {
      const entityNames = selectedEntities
        .map((id) => entities.find((e) => e.id === id)?.name || 'Unknown')
        .join(', ');
      announceToScreenReader(`Selected: ${entityNames}`, 'polite');
    }
  }, [selectedEntities, entities]);

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

    announceToScreenReader(`Created entity ${newEntity.name}`, 'polite');
  }, [entities.length, addEntity, removeEntity]);

  const handleEntityDelete = useCallback(async (entityId: string) => {
    const entity = findEntityById(entityId);
    if (!entity) return;

    const command = new DeleteEntityCommand(entity, removeEntity, addEntity);
    await historyManagerRef.current.executeCommand(command);

    announceToScreenReader(`Deleted entity ${entity.name}`, 'assertive');
  }, [findEntityById, removeEntity, addEntity]);

  const handleEntityRename = useCallback(async (entityId: string, newName: string) => {
    const entity = findEntityById(entityId);
    if (!entity) return;

    const command = new RenameEntityCommand(entityId, entity.name, newName, updateEntityName);
    await historyManagerRef.current.executeCommand(command);

    announceToScreenReader(`Renamed entity to ${newName}`, 'polite');
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

    announceToScreenReader(
      `${entity.name} is now ${newVisibility ? 'visible' : 'hidden'}`,
      'polite'
    );
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

    announceToScreenReader(
      `${entity.name} is now ${newLock ? 'locked' : 'unlocked'}`,
      'polite'
    );
  }, [findEntityById]);

  // Copy/Paste functionality
  const handleEntityCopy = useCallback(() => {
    if (selectedEntities.length !== 1) {
      announceToScreenReader('Cannot copy: Select exactly one entity', 'assertive');
      return;
    }
    const entity = findEntityById(selectedEntities[0]);
    if (entity) {
      setCopiedEntity(entity);
      announceToScreenReader(`Copied ${entity.name}`, 'polite');
    }
  }, [selectedEntities, findEntityById]);

  const handleEntityPaste = useCallback(async () => {
    if (!copiedEntity) {
      announceToScreenReader('Nothing to paste', 'assertive');
      return;
    }

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

    announceToScreenReader(`Pasted ${newEntity.name}`, 'polite');
  }, [copiedEntity, addEntity, removeEntity]);

  // Undo/Redo handlers
  const handleUndo = useCallback(async () => {
    await historyManagerRef.current.undo();
    announceToScreenReader('Undo performed', 'polite');
  }, []);

  const handleRedo = useCallback(async () => {
    await historyManagerRef.current.redo();
    announceToScreenReader('Redo performed', 'polite');
  }, []);

  // Keyboard shortcuts with accessibility enhancements
  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      // Ignore if user is typing in an input
      const target = e.target as HTMLElement;
      if (
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.isContentEditable
      ) {
        return;
      }

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
      // Delete or Backspace: Delete selected entity
      if ((e.key === 'Delete' || e.key === 'Backspace') && selectedEntities.length > 0) {
        e.preventDefault();
        await handleEntityDelete(selectedEntities[0]);
      }
      // Escape: Deselect all
      if (e.key === 'Escape') {
        setSelectedEntities([]);
        announceToScreenReader('Deselected all entities', 'polite');
      }
      // W: Translate mode
      if (e.key === 'w' || e.key === 'W') {
        setTransformMode(TransformMode.Translate);
        announceToScreenReader('Translate mode activated', 'polite');
      }
      // E: Rotate mode
      if (e.key === 'e' || e.key === 'E') {
        setTransformMode(TransformMode.Rotate);
        announceToScreenReader('Rotate mode activated', 'polite');
      }
      // R: Scale mode
      if (e.key === 'r' || e.key === 'R') {
        setTransformMode(TransformMode.Scale);
        announceToScreenReader('Scale mode activated', 'polite');
      }
      // F12: Toggle performance dashboard
      if (e.key === 'F12') {
        setShowPerformanceDashboard((prev) => {
          const newState = !prev;
          announceToScreenReader(
            `Performance dashboard ${newState ? 'opened' : 'closed'}`,
            'polite'
          );
          return newState;
        });
      }
      // Ctrl/Cmd + O: Open Asset Browser
      if ((e.ctrlKey || e.metaKey) && e.key === 'o') {
        e.preventDefault();
        setShowAssetBrowser(true);
        announceToScreenReader('Asset browser opened', 'polite');
      }
      // Ctrl/Cmd + T: Toggle Timeline
      if ((e.ctrlKey || e.metaKey) && e.key === 't') {
        e.preventDefault();
        setShowTimeline((prev) => {
          const newState = !prev;
          announceToScreenReader(
            `Timeline ${newState ? 'shown' : 'hidden'}`,
            'polite'
          );
          return newState;
        });
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [
    handleUndo,
    handleRedo,
    handleEntityCopy,
    handleEntityPaste,
    handleEntityDelete,
    selectedEntities,
  ]);

  // Selection handler
  const handleEntitySelect = useCallback((entityIds: string[]) => {
    setSelectedEntities(entityIds);
  }, []);

  // Playback controls with announcements
  const handlePlay = () => {
    setIsPlaying(true);
    setIsPaused(false);
    announceToScreenReader('Playing scene', 'polite');
  };

  const handlePause = () => {
    setIsPaused(true);
    announceToScreenReader('Scene paused', 'polite');
  };

  const handleStop = () => {
    setIsPlaying(false);
    setIsPaused(false);
    announceToScreenReader('Scene stopped', 'polite');
  };

  // Modal close handlers with focus restoration
  const handlePerformanceDashboardClose = useCallback(() => {
    setShowPerformanceDashboard(false);
    // Restore focus to the button that opened the modal
    lastFocusedElementRef.current?.focus();
  }, []);

  const handleAssetBrowserClose = useCallback(() => {
    setShowAssetBrowser(false);
    lastFocusedElementRef.current?.focus();
  }, []);

  return (
    <div
      ref={appContainerRef}
      className="h-screen w-screen flex flex-col bg-slate-900 text-slate-200"
      role="application"
      aria-label="Game Engine Editor"
      tabIndex={0}
    >
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
      <div className="flex-1 flex overflow-hidden" role="main" aria-label="Editor workspace">
        {/* Left Panel - Entity Tree */}
        <div
          className="w-64 border-r border-slate-700"
          role="region"
          aria-label="Scene hierarchy panel"
        >
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
        <div
          className="flex-1"
          role="region"
          aria-label="3D viewport"
        >
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
        <div
          className="w-80 border-l border-slate-700"
          role="region"
          aria-label="Property inspector panel"
        >
          <PropertyInspector
            entities={entities}
            selectedEntities={selectedEntities}
            onTransformChange={handleTransformChange}
          />
        </div>
      </div>

      {/* Bottom Status Bar */}
      <div
        className="h-8 bg-slate-800 border-t border-slate-700 px-4 flex items-center justify-between text-xs text-slate-400 animate-fade-in"
        role="contentinfo"
        aria-label="Status bar"
      >
        <div className="flex items-center gap-4">
          <span className="animate-fade-in">Game Engine Editor v0.1.0</span>
          <span aria-hidden="true">|</span>
          <span aria-label={`Number of entities: ${entities.length}`}>
            {entities.length} entities
          </span>
          <span aria-hidden="true">|</span>
          <span aria-label={`Number of selected entities: ${selectedEntities.length}`}>
            {selectedEntities.length} selected
          </span>
        </div>
        <div className="flex items-center gap-4">
          <span
            className={isPlaying ? 'text-green-400 animate-pulse-custom' : 'text-slate-400'}
            aria-label={`Scene is ${isPlaying ? (isPaused ? 'paused' : 'playing') : 'stopped'}`}
          >
            {isPlaying ? (isPaused ? '● Paused' : '● Playing') : '○ Stopped'}
          </span>
          <span aria-hidden="true">|</span>
          <span>Ready</span>
          <span aria-hidden="true">|</span>
          <button
            onClick={() => {
              lastFocusedElementRef.current = document.activeElement as HTMLElement;
              setShowPerformanceDashboard(true);
            }}
            className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md hover:bg-slate-700 active:bg-slate-600 transition-smooth hover-lift text-slate-300 hover:text-slate-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
            title="Open Performance Monitor (F12)"
            aria-label="Open Performance Monitor"
          >
            <Activity className="w-3.5 h-3.5" aria-hidden="true" />
            <span>Performance</span>
          </button>
          <span aria-hidden="true">|</span>
          <button
            onClick={() => {
              lastFocusedElementRef.current = document.activeElement as HTMLElement;
              setShowAssetBrowser(true);
            }}
            className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md hover:bg-slate-700 active:bg-slate-600 transition-smooth hover-lift text-slate-300 hover:text-slate-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
            title="Open Asset Browser (Ctrl+O)"
            aria-label="Open Asset Browser"
          >
            <Folder className="w-3.5 h-3.5" aria-hidden="true" />
            <span>Assets</span>
          </button>
          <span aria-hidden="true">|</span>
          <button
            onClick={() => setShowTimeline((prev) => !prev)}
            className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md hover:bg-slate-700 active:bg-slate-600 transition-smooth hover-lift text-slate-300 hover:text-slate-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
            title="Toggle Timeline (Ctrl+T)"
            aria-label="Toggle Timeline"
            aria-pressed={showTimeline}
          >
            <Film className="w-3.5 h-3.5" aria-hidden="true" />
            <span>Timeline</span>
          </button>
        </div>
      </div>

      {/* Performance Dashboard - Lazy Loaded */}
      {showPerformanceDashboard && (
        <div role="dialog" aria-modal="true" aria-label="Performance Dashboard">
          <LazyPerformanceDashboard onClose={handlePerformanceDashboardClose} />
        </div>
      )}

      {/* Asset Browser - Lazy Loaded */}
      {showAssetBrowser && (
        <div role="dialog" aria-modal="true" aria-label="Asset Browser">
          <LazyAssetBrowser isOpen={showAssetBrowser} onClose={handleAssetBrowserClose} />
        </div>
      )}

      {/* Timeline - Lazy Loaded */}
      {showTimeline && (
        <div
          className="fixed bottom-0 left-0 right-0 z-50"
          role="region"
          aria-label="Animation timeline"
        >
          <LazyTimeline
            clip={currentClip}
            onClipChange={(clip) => {
              setAnimationClips((prev) => {
                const index = prev.findIndex((c) => c.id === clip.id);
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
