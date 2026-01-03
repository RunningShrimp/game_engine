import React, { useState, useEffect } from 'react';
import { Search, Filter, PlayCircle, Clock, Star, Lock } from 'lucide-react';
import { Tutorial } from '../types/tutorial';
import { invoke } from '@tauri-apps/api/core';

interface TutorialLibraryProps {
  onSelectTutorial: (tutorialId: string) => void;
  onCreateNew: () => void;
}

const TutorialLibrary: React.FC<TutorialLibraryProps> = ({
  onSelectTutorial,
  onCreateNew
}) => {
  const [tutorials, setTutorials] = useState<Tutorial[]>([]);
  const [filteredTutorials, setFilteredTutorials] = useState<Tutorial[]>([]);
  const [selectedCategory, setSelectedCategory] = useState<'all' | 'beginner' | 'intermediate' | 'advanced'>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    loadTutorials();
  }, []);

  useEffect(() => {
    filterTutorials();
  }, [tutorials, selectedCategory, searchQuery]);

  const loadTutorials = async () => {
    setIsLoading(true);
    try {
      const data = await invoke<Tutorial[]>('get_tutorials');

      // 如果没有教程，从磁盘加载
      if (data.length === 0) {
        await invoke('load_tutorials_from_disk', {
          tutorialsDir: '/public/tutorials'
        });
        const reloaded = await invoke<Tutorial[]>('get_tutorials');
        setTutorials(reloaded);
      } else {
        setTutorials(data);
      }
    } catch (error) {
      console.error('Failed to load tutorials:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const filterTutorials = () => {
    let filtered = [...tutorials];

    // 按类别筛选
    if (selectedCategory !== 'all') {
      filtered = filtered.filter(t => t.category === selectedCategory);
    }

    // 按搜索词筛选
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(t =>
        t.title.toLowerCase().includes(query) ||
        t.description.toLowerCase().includes(query) ||
        t.skills.some(s => s.toLowerCase().includes(query))
      );
    }

    setFilteredTutorials(filtered);
  };

  const getCategoryColor = (category: string) => {
    switch (category) {
      case 'beginner': return 'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-200';
      case 'intermediate': return 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-200';
      case 'advanced': return 'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-200';
      default: return 'bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200';
    }
  };

  const getDifficultyColor = (difficulty: string) => {
    switch (difficulty) {
      case 'easy': return 'bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-200';
      case 'medium': return 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-200';
      case 'hard': return 'bg-orange-100 dark:bg-orange-900/30 text-orange-800 dark:text-orange-200';
      case 'expert': return 'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-200';
      default: return 'bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200';
    }
  };

  return (
    <div className="h-full flex flex-col bg-gray-50 dark:bg-gray-900">
      {/* 头部 */}
      <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-4">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">教程库</h2>
          <button
            onClick={onCreateNew}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium"
          >
            创建新教程
          </button>
        </div>

        {/* 搜索栏 */}
        <div className="flex items-center gap-4">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="搜索教程..."
              className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <div className="flex items-center gap-2">
            <Filter className="w-5 h-5 text-gray-400" />
            <select
              value={selectedCategory}
              onChange={(e) => setSelectedCategory(e.target.value as any)}
              className="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
            >
              <option value="all">全部类别</option>
              <option value="beginner">入门</option>
              <option value="intermediate">进阶</option>
              <option value="advanced">高级</option>
            </select>
          </div>
        </div>
      </div>

      {/* 教程列表 */}
      <div className="flex-1 overflow-y-auto p-6">
        {isLoading ? (
          <div className="flex items-center justify-center h-full">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
          </div>
        ) : filteredTutorials.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-gray-500 dark:text-gray-400">
            <BookOpen className="w-16 h-16 mb-4 opacity-50" />
            <p className="text-lg">暂无教程</p>
            <p className="text-sm mt-2">点击上方按钮创建新教程</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {filteredTutorials.map((tutorial) => (
              <TutorialCard
                key={tutorial.id}
                tutorial={tutorial}
                onStart={() => onSelectTutorial(tutorial.id)}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

const TutorialCard: React.FC<{
  tutorial: Tutorial;
  onStart: () => void;
}> = ({ tutorial, onStart }) => {
  const getCategoryLabel = (category: string) => {
    switch (category) {
      case 'beginner': return '入门';
      case 'intermediate': return '进阶';
      case 'advanced': return '高级';
      default: return category;
    }
  };

  const getDifficultyLabel = (difficulty: string) => {
    switch (difficulty) {
      case 'easy': return '简单';
      case 'medium': return '中等';
      case 'hard': return '困难';
      case 'expert': return '专家';
      default: return difficulty;
    }
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm hover:shadow-md transition-shadow border border-gray-200 dark:border-gray-700 overflow-hidden">
      {/* 封面图（可选） */}
      <div className="h-40 bg-gradient-to-br from-blue-400 to-purple-500 flex items-center justify-center">
        <BookOpen className="w-16 h-16 text-white opacity-80" />
      </div>

      {/* 内容 */}
      <div className="p-4">
        <div className="flex items-start justify-between mb-2">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white flex-1">
            {tutorial.title}
          </h3>
        </div>

        <p className="text-sm text-gray-600 dark:text-gray-400 mb-3 line-clamp-2">
          {tutorial.description}
        </p>

        <div className="flex flex-wrap gap-2 mb-3">
          <span className={`px-2 py-1 rounded text-xs font-medium`}>
            {getCategoryLabel(tutorial.category)}
          </span>
          <span className={`px-2 py-1 rounded text-xs font-medium`}>
            {getDifficultyLabel(tutorial.difficulty)}
          </span>
        </div>

        <div className="flex items-center justify-between text-sm text-gray-500 dark:text-gray-400">
          <div className="flex items-center gap-1">
            <Clock className="w-4 h-4" />
            <span>{tutorial.estimatedTime} 分钟</span>
          </div>
          <div className="flex items-center gap-1 text-yellow-600 dark:text-yellow-400 font-medium">
            <Star className="w-4 h-4" />
            <span>{tutorial.xpReward} XP</span>
          </div>
        </div>

        {/* 技能标签 */}
        {tutorial.skills.length > 0 && (
          <div className="mt-3 flex flex-wrap gap-1">
            {tutorial.skills.slice(0, 3).map((skill, index) => (
              <span
                key={index}
                className="px-2 py-0.5 bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-200 rounded text-xs"
              >
                {skill}
              </span>
            ))}
            {tutorial.skills.length > 3 && (
              <span className="px-2 py-0.5 bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 rounded text-xs">
                +{tutorial.skills.length - 3}
              </span>
            )}
          </div>
        )}

        {/* 前置条件 */}
        {tutorial.prerequisites.length > 0 && (
          <div className="mt-3 p-2 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded text-xs text-yellow-800 dark:text-yellow-200">
            <Lock className="w-3 h-3 inline mr-1" />
            需要完成: {tutorial.prerequisites.length} 个前置教程
          </div>
        )}

        {/* 开始按钮 */}
        <button
          onClick={onStart}
          className="w-full mt-4 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium flex items-center justify-center gap-2 transition-colors"
        >
          <PlayCircle className="w-4 h-4" />
          开始学习
        </button>
      </div>
    </div>
  );
};

export default TutorialLibrary;
