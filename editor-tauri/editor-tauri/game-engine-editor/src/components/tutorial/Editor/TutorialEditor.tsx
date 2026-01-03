import React, { useState, useCallback } from 'react';
import { Save, Eye, Play, Plus, Trash2, ChevronUp, ChevronDown, BookOpen } from 'lucide-react';
import { Tutorial, TutorialStep, TutorialChallenge, TutorialEditor as IEditor } from '../../types/tutorial';
import StepEditor from './StepEditor';
import ChallengeEditor from './ChallengeEditor';
import MetadataEditor from './MetadataEditor';
import TutorialPreview from './TutorialPreview';

const TutorialEditor: React.FC = () => {
  const [tutorial, setTutorial] = useState<Partial<Tutorial>>({
    title: '',
    description: '',
    category: 'beginner',
    difficulty: 'easy',
    estimatedTime: 30,
    prerequisites: [],
    skills: [],
    xpReward: 100,
    badges: [],
    steps: []
  });

  const [currentStepIndex, setCurrentStepIndex] = useState<number | null>(null);
  const [currentChallengeIndex, setCurrentChallengeIndex] = useState<number | null>(null);
  const [showPreview, setShowPreview] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [activeTab, setActiveTab] = useState<'metadata' | 'steps' | 'challenges'>('metadata');

  // 保存教程
  const saveTutorial = async () => {
    setIsSaving(true);
    try {
      // 调用后端API保存
      await invoke('save_tutorial', { tutorial });
      alert('教程保存成功！');
    } catch (error) {
      console.error('保存失败:', error);
      alert('保存失败，请重试');
    } finally {
      setIsSaving(false);
    }
  };

  // 发布教程
  const publishTutorial = async () => {
    if (!confirm('确定要发布这个教程吗？发布后将无法修改。')) return;

    try {
      await invoke('publish_tutorial', { tutorialId: tutorial.id });
      alert('教程发布成功！');
    } catch (error) {
      console.error('发布失败:', error);
      alert('发布失败，请重试');
    }
  };

  // 添加步骤
  const addStep = useCallback(() => {
    const newStep: TutorialStep = {
      id: `step-${Date.now()}`,
      title: '新步骤',
      content: '<p>在这里输入步骤内容...</p>',
      type: 'concept',
      hints: [],
      order: (tutorial.steps?.length || 0)
    };

    setTutorial(prev => ({
      ...prev,
      steps: [...(prev.steps || []), newStep]
    }));
    setCurrentStepIndex(tutorial.steps?.length || 0);
  }, [tutorial.steps]);

  // 更新步骤
  const updateStep = useCallback((stepIndex: number, updates: Partial<TutorialStep>) => {
    setTutorial(prev => ({
      ...prev,
      steps: prev.steps?.map((step, index) =>
        index === stepIndex ? { ...step, ...updates } : step
      )
    }));
  }, []);

  // 删除步骤
  const deleteStep = useCallback((stepIndex: number) => {
    if (!confirm('确定要删除这个步骤吗？')) return;

    setTutorial(prev => ({
      ...prev,
      steps: prev.steps?.filter((_, index) => index !== stepIndex)
    }));

    if (currentStepIndex === stepIndex) {
      setCurrentStepIndex(null);
    } else if (currentStepIndex !== null && currentStepIndex > stepIndex) {
      setCurrentStepIndex(currentStepIndex - 1);
    }
  }, [currentStepIndex]);

  // 上移步骤
  const moveStepUp = useCallback((stepIndex: number) => {
    if (stepIndex === 0) return;

    setTutorial(prev => {
      const steps = [...(prev.steps || [])];
      [steps[stepIndex - 1], steps[stepIndex]] = [steps[stepIndex], steps[stepIndex - 1]];

      // 更新order
      steps.forEach((step, index) => step.order = index);

      return { ...prev, steps };
    });

    if (currentStepIndex === stepIndex) {
      setCurrentStepIndex(stepIndex - 1);
    } else if (currentStepIndex === stepIndex - 1) {
      setCurrentStepIndex(stepIndex);
    }
  }, [currentStepIndex]);

  // 下移步骤
  const moveStepDown = useCallback((stepIndex: number) => {
    if (!tutorial.steps || stepIndex >= tutorial.steps.length - 1) return;

    setTutorial(prev => {
      const steps = [...(prev.steps || [])];
      [steps[stepIndex], steps[stepIndex + 1]] = [steps[stepIndex + 1], steps[stepIndex]];

      // 更新order
      steps.forEach((step, index) => step.order = index);

      return { ...prev, steps };
    });

    if (currentStepIndex === stepIndex) {
      setCurrentStepIndex(stepIndex + 1);
    } else if (currentStepIndex === stepIndex + 1) {
      setCurrentStepIndex(stepIndex);
    }
  }, [currentStepIndex, tutorial.steps]);

  // 添加挑战
  const addChallenge = useCallback(() => {
    const newChallenge: TutorialChallenge = {
      id: `challenge-${Date.now()}`,
      title: '新挑战',
      description: '挑战描述',
      type: 'implement',
      difficulty: 'easy',
      xpReward: 50
    };

    setTutorial(prev => ({
      ...prev,
      challenges: [...(prev.challenges || []), newChallenge]
    }));
    setCurrentChallengeIndex((tutorial.challenges?.length || 0));
  }, [tutorial.challenges]);

  // 更新挑战
  const updateChallenge = useCallback((challengeIndex: number, updates: Partial<TutorialChallenge>) => {
    setTutorial(prev => ({
      ...prev,
      challenges: prev.challenges?.map((challenge, index) =>
        index === challengeIndex ? { ...challenge, ...updates } : challenge
      )
    }));
  }, []);

  // 删除挑战
  const deleteChallenge = useCallback((challengeIndex: number) => {
    if (!confirm('确定要删除这个挑战吗？')) return;

    setTutorial(prev => ({
      ...prev,
      challenges: prev.challenges?.filter((_, index) => index !== challengeIndex)
    }));

    if (currentChallengeIndex === challengeIndex) {
      setCurrentChallengeIndex(null);
    }
  }, [currentChallengeIndex]);

  // 导出教程
  const exportTutorial = () => {
    const json = JSON.stringify(tutorial, null, 2);
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${tutorial.title || 'tutorial'}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  if (showPreview) {
    return (
      <TutorialPreview
        tutorial={tutorial as Tutorial}
        onEdit={() => setShowPreview(false)}
        onSave={saveTutorial}
        onPublish={publishTutorial}
      />
    );
  }

  return (
    <div className="h-screen flex flex-col bg-gray-50 dark:bg-gray-900">
      {/* 头部工具栏 */}
      <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <BookOpen className="w-6 h-6 text-blue-600" />
            <h1 className="text-xl font-bold text-gray-900 dark:text-white">
              教程编辑器
            </h1>
            <input
              type="text"
              value={tutorial.title || ''}
              onChange={(e) => setTutorial(prev => ({ ...prev, title: e.target.value }))}
              placeholder="教程标题"
              className="px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <div className="flex items-center gap-2">
            <button
              onClick={() => setShowPreview(true)}
              className="px-4 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg font-medium flex items-center gap-2"
            >
              <Eye className="w-4 h-4" />
              预览
            </button>

            <button
              onClick={exportTutorial}
              className="px-4 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg font-medium"
            >
              导出
            </button>

            <button
              onClick={saveTutorial}
              disabled={isSaving}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium flex items-center gap-2 disabled:opacity-50"
            >
              <Save className="w-4 h-4" />
              {isSaving ? '保存中...' : '保存'}
            </button>

            <button
              onClick={publishTutorial}
              className="px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg font-medium flex items-center gap-2"
            >
              <Play className="w-4 h-4" />
              发布
            </button>
          </div>
        </div>

        {/* 标签页 */}
        <div className="flex gap-2 mt-4">
          <button
            onClick={() => setActiveTab('metadata')}
            className={`px-4 py-2 rounded-lg font-medium transition-colors ${
              activeTab === 'metadata'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 hover:bg-gray-300 dark:hover:bg-gray-600'
            }`}
          >
            元数据
          </button>
          <button
            onClick={() => setActiveTab('steps')}
            className={`px-4 py-2 rounded-lg font-medium transition-colors ${
              activeTab === 'steps'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 hover:bg-gray-300 dark:hover:bg-gray-600'
            }`}
          >
            步骤 ({tutorial.steps?.length || 0})
          </button>
          <button
            onClick={() => setActiveTab('challenges')}
            className={`px-4 py-2 rounded-lg font-medium transition-colors ${
              activeTab === 'challenges'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 hover:bg-gray-300 dark:hover:bg-gray-600'
            }`}
          >
            挑战 ({tutorial.challenges?.length || 0})
          </button>
        </div>
      </div>

      {/* 主内容区 */}
      <div className="flex-1 flex overflow-hidden">
        {/* 左侧：列表 */}
        <div className="w-80 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 overflow-y-auto">
          {activeTab === 'metadata' && (
            <div className="p-4">
              <h3 className="font-semibold text-gray-900 dark:text-white mb-2">基本信息</h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                在右侧面板编辑教程的元数据信息
              </p>
            </div>
          )}

          {activeTab === 'steps' && (
            <div className="p-4">
              <div className="flex items-center justify-between mb-4">
                <h3 className="font-semibold text-gray-900 dark:text-white">步骤列表</h3>
                <button
                  onClick={addStep}
                  className="p-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg"
                >
                  <Plus className="w-4 h-4" />
                </button>
              </div>

              <div className="space-y-2">
                {tutorial.steps?.map((step, index) => (
                  <div
                    key={step.id}
                    onClick={() => setCurrentStepIndex(index)}
                    className={`p-3 rounded-lg cursor-pointer transition-colors ${
                      currentStepIndex === index
                        ? 'bg-blue-100 dark:bg-blue-900/30 border-2 border-blue-500'
                        : 'bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600'
                    }`}
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex-1">
                        <p className="font-medium text-gray-900 dark:text-white text-sm">
                          {index + 1}. {step.title}
                        </p>
                        <p className="text-xs text-gray-600 dark:text-gray-400 mt-1">
                          {step.type}
                        </p>
                      </div>
                      <div className="flex gap-1">
                        <button
                          onClick={(e) => { e.stopPropagation(); moveStepUp(index); }}
                          disabled={index === 0}
                          className="p-1 hover:bg-gray-300 dark:hover:bg-gray-600 rounded disabled:opacity-50"
                        >
                          <ChevronUp className="w-3 h-3" />
                        </button>
                        <button
                          onClick={(e) => { e.stopPropagation(); moveStepDown(index); }}
                          disabled={index >= (tutorial.steps?.length || 0) - 1}
                          className="p-1 hover:bg-gray-300 dark:hover:bg-gray-600 rounded disabled:opacity-50"
                        >
                          <ChevronDown className="w-3 h-3" />
                        </button>
                        <button
                          onClick={(e) => { e.stopPropagation(); deleteStep(index); }}
                          className="p-1 hover:bg-red-200 dark:hover:bg-red-900 rounded text-red-600"
                        >
                          <Trash2 className="w-3 h-3" />
                        </button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {activeTab === 'challenges' && (
            <div className="p-4">
              <div className="flex items-center justify-between mb-4">
                <h3 className="font-semibold text-gray-900 dark:text-white">挑战列表</h3>
                <button
                  onClick={addChallenge}
                  className="p-2 bg-purple-600 hover:bg-purple-700 text-white rounded-lg"
                >
                  <Plus className="w-4 h-4" />
                </button>
              </div>

              <div className="space-y-2">
                {tutorial.challenges?.map((challenge, index) => (
                  <div
                    key={challenge.id}
                    onClick={() => setCurrentChallengeIndex(index)}
                    className={`p-3 rounded-lg cursor-pointer transition-colors ${
                      currentChallengeIndex === index
                        ? 'bg-purple-100 dark:bg-purple-900/30 border-2 border-purple-500'
                        : 'bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600'
                    }`}
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex-1">
                        <p className="font-medium text-gray-900 dark:text-white text-sm">
                          {challenge.title}
                        </p>
                        <p className="text-xs text-gray-600 dark:text-gray-400 mt-1">
                          {challenge.difficulty} · {challenge.xpReward} XP
                        </p>
                      </div>
                      <button
                        onClick={(e) => { e.stopPropagation(); deleteChallenge(index); }}
                        className="p-1 hover:bg-red-200 dark:hover:bg-red-900 rounded text-red-600"
                      >
                        <Trash2 className="w-3 h-3" />
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* 右侧：编辑器 */}
        <div className="flex-1 overflow-y-auto">
          {activeTab === 'metadata' && (
            <MetadataEditor
              tutorial={tutorial}
              onChange={setTutorial}
            />
          )}

          {activeTab === 'steps' && currentStepIndex !== null && (
            <StepEditor
              step={tutorial.steps![currentStepIndex]}
              onChange={(updates) => updateStep(currentStepIndex, updates)}
            />
          )}

          {activeTab === 'challenges' && currentChallengeIndex !== null && (
            <ChallengeEditor
              challenge={tutorial.challenges![currentChallengeIndex]}
              onChange={(updates) => updateChallenge(currentChallengeIndex, updates)}
            />
          )}

          {activeTab === 'steps' && currentStepIndex === null && (
            <div className="flex items-center justify-center h-full">
              <div className="text-center text-gray-500">
                <Plus className="w-12 h-12 mx-auto mb-4 opacity-50" />
                <p>点击左侧列表选择步骤</p>
                <p className="text-sm mt-2">或点击 + 添加新步骤</p>
              </div>
            </div>
          )}

          {activeTab === 'challenges' && currentChallengeIndex === null && (
            <div className="flex items-center justify-center h-full">
              <div className="text-center text-gray-500">
                <Plus className="w-12 h-12 mx-auto mb-4 opacity-50" />
                <p>点击左侧列表选择挑战</p>
                <p className="text-sm mt-2">或点击 + 添加新挑战</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default TutorialEditor;
