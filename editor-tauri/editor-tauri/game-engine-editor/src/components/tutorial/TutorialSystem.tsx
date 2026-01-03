import React, { useState } from 'react';
import { BookOpen, Edit, Trophy, List, Play, Settings } from 'lucide-react';
import TutorialPlayer from './Player/TutorialPlayer';
import TutorialEditor from './Editor/TutorialEditor';
import ProgressDashboard from './Progress/ProgressDashboard';
import TutorialLibrary from './TutorialLibrary';

type TutorialView = 'library' | 'player' | 'editor' | 'progress';

const TutorialSystem: React.FC = () => {
  const [currentView, setCurrentView] = useState<TutorialView>('library');
  const [selectedTutorialId, setSelectedTutorialId] = useState<string | null>(null);

  const renderView = () => {
    switch (currentView) {
      case 'library':
        return (
          <TutorialLibrary
            onSelectTutorial={(id) => {
              setSelectedTutorialId(id);
              setCurrentView('player');
            }}
            onCreateNew={() => setCurrentView('editor')}
          />
        );

      case 'player':
        return selectedTutorialId ? (
          <TutorialPlayer
            tutorialId={selectedTutorialId}
            onComplete={() => setCurrentView('progress')}
            onProgressChange={(progress) => {
              console.log('Progress updated:', progress);
            }}
          />
        ) : null;

      case 'editor':
        return <TutorialEditor />;

      case 'progress':
        return (
          <ProgressDashboard
            userId="default-user"
          />
        );

      default:
        return null;
    }
  };

  return (
    <div className="h-screen flex flex-col bg-gray-50 dark:bg-gray-900">
      {/* 顶部导航栏 */}
      <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-2">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <BookOpen className="w-6 h-6 text-blue-600" />
            <h1 className="text-xl font-bold text-gray-900 dark:text-white">
              教程系统
            </h1>
          </div>

          <div className="flex items-center gap-1 bg-gray-100 dark:bg-gray-700 rounded-lg p-1">
            <NavigationButton
              icon={<List className="w-4 h-4" />}
              label="教程库"
              active={currentView === 'library'}
              onClick={() => setCurrentView('library')}
            />
            <NavigationButton
              icon={<Play className="w-4 h-4" />}
              label="正在学习"
              active={currentView === 'player'}
              onClick={() => setCurrentView('player')}
              disabled={!selectedTutorialId}
            />
            <NavigationButton
              icon={<Edit className="w-4 h-4" />}
              label="编辑器"
              active={currentView === 'editor'}
              onClick={() => setCurrentView('editor')}
            />
            <NavigationButton
              icon={<Trophy className="w-4 h-4" />}
              label="进度"
              active={currentView === 'progress'}
              onClick={() => setCurrentView('progress')}
            />
          </div>

          <div className="flex items-center gap-2">
            <button
              className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
              title="设置"
            >
              <Settings className="w-5 h-5 text-gray-600 dark:text-gray-400" />
            </button>
          </div>
        </div>
      </div>

      {/* 主内容区 */}
      <div className="flex-1 overflow-hidden">
        {renderView()}
      </div>
    </div>
  );
};

// 导航按钮组件
const NavigationButton: React.FC<{
  icon: React.ReactNode;
  label: string;
  active: boolean;
  onClick: () => void;
  disabled?: boolean;
}> = ({ icon, label, active, onClick, disabled }) => {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={`flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-colors ${
        active
          ? 'bg-blue-600 text-white'
          : 'text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'
      } ${disabled ? 'opacity-50 cursor-not-allowed' : ''}`}
    >
      {icon}
      <span className="text-sm">{label}</span>
    </button>
  );
};

export default TutorialSystem;
