// 教程系统核心类型定义

export interface Tutorial {
  id: string;
  title: string;
  description: string;
  category: TutorialCategory;
  difficulty: TutorialDifficulty;
  estimatedTime: number; // 分钟
  prerequisites: string[]; // 前置教程ID
  skills: string[]; // 涉及的技能
  xpReward: number; // 经验值奖励
  badges: string[]; // 获得的徽章
  steps: TutorialStep[];
  challenges?: TutorialChallenge[];
  createdAt: Date;
  updatedAt: Date;
}

export type TutorialCategory = 'beginner' | 'intermediate' | 'advanced';
export type TutorialDifficulty = 'easy' | 'medium' | 'hard' | 'expert';

export interface TutorialStep {
  id: string;
  title: string;
  content: string; // Markdown格式
  type: StepType;
  codeTemplate?: string;
  expectedOutput?: string;
  hints: string[];
  verifyFn?: string; // 验证函数名
  resources?: StepResource[];
  order: number;
}

export type StepType =
  | 'concept'     // 概念讲解
  | 'demo'        // 演示
  | 'exercise'    // 练习题
  | 'challenge'   // 挑战
  | 'quiz';       // 测验

export interface StepResource {
  type: 'image' | 'video' | 'code' | 'document';
  url: string;
  title: string;
}

export interface TutorialChallenge {
  id: string;
  title: string;
  description: string;
  type: ChallengeType;
  difficulty: TutorialDifficulty;
  timeLimit?: number; // 秒
  starterCode?: string;
  solution?: string;
  testCases?: TestCase[];
  xpReward: number;
}

export type ChallengeType =
  | 'fill-blank'    // 填空题
  | 'debug'         // Bug修复
  | 'implement'     // 代码实现
  | 'optimization'  // 优化任务
  | 'creative';     // 创意任务

export interface TestCase {
  input: any;
  expectedOutput: any;
  description: string;
}

export interface TutorialProgress {
  userId: string;
  tutorialId: string;
  currentStep: number;
  completedSteps: string[];
  startedAt: Date;
  completedAt?: Date;
  timeSpent: number; // 秒
  attempts: number;
  hintsUsed: number;
}

export interface UserStats {
  userId: string;
  totalXP: number;
  level: number;
  currentLevelXP: number;
  nextLevelXP: number;
  completedTutorials: string[];
  inProgressTutorials: string[];
  badges: Badge[];
  skills: SkillProgress[];
  streakDays: number;
  lastActiveDate: Date;
  achievements: Achievement[];
}

export interface Badge {
  id: string;
  name: string;
  description: string;
  icon: string;
  rarity: 'common' | 'rare' | 'epic' | 'legendary';
  earnedAt: Date;
}

export interface SkillProgress {
  name: string;
  level: number;
  progress: number; // 0-100
  tutorialsCompleted: string[];
}

export interface Achievement {
  id: string;
  title: string;
  description: string;
  icon: string;
  xpReward: number;
  unlockedAt: Date;
  progress?: number;
  total?: number;
}

export interface TutorialSession {
  sessionId: string;
  tutorialId: string;
  stepId: string;
  code: string;
  output: string;
  status: 'running' | 'completed' | 'failed';
  timestamp: Date;
}

// API 接口
export interface TutorialAPI {
  // 教程管理
  getTutorials(): Promise<Tutorial[]>;
  getTutorial(id: string): Promise<Tutorial>;
  createTutorial(tutorial: Omit<Tutorial, 'id' | 'createdAt' | 'updatedAt'>): Promise<Tutorial>;
  updateTutorial(id: string, tutorial: Partial<Tutorial>): Promise<Tutorial>;
  deleteTutorial(id: string): Promise<void>;

  // 进度管理
  getProgress(tutorialId: string): Promise<TutorialProgress>;
  startTutorial(tutorialId: string): Promise<void>;
  completeStep(tutorialId: string, stepId: string): Promise<void>;
  saveProgress(tutorialId: string, progress: Partial<TutorialProgress>): Promise<void>;

  // 代码执行
  executeCode(code: string, language: string): Promise<string>;

  // 用户统计
  getUserStats(): Promise<UserStats>;
  getLeaderboard(limit?: number): Promise<UserStats[]>;
}

// 编辑器接口
export interface TutorialEditor {
  tutorial: Partial<Tutorial>;
  currentStep: number;

  // 基础操作
  createNewTutorial(): void;
  loadTutorial(id: string): Promise<void>;
  saveTutorial(): Promise<void>;
  publishTutorial(): Promise<void>;

  // 步骤管理
  addStep(step: Omit<TutorialStep, 'id' | 'order'>): void;
  updateStep(stepId: string, updates: Partial<TutorialStep>): void;
  deleteStep(stepId: string): void;
  reorderSteps(steps: TutorialStep[]): void;

  // 挑战管理
  addChallenge(challenge: Omit<TutorialChallenge, 'id'>): void;
  updateChallenge(challengeId: string, updates: Partial<TutorialChallenge>): void;
  deleteChallenge(challengeId: string): void;

  // 预览
  previewTutorial(): void;
  exportTutorial(): string;
}

// 播放器接口
export interface TutorialPlayer {
  tutorial: Tutorial | null;
  progress: TutorialProgress | null;
  currentStep: TutorialStep | null;

  // 导航
  loadTutorial(tutorialId: string): Promise<void>;
  nextStep(): void;
  previousStep(): void;
  goToStep(stepId: string): void;

  // 交互
  submitAnswer(answer: any): Promise<boolean>;
  showHint(hintIndex: number): void;
  skipStep(): void;

  // 进度
  getProgress(): TutorialProgress;
  saveProgress(): Promise<void>;
  completeTutorial(): Promise<void>;

  // 代码执行
  runCode(code: string): Promise<string>;
  resetCode(): void;
}

// 游戏化系统
export interface GamificationSystem {
  // 经验值
  addXP(amount: number, source: string): void;
  calculateLevel(totalXP: number): number;

  // 成就
  unlockAchievement(achievementId: string): void;
  checkAchievements(): void;

  // 徽章
  awardBadge(badgeId: string): void;
  hasBadge(badgeId: string): boolean;

  // 技能树
  updateSkill(skillName: string, xp: number): void;
  getSkillLevel(skillName: string): number;

  // 连续打卡
  updateStreak(): void;
}

// 事件类型
export type TutorialEvent =
  | { type: 'tutorial_started'; tutorialId: string }
  | { type: 'tutorial_completed'; tutorialId: string; duration: number }
  | { type: 'step_started'; stepId: string }
  | { type: 'step_completed'; stepId: string; attempts: number }
  | { type: 'hint_used'; stepId: string; hintIndex: number }
  | { type: 'challenge_completed'; challengeId: string; duration: number }
  | { type: 'achievement_unlocked'; achievementId: string }
  | { type: 'code_executed'; success: boolean; duration: number };
