import React, { useState } from 'react';
import { TutorialStep } from '../../types/tutorial';
import CodeEditor from '../CodeEditor';

interface StepEditorProps {
  step: TutorialStep;
  onChange: (updates: Partial<TutorialStep>) => void;
}

const StepEditor: React.FC<StepEditorProps> = ({ step, onChange }) => {
  const [showPreview, setShowPreview] = useState(false);

  return (
    <div className="p-6 max-w-5xl mx-auto">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">编辑步骤</h2>
        <button
          onClick={() => setShowPreview(!showPreview)}
          className="px-4 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg font-medium"
        >
          {showPreview ? '编辑' : '预览'}
        </button>
      </div>

      <div className="space-y-6">
        {/* 基本信息 */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                标题 *
              </label>
              <input
                type="text"
                value={step.title}
                onChange={(e) => onChange({ title: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                类型
              </label>
              <select
                value={step.type}
                onChange={(e) => onChange({ type: e.target.value as any })}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
              >
                <option value="concept">概念讲解</option>
                <option value="demo">演示</option>
                <option value="exercise">练习题</option>
                <option value="challenge">挑战</option>
                <option value="quiz">测验</option>
              </select>
            </div>
          </div>
        </div>

        {/* 内容编辑器 */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-4">
            步骤内容（支持Markdown和HTML）
          </label>

          {showPreview ? (
            <div className="border border-gray-300 dark:border-gray-600 rounded-lg p-4 prose dark:prose-invert max-w-none">
              <div dangerouslySetInnerHTML={{ __html: step.content }} />
            </div>
          ) : (
            <textarea
              value={step.content}
              onChange={(e) => onChange({ content: e.target.value })}
              rows={12}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white font-mono text-sm focus:ring-2 focus:ring-blue-500"
              placeholder="<p>在这里输入步骤内容，支持HTML和Markdown格式</p>"
            />
          )}
        </div>

        {/* 代码模板 */}
        {step.type !== 'concept' && (
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-4">
              代码模板（可选）
            </label>

            <CodeEditor
              value={step.codeTemplate || ''}
              onChange={(code) => onChange({ codeTemplate: code })}
              language="rust"
              height="200px"
            />

            <div className="mt-4">
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                预期输出（用于自动验证）
              </label>
              <input
                type="text"
                value={step.expectedOutput || ''}
                onChange={(e) => onChange({ expectedOutput: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white font-mono text-sm focus:ring-2 focus:ring-blue-500"
                placeholder="预期输出内容"
              />
            </div>
          </div>
        )}

        {/* 提示 */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
          <div className="flex items-center justify-between mb-4">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              提示（按顺序显示）
            </label>
            <button
              onClick={() => onChange({ hints: [...(step.hints || []), ''] })}
              className="px-3 py-1 bg-blue-600 hover:bg-blue-700 text-white rounded text-sm font-medium"
            >
              添加提示
            </button>
          </div>

          <div className="space-y-3">
            {step.hints?.map((hint, index) => (
              <div key={index} className="flex gap-2">
                <span className="text-sm text-gray-500 dark:text-gray-400 py-2 w-8">
                  {index + 1}.
                </span>
                <textarea
                  value={hint}
                  onChange={(e) => {
                    const newHints = [...(step.hints || [])];
                    newHints[index] = e.target.value;
                    onChange({ hints: newHints });
                  }}
                  rows={2}
                  className="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-blue-500"
                  placeholder={`提示 ${index + 1} 内容`}
                />
                <button
                  onClick={() => {
                    const newHints = step.hints?.filter((_, i) => i !== index) || [];
                    onChange({ hints: newHints });
                  }}
                  className="px-3 py-1 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded"
                >
                  删除
                </button>
              </div>
            ))}
          </div>

          {!step.hints || step.hints.length === 0 && (
            <p className="text-sm text-gray-500 dark:text-gray-400 text-center py-4">
              暂无提示，点击上方按钮添加
            </p>
          )}
        </div>

        {/* 验证函数 */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            验证函数名（可选）
          </label>
          <input
            type="text"
            value={step.verifyFn || ''}
            onChange={(e) => onChange({ verifyFn: e.target.value })}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white font-mono text-sm focus:ring-2 focus:ring-blue-500"
            placeholder="verify_user_answer"
          />
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            如果不指定，将使用默认验证逻辑
          </p>
        </div>
      </div>
    </div>
  );
};

export default StepEditor;
