# 交互式教程系统 - 完整文档

## 目录

1. [系统概述](#系统概述)
2. [快速开始](#快速开始)
3. [用户指南](#用户指南)
4. [教程创作指南](#教程创作指南)
5. [开发者指南](#开发者指南)
6. [API参考](#api参考)
7. [架构设计](#架构设计)

---

## 系统概述

### 功能特性

交互式教程系统是一个游戏化的学习平台，帮助开发者通过实践学习游戏引擎的使用。

#### 核心功能

1. **教程播放器**
   - 渐进式学习体验
   - 实时代码执行和验证
   - 智能提示系统
   - 进度自动保存

2. **教程编辑器**
   - 可视化教程创建
   - 富文本和Markdown支持
   - 实时预览
   - 一键发布

3. **进度追踪**
   - 详细的学习统计
   - 技能树系统
   - 成就徽章
   - 学习路径规划

4. **游戏化元素**
   - 经验值（XP）系统
   - 等级进阶
   - 徽章收集
   - 排行榜（可选）

### 技术栈

**前端**
- React 19.1.0
- TypeScript 5.8.3
- Tailwind CSS 4.1.18
- Tauri 2.0

**后端**
- Rust
- Tauri
- Serde (序列化)
- Tokio (异步运行时)

**数据存储**
- JSON文件存储（教程内容）
- 内存状态管理（用户进度）

---

## 快速开始

### 安装

系统已集成到编辑器中，无需额外安装。

### 启动教程系统

1. 启动游戏引擎编辑器
2. 点击主菜单中的"教程"选项
3. 进入教程系统界面

### 开始学习

1. **浏览教程库**
   - 查看所有可用教程
   - 按类别/难度筛选
   - 搜索特定主题

2. **选择教程**
   - 点击任意教程卡片
   - 查看教程详情
   - 检查前置条件

3. **开始学习**
   - 跟随步骤指导
   - 完成代码练习
   - 获得经验值和徽章

---

## 用户指南

### 教程界面

#### 教程库

- **教程卡片**：显示教程标题、描述、难度、时间、奖励
- **筛选器**：按类别（入门/进阶/高级）和难度筛选
- **搜索**：按标题、描述、技能标签搜索

#### 教程播放器

```
┌─────────────────────────────────────────────────┐
│ 教程标题                    进度条  ████░░░░  │
│ 第 X / Y 步                                XP  │
├─────────────────────────────────────────────────┤
│                                                 │
│  教程内容区（左侧）                             │
│  - 概念讲解                                     │
│  - 代码示例                                     │
│  - 练习说明                                     │
│                                                 │
├─────────────────────────────────────────────────┤
│  代码编辑器（右侧）                              │
│  ┌───────────────────────────────────────────┐  │
│  │ // 在这里编写代码                          │  │
│  │ fn main() {                               │  │
│  │     println!("Hello");                    │  │
│  │ }                                         │  │
│  └───────────────────────────────────────────┘  │
│                                                 │
│  [运行代码]  [输出显示区]                       │
│                                                 │
├─────────────────────────────────────────────────┤
│  [提示] [跳过]           [上一步]      [下一步] │
└─────────────────────────────────────────────────┘
```

#### 进度面板

- **总览**：XP、等级、完成教程数、徽章数
- **技能树**：各技能的熟练度
- **成就**：已解锁和未解锁的成就
- **学习路径**：推荐的学习顺序

### 操作说明

#### 导航教程

- **下一步**：完成当前步骤后进入下一步
- **上一步**：返回前一步骤
- **跳过**：跳过当前步骤（无XP奖励）
- **提示**：获取帮助（会减少XP奖励）

#### 代码练习

1. 阅读步骤说明
2. 在代码编辑器中编写代码
3. 点击"运行代码"测试
4. 查看输出结果
5. 修改并重试直到正确

#### 获得奖励

- **完成步骤**：基础XP
- **完成教程**：大量XP + 徽章
- **挑战任务**：额外XP
- **成就**：特殊奖励

---

## 教程创作指南

### 教程结构

一个完整的教程包含以下部分：

```json
{
  "id": "tutorial-unique-id",
  "title": "教程标题",
  "description": "简短描述",
  "category": "beginner|intermediate|advanced",
  "difficulty": "easy|medium|hard|expert",
  "estimated_time": 30,
  "prerequisites": ["tutorial-id-1", "tutorial-id-2"],
  "skills": ["技能1", "技能2"],
  "xp_reward": 100,
  "badges": ["badge-id"],
  "steps": [...],
  "challenges": [...]
}
```

### 步骤类型

#### 1. 概念讲解 (concept)

用于介绍理论知识：

```json
{
  "type": "concept",
  "title": "概念名称",
  "content": "<p>HTML格式的教学内容</p>",
  "hints": ["提示1", "提示2"]
}
```

#### 2. 演示 (demo)

展示代码示例：

```json
{
  "type": "demo",
  "title": "代码演示",
  "content": "<p>说明文字</p>",
  "code_template": "// 示例代码",
  "expected_output": "预期输出"
}
```

#### 3. 练习题 (exercise)

用户需要编写代码：

```json
{
  "type": "exercise",
  "title": "练习任务",
  "content": "<p>任务说明</p>",
  "code_template": "// 预填充代码\n// TODO: 学生需要完成的部分",
  "expected_output": "验证输出",
  "hints": ["提示1", "提示2", "提示3"]
}
```

#### 4. 挑战 (challenge)

复杂任务：

```json
{
  "type": "challenge",
  "title": "挑战任务",
  "content": "<p>任务描述</p>",
  "code_template": "// 初始代码",
  "verify_fn": "custom_verify_function"
}
```

### 挑战设计

```json
{
  "id": "challenge-id",
  "title": "挑战标题",
  "description": "详细描述",
  "type": "fill-blank|debug|implement|optimization|creative",
  "difficulty": "easy",
  "time_limit": 300,
  "starter_code": "// 初始代码",
  "solution": "// 参考答案",
  "test_cases": [
    {
      "input": {...},
      "expected_output": {...},
      "description": "测试描述"
    }
  ],
  "xp_reward": 50
}
```

### 最佳实践

1. **渐进式难度**
   - 从简单到复杂
   - 每步建立在前一步基础上
   - 提供足够的脚手架

2. **清晰的说明**
   - 使用简洁的语言
   - 提供代码注释
   - 添加视觉辅助（图片、图表）

3. **合理的提示**
   - 第一级提示：轻微引导
   - 第二级提示：更具体
   - 第三级提示：接近答案

4. **及时反馈**
   - 即时代码验证
   - 清晰的错误信息
   - 成功时的鼓励

5. **奖励设计**
   - 完成奖励：50-200 XP
   - 挑战奖励：额外 50-150 XP
   - 难度越高，奖励越多

---

## 开发者指南

### 项目结构

```
tutorial-system/
├── frontend/
│   ├── Player/              # 教程播放器
│   │   ├── TutorialPlayer.tsx
│   │   ├── ProgressBar.tsx
│   │   ├── HintSystem.tsx
│   │   ├── ChallengePanel.tsx
│   │   └── AchievementPopup.tsx
│   ├── Editor/              # 教程编辑器
│   │   ├── TutorialEditor.tsx
│   │   ├── StepEditor.tsx
│   │   ├── ChallengeEditor.tsx
│   │   ├── MetadataEditor.tsx
│   │   └── TutorialPreview.tsx
│   ├── Progress/            # 进度系统
│   │   ├── ProgressDashboard.tsx
│   │   ├── SkillTree.tsx
│   │   ├── AchievementsList.tsx
│   │   └── LearningPath.tsx
│   ├── CodeEditor.tsx       # 代码编辑器
│   ├── TutorialSystem.tsx   # 主入口
│   └── TutorialLibrary.tsx  # 教程库
├── backend/
│   └── tutorial/
│       └── mod.rs           # Rust后端API
├── content/
│   ├── beginner/            # 入门教程
│   ├── intermediate/        # 进阶教程
│   └── advanced/            # 高级教程
└── types/
    └── tutorial.ts          # TypeScript类型定义
```

### 前端组件

#### TutorialPlayer

教程播放器主组件，管理整个学习流程。

```typescript
interface TutorialPlayerProps {
  tutorialId: string;
  onComplete?: () => void;
  onProgressChange?: (progress: TutorialProgress) => void;
}
```

**主要方法**：
- `loadTutorial()` - 加载教程数据
- `nextStep()` - 进入下一步
- `previousStep()` - 返回上一步
- `runCode()` - 执行用户代码
- `submitAnswer()` - 提交答案

#### TutorialEditor

教程创建和编辑工具。

```typescript
interface TutorialEditor {
  createNewTutorial(): void;
  addStep(step: StepContent): void;
  updateStep(stepId: string, updates: Partial<Step>): void;
  deleteStep(stepId: string): void;
  publish(): Promise<void>;
}
```

#### ProgressDashboard

显示用户学习进度和成就。

```typescript
interface ProgressDashboardProps {
  userId: string;
}
```

### 后端API

所有教程相关的Tauri命令：

```rust
// 教程管理
get_tutorials() -> Result<Vec<Tutorial>>
get_tutorial(id: String) -> Result<Tutorial>
create_tutorial(tutorial: Tutorial) -> Result<Tutorial>
update_tutorial(id: String, tutorial: Tutorial) -> Result<Tutorial>
delete_tutorial(id: String) -> Result<()>

// 进度管理
get_tutorial_progress(tutorial_id: String, user_id: String) -> Result<UserProgress>
start_tutorial(tutorial_id: String, user_id: String) -> Result<()>
complete_tutorial_step(tutorial_id: String, step_id: String, user_id: String) -> Result<()>
save_tutorial_progress(...) -> Result<()>

// 用户统计
get_user_stats(user_id: String) -> Result<UserStats>
get_leaderboard(limit: usize) -> Result<Vec<UserStats>>
add_user_xp(user_id: String, amount: u32, source: String) -> Result<UserStats>
award_badge(user_id: String, badge_id: String) -> Result<Badge>

// 代码执行
execute_tutorial_code(code: String, language: String) -> Result<String>
verify_tutorial_answer(...) -> Result<bool>
```

### 数据模型

#### Tutorial

```typescript
interface Tutorial {
  id: string;
  title: string;
  description: string;
  category: 'beginner' | 'intermediate' | 'advanced';
  difficulty: 'easy' | 'medium' | 'hard' | 'expert';
  estimatedTime: number;
  prerequisites: string[];
  skills: string[];
  xpReward: number;
  badges: string[];
  steps: TutorialStep[];
  challenges?: TutorialChallenge[];
  createdAt: Date;
  updatedAt: Date;
}
```

#### UserStats

```typescript
interface UserStats {
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
```

---

## API参考

### 前端API

#### 加载教程列表

```typescript
import { invoke } from '@tauri-apps/api/core';

const tutorials = await invoke<Tutorial[]>('get_tutorials');
```

#### 开始教程

```typescript
await invoke('start_tutorial', {
  tutorialId: 'tutorial-001',
  userId: 'user-123'
});
```

#### 提交步骤

```typescript
await invoke('complete_tutorial_step', {
  tutorialId: 'tutorial-001',
  stepId: 'step-001-01',
  userId: 'user-123'
});
```

#### 执行代码

```typescript
const output = await invoke<string>('execute_tutorial_code', {
  code: 'println!("Hello")',
  language: 'rust'
});
```

### 后端API扩展

#### 添加自定义验证函数

```rust
pub fn verify_custom_answer(
    tutorial_id: &str,
    step_id: &str,
    answer: &serde_json::Value,
) -> Result<bool, String> {
    // 自定义验证逻辑
    match step_id {
        "step-001-03" => {
            // 特定步骤的验证
            Ok(true)
        }
        _ => Ok(false)
    }
}
```

#### 自定义成就系统

```rust
fn check_custom_achievements(stats: &mut UserStats) {
    // 添加自定义成就检查
    if stats.total_xp >= 10000 {
        stats.achievements.push(Achievement {
            id: "xp_master".to_string(),
            title: "经验大师".to_string(),
            description: "累计获得10000 XP".to_string(),
            icon: "💎".to_string(),
            xp_reward: 500,
            unlocked_at: Utc::now(),
            progress: Some(stats.total_xp),
            total: Some(10000),
        });
    }
}
```

---

## 架构设计

### 系统架构图

```
┌─────────────────────────────────────────────────┐
│              前端 (React + TypeScript)           │
├─────────────────────────────────────────────────┤
│  TutorialSystem  │  Player  │  Editor  │  Progress │
└─────────────────┬───────────────────────────────┘
                  │
                  │ Tauri IPC
                  │
┌─────────────────▼───────────────────────────────┐
│              后端 (Rust)                        │
├─────────────────────────────────────────────────┤
│  Tutorial Manager  │  Progress  │  Gamification  │
└─────────────────┬───────────────────────────────┘
                  │
                  │ File I/O
                  │
┌─────────────────▼───────────────────────────────┐
│              数据层                             │
├─────────────────────────────────────────────────┤
│  Tutorial Files (.json)  │  User Progress Data │
└─────────────────────────────────────────────────┘
```

### 数据流

1. **加载教程**
   ```
   前端请求 → get_tutorials()
   → 后端读取JSON文件
   → 返回教程列表
   → 前端显示
   ```

2. **学习流程**
   ```
   用户操作 → 更新进度
   → save_tutorial_progress()
   → 后端保存到内存
   → 返回确认
   → 前端更新UI
   ```

3. **代码执行**
   ```
   用户输入代码 → execute_tutorial_code()
   → 后端执行沙箱
   → 返回输出
   → 前端显示结果
   ```

### 状态管理

- **教程状态**：后端内存，程序启动时从文件加载
- **用户进度**：后端内存，可持久化到本地存储
- **临时状态**：前端React state

### 安全考虑

1. **代码执行沙箱**
   - 使用独立进程执行用户代码
   - 限制资源使用（CPU、内存）
   - 超时机制防止无限循环

2. **输入验证**
   - 所有用户输入都经过验证
   - 防止XSS攻击（富文本内容）
   - 文件路径安全检查

3. **数据持久化**
   - 定期保存用户进度
   - 数据备份机制
   - 版本控制

---

## 扩展和定制

### 添加新的步骤类型

1. 在 `types/tutorial.ts` 中添加新类型：

```typescript
export type StepType =
  | 'concept'
  | 'demo'
  | 'exercise'
  | 'challenge'
  | 'quiz'
  | 'interactive'; // 新类型
```

2. 在 `TutorialPlayer.tsx` 中添加渲染逻辑：

```typescript
if (currentStep.type === 'interactive') {
  return <InteractiveStep step={currentStep} />;
}
```

3. 在后端添加相应的验证逻辑。

### 自定义奖励系统

```rust
// 修改 XP 计算公式
fn calculate_xp_reward(
    base_xp: u32,
    time_used: u64,
    hints_used: u32,
) -> u32 {
    let time_bonus = if time_used < expected_time {
        base_xp / 10
    } else {
        0
    };

    let hint_penalty = (hints_used as u32) * 10;

    base_xp + time_bonus - hint_penalty
}
```

### 多语言支持

1. 创建语言文件：

```json
// public/locales/zh-CN.json
{
  "tutorial": {
    "start": "开始学习",
    "next": "下一步",
    "previous": "上一步"
  }
}
```

2. 使用 i18n 库加载翻译

---

## 故障排除

### 常见问题

1. **教程无法加载**
   - 检查JSON文件格式是否正确
   - 确认文件路径正确
   - 查看浏览器控制台错误

2. **代码执行失败**
   - 检查代码语法是否正确
   - 确认执行沙箱正常运行
   - 查看后端日志

3. **进度未保存**
   - 检查本地存储权限
   - 确认网络连接（如果使用云同步）
   - 尝试手动保存

### 调试模式

启用调试日志：

```rust
// 在 .env 文件中
TAURI_DEBUG=true
RUST_LOG=debug
```

---

## 最佳实践总结

### 教程设计

1. **保持简洁**：每步聚焦一个概念
2. **实践优先**：让用户动手写代码
3. **及时反馈**：立即显示执行结果
4. **渐进难度**：从简单到复杂
5. **真实场景**：使用实际游戏开发案例

### 系统使用

1. **定期保存**：确保进度不丢失
2. **合理使用提示**：避免过度依赖
3. **完成挑战**：获得额外XP
4. **关注技能树**：平衡发展各项技能
5. **参与社区**：分享学习经验

---

## 未来计划

- [ ] 在线社区和分享功能
- [ ] 多人协作学习
- [ ] VR/AR教程支持
- [ ] AI辅助学习助手
- [ ] 移动端适配
- [ ] 离线模式支持
- [ ] 视频教程集成
- [ ] 实时代码协作

---

## 贡献指南

欢迎贡献教程和功能改进！

1. Fork 项目
2. 创建特性分支
3. 提交更改
4. 创建 Pull Request

---

## 许可证

MIT License

---

## 联系方式

- GitHub Issues: [项目地址]
- 邮件: support@example.com
- 文档: [在线文档链接]

---

**最后更新**: 2026-01-02
**版本**: 1.0.0
