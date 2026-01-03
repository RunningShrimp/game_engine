import React from 'react';
import { Tutorial } from '../../types/tutorial';

interface MetadataEditorProps {
  tutorial: Partial<Tutorial>;
  onChange: (tutorial: Partial<Tutorial>) => void;
}

const MetadataEditor: React.FC<MetadataEditorProps> = ({ tutorial, onChange }) => {
  const handleChange = (field: keyof Tutorial, value: any) => {
    onChange({ ...tutorial, [field]: value });
  };

  const handleArrayChange = (field: 'prerequisites' | 'skills' | 'badges', value: string[]) => {
    onChange({ ...tutorial, [field]: value });
  };

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">教程元数据</h2>

      <div className="space-y-6">
        {/* 基本信息 */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">基本信息</h3>

          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                标题 *
              </label>
              <input
                type="text"
                value={tutorial.title || ''}
                onChange={(e) => handleChange('title', e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
                placeholder="输入教程标题"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                描述 *
              </label>
              <textarea
                value={tutorial.description || ''}
                onChange={(e) => handleChange('description', e.target.value)}
                rows={3}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
                placeholder="简要描述教程内容"
              />
            </div>
          </div>
        </div>

        {/* 分类和难度 */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">分类和难度</h3>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                类别
              </label>
              <select
                value={tutorial.category || 'beginner'}
                onChange={(e) => handleChange('category', e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
              >
                <option value="beginner">入门</option>
                <option value="intermediate">进阶</option>
                <option value="advanced">高级</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                难度
              </label>
              <select
                value={tutorial.difficulty || 'easy'}
                onChange={(e) => handleChange('difficulty', e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
              >
                <option value="easy">简单</option>
                <option value="medium">中等</option>
                <option value="hard">困难</option>
                <option value="expert">专家</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                预计时间（分钟）
              </label>
              <input
                type="number"
                value={tutorial.estimatedTime || 30}
                onChange={(e) => handleChange('estimatedTime', parseInt(e.target.value))}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
                min="1"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                经验值奖励
              </label>
              <input
                type="number"
                value={tutorial.xpReward || 100}
                onChange={(e) => handleChange('xpReward', parseInt(e.target.value))}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
                min="0"
              />
            </div>
          </div>
        </div>

        {/* 前置条件 */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            前置条件（教程ID）
          </h3>

          <TagInput
            tags={tutorial.prerequisites || []}
            onChange={(tags) => handleArrayChange('prerequisites', tags)}
            placeholder="输入前置教程ID并按回车"
          />
        </div>

        {/* 技能标签 */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            技能标签
          </h3>

          <TagInput
            tags={tutorial.skills || []}
            onChange={(tags) => handleArrayChange('skills', tags)}
            placeholder="输入技能标签并按回车，如：Rust、3D渲染、物理系统"
          />
        </div>

        {/* 徽章 */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            完成徽章（徽章ID）
          </h3>

          <TagInput
            tags={tutorial.badges || []}
            onChange={(tags) => handleArrayChange('badges', tags)}
            placeholder="输入徽章ID并按回车"
          />
        </div>
      </div>
    </div>
  );
};

// 标签输入组件
const TagInput: React.FC<{
  tags: string[];
  onChange: (tags: string[]) => void;
  placeholder: string;
}> = ({ tags, onChange, placeholder }) => {
  const [input, setInput] = React.useState('');

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && input.trim()) {
      e.preventDefault();
      if (!tags.includes(input.trim())) {
        onChange([...tags, input.trim()]);
      }
      setInput('');
    }
  };

  const removeTag = (tagToRemove: string) => {
    onChange(tags.filter(tag => tag !== tagToRemove));
  };

  return (
    <div className="flex flex-wrap gap-2 p-3 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900">
      {tags.map(tag => (
        <span
          key={tag}
          className="inline-flex items-center gap-1 px-3 py-1 bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-200 rounded-full text-sm"
        >
          {tag}
          <button
            onClick={() => removeTag(tag)}
            className="hover:bg-blue-200 dark:hover:bg-blue-800 rounded-full p-0.5"
          >
            ×
          </button>
        </span>
      ))}
      <input
        type="text"
        value={input}
        onChange={(e) => setInput(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        className="flex-1 min-w-40 outline-none bg-transparent text-gray-900 dark:text-white placeholder-gray-500"
      />
    </div>
  );
};

export default MetadataEditor;
