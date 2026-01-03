import React, { useState, useEffect, useCallback } from 'react';
import { Play, Pause, SkipForward, SkipBack, Lightbulb, CheckCircle, XCircle, Trophy, Target } from 'lucide-react';
import { Tutorial, TutorialStep, TutorialProgress, ChallengeType } from '../../types/tutorial';
import { invoke } from '@tauri-apps/api/core';
import CodeEditor from '../CodeEditor';
import ProgressBar from './ProgressBar';
import HintSystem from './HintSystem';
import ChallengePanel from './ChallengePanel';
import AchievementPopup from './AchievementPopup';

interface TutorialPlayerProps {
  tutorialId: string;
  onComplete?: () => void;
  onProgressChange?: (progress: TutorialProgress) => void;
}

const TutorialPlayer: React.FC<TutorialPlayerProps> = ({
  tutorialId,
  onComplete,
  onProgressChange
}) => {
  const [tutorial, setTutorial] = useState<Tutorial | null>(null);
  const [progress, setProgress] = useState<TutorialProgress | null>(null);
  const [currentStep, setCurrentStep] = useState<TutorialStep | null>(null);
  const [currentStepIndex, setCurrentStepIndex] = useState(0);
  const [code, setCode] = useState('');
  const [output, setOutput] = useState('');
  const [isRunning, setIsRunning] = useState(false);
  const [showHints, setShowHints] = useState(false);
  const [currentHintIndex, setCurrentHintIndex] = useState(0);
  const [challengeResult, setChallengeResult] = useState<{ success: boolean; message: string } | null>(null);
  const [achievements, setAchievements] = useState<string[]>([]);
  const [showAchievement, setShowAchievement] = useState<string | null>(null);
  const [timeSpent, setTimeSpent] = useState(0);

  // 加载教程
  useEffect(() => {
    loadTutorial();
  }, [tutorialId]);

  // 计时器
  useEffect(() => {
    const timer = setInterval(() => {
      if (tutorial && progress) {
        setTimeSpent(prev => prev + 1);
      }
    }, 1000);

    return () => clearInterval(timer);
  }, [tutorial, progress]);

  const loadTutorial = async () => {
    try {
      const data = await invoke<Tutorial>('get_tutorial', { tutorialId });
      setTutorial(data);

      const prog = await invoke<TutorialProgress>('get_tutorial_progress', { tutorialId });
      setProgress(prog);

      if (data && prog) {
        const stepIndex = data.steps.findIndex(s => s.id === prog.currentStep);
        setCurrentStepIndex(stepIndex >= 0 ? stepIndex : 0);
        setCurrentStep(data.steps[0]);
        setCode(data.steps[0]?.codeTemplate || '');
      }
    } catch (error) {
      console.error('Failed to load tutorial:', error);
    }
  };

  const saveProgress = async () => {
    if (!tutorial || !progress) return;

    const updatedProgress: Partial<TutorialProgress> = {
      currentStep: tutorial.steps[currentStepIndex].id,
      timeSpent: timeSpent
    };

    await invoke('save_tutorial_progress', {
      tutorialId,
      progress: updatedProgress
    });

    onProgressChange?.({ ...progress, ...updatedProgress });
  };

  const nextStep = async () => {
    if (!tutorial || currentStepIndex >= tutorial.steps.length - 1) {
      // 教程完成
      await completeTutorial();
      return;
    }

    const nextIndex = currentStepIndex + 1;
    const nextStepData = tutorial.steps[nextIndex];

    setCurrentStepIndex(nextIndex);
    setCurrentStep(nextStepData);
    setCode(nextStepData.codeTemplate || '');
    setOutput('');
    setChallengeResult(null);
    setCurrentHintIndex(0);
    setShowHints(false);

    await saveProgress();
  };

  const previousStep = () => {
    if (currentStepIndex <= 0) return;

    const prevIndex = currentStepIndex - 1;
    const prevStepData = tutorial!.steps[prevIndex];

    setCurrentStepIndex(prevIndex);
    setCurrentStep(prevStepData);
    setCode(prevStepData.codeTemplate || '');
    setOutput('');
    setChallengeResult(null);
    setCurrentHintIndex(0);
    setShowHints(false);

    saveProgress();
  };

  const runCode = async () => {
    if (!currentStep) return;

    setIsRunning(true);
    try {
      const result = await invoke<string>('execute_tutorial_code', {
        code,
        language: 'rust'
      });
      setOutput(result);

      // 自动验证输出
      if (currentStep.expectedOutput && result.trim() === currentStep.expectedOutput.trim()) {
        setChallengeResult({ success: true, message: '输出正确！' });
        // 添加经验值
        await addXP(10, 'code_execution');
      }
    } catch (error) {
      setOutput(`错误: ${error}`);
      setChallengeResult({ success: false, message: '代码执行失败' });
    } finally {
      setIsRunning(false);
    }
  };

  const submitAnswer = async (answer: any) => {
    if (!currentStep || !tutorial) return;

    try {
      const result = await invoke<boolean>('verify_tutorial_answer', {
        tutorialId: tutorial.id,
        stepId: currentStep.id,
        answer
      });

      if (result) {
        setChallengeResult({ success: true, message: '答案正确！' });

        // 完成步骤
        await invoke('complete_tutorial_step', {
          tutorialId: tutorial.id,
          stepId: currentStep.id
        });

        // 检查成就
        await checkAchievements();
      } else {
        setChallengeResult({ success: false, message: '答案不正确，请再试一次' });
      }
    } catch (error) {
      console.error('Failed to submit answer:', error);
    }
  };

  const showHint = () => {
    if (!currentStep || currentHintIndex >= currentStep.hints.length) return;

    setShowHints(true);
    setCurrentHintIndex(currentHintIndex);

    // 记录提示使用
    if (tutorial && currentStep) {
      invoke('log_tutorial_hint', {
        tutorialId: tutorial.id,
        stepId: currentStep.id,
        hintIndex: currentHintIndex
      });
    }
  };

  const skipStep = async () => {
    if (!tutorial || !currentStep) return;

    const confirmed = confirm('确定要跳过这一步吗？你将不会获得这一步的经验值。');
    if (!confirmed) return;

    await nextStep();
  };

  const completeTutorial = async () => {
    if (!tutorial || !progress) return;

    try {
      await invoke('complete_tutorial', {
        tutorialId: tutorial.id
      });

      // 发放奖励
      await addXP(tutorial.xpReward, 'tutorial_completion');

      // 解锁徽章
      for (const badge of tutorial.badges) {
        await awardBadge(badge);
      }

      // 显示完成界面
      onComplete?.();
    } catch (error) {
      console.error('Failed to complete tutorial:', error);
    }
  };

  const addXP = async (amount: number, source: string) => {
    try {
      const stats = await invoke('add_user_xp', { amount, source });
      // 检查是否升级
      if (stats.levelUp) {
        setShowAchievement(`等级提升！当前等级: ${stats.level}`);
        setTimeout(() => setShowAchievement(null), 3000);
      }
    } catch (error) {
      console.error('Failed to add XP:', error);
    }
  };

  const awardBadge = async (badgeId: string) => {
    try {
      const badge = await invoke('award_badge', { badgeId });
      setShowAchievement(`获得徽章: ${badge.name}`);
      setTimeout(() => setShowAchievement(null), 3000);
    } catch (error) {
      console.error('Failed to award badge:', error);
    }
  };

  const checkAchievements = async () => {
    try {
      const newAchievements = await invoke<string[]>('check_user_achievements');
      if (newAchievements.length > 0) {
        setAchievements(prev => [...prev, ...newAchievements]);
        newAchievements.forEach(achievementId => {
          setShowAchievement(`解锁成就: ${achievementId}`);
          setTimeout(() => setShowAchievement(null), 3000);
        });
      }
    } catch (error) {
      console.error('Failed to check achievements:', error);
    }
  };

  if (!tutorial || !currentStep) {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p className="text-gray-600">加载教程中...</p>
        </div>
      </div>
    );
  }

  const progressPercent = progress ? ((currentStepIndex + 1) / tutorial.steps.length) * 100 : 0;

  return (
    <div className="h-screen flex flex-col bg-gray-50 dark:bg-gray-900">
      {/* 头部 */}
      <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-4">
        <div className="flex items-center justify-between mb-4">
          <div>
            <h1 className="text-2xl font-bold text-gray-900 dark:text-white">{tutorial.title}</h1>
            <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">{currentStep.title}</p>
          </div>
          <div className="flex items-center gap-4">
            <div className="text-right">
              <p className="text-sm text-gray-600 dark:text-gray-400">经验值</p>
              <p className="text-lg font-bold text-yellow-600 dark:text-yellow-400">{tutorial.xpReward} XP</p>
            </div>
            <Trophy className="w-8 h-8 text-yellow-500" />
          </div>
        </div>

        <ProgressBar
          current={currentStepIndex + 1}
          total={tutorial.steps.length}
          percent={progressPercent}
        />

        <div className="flex items-center justify-between mt-4 text-sm text-gray-600 dark:text-gray-400">
          <div className="flex items-center gap-4">
            <span>难度: {tutorial.difficulty}</span>
            <span>预计时间: {tutorial.estimatedTime} 分钟</span>
            <span>已用时间: {Math.floor(timeSpent / 60)}:{(timeSpent % 60).toString().padStart(2, '0')}</span>
          </div>
          <div className="flex items-center gap-2">
            <Target className="w-4 h-4" />
            <span>第 {currentStepIndex + 1} / {tutorial.steps.length} 步</span>
          </div>
        </div>
      </div>

      {/* 主内容区 */}
      <div className="flex-1 flex overflow-hidden">
        {/* 左侧：教程内容 */}
        <div className="flex-1 overflow-y-auto p-6">
          <div className="max-w-3xl mx-auto bg-white dark:bg-gray-800 rounded-lg shadow-sm p-8">
            <div className="prose dark:prose-invert max-w-none">
              <div dangerouslySetInnerHTML={{ __html: currentStep.content }} />
            </div>

            {/* 步骤类型标签 */}
            <div className="mt-6 flex items-center gap-2">
              <span className="px-3 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded-full text-sm font-medium">
                {currentStep.type}
              </span>
              {tutorial.skills.map(skill => (
                <span key={skill} className="px-3 py-1 bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200 rounded-full text-sm">
                  {skill}
                </span>
              ))}
            </div>
          </div>
        </div>

        {/* 右侧：代码编辑器和交互区域 */}
        <div className="w-1/2 border-l border-gray-200 dark:border-gray-700 flex flex-col">
          {/* 代码编辑器 */}
          {currentStep.codeTemplate && (
            <div className="flex-1 flex flex-col">
              <div className="bg-gray-100 dark:bg-gray-800 px-4 py-2 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">代码编辑器</span>
                <button
                  onClick={runCode}
                  disabled={isRunning}
                  className="px-4 py-1 bg-green-600 hover:bg-green-700 text-white rounded-md text-sm font-medium disabled:opacity-50 flex items-center gap-2"
                >
                  {isRunning ? (
                    <>
                      <Pause className="w-4 h-4" />
                      运行中...
                    </>
                  ) : (
                    <>
                      <Play className="w-4 h-4" />
                      运行代码
                    </>
                  )}
                </button>
              </div>
              <div className="flex-1">
                <CodeEditor
                  value={code}
                  onChange={setCode}
                  language="rust"
                  height="100%"
                />
              </div>

              {/* 输出 */}
              {output && (
                <div className="bg-gray-900 text-gray-100 p-4 font-mono text-sm overflow-auto max-h-48">
                  <pre>{output}</pre>
                </div>
              )}

              {/* 验证结果 */}
              {challengeResult && (
                <div className={`p-4 ${challengeResult.success ? 'bg-green-50 dark:bg-green-900/20' : 'bg-red-50 dark:bg-red-900/20'} border-t border-gray-200 dark:border-gray-700`}>
                  <div className="flex items-center gap-2">
                    {challengeResult.success ? (
                      <CheckCircle className="w-5 h-5 text-green-600 dark:text-green-400" />
                    ) : (
                      <XCircle className="w-5 h-5 text-red-600 dark:text-red-400" />
                    )}
                    <span className={challengeResult.success ? 'text-green-800 dark:text-green-200' : 'text-red-800 dark:text-red-200'}>
                      {challengeResult.message}
                    </span>
                  </div>
                </div>
              )}
            </div>
          )}

          {/* 挑战面板 */}
          {currentStep.type === 'challenge' && tutorial.challenges && (
            <ChallengePanel
              challenges={tutorial.challenges}
              onSubmit={submitAnswer}
            />
          )}

          {/* 提示系统 */}
          {showHints && currentStep.hints.length > 0 && (
            <HintSystem
              hints={currentStep.hints}
              currentIndex={currentHintIndex}
              onNext={() => setCurrentHintIndex(Math.min(currentHintIndex + 1, currentStep.hints.length - 1))}
              onPrevious={() => setCurrentHintIndex(Math.max(currentHintIndex - 1, 0))}
            />
          )}
        </div>
      </div>

      {/* 底部导航栏 */}
      <div className="bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 px-6 py-4">
        <div className="flex items-center justify-between">
          <button
            onClick={previousStep}
            disabled={currentStepIndex === 0}
            className="px-6 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg font-medium disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
          >
            <SkipBack className="w-4 h-4" />
            上一步
          </button>

          <div className="flex items-center gap-4">
            {currentStep.hints.length > 0 && (
              <button
                onClick={showHint}
                className="px-4 py-2 bg-yellow-100 dark:bg-yellow-900/20 hover:bg-yellow-200 dark:hover:bg-yellow-900/30 text-yellow-800 dark:text-yellow-200 rounded-lg font-medium flex items-center gap-2"
              >
                <Lightbulb className="w-4 h-4" />
                提示 ({currentHintIndex + 1}/{currentStep.hints.length})
              </button>
            )}

            <button
              onClick={skipStep}
              className="px-4 py-2 text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200 font-medium"
            >
              跳过
            </button>
          </div>

          <button
            onClick={nextStep}
            className="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium flex items-center gap-2"
          >
            {currentStepIndex === tutorial.steps.length - 1 ? '完成' : '下一步'}
            <SkipForward className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* 成就弹窗 */}
      {showAchievement && (
        <AchievementPopup
          message={showAchievement}
          onClose={() => setShowAchievement(null)}
        />
      )}
    </div>
  );
};

export default TutorialPlayer;
