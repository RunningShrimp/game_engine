// 交互式教程系统 - 组件导出索引

// 主组件
export { default as TutorialSystem } from './TutorialSystem';
export { default as TutorialLibrary } from './TutorialLibrary';
export { default as CodeEditor } from './CodeEditor';

// 播放器组件
export { default as TutorialPlayer } from './Player/TutorialPlayer';
export { default as ProgressBar } from './Player/ProgressBar';
export { default as HintSystem } from './Player/HintSystem';
export { default as ChallengePanel } from './Player/ChallengePanel';
export { default as AchievementPopup } from './Player/AchievementPopup';

// 编辑器组件
export { default as TutorialEditor } from './Editor/TutorialEditor';
export { default as StepEditor } from './Editor/StepEditor';
export { default as ChallengeEditor } from './Editor/ChallengeEditor';
export { default as MetadataEditor } from './Editor/MetadataEditor';
export { default as TutorialPreview } from './Editor/TutorialPreview';

// 进度系统组件
export { default as ProgressDashboard } from './Progress/ProgressDashboard';
export { default as SkillTree } from './Progress/SkillTree';
export { default as AchievementsList } from './Progress/AchievementsList';
export { default as LearningPath } from './Progress/LearningPath';

// 类型定义
export type {
  Tutorial,
  TutorialStep,
  TutorialChallenge,
  TutorialProgress,
  UserStats,
  Badge,
  Achievement,
  SkillProgress,
  TutorialSession,
  TutorialCategory,
  TutorialDifficulty,
  StepType,
  ChallengeType,
} from '../../types/tutorial';

// API接口类型
export type {
  TutorialAPI,
  TutorialEditor as TutorialEditorInterface,
  TutorialPlayer as TutorialPlayerInterface,
  GamificationSystem,
  TutorialEvent,
} from '../../types/tutorial';
