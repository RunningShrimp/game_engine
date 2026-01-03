import React from 'react';
import { Eye, Edit, Save, Play } from 'lucide-react';
import { Tutorial } from '../../types/tutorial';

interface TutorialPreviewProps {
  tutorial: Tutorial;
  onEdit: () => void;
  onSave: () => Promise<void>;
  onPublish: () => Promise<void>;
}

const TutorialPreview: React.FC<TutorialPreviewProps> = ({
  tutorial,
  onEdit,
  onSave,
  onPublish
}) => {
  return (
    <div className="h-screen flex flex-col bg-gray-50 dark:bg-gray-900">
      {/* 头部 */}
      <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Eye className="w-6 h-6 text-blue-600" />
            <h1 className="text-xl font-bold text-gray-900 dark:text-white">
              教程预览
            </h1>
          </div>

          <div className="flex items-center gap-2">
            <button
              onClick={onEdit}
              className="px-4 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg font-medium flex items-center gap-2"
            >
              <Edit className="w-4 h-4" />
              返回编辑
            </button>

            <button
              onClick={onSave}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium flex items-center gap-2"
            >
              <Save className="w-4 h-4" />
              保存
            </button>

            <button
              onClick={onPublish}
              className="px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg font-medium flex items-center gap-2"
            >
              <Play className="w-4 h-4" />
              发布
            </button>
          </div>
        </div>
      </div>

      {/* 预览内容 */}
      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-4xl mx-auto bg-white dark:bg-gray-800 rounded-lg shadow-sm p-8">
          {/* 基本信息 */}
          <div className="border-b border-gray-200 dark:border-gray-700 pb-6 mb-6">
            <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-4">
              {tutorial.title || '未命名教程'}
            </h1>

            <p className="text-lg text-gray-600 dark:text-gray-400 mb-6">
              {tutorial.description || '暂无描述'}
            </p>

            <div className="flex flex-wrap gap-3">
              <span className={`px-3 py-1 rounded-full text-sm font-medium ${
                tutorial.category === 'beginner' ? 'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-200' :
                tutorial.category === 'intermediate' ? 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-200' :
                'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-200'
              }`}>
                {tutorial.category === 'beginner' ? '入门' : tutorial.category === 'intermediate' ? '进阶' : '高级'}
              </span>

              <span className={`px-3 py-1 rounded-full text-sm font-medium ${
                tutorial.difficulty === 'easy' ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-200' :
                tutorial.difficulty === 'medium' ? 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-200' :
                tutorial.difficulty === 'hard' ? 'bg-orange-100 dark:bg-orange-900/30 text-orange-800 dark:text-orange-200' :
                'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-200'
              }`}>
                {tutorial.difficulty === 'easy' ? '简单' : tutorial.difficulty === 'medium' ? '中等' : tutorial.difficulty === 'hard' ? '困难' : '专家'}
              </span>

              <span className="px-3 py-1 bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200 rounded-full text-sm">
                {tutorial.estimatedTime} 分钟
              </span>

              <span className="px-3 py-1 bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-200 rounded-full text-sm font-medium">
                {tutorial.xpReward} XP
              </span>
            </div>
          </div>

          {/* 技能标签 */}
          {tutorial.skills && tutorial.skills.length > 0 && (
            <div className="mb-6">
              <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">涉及技能</h3>
              <div className="flex flex-wrap gap-2">
                {tutorial.skills.map((skill, index) => (
                  <span
                    key={index}
                    className="px-3 py-1 bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-200 rounded-full text-sm"
                  >
                    {skill}
                  </span>
                ))}
              </div>
            </div>
          )}

          {/* 前置条件 */}
          {tutorial.prerequisites && tutorial.prerequisites.length > 0 && (
            <div className="mb-6">
              <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">前置条件</h3>
              <ul className="list-disc list-inside text-sm text-gray-600 dark:text-gray-400">
                {tutorial.prerequisites.map((prereq, index) => (
                  <li key={index}>{prereq}</li>
                ))}
              </ul>
            </div>
          )}

          {/* 步骤列表 */}
          <div className="mb-6">
            <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-4">
              教程步骤 ({tutorial.steps?.length || 0})
            </h3>

            <div className="space-y-4">
              {tutorial.steps?.map((step, index) => (
                <div
                  key={step.id}
                  className="border border-gray-200 dark:border-gray-700 rounded-lg p-4 hover:border-blue-300 dark:hover:border-blue-600 transition-colors"
                >
                  <div className="flex items-start gap-3">
                    <div className="flex-shrink-0 w-8 h-8 bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-200 rounded-full flex items-center justify-center font-semibold text-sm">
                      {index + 1}
                    </div>

                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2">
                        <h4 className="font-semibold text-gray-900 dark:text-white">
                          {step.title}
                        </h4>
                        <span className="px-2 py-0.5 bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200 rounded text-xs">
                          {step.type}
                        </span>
                      </div>

                      <div
                        className="prose dark:prose-invert max-w-none text-sm text-gray-600 dark:text-gray-400"
                        dangerouslySetInnerHTML={{ __html: step.content }}
                      />

                      {step.hints && step.hints.length > 0 && (
                        <div className="mt-3 text-xs text-gray-500 dark:text-gray-500">
                          提示数量: {step.hints.length}
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* 挑战列表 */}
          {tutorial.challenges && tutorial.challenges.length > 0 && (
            <div>
              <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-4">
                挑战任务 ({tutorial.challenges.length})
              </h3>

              <div className="grid grid-cols-2 gap-4">
                {tutorial.challenges.map((challenge, index) => (
                  <div
                    key={challenge.id}
                    className="border border-purple-200 dark:border-purple-800 rounded-lg p-4 bg-purple-50 dark:bg-purple-900/10"
                  >
                    <div className="flex items-center justify-between mb-2">
                      <h4 className="font-semibold text-gray-900 dark:text-white">
                        {challenge.title}
                      </h4>
                      <span className="text-sm text-yellow-600 dark:text-yellow-400 font-medium">
                        {challenge.xpReward} XP
                      </span>
                    </div>

                    <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">
                      {challenge.description}
                    </p>

                    <div className="flex gap-2">
                      <span className={`px-2 py-1 rounded text-xs font-medium ${
                        challenge.difficulty === 'easy' ? 'bg-green-100 text-green-800' :
                        challenge.difficulty === 'medium' ? 'bg-yellow-100 text-yellow-800' :
                        challenge.difficulty === 'hard' ? 'bg-orange-100 text-orange-800' :
                        'bg-red-100 text-red-800'
                      }`}>
                        {challenge.difficulty}
                      </span>
                      <span className="px-2 py-1 bg-purple-100 dark:bg-purple-900/30 text-purple-800 dark:text-purple-200 rounded text-xs">
                        {challenge.type}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* 徽章 */}
          {tutorial.badges && tutorial.badges.length > 0 && (
            <div className="mt-6 pt-6 border-t border-gray-200 dark:border-gray-700">
              <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                完成奖励徽章
              </h3>
              <div className="flex flex-wrap gap-2">
                {tutorial.badges.map((badge, index) => (
                  <span
                    key={index}
                    className="px-3 py-1 bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-200 rounded-full text-sm flex items-center gap-1"
                  >
                    🏆 {badge}
                  </span>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default TutorialPreview;
