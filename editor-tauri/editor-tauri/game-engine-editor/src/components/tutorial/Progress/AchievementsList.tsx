import React from 'react';
import { Achievement, Badge } from '../../types/tutorial';
import { Trophy, Medal, Star, Crown } from 'lucide-react';

interface AchievementsListProps {
  achievements: Achievement[];
  badges: Badge[];
}

const AchievementsList: React.FC<AchievementsListProps> = ({ achievements, badges }) => {
  return (
    <div className="space-y-6">
      {/* 徽章 */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">
          <Medal className="w-6 h-6 inline mr-2" />
          徽章收藏
        </h2>

        <div className="grid grid-cols-4 gap-4">
          {badges.map((badge, index) => (
            <BadgeCard key={index} badge={badge} />
          ))}
        </div>

        {badges.length === 0 && (
          <div className="text-center py-12 text-gray-500 dark:text-gray-400">
            <Medal className="w-12 h-12 mx-auto mb-4 opacity-50" />
            <p>完成教程以获得徽章</p>
          </div>
        )}
      </div>

      {/* 成就 */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">
          <Trophy className="w-6 h-6 inline mr-2" />
          成就
        </h2>

        <div className="grid grid-cols-2 gap-4">
          {achievements.map((achievement, index) => (
            <AchievementCard key={index} achievement={achievement} />
          ))}
        </div>

        {achievements.length === 0 && (
          <div className="text-center py-12 text-gray-500 dark:text-gray-400">
            <Trophy className="w-12 h-12 mx-auto mb-4 opacity-50" />
            <p>完成特定任务以解锁成就</p>
          </div>
        )}
      </div>
    </div>
  );
};

const BadgeCard: React.FC<{ badge: Badge }> = ({ badge }) => {
  const getRarityColor = (rarity: string) => {
    switch (rarity) {
      case 'legendary': return 'from-yellow-400 to-orange-500';
      case 'epic': return 'from-purple-400 to-pink-500';
      case 'rare': return 'from-blue-400 to-cyan-500';
      default: return 'from-gray-400 to-gray-500';
    }
  };

  const getRarityBorder = (rarity: string) => {
    switch (rarity) {
      case 'legendary': return 'border-yellow-400 dark:border-yellow-600';
      case 'epic': return 'border-purple-400 dark:border-purple-600';
      case 'rare': return 'border-blue-400 dark:border-blue-600';
      default: return 'border-gray-400 dark:border-gray-600';
    }
  };

  return (
    <div className={`border-2 ${getRarityBorder(badge.rarity)} rounded-lg p-4 bg-gradient-to-br ${getRarityColor(badge.rarity)} bg-opacity-10`}>
      <div className="text-center">
        <div className="text-4xl mb-2">{badge.icon}</div>
        <h3 className="font-bold text-gray-900 dark:text-white mb-1">{badge.name}</h3>
        <p className="text-xs text-gray-700 dark:text-gray-300 mb-2">{badge.description}</p>
        <span className={`text-xs px-2 py-1 rounded-full font-medium ${
          badge.rarity === 'legendary' ? 'bg-yellow-100 text-yellow-800' :
          badge.rarity === 'epic' ? 'bg-purple-100 text-purple-800' :
          badge.rarity === 'rare' ? 'bg-blue-100 text-blue-800' :
          'bg-gray-100 text-gray-800'
        }`}>
          {badge.rarity}
        </span>
        <p className="text-xs text-gray-600 dark:text-gray-400 mt-2">
          {new Date(badge.earnedAt).toLocaleDateString()}
        </p>
      </div>
    </div>
  );
};

const AchievementCard: React.FC<{ achievement: Achievement }> = ({ achievement }) => {
  const isCompleted = !achievement.progress || achievement.progress === achievement.total;

  return (
    <div className={`border-2 rounded-lg p-4 ${
      isCompleted
        ? 'border-green-400 dark:border-green-600 bg-green-50 dark:bg-green-900/20'
        : 'border-gray-200 dark:border-gray-700'
    }`}>
      <div className="flex items-start gap-3">
        <div className="text-3xl">{achievement.icon}</div>
        <div className="flex-1">
          <div className="flex items-center justify-between mb-1">
            <h3 className="font-semibold text-gray-900 dark:text-white">
              {achievement.title}
            </h3>
            {!isCompleted && (
              <span className="text-sm text-gray-600 dark:text-gray-400">
                {achievement.progress}/{achievement.total}
              </span>
            )}
          </div>

          <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">
            {achievement.description}
          </p>

          {achievement.progress !== undefined && achievement.total !== undefined && (
            <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2 overflow-hidden">
              <div
                className={`h-full transition-all duration-300 ${
                  isCompleted ? 'bg-green-500' : 'bg-blue-500'
                }`}
                style={{ width: `${(achievement.progress / achievement.total) * 100}%` }}
              />
            </div>
          )}

          <div className="flex items-center justify-between mt-3">
            <span className="text-xs text-gray-500 dark:text-gray-400">
              {isCompleted
                ? `解锁于 ${new Date(achievement.unlockedAt).toLocaleDateString()}`
                : '进行中...'}
            </span>
            <span className="text-sm font-medium text-yellow-600 dark:text-yellow-400">
              +{achievement.xpReward} XP
            </span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default AchievementsList;
