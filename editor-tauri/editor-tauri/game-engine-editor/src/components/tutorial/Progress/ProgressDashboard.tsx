import React, { useState, useEffect } from 'react';
import { Trophy, Target, Flame, TrendingUp, Award, BookOpen } from 'lucide-react';
import { UserStats, Tutorial } from '../../types/tutorial';
import { invoke } from '@tauri-apps/api/core';
import SkillTree from './SkillTree';
import AchievementsList from './AchievementsList';
import LearningPath from './LearningPath';

interface ProgressDashboardProps {
  userId: string;
}

const ProgressDashboard: React.FC<ProgressDashboardProps> = ({ userId }) => {
  const [stats, setStats] = useState<UserStats | null>(null);
  const [leaderboard, setLeaderboard] = useState<UserStats[]>([]);
  const [activeTab, setActiveTab] = useState<'overview' | 'skills' | 'achievements' | 'path'>('overview');
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    loadData();
  }, [userId]);

  const loadData = async () => {
    setIsLoading(true);
    try {
      const [userStats, leaderboardData] = await Promise.all([
        invoke<UserStats>('get_user_stats'),
        invoke<UserStats[]>('get_leaderboard', { limit: 10 })
      ]);
      setStats(userStats);
      setLeaderboard(leaderboardData);
    } catch (error) {
      console.error('Failed to load progress data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  if (!stats) {
    return <div>无法加载进度数据</div>;
  }

  const levelProgress = (stats.currentLevelXP / stats.nextLevelXP) * 100;

  return (
    <div className="h-screen flex flex-col bg-gray-50 dark:bg-gray-900">
      {/* 头部 */}
      <div className="bg-gradient-to-r from-blue-600 to-purple-600 dark:from-blue-800 dark:to-purple-800 text-white px-6 py-8">
        <div className="max-w-7xl mx-auto">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold mb-2">学习进度</h1>
              <p className="text-blue-100">继续你的游戏引擎开发之旅</p>
            </div>

            <div className="flex items-center gap-4">
              <div className="text-center">
                <div className="flex items-center gap-2">
                  <Flame className="w-6 h-6 text-orange-300" />
                  <span className="text-2xl font-bold">{stats.streakDays}</span>
                </div>
                <p className="text-sm text-blue-100">连续打卡天数</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* 标签页 */}
      <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6">
        <div className="max-w-7xl mx-auto flex gap-4">
          <button
            onClick={() => setActiveTab('overview')}
            className={`px-4 py-3 font-medium transition-colors border-b-2 ${
              activeTab === 'overview'
                ? 'border-blue-600 text-blue-600 dark:text-blue-400'
                : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
            }`}
          >
            总览
          </button>
          <button
            onClick={() => setActiveTab('skills')}
            className={`px-4 py-3 font-medium transition-colors border-b-2 ${
              activeTab === 'skills'
                ? 'border-blue-600 text-blue-600 dark:text-blue-400'
                : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
            }`}
          >
            技能树
          </button>
          <button
            onClick={() => setActiveTab('achievements')}
            className={`px-4 py-3 font-medium transition-colors border-b-2 ${
              activeTab === 'achievements'
                ? 'border-blue-600 text-blue-600 dark:text-blue-400'
                : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
            }`}
          >
            成就
          </button>
          <button
            onClick={() => setActiveTab('path')}
            className={`px-4 py-3 font-medium transition-colors border-b-2 ${
              activeTab === 'path'
                ? 'border-blue-600 text-blue-600 dark:text-blue-400'
                : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
            }`}
          >
            学习路径
          </button>
        </div>
      </div>

      {/* 内容区 */}
      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-7xl mx-auto">
          {activeTab === 'overview' && (
            <div className="space-y-6">
              {/* 统计卡片 */}
              <div className="grid grid-cols-4 gap-4">
                <StatsCard
                  icon={<Trophy className="w-6 h-6 text-yellow-600" />}
                  label="总经验值"
                  value={stats.totalXP}
                  color="yellow"
                />
                <StatsCard
                  icon={<Target className="w-6 h-6 text-blue-600" />}
                  label="当前等级"
                  value={stats.level}
                  color="blue"
                />
                <StatsCard
                  icon={<BookOpen className="w-6 h-6 text-green-600" />}
                  label="完成教程"
                  value={stats.completedTutorials.length}
                  color="green"
                />
                <StatsCard
                  icon={<Award className="w-6 h-6 text-purple-600" />}
                  label="获得徽章"
                  value={stats.badges.length}
                  color="purple"
                />
              </div>

              {/* 等级进度 */}
              <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                  等级进度
                </h3>
                <div className="mb-2 flex items-center justify-between">
                  <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                    Level {stats.level}
                  </span>
                  <span className="text-sm text-gray-600 dark:text-gray-400">
                    {stats.currentLevelXP} / {stats.nextLevelXP} XP
                  </span>
                </div>
                <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3 overflow-hidden">
                  <div
                    className="bg-gradient-to-r from-blue-500 to-purple-600 h-full transition-all duration-500"
                    style={{ width: `${levelProgress}%` }}
                  />
                </div>
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-2">
                  距离下一级还需 {stats.nextLevelXP - stats.currentLevelXP} XP
                </p>
              </div>

              {/* 最近活动 */}
              <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                  正在学习
                </h3>
                {stats.inProgressTutorials.length > 0 ? (
                  <div className="space-y-3">
                    {stats.inProgressTutorials.map(tutorialId => (
                      <div key={tutorialId} className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
                        <span className="text-gray-900 dark:text-white">{tutorialId}</span>
                        <button className="px-3 py-1 bg-blue-600 hover:bg-blue-700 text-white rounded text-sm">
                          继续
                        </button>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-gray-500 dark:text-gray-400 text-center py-4">
                    暂无正在学习的教程
                  </p>
                )}
              </div>

              {/* 排行榜 */}
              <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                  <TrendingUp className="w-5 h-5 inline mr-2" />
                  排行榜
                </h3>
                <div className="space-y-2">
                  {leaderboard.map((user, index) => (
                    <div
                      key={user.userId}
                      className={`flex items-center justify-between p-3 rounded-lg ${
                        user.userId === userId
                          ? 'bg-blue-50 dark:bg-blue-900/20 border-2 border-blue-500'
                          : 'bg-gray-50 dark:bg-gray-700'
                      }`}
                    >
                      <div className="flex items-center gap-3">
                        <span className={`font-bold text-lg ${
                          index === 0 ? 'text-yellow-600' :
                          index === 1 ? 'text-gray-500' :
                          index === 2 ? 'text-orange-600' :
                          'text-gray-400'
                        }`}>
                          #{index + 1}
                        </span>
                        <span className="font-medium text-gray-900 dark:text-white">
                          {user.userId === userId ? '你' : `用户${user.userId}`}
                        </span>
                      </div>
                      <div className="flex items-center gap-4">
                        <span className="text-sm text-gray-600 dark:text-gray-400">
                          Level {user.level}
                        </span>
                        <span className="font-semibold text-blue-600 dark:text-blue-400">
                          {user.totalXP} XP
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {activeTab === 'skills' && (
            <SkillTree skills={stats.skills} />
          )}

          {activeTab === 'achievements' && (
            <AchievementsList achievements={stats.achievements} badges={stats.badges} />
          )}

          {activeTab === 'path' && (
            <LearningPath
              completedTutorials={stats.completedTutorials}
              inProgressTutorials={stats.inProgressTutorials}
            />
          )}
        </div>
      </div>
    </div>
  );
};

// 统计卡片组件
const StatsCard: React.FC<{
  icon: React.ReactNode;
  label: string;
  value: number;
  color: 'yellow' | 'blue' | 'green' | 'purple';
}> = ({ icon, label, value, color }) => {
  const colorClasses = {
    yellow: 'bg-yellow-50 dark:bg-yellow-900/20 border-yellow-200 dark:border-yellow-800',
    blue: 'bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800',
    green: 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800',
    purple: 'bg-purple-50 dark:bg-purple-900/20 border-purple-200 dark:border-purple-800'
  };

  return (
    <div className={`border-2 rounded-lg p-4 ${colorClasses[color]}`}>
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm text-gray-600 dark:text-gray-400 mb-1">{label}</p>
          <p className="text-3xl font-bold text-gray-900 dark:text-white">{value}</p>
        </div>
        <div className="p-2 bg-white dark:bg-gray-800 rounded-lg">
          {icon}
        </div>
      </div>
    </div>
  );
};

export default ProgressDashboard;
