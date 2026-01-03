import React, { useState, useEffect } from 'react';
import { CheckCircle, Circle, PlayCircle, Lock } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { Tutorial } from '../../types/tutorial';

interface LearningPathProps {
  completedTutorials: string[];
  inProgressTutorials: string[];
}

const LearningPath: React.FC<LearningPathProps> = ({
  completedTutorials,
  inProgressTutorials
}) => {
  const [allTutorials, setAllTutorials] = useState<Tutorial[]>([]);
  const [selectedCategory, setSelectedCategory] = useState<'beginner' | 'intermediate' | 'advanced'>('beginner');

  useEffect(() => {
    loadTutorials();
  }, []);

  const loadTutorials = async () => {
    try {
      const tutorials = await invoke<Tutorial[]>('get_all_tutorials');
      setAllTutorials(tutorials);
    } catch (error) {
      console.error('Failed to load tutorials:', error);
    }
  };

  const filteredTutorials = allTutorials.filter(t => t.category === selectedCategory);

  const getTutorialStatus = (tutorialId: string) => {
    if (completedTutorials.includes(tutorialId)) return 'completed';
    if (inProgressTutorials.includes(tutorialId)) return 'in-progress';
    return 'locked';
  };

  const canAccessTutorial = (tutorial: Tutorial) => {
    if (completedTutorials.includes(tutorial.id)) return true;
    if (inProgressTutorials.includes(tutorial.id)) return true;

    // 检查前置条件
    if (tutorial.prerequisites.length === 0) return true;
    return tutorial.prerequisites.every(prereq => completedTutorials.includes(prereq));
  };

  return (
    <div className="space-y-6">
      {/* 类别选择 */}
      <div className="flex gap-2">
        <CategoryButton
          category="beginner"
          label="入门"
          selected={selectedCategory === 'beginner'}
          onClick={() => setSelectedCategory('beginner')}
        />
        <CategoryButton
          category="intermediate"
          label="进阶"
          selected={selectedCategory === 'intermediate'}
          onClick={() => setSelectedCategory('intermediate')}
        />
        <CategoryButton
          category="advanced"
          label="高级"
          selected={selectedCategory === 'advanced'}
          onClick={() => setSelectedCategory('advanced')}
        />
      </div>

      {/* 学习路径 */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">
          学习路径
        </h2>

        <div className="space-y-4">
          {filteredTutorials.map((tutorial, index) => {
            const status = getTutorialStatus(tutorial.id);
            const canAccess = canAccessTutorial(tutorial);

            return (
              <TutorialPathCard
                key={tutorial.id}
                tutorial={tutorial}
                status={status}
                canAccess={canAccess}
                index={index}
              />
            );
          })}
        </div>

        {filteredTutorials.length === 0 && (
          <div className="text-center py-12 text-gray-500 dark:text-gray-400">
            <Lock className="w-12 h-12 mx-auto mb-4 opacity-50" />
            <p>该类别暂无教程</p>
          </div>
        )}
      </div>

      {/* 统计 */}
      <div className="grid grid-cols-3 gap-4">
        <StatCard
          label="已完成"
          value={completedTutorials.length}
          color="green"
        />
        <StatCard
          label="进行中"
          value={inProgressTutorials.length}
          color="blue"
        />
        <StatCard
          label="总计"
          value={allTutorials.length}
          color="gray"
        />
      </div>
    </div>
  );
};

const CategoryButton: React.FC<{
  category: 'beginner' | 'intermediate' | 'advanced';
  label: string;
  selected: boolean;
  onClick: () => void;
}> = ({ category, label, selected, onClick }) => {
  const colors = {
    beginner: selected ? 'bg-green-600 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200',
    intermediate: selected ? 'bg-yellow-600 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200',
    advanced: selected ? 'bg-red-600 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200'
  };

  return (
    <button
      onClick={onClick}
      className={`px-6 py-2 rounded-lg font-medium transition-colors ${colors[category]}`}
    >
      {label}
    </button>
  );
};

const TutorialPathCard: React.FC<{
  tutorial: Tutorial;
  status: 'completed' | 'in-progress' | 'locked';
  canAccess: boolean;
  index: number;
}> = ({ tutorial, status, canAccess, index }) => {
  const getStatusIcon = () => {
    switch (status) {
      case 'completed':
        return <CheckCircle className="w-6 h-6 text-green-600" />;
      case 'in-progress':
        return <PlayCircle className="w-6 h-6 text-blue-600" />;
      default:
        return <Circle className="w-6 h-6 text-gray-400" />;
    }
  };

  const getStatusText = () => {
    switch (status) {
      case 'completed':
        return '已完成';
      case 'in-progress':
        return '进行中';
      default:
        return '未开始';
    }
  };

  return (
    <div
      className={`border-2 rounded-lg p-4 transition-all ${
        !canAccess
          ? 'border-gray-200 dark:border-gray-700 opacity-50'
          : status === 'completed'
          ? 'border-green-400 dark:border-green-600 bg-green-50 dark:bg-green-900/20'
          : status === 'in-progress'
          ? 'border-blue-400 dark:border-blue-600 bg-blue-50 dark:bg-blue-900/20'
          : 'border-gray-200 dark:border-gray-700 hover:border-blue-300 dark:hover:border-blue-600'
      }`}
    >
      <div className="flex items-start gap-4">
        <div className="flex-shrink-0 mt-1">
          {getStatusIcon()}
        </div>

        <div className="flex-1">
          <div className="flex items-start justify-between mb-2">
            <div>
              <div className="flex items-center gap-2 mb-1">
                <span className="text-sm text-gray-500 dark:text-gray-400">
                  #{index + 1}
                </span>
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                  {tutorial.title}
                </h3>
              </div>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                {tutorial.description}
              </p>
            </div>

            <div className="text-right">
              <span className={`px-3 py-1 rounded-full text-sm font-medium ${
                status === 'completed' ? 'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-200' :
                status === 'in-progress' ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-200' :
                'bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200'
              }`}>
                {getStatusText()}
              </span>
            </div>
          </div>

          <div className="flex items-center gap-4 text-sm text-gray-600 dark:text-gray-400">
            <span>{tutorial.estimatedTime} 分钟</span>
            <span>·</span>
            <span className="px-2 py-0.5 bg-gray-200 dark:bg-gray-700 rounded text-xs">
              {tutorial.difficulty}
            </span>
            <span>·</span>
            <span className="text-yellow-600 dark:text-yellow-400 font-medium">
              +{tutorial.xpReward} XP
            </span>
          </div>

          {tutorial.prerequisites.length > 0 && !canAccess && (
            <div className="mt-3 p-2 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded text-sm text-yellow-800 dark:text-yellow-200">
              <Lock className="w-4 h-4 inline mr-1" />
              需要先完成: {tutorial.prerequisites.join(', ')}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

const StatCard: React.FC<{
  label: string;
  value: number;
  color: 'green' | 'blue' | 'gray';
}> = ({ label, value, color }) => {
  const colors = {
    green: 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800',
    blue: 'bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800',
    gray: 'bg-gray-50 dark:bg-gray-700 border-gray-200 dark:border-gray-700'
  };

  return (
    <div className={`border-2 rounded-lg p-4 ${colors[color]}`}>
      <div className="text-center">
        <p className="text-3xl font-bold text-gray-900 dark:text-white">{value}</p>
        <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">{label}</p>
      </div>
    </div>
  );
};

export default LearningPath;
