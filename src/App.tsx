import React, { useState, useCallback, useEffect, useRef, useMemo } from 'react';
import { Activity, BarChart3, FolderOpen, Film } from 'lucide-react';
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
import './utils/lazyLoad';
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
import './styles/animations.css';

const PANEL_SIZES_STORAGE_KEY = 'game-engine-editor-panel-sizes';
const DEFAULT_PANEL_SIZES = { left: 280, right: 320 };
const MIN_PANEL_SIZES = { left: 200, right: 250 };
const MAX_PANEL_SIZES = { left: 500, right: 600 };

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

interface ResizablePanelProps {
  children: React.ReactNode;
  width: number;
  minWidth: number;
  maxWidth: number;
  position: 'left' | 'right';
  onWidthChange: (width: number) => void;
  className?: string;
}

const ResizablePanel = React.memo<ResizablePanelProps>(({
  children,
  width,
  minWidth,
  maxWidth,
  position,
  onWidthChange,
  className = '',
}) => {
  const [isResizing, setIsResizing] = useState(false);
  const startX = useRef(0);
  const startWidth = useRef(0);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
    startX.current = e.clientX;
    startWidth.current = width;
  }, [width]);

  useEffect(() => {
    if (!isResizing) return;

    const handleMouseMove = (e: MouseEvent) => {
      const deltaX = position === 'left' 
        ? e.clientX - startX.current 
        : startX.current - e.clientX;
      const newWidth = Math.max(minWidth, Math.min(maxWidth, startWidth.current + deltaX));
      onWidthChange(newWidth);
    };

    const handleMouseUp = () => setIsResizing(false);

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isResizing, minWidth, maxWidth, position, onWidthChange, startX, startWidth]);

  return (
    <div
      className={`relative flex-shrink-0 transition-all duration-150 ease-in-out ${className}`}
      style={{ width: `${width}px` }}
    >
      {children}
      <div
        className={`absolute top-0 bottom-0 w-1 cursor-col-resize hover:bg-blue-500 transition-colors ${
          isResizing ? 'bg-blue-500' : 'bg-transparent'
        } ${position === 'left' ? 'right-0' : 'left-0'}`}
        onMouseDown={handleMouseDown}
      />
    </div>
  );
});

ResizablePanel.displayName = 'ResizablePanel';

interface StatusBarProps {
  entitiesCount: number;
  selectedCount: number;
  isPlaying: boolean;
  isPaused: boolean;
  onOpenPerformance: () => void;
  onOpenAssets: () => void;
  onToggleTimeline: () => void;
}

const StatusBar = React.memo<StatusBarProps>(({
  entitiesCount,
  selectedCount,
  isPlaying,
  isPaused,
  onOpenPerformance,
  onOpenAssets,
  onToggleTimeline,
}) => {
  const getStatusText = useCallback(() => {
    if (isPlaying) {
      return isPaused ? 'Paused' : 'Playing';
    }
    return 'Ready';
  }, [isPlaying, isPaused]);

  const getStatusColor = useCallback(() => {
    if (isPlaying) {
      return isPaused ? 'text-yellow-400' : 'text-green-400';
    }
    return 'text-slate-400';
  }, [isPlaying, isPaused]);

  return (
    <div className="h-8 bg-slate-800 border-t border-slate-700 px-4 flex items-center justify-between text-xs text-slate-400">
      <div className="flex items-center gap-3">
        <span className="font-medium">Game Engine Editor v0.1.0</span>
        <span className="text-slate-600">|</span>
        <span>{entitiesCount} entities</span>
        <span className="text-slate-600">|</span>
        <span>{selectedCount} selected</span>
      </div>

      <div className="flex items-center gap-2">
        <div className="flex items-center gap-2 pr-3 border-r border-slate-700">
          <Activity className={`w-3.5 h-3.5 ${getStatusColor()}`} />
          <span className={getStatusColor()}>{getStatusText()}</span>
        </div>

        <div className="flex items-center gap-1">
          <StatusBarButton
            icon={BarChart3}
            label="Performance"
            shortcut="F12"
            onClick={onOpenPerformance}
          />
          <StatusBarButton
            icon={FolderOpen}
            label="Assets"
            shortcut="Ctrl+O"
            onClick={onOpenAssets}
          />
          <StatusBarButton
            icon={Film}
            label="Timeline"
            shortcut="Ctrl+T"
            onClick={onToggleTimeline}
          />
        </div>
      </div>
    </div>
  );
});

StatusBar.displayName = 'StatusBar';

interface StatusBarButtonProps {
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  shortcut: string;
  onClick: () => void;
}

const StatusBarButton = React.memo<StatusBarButtonProps>(({ 
  icon: Icon, 
  label, 
  shortcut, 
  onClick 
}) => (
  <button
    onClick={onClick}
    className="flex items-center gap-1.5 px-2 py-1 hover:text-slate-200 hover:bg-slate-700 rounded transition-all duration-150"
    title={`${label} (${shortcut})`}
  >
    <Icon className="w-3.5 h-3.5" />
    <span>{label}</span>
  </button>
));

StatusBarButton.displayName = 'StatusBarButton';

function App() {
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

  const [leftPanelWidth, setLeftPanelWidth] = useState(() => {
    try {
      const saved = localStorage.getItem(PANEL_SIZES_STORAGE_KEY);
      if (saved) {
        const sizes = JSON.parse(saved);
        return Math.max(MIN_PANEL_SIZES.left, Math.min(MAX_PANEL_SIZES.left, sizes.left || DEFAULT_PANEL_SIZES.left));
      }
    } catch (e) {
      console.warn('Failed to load panel sizes from localStorage:', e);
    }
    return DEFAULT_PANEL_SIZES.left;
  });

  const [rightPanelWidth, setRightPanelWidth] = useState(() => {
    try {
      const saved = localStorage.getItem(PANEL_SIZES_STORAGE_KEY);
      if (saved) {
        const sizes = JSON.parse(saved);
        return Math.max(MIN_PANEL_SIZES.right, Math.min(MAX_PANEL_SIZES.right, sizes.right || DEFAULT_PANEL_SIZES.right));
      }
    } catch (e) {
      console.warn('Failed to load panel sizes from localStorage:', e);
    }
    return DEFAULT_PANEL_SIZES.right;
  });

  const gridSize = 1;
  const snapValue = 0.1;
  const historyManagerRef = useRef<HistoryManager>(new HistoryManager(100));
  const [canUndo, setCanUndo] = useState(false);
  const [canRedo, setCanRedo] = useState(false);

  useEffect(() => {
    try {
      localStorage.setItem(PANEL_SIZES_STORAGE_KEY, JSON.stringify({
        left: leftPanelWidth,
        right: rightPanelWidth,
      }));
    } catch (e) {
      console.warn('Failed to save panel sizes to localStorage:', e);
    }
  }, [leftPanelWidth, rightPanelWidth]);

  useEffect(() => {
    const unsubscribe = historyManagerRef.current.subscribe((state) => {
      setCanUndo(state.canUndo);
      setCanRedo(state.canRedo);
    });
    return unsubscribe;
  }, []);

  useEffect(() => {
    const timer = setTimeout(() => {
      initPreloadStrategies();

      if ('requestIdleCallback' in window) {
        (window as any).requestIdleCallback(
          () => preloadAllEditors(),
          { timeout: 5000 }
        );
      } else {
        setTimeout(() => preloadAllEditors(), 5000);
      }
    }, 2000);

    return () => clearTimeout(timer);
  }, []);

  const cloneEntity = useCallback((entity: Entity): Entity => {
    return JSON.parse(JSON.stringify(entity));
  }, []);

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
  }, [copiedEntity, cloneEntity, addEntity, removeEntity]);

  const handleUndo = useCallback(async () => {
    await historyManagerRef.current.undo();
  }, []);

  const handleRedo = useCallback(async () => {
    await historyManagerRef.current.redo();
  }, []);

  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
        e.preventDefault();
        await handleUndo();
      }
      if ((e.ctrlKey || e.metaKey) && (e.key === 'y' || (e.key === 'z' && e.shiftKey))) {
        e.preventDefault();
        await handleRedo();
      }
      if ((e.ctrlKey || e.metaKey) && e.key === 'c') {
        handleEntityCopy();
      }
      if ((e.ctrlKey || e.metaKey) && e.key === 'v') {
        await handleEntityPaste();
      }
      if (e.key === 'Delete' && selectedEntities.length > 0) {
        await handleEntityDelete(selectedEntities[0]);
      }
      if (e.key === 'w' || e.key === 'W') {
        setTransformMode(TransformMode.Translate);
      }
      if (e.key === 'e' || e.key === 'E') {
        setTransformMode(TransformMode.Rotate);
      }
      if (e.key === 'r' || e.key === 'R') {
        setTransformMode(TransformMode.Scale);
      }
      if (e.key === 'F12') {
        setShowPerformanceDashboard((prev) => !prev);
      }
      if ((e.ctrlKey || e.metaKey) && e.key === 'o') {
        e.preventDefault();
        setShowAssetBrowser(true);
      }
      if ((e.ctrlKey || e.metaKey) && e.key === 't') {
        e.preventDefault();
        setShowTimeline(prev => !prev);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleUndo, handleRedo, handleEntityCopy, handleEntityPaste, handleEntityDelete, selectedEntities]);

  const handleEntitySelect = useCallback((entityIds: string[]) => {
    setSelectedEntities(entityIds);
  }, []);

  const handlePlay = useCallback(() => {
    setIsPlaying(true);
    setIsPaused(false);
  }, []);

  const handlePause = useCallback(() => {
    setIsPaused(true);
  }, []);

  const handleStop = useCallback(() => {
    setIsPlaying(false);
    setIsPaused(false);
  }, []);

  const viewportProps = useMemo(() => ({
    entities,
    selectedEntities,
    transformMode,
    space,
    gridSize,
    snapEnabled,
    snapValue,
    showGrid: true,
    showStats: true,
  }), [entities, selectedEntities, transformMode, space, snapEnabled]);

  return (
    <div className="h-screen w-screen flex flex-col bg-slate-900 text-slate-200">
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

      <div className="flex-1 flex overflow-hidden">
        <ResizablePanel
          width={leftPanelWidth}
          minWidth={MIN_PANEL_SIZES.left}
          maxWidth={MAX_PANEL_SIZES.left}
          position="left"
          onWidthChange={setLeftPanelWidth}
          className="border-r border-slate-700"
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
        </ResizablePanel>

        <div className="flex-1 min-w-0">
          <Viewport {...viewportProps} />
        </div>

        <ResizablePanel
          width={rightPanelWidth}
          minWidth={MIN_PANEL_SIZES.right}
          maxWidth={MAX_PANEL_SIZES.right}
          position="right"
          onWidthChange={setRightPanelWidth}
          className="border-l border-slate-700"
        >
          <PropertyInspector
            entities={entities}
            selectedEntities={selectedEntities}
            onTransformChange={handleTransformChange}
          />
        </ResizablePanel>
      </div>

      <StatusBar
        entitiesCount={entities.length}
        selectedCount={selectedEntities.length}
        isPlaying={isPlaying}
        isPaused={isPaused}
        onOpenPerformance={() => setShowPerformanceDashboard(true)}
        onOpenAssets={() => setShowAssetBrowser(true)}
        onToggleTimeline={() => setShowTimeline(prev => !prev)}
      />

      {showPerformanceDashboard && (
        <LazyPerformanceDashboard onClose={() => setShowPerformanceDashboard(false)} />
      )}

      {showAssetBrowser && (
        <LazyAssetBrowser isOpen={showAssetBrowser} onClose={() => setShowAssetBrowser(false)} />
      )}

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
