import React from 'react';
import { TutorialChallenge } from '../../types/tutorial';
import CodeEditor from '../CodeEditor';

interface ChallengeEditorProps {
  challenge: TutorialChallenge;
  onChange: (updates: Partial<TutorialChallenge>) => void;
}

const ChallengeEditor: React.FC<ChallengeEditorProps> = ({ challenge, onChange }) => {
  return (
    <div className="p-6 max-w-4xl mx-auto">
      <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">编辑挑战</h2>

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
                value={challenge.title}
                onChange={(e) => onChange({ title: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                类型
              </label>
              <select
                value={challenge.type}
                onChange={(e) => onChange({ type: e.target.value as any })}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500"
              >
                <option value="fill-blank">填空题</option>
                <option value="debug">Bug修复</option>
                <option value="implement">代码实现</option>
                <option value="optimization">优化任务</option>
                <option value="creative">创意任务</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                难度
              </label>
              <select
                value={challenge.difficulty}
                onChange={(e) => onChange({ difficulty: e.target.value as any })}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500"
              >
                <option value="easy">简单</option>
                <option value="medium">中等</option>
                <option value="hard">困难</option>
                <option value="expert">专家</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                经验值奖励
              </label>
              <input
                type="number"
                value={challenge.xpReward}
                onChange={(e) => onChange({ xpReward: parseInt(e.target.value) })}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500"
                min="0"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                时间限制（秒，可选）
              </label>
              <input
                type="number"
                value={challenge.timeLimit || ''}
                onChange={(e) => onChange({ timeLimit: e.target.value ? parseInt(e.target.value) : undefined })}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500"
                min="0"
                placeholder="不限制"
              />
            </div>
          </div>
        </div>

        {/* 描述 */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            挑战描述
          </label>
          <textarea
            value={challenge.description}
            onChange={(e) => onChange({ description: e.target.value })}
            rows={4}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500"
            placeholder="描述挑战的目标和要求"
          />
        </div>

        {/* 初始代码 */}
        {(challenge.type === 'implement' || challenge.type === 'debug' || challenge.type === 'optimization') && (
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-4">
              初始代码
            </label>
            <CodeEditor
              value={challenge.starterCode || ''}
              onChange={(code) => onChange({ starterCode: code })}
              language="rust"
              height="300px"
            />
          </div>
        )}

        {/* 参考答案 */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-4">
            参考答案（仅供编辑者查看）
          </label>
          <CodeEditor
            value={challenge.solution || ''}
            onChange={(code) => onChange({ solution: code })}
            language="rust"
            height="300px"
          />
        </div>

        {/* 测试用例 */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-sm">
          <div className="flex items-center justify-between mb-4">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              测试用例
            </label>
            <button
              onClick={() => {
                const newTestCases = [
                  ...(challenge.testCases || []),
                  { input: '', expectedOutput: '', description: '' }
                ];
                onChange({ testCases: newTestCases });
              }}
              className="px-3 py-1 bg-purple-600 hover:bg-purple-700 text-white rounded text-sm font-medium"
            >
              添加测试用例
            </button>
          </div>

          <div className="space-y-4">
            {challenge.testCases?.map((testCase, index) => (
              <div key={index} className="border border-gray-200 dark:border-gray-700 rounded-lg p-4">
                <div className="flex items-center justify-between mb-3">
                  <span className="font-medium text-gray-900 dark:text-white">测试用例 {index + 1}</span>
                  <button
                    onClick={() => {
                      const newTestCases = challenge.testCases?.filter((_, i) => i !== index);
                      onChange({ testCases: newTestCases });
                    }}
                    className="text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 px-2 py-1 rounded text-sm"
                  >
                    删除
                  </button>
                </div>

                <div className="space-y-3">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      描述
                    </label>
                    <input
                      type="text"
                      value={testCase.description}
                      onChange={(e) => {
                        const newTestCases = [...(challenge.testCases || [])];
                        newTestCases[index] = { ...testCase, description: e.target.value };
                        onChange({ testCases: newTestCases });
                      }}
                      className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white text-sm"
                      placeholder="测试用例描述"
                    />
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      输入（JSON格式）
                    </label>
                    <textarea
                      value={typeof testCase.input === 'string' ? testCase.input : JSON.stringify(testCase.input)}
                      onChange={(e) => {
                        try {
                          const input = JSON.parse(e.target.value);
                          const newTestCases = [...(challenge.testCases || [])];
                          newTestCases[index] = { ...testCase, input };
                          onChange({ testCases: newTestCases });
                        } catch {
                          // 忽略JSON解析错误
                        }
                      }}
                      rows={2}
                      className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white font-mono text-sm"
                      placeholder='{"key": "value"}'
                    />
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      预期输出（JSON格式）
                    </label>
                    <textarea
                      value={typeof testCase.expectedOutput === 'string' ? testCase.expectedOutput : JSON.stringify(testCase.expectedOutput)}
                      onChange={(e) => {
                        try {
                          const expectedOutput = JSON.parse(e.target.value);
                          const newTestCases = [...(challenge.testCases || [])];
                          newTestCases[index] = { ...testCase, expectedOutput };
                          onChange({ testCases: newTestCases });
                        } catch {
                          // 忽略JSON解析错误
                        }
                      }}
                      rows={2}
                      className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white font-mono text-sm"
                      placeholder='{"result": "expected"}'
                    />
                  </div>
                </div>
              </div>
            ))}
          </div>

          {!challenge.testCases || challenge.testCases.length === 0 && (
            <p className="text-sm text-gray-500 dark:text-gray-400 text-center py-4">
              暂无测试用例，点击上方按钮添加
            </p>
          )}
        </div>
      </div>
    </div>
  );
};

export default ChallengeEditor;
