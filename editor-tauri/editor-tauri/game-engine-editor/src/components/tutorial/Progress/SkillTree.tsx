import React from 'react';
import { SkillProgress } from '../../types/tutorial';
import { Zap, Lock } from 'lucide-react';

interface SkillTreeProps {
  skills: SkillProgress[];
}

const SkillTree: React.FC<SkillTreeProps> = ({ skills }) => {
  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
      <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">技能树</h2>

      <div className="grid grid-cols-3 gap-6">
        {skills.map((skill, index) => (
          <SkillCard key={index} skill={skill} />
        ))}
      </div>

      {skills.length === 0 && (
        <div className="text-center py-12 text-gray-500 dark:text-gray-400">
          <Lock className="w-12 h-12 mx-auto mb-4 opacity-50" />
          <p>开始学习教程以解锁技能</p>
        </div>
      )}
    </div>
  );
};

const SkillCard: React.FC<{ skill: SkillProgress }> = ({ skill }) => {
  const getLevelColor = (level: number) => {
    if (level >= 5) return 'text-purple-600';
    if (level >= 4) return 'text-blue-600';
    if (level >= 3) return 'text-green-600';
    if (level >= 2) return 'text-yellow-600';
    return 'text-gray-600';
  };

  const getProgressColor = (level: number) => {
    if (level >= 5) return 'bg-purple-500';
    if (level >= 4) return 'bg-blue-500';
    if (level >= 3) return 'bg-green-500';
    if (level >= 2) return 'bg-yellow-500';
    return 'bg-gray-500';
  };

  return (
    <div className="border border-gray-200 dark:border-gray-700 rounded-lg p-4 hover:border-blue-300 dark:hover:border-blue-600 transition-colors">
      <div className="flex items-center justify-between mb-3">
        <h3 className="font-semibold text-gray-900 dark:text-white">{skill.name}</h3>
        <div className="flex items-center gap-1">
          <Zap className={`w-4 h-4 ${getLevelColor(skill.level)}`} />
          <span className={`font-bold ${getLevelColor(skill.level)}`}>
            Lv.{skill.level}
          </span>
        </div>
      </div>

      <div className="mb-2">
        <div className="flex items-center justify-between text-sm mb-1">
          <span className="text-gray-600 dark:text-gray-400">进度</span>
          <span className="font-medium text-gray-900 dark:text-white">{skill.progress}%</span>
        </div>
        <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2 overflow-hidden">
          <div
            className={`${getProgressColor(skill.level)} h-full transition-all duration-300`}
            style={{ width: `${skill.progress}%` }}
          />
        </div>
      </div>

      <div className="text-xs text-gray-500 dark:text-gray-400">
        已完成 {skill.tutorialsCompleted.length} 个教程
      </div>
    </div>
  );
};

export default SkillTree;
