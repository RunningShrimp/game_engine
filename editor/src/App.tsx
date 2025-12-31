import { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { SceneView } from './components/SceneView';
import { Hierarchy } from './components/Hierarchy';
import { Inspector } from './components/Inspector';
import { AssetBrowser } from './components/AssetBrowser';
import { Console } from './components/Console';

function App() {
  const [selectedEntity, setSelectedEntity] = useState<number | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [isInitialized, setIsInitialized] = useState(false);

  const handleInitializeEngine = async () => {
    try {
      const result = await invoke<string>('create_engine');
      console.log(result);
      setIsInitialized(true);
    } catch (error) {
      console.error('Failed to initialize engine:', error);
    }
  };

  const handlePlayScene = async () => {
    try {
      await invoke('play_scene');
      setIsPlaying(true);
    } catch (error) {
      console.error('Failed to play scene:', error);
    }
  };

  const handleStopScene = async () => {
    try {
      await invoke('stop_scene');
      setIsPlaying(false);
    } catch (error) {
      console.error('Failed to stop scene:', error);
    }
  };

  const handlePauseScene = async () => {
    try {
      await invoke('pause_scene');
    } catch (error) {
      console.error('Failed to pause scene:', error);
    }
  };

  const handleSaveScene = async () => {
    try {
      await invoke('save_scene', { scenePath: '/scenes/default.scene' });
      console.log('Scene saved');
    } catch (error) {
      console.error('Failed to save scene:', error);
    }
  };

  const handleLoadScene = async () => {
    try {
      await invoke('load_scene', { scenePath: '/scenes/default.scene' });
      console.log('Scene loaded');
    } catch (error) {
      console.error('Failed to load scene:', error);
    }
  };

  // 初始化引擎
  if (!isInitialized) {
    return (
      <div className="flex items-center justify-center h-screen bg-gray-900">
        <div className="text-center">
          <h1 className="text-3xl font-bold text-white mb-4">
            Game Engine Editor
          </h1>
          <p className="text-gray-400 mb-8">
            Cross-platform game engine editor built with Tauri
          </p>
          <button
            onClick={handleInitializeEngine}
            className="px-6 py-3 bg-blue-600 text-white rounded hover:bg-blue-700"
          >
            Initialize Engine
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-screen bg-gray-900 text-white">
      {/* Menu Bar */}
      <div className="h-12 bg-gray-800 flex items-center px-4 border-b border-gray-700">
        <h1 className="text-lg font-bold mr-8">
          Game Engine Editor
          <span className="text-xs text-gray-400 ml-2">v0.1.0</span>
        </h1>

        <div className="flex gap-2">
          <button onClick={handleSaveScene} className="menu-button">
            Save
          </button>
          <button onClick={handleLoadScene} className="menu-button">
            Load
          </button>
        </div>

        <div className="ml-auto flex gap-2 items-center">
          {isPlaying ? (
            <>
              <button
                onClick={handlePauseScene}
                className="playback-button pause"
                title="Pause"
              >
                ⏸
              </button>
              <button
                onClick={handleStopScene}
                className="playback-button stop"
                title="Stop"
              >
                ⏹
              </button>
            </>
          ) : (
            <button
              onClick={handlePlayScene}
              className="playback-button play"
              title="Play"
            >
              ▶
            </button>
          )}
        </div>
      </div>

      {/* Main Content */}
      <div className="flex flex-1 overflow-hidden">
        {/* Left Panel */}
        <div className="w-72 bg-gray-800 flex flex-col border-r border-gray-700 overflow-hidden">
          <div className="flex-1 overflow-y-auto">
            <Hierarchy
              selectedEntity={selectedEntity}
              onEntitySelect={setSelectedEntity}
            />
          </div>
          <div className="h-1/3 border-t border-gray-700 overflow-y-auto">
            <AssetBrowser />
          </div>
        </div>

        {/* Center View */}
        <div className="flex-1 flex flex-col overflow-hidden">
          <div className="flex-1 overflow-hidden">
            <SceneView
              selectedEntity={selectedEntity}
              onEntitySelect={setSelectedEntity}
            />
          </div>
          <div className="h-48 border-t border-gray-700 overflow-y-auto">
            <Console />
          </div>
        </div>

        {/* Right Panel */}
        <div className="w-80 bg-gray-800 border-l border-gray-700 overflow-y-auto">
          <Inspector entityId={selectedEntity} />
        </div>
      </div>
    </div>
  );
}

export default App;
