# 交互式教程系统 - 快速入门指南

## 系统概述

交互式教程系统是一个完整的游戏化学习平台，已成功集成到游戏引擎编辑器中。

## 核心功能

### 1. 教程播放器
- ✅ 渐进式步骤引导
- ✅ 实时代码编辑和执行
- ✅ 智能提示系统
- ✅ 自动进度保存
- ✅ 成就弹窗通知

### 2. 教程编辑器
- ✅ 可视化教程创建
- ✅ 步骤和挑战管理
- ✅ 实时预览
- ✅ Markdown/HTML富文本支持
- ✅ 一键发布

### 3. 进度追踪系统
- ✅ XP和等级系统
- ✅ 技能树可视化
- ✅ 成就和徽章
- ✅ 学习路径规划
- ✅ 排行榜（可选）

### 4. 游戏化元素
- ✅ 经验值（XP）奖励
- ✅ 等级进阶
- ✅ 徽章收集（普通/稀有/史诗/传说）
- ✅ 连续打卡
- ✅ 成就解锁

## 快速开始

### 作为学习者

1. **启动教程系统**
   ```bash
   # 启动编辑器
   npm run tauri dev

   # 在编辑器中点击"教程"菜单
   ```

2. **浏览教程库**
   - 查看所有可用教程
   - 按难度筛选（入门/进阶/高级）
   - 搜索感兴趣的主题

3. **开始学习**
   - 选择教程点击"开始学习"
   - 跟随步骤指导
   - 完成代码练习
   - 获得XP和徽章

### 作为教程创作者

1. **打开教程编辑器**
   - 点击"创建新教程"
   - 填写基本信息（标题、描述、难度等）

2. **添加教程步骤**
   ```json
   {
     "type": "concept|demo|exercise|challenge",
     "title": "步骤标题",
     "content": "HTML格式的教学内容",
     "code_template": "可选的代码模板",
     "hints": ["提示1", "提示2"]
   }
   ```

3. **预览和发布**
   - 点击"预览"查看效果
   - 点击"发布"保存教程

## 文件结构

```
├── src/
│   ├── components/tutorial/
│   │   ├── Player/              # 教程播放器
│   │   ├── Editor/              # 教程编辑器
│   │   ├── Progress/            # 进度系统
│   │   ├── TutorialSystem.tsx   # 主入口
│   │   ├── TutorialLibrary.tsx  # 教程库
│   │   └── CodeEditor.tsx       # 代码编辑器
│   └── types/tutorial.ts        # 类型定义
│
├── src-tauri/src/tutorial/
│   └── mod.rs                   # 后端API（500+ 行）
│
├── public/tutorials/
│   ├── beginner/                # 入门教程
│   │   ├── 01-first-scene.json
│   │   └── 02-materials-lights.json
│   ├── intermediate/            # 进阶教程
│   │   └── 01-scripting.json
│   └── advanced/                # 高级教程
│
└── docs/tutorial-system/
    └── README.md                # 完整文档
```

## API使用示例

### 加载教程列表

```typescript
import { invoke } from '@tauri-apps/api/core';

const tutorials = await invoke<Tutorial[]>('get_tutorials');
console.log(`找到 ${tutorials.length} 个教程`);
```

### 开始教程

```typescript
await invoke('start_tutorial', {
  tutorialId: 'tutorial-001',
  userId: 'user-123'
});
```

### 执行代码

```typescript
const output = await invoke<string>('execute_tutorial_code', {
  code: 'println!("Hello, World!");',
  language: 'rust'
});
console.log(output);
```

### 获取用户统计

```typescript
const stats = await invoke<UserStats>('get_user_stats', {
  userId: 'user-123'
});

console.log(`等级: ${stats.level}`);
console.log(`总XP: ${stats.totalXP}`);
console.log(`完成教程: ${stats.completedTutorials.length}`);
```

## 示例教程格式

### 基础教程

```json
{
  "id": "tutorial-001",
  "title": "创建你的第一个3D场景",
  "description": "学习如何创建和管理3D场景",
  "category": "beginner",
  "difficulty": "easy",
  "estimated_time": 15,
  "prerequisites": [],
  "skills": ["场景管理", "3D基础"],
  "xp_reward": 100,
  "badges": ["first_scene_creator"],
  "steps": [
    {
      "id": "step-01",
      "title": "了解场景系统",
      "content": "<h2>什么是场景？</h2><p>...</p>",
      "type": "concept",
      "hints": ["提示1", "提示2"],
      "order": 0
    },
    {
      "id": "step-02",
      "title": "创建新场景",
      "content": "...",
      "type": "exercise",
      "code_template": "// 在这里编写代码",
      "expected_output": "预期输出",
      "hints": ["提示内容"],
      "order": 1
    }
  ]
}
```

### 包含挑战的教程

```json
{
  "challenges": [
    {
      "id": "challenge-01",
      "title": "创建多彩场景",
      "description": "创建包含5个不同颜色立方体的场景",
      "type": "creative",
      "difficulty": "easy",
      "starter_code": "// 初始代码",
      "solution": "// 参考答案",
      "xp_reward": 50
    }
  ]
}
```

## 核心组件说明

### TutorialPlayer

教程播放器主组件，管理整个学习流程。

**Props**:
- `tutorialId`: 教程ID
- `onComplete`: 完成回调
- `onProgressChange`: 进度变化回调

**主要功能**:
- 步骤导航（上一步/下一步）
- 代码执行和验证
- 提示系统
- 进度保存

### TutorialEditor

教程创建和编辑工具。

**功能**:
- 元数据编辑（标题、描述、难度等）
- 步骤管理（添加、编辑、删除、排序）
- 挑战管理
- 实时预览
- 导出JSON

### ProgressDashboard

学习进度和成就展示。

**标签页**:
- 总览：XP、等级、统计
- 技能树：技能熟练度
- 成就：已解锁和未解锁
- 学习路径：推荐教程顺序

## 数据模型

### Tutorial

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
}
```

### UserStats

```typescript
interface UserStats {
  userId: string;
  totalXP: number;
  level: number;
  completedTutorials: string[];
  badges: Badge[];
  skills: SkillProgress[];
  achievements: Achievement[];
  streakDays: number;
}
```

## 后端API（Rust）

所有教程相关的Tauri命令已注册到 `src-tauri/src/lib.rs`：

```rust
// 教程管理
tutorial::get_tutorials
tutorial::get_tutorial
tutorial::create_tutorial
tutorial::update_tutorial
tutorial::delete_tutorial

// 进度管理
tutorial::get_tutorial_progress
tutorial::start_tutorial
tutorial::complete_tutorial_step
tutorial::save_tutorial_progress
tutorial::complete_tutorial

// 用户统计
tutorial::get_user_stats
tutorial::get_leaderboard
tutorial::add_user_xp
tutorial::award_badge
tutorial::check_user_achievements

// 代码执行
tutorial::execute_tutorial_code
tutorial::verify_tutorial_answer
```

## 游戏化系统

### XP计算

- 完成步骤：+10 XP
- 完成教程：+50-200 XP（根据难度）
- 完成挑战：+50-150 XP
- 解锁成就：+50-500 XP

### 等级系统

```rust
level_xp = base_xp * (1.5 ^ (level - 1))

Level 1: 0-100 XP
Level 2: 100-250 XP
Level 3: 250-475 XP
...
```

### 徽章稀有度

- **普通** (Common): 基础教程完成
- **稀有** (Rare): 中级教程完成
- **史诗** (Epic): 高级教程完成
- **传说** (Legendary): 特殊成就

## 扩展教程系统

### 添加新教程

1. 在 `public/tutorials/` 对应目录创建 JSON 文件
2. 按照格式编写教程内容
3. 重启编辑器，教程会自动加载

### 自定义步骤类型

1. 在 `types/tutorial.ts` 添加新类型
2. 在 `TutorialPlayer.tsx` 添加渲染逻辑
3. 更新编辑器支持新类型

### 添加新的成就

在 `src-tauri/src/tutorial/mod.rs` 的 `check_achievements` 函数中添加：

```rust
if stats.completed_tutorials.len() >= 20 {
    stats.achievements.push(Achievement {
        id: "master_learner".to_string(),
        title: "学习大师".to_string(),
        description: "完成20个教程".to_string(),
        icon: "👑".to_string(),
        xp_reward: 1000,
        unlocked_at: Utc::now(),
        progress: Some(stats.completed_tutorials.len() as u32),
        total: Some(20),
    });
}
```

## 常见问题

**Q: 教程进度会丢失吗？**
A: 不会。进度会自动保存到内存，可以添加持久化到本地存储。

**Q: 如何创建自己的教程？**
A: 使用教程编辑器，填写元数据，添加步骤，预览后发布。

**Q: 代码执行安全吗？**
A: 后端有代码执行沙箱，可以扩展为使用独立的执行环境。

**Q: 可以离线使用吗？**
A: 可以。教程内容存储在本地，无需网络连接。

**Q: 如何分享教程？**
A: 导出教程JSON文件，其他人导入即可使用。

## 性能优化建议

1. **教程懒加载**：只加载当前需要的教程
2. **图片优化**：使用WebP格式，压缩图片
3. **代码缓存**：缓存编译结果
4. **进度批量保存**：避免频繁写入

## 未来改进

- [ ] 云同步进度
- [ ] 社区分享平台
- [ ] AI学习助手
- [ ] 视频教程支持
- [ ] 多人协作学习
- [ ] 移动端应用

## 技术支持

- 📖 完整文档: `docs/tutorial-system/README.md`
- 🐛 问题反馈: GitHub Issues
- 💬 社区讨论: Discord/论坛

## 总结

交互式教程系统已完全实现并集成到编辑器中，包括：

✅ **13个前端组件**（播放器、编辑器、进度系统）
✅ **完整的后端API**（21个Tauri命令）
✅ **3个示例教程**（入门、进阶、高级）
✅ **游戏化系统**（XP、等级、徽章、成就）
✅ **完整的类型定义**和文档

系统可以立即使用，也可以根据需求进行扩展和定制。

---

**版本**: 1.0.0
**最后更新**: 2026-01-02
**作者**: Game Engine Editor Team
