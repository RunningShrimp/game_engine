import React, { useState } from 'react';
import { Target, CheckCircle2, Clock } from 'lucide-react';
import { TutorialChallenge } from '../../types/tutorial';

interface ChallengePanelProps {
  challenges: TutorialChallenge[];
  onSubmit: (answer: any) => Promise<void>;
}

const ChallengePanel: React.FC<ChallengePanelProps> = ({ challenges, onSubmit }) => {
  const [currentChallenge, setCurrentChallenge] = useState(0);
  const [userAnswer, setUserAnswer] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [result, setResult] = useState<{ success: boolean; message: string } | null>(null);

  const challenge = challenges[currentChallenge];

  const handleSubmit = async () => {
    setIsSubmitting(true);
    try {
      await onSubmit({ challengeId: challenge.id, answer: userAnswer });
      setResult({ success: true, message: '挑战完成！' });
    } catch (error) {
      setResult({ success: false, message: '答案不正确，请再试一次' });
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 p-6">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <Target className="w-5 h-5 text-purple-600 dark:text-purple-400" />
          <h3 className="font-semibold text-gray-900 dark:text-white">
            挑战 {currentChallenge + 1} / {challenges.length}
          </h3>
        </div>
        {challenge.timeLimit && (
          <div className="flex items-center gap-1 text-sm text-gray-600 dark:text-gray-400">
            <Clock className="w-4 h-4" />
            {Math.floor(challenge.timeLimit / 60)}:{(challenge.timeLimit % 60).toString().padStart(2, '0')}
          </div>
        )}
      </div>

      <div className="space-y-4">
        <div>
          <h4 className="font-medium text-gray-900 dark:text-white mb-2">
            {challenge.title}
          </h4>
          <p className="text-sm text-gray-600 dark:text-gray-400">
            {challenge.description}
          </p>
        </div>

        {/* 难度标签 */}
        <div className="flex items-center gap-2">
          <span className={`px-2 py-1 rounded text-xs font-medium ${
            challenge.difficulty === 'easy' ? 'bg-green-100 text-green-800' :
            challenge.difficulty === 'medium' ? 'bg-yellow-100 text-yellow-800' :
            challenge.difficulty === 'hard' ? 'bg-orange-100 text-orange-800' :
            'bg-red-100 text-red-800'
          }`}>
            {challenge.difficulty}
          </span>
          <span className="text-sm text-gray-600 dark:text-gray-400">
            奖励: {challenge.xpReward} XP
          </span>
        </div>

        {/* 答案输入 */}
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            你的答案
          </label>
          <textarea
            value={userAnswer}
            onChange={(e) => setUserAnswer(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500 focus:border-transparent"
            rows={4}
            placeholder="输入你的答案..."
          />
        </div>

        {/* 结果反馈 */}
        {result && (
          <div className={`p-3 rounded-lg ${
            result.success
              ? 'bg-green-50 dark:bg-green-900/20 text-green-800 dark:text-green-200'
              : 'bg-red-50 dark:bg-red-900/20 text-red-800 dark:text-red-200'
          }`}>
            <div className="flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4" />
              <span className="text-sm font-medium">{result.message}</span>
            </div>
          </div>
        )}

        {/* 提交按钮 */}
        <div className="flex justify-between">
          <button
            onClick={() => setCurrentChallenge(Math.max(0, currentChallenge - 1))}
            disabled={currentChallenge === 0}
            className="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
          >
            上一个挑战
          </button>
          <button
            onClick={handleSubmit}
            disabled={isSubmitting || !userAnswer.trim()}
            className="px-6 py-2 bg-purple-600 hover:bg-purple-700 text-white rounded-lg font-medium disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isSubmitting ? '提交中...' : '提交答案'}
          </button>
        </div>
      </div>
    </div>
  );
};

export default ChallengePanel;
