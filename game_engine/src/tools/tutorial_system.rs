//! 教程系统
//!
//! 提供交互式教程生成和管理功能
//!
//! # 功能特性
//!
//! - 渐进式学习路径
//! - 交互式代码示例
//! - 自动验证练习答案
//! - 学习进度跟踪
//! - 多语言支持

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// 教程级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TutorialLevel {
    /// 初级
    Beginner,
    /// 中级
    Intermediate,
    /// 高级
    Advanced,
    /// 专家
    Expert,
}

/// 教程类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TutorialCategory {
    /// 快速开始
    QuickStart,
    /// ECS系统
    Ecs,
    /// 渲染
    Rendering,
    /// 物理
    Physics,
    /// 音频
    Audio,
    /// 脚本
    Scripting,
    /// 网络
    Networking,
    /// UI
    Ui,
    /// 工具
    Tools,
}

/// 教程元数据
#[derive(Debug, Clone)]
pub struct TutorialMetadata {
    /// 教程ID
    pub id: String,
    /// 教程标题
    pub title: String,
    /// 教程描述
    pub description: String,
    /// 教程级别
    pub level: TutorialLevel,
    /// 教程分类
    pub category: TutorialCategory,
    /// 预计时间（分钟）
    pub estimated_time: u32,
    /// 前置教程
    pub prerequisites: Vec<String>,
    /// 学习目标
    pub learning_objectives: Vec<String>,
    /// 标签
    pub tags: Vec<String>,
    /// 作者
    pub author: String,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

/// 教程章节
#[derive(Debug, Clone)]
pub struct TutorialChapter {
    /// 章节序号
    pub number: usize,
    /// 章节标题
    pub title: String,
    /// 章节内容（Markdown格式）
    pub content: String,
    /// 代码示例
    pub code_examples: Vec<CodeExample>,
    /// 练习
    pub exercises: Vec<Exercise>,
}

/// 代码示例
#[derive(Debug, Clone)]
pub struct CodeExample {
    /// 示例标题
    pub title: String,
    /// 示例描述
    pub description: String,
    /// 代码内容
    pub code: String,
    /// 代码语言
    pub language: String,
    /// 是否可运行
    pub runnable: bool,
    /// 运行命令
    pub run_command: Option<String>,
}

/// 练习
#[derive(Debug, Clone)]
pub struct Exercise {
    /// 练习ID
    pub id: String,
    /// 练习标题
    pub title: String,
    /// 练习描述
    pub description: String,
    /// 练习类型
    pub exercise_type: ExerciseType,
    /// 初始代码
    pub initial_code: Option<String>,
    /// 参考答案
    pub solution: String,
    /// 提示
    pub hints: Vec<String>,
    /// 难度（1-5）
    pub difficulty: u8,
}

/// 练习类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExerciseType {
    /// 选择题
    MultipleChoice { options: Vec<String>, correct: usize },
    /// 填空题
    FillInBlank,
    /// 代码补全
    CodeCompletion,
    /// 编程题
    Coding,
    /// 调试题
    Debugging,
}

/// 学习进度
#[derive(Debug, Clone)]
pub struct LearningProgress {
    /// 教程ID
    pub tutorial_id: String,
    /// 完成的章节
    pub completed_chapters: Vec<usize>,
    /// 当前章节
    pub current_chapter: usize,
    /// 完成的练习
    pub completed_exercises: Vec<String>,
    /// 练习得分
    pub exercise_scores: HashMap<String, u8>,
    /// 开始时间
    pub started_at: String,
    /// 最后更新时间
    pub last_updated: String,
    /// 完成状态
    pub is_completed: bool,
}

/// 教程生成器
pub struct TutorialGenerator {
    /// 教程目录
    tutorials_dir: PathBuf,
    /// 输出目录
    output_dir: PathBuf,
    /// 所有教程元数据
    tutorials: HashMap<String, TutorialMetadata>,
    /// 学习进度
    progress: HashMap<String, LearningProgress>,
}

impl TutorialGenerator {
    /// 创建新的教程生成器
    pub fn new(tutorials_dir: PathBuf, output_dir: PathBuf) -> Self {
        Self {
            tutorials_dir,
            output_dir,
            tutorials: HashMap::new(),
            progress: HashMap::new(),
        }
    }

    /// 生成所有教程
    pub fn generate_all(&mut self) -> Result<(), String> {
        println!("📚 开始生成教程系统...");

        // 创建输出目录
        fs::create_dir_all(&self.output_dir)
            .map_err(|e| format!("无法创建输出目录: {}", e))?;

        // 生成快速开始教程
        self.generate_quickstart_tutorials()?;

        // 生成ECS教程
        self.generate_ecs_tutorials()?;

        // 生成渲染教程
        self.generate_rendering_tutorials()?;

        // 生成物理教程
        self.generate_physics_tutorials()?;

        // 生成脚本教程
        self.generate_scripting_tutorials()?;

        // 生成教程索引
        self.generate_tutorial_index()?;

        // 生成学习路径
        self.generate_learning_paths()?;

        println!("✅ 教程系统生成完成！共生成 {} 个教程", self.tutorials.len());

        Ok(())
    }

    /// 生成快速开始教程
    fn generate_quickstart_tutorials(&mut self) -> Result<(), String> {
        println!("📖 生成快速开始教程...");

        // 教程1: Hello World
        let tutorial1 = self.create_hello_world_tutorial()?;

        // 教程2: 第一个游戏
        let tutorial2 = self.create_first_game_tutorial()?;

        // 教程3: 理解ECS
        let tutorial3 = self.create_understanding_ecs_tutorial()?;

        Ok(())
    }

    /// 创建Hello World教程
    fn create_hello_world_tutorial(&mut self) -> Result<(), String> {
        let metadata = TutorialMetadata {
            id: "hello_world".to_string(),
            title: "Hello World".to_string(),
            description: "学习如何创建第一个游戏引擎程序".to_string(),
            level: TutorialLevel::Beginner,
            category: TutorialCategory::QuickStart,
            estimated_time: 15,
            prerequisites: vec![],
            learning_objectives: vec![
                "理解游戏引擎的基本结构".to_string(),
                "创建并运行第一个程序".to_string(),
                "学习基本的窗口设置".to_string(),
            ],
            tags: vec!["快速开始".to_string(), "基础".to_string()],
            author: "Game Engine Team".to_string(),
            created_at: "2026-01-03".to_string(),
            updated_at: "2026-01-03".to_string(),
        };

        let chapters = vec![
            TutorialChapter {
                number: 1,
                title: "什么是游戏引擎".to_string(),
                content: r#"
# 什么是游戏引擎

游戏引擎是一个软件开发框架，旨在为开发电子游戏提供核心功能。

## 核心功能

- **渲染系统**: 处理图形和视觉效果
- **物理系统**: 模拟物理交互
- **音频系统**: 管理声音和音乐
- **脚本系统**: 支持游戏逻辑编程
- **工具集**: 提供编辑器和开发工具

## 为什么选择Rust游戏引擎

- **性能**: Rust提供C++级别的性能
- **安全性**: 编译时内存安全保证
- **现代性**: 现代化的工具链和生态系统
- **跨平台**: 支持多平台编译
                "#.to_string(),
                code_examples: vec![],
                exercises: vec![],
            },
            TutorialChapter {
                number: 2,
                title: "创建你的第一个程序".to_string(),
                content: r#"
# 创建你的第一个程序

让我们创建一个简单的窗口程序。

## 代码示例

```rust
use game_engine::prelude::*;

fn main() {
    // 创建应用
    Application::new()
        .with_window(WindowConfig {
            title: "Hello World".to_string(),
            width: 800,
            height: 600,
            ..Default::default()
        })
        .run();
}
```

## 代码解释

1. **导入预定义模块**: `use game_engine::prelude::*;`
2. **创建应用**: `Application::new()`
3. **配置窗口**: 设置标题和尺寸
4. **运行应用**: `.run()`
                "#.to_string(),
                code_examples: vec![
                    CodeExample {
                        title: "基础窗口程序".to_string(),
                        description: "创建一个简单的游戏窗口".to_string(),
                        code: r#"
use game_engine::prelude::*;

fn main() {
    Application::new()
        .with_window(WindowConfig {
            title: "Hello World".to_string(),
            width: 800,
            height: 600,
            ..Default::default()
        })
        .run();
}
                        "#.to_string(),
                        language: "rust".to_string(),
                        runnable: true,
                        run_command: Some("cargo run --example hello_world".to_string()),
                    }
                ],
                exercises: vec![
                    Exercise {
                        id: "ex1".to_string(),
                        title: "修改窗口标题".to_string(),
                        description: "将窗口标题修改为你的名字".to_string(),
                        exercise_type: ExerciseType::Coding,
                        initial_code: Some(r#"
use game_engine::prelude::*;

fn main() {
    Application::new()
        .with_window(WindowConfig {
            title: // TODO: 修改这里
            width: 800,
            height: 600,
            ..Default::default()
        })
        .run();
}
                        "#.to_string()),
                        solution: r#"
use game_engine::prelude::*;

fn main() {
    Application::new()
        .with_window(WindowConfig {
            title: "你的名字".to_string(),
            width: 800,
            height: 600,
            ..Default::default()
        })
        .run();
}
                        "#.to_string(),
                        hints: vec![
                            "字符串字面量使用双引号".to_string(),
                            "使用.to_string()将&str转换为String".to_string(),
                        ],
                        difficulty: 1,
                    }
                ],
            },
        ];

        self.write_tutorial(&metadata, &chapters)?;
        self.tutorials.insert(metadata.id.clone(), metadata);

        Ok(())
    }

    /// 创建第一个游戏教程
    fn create_first_game_tutorial(&mut self) -> Result<(), String> {
        let metadata = TutorialMetadata {
            id: "first_game".to_string(),
            title: "你的第一个游戏".to_string(),
            description: "创建一个简单的移动方块游戏".to_string(),
            level: TutorialLevel::Beginner,
            category: TutorialCategory::QuickStart,
            estimated_time: 30,
            prerequisites: vec!["hello_world".to_string()],
            learning_objectives: vec![
                "理解ECS架构".to_string(),
                "创建简单的游戏循环".to_string(),
                "处理用户输入".to_string(),
            ],
            tags: vec!["游戏开发".to_string(), "ECS".to_string()],
            author: "Game Engine Team".to_string(),
            created_at: "2026-01-03".to_string(),
            updated_at: "2026-01-03".to_string(),
        };

        let chapters = vec![
            TutorialChapter {
                number: 1,
                title: "理解ECS架构".to_string(),
                content: r#"
# 理解ECS架构

ECS（Entity Component System）是一种游戏开发架构模式。

## 核心概念

- **Entity（实体）**: 游戏对象的唯一标识符
- **Component（组件）**: 纯数据，存储游戏状态
- **System（系统）**: 纯逻辑，处理游戏行为

## 示例

```rust
// 定义组件
struct Position {
    x: f32,
    y: f32,
}

struct Velocity {
    dx: f32,
    dy: f32,
}

// 定义系统
fn movement_system(query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in query.iter_mut() {
        pos.x += vel.dx;
        pos.y += vel.dy;
    }
}
```
                "#.to_string(),
                code_examples: vec![],
                exercises: vec![],
            },
            TutorialChapter {
                number: 2,
                title: "创建玩家实体".to_string(),
                content: r#"
# 创建玩家实体

让我们创建一个可以移动的方块。

## 代码示例

```rust
use game_engine::prelude::*;

fn main() {
    let mut app = Application::new();

    // 启动2D渲染
    app.enable_2d_rendering();

    // 创建玩家
    app.world.spawn((
        Position { x: 0.0, y: 0.0 },
        Velocity { dx: 0.0, dy: 0.0 },
        Sprite::color(Color::BLUE),
        Transform::default(),
    ));

    // 添加输入系统
    app.add_system(input_system);
    app.add_system(movement_system);

    app.run();
}

fn input_system(keys: Res<Input<KeyCode>>, mut query: Query<&mut Velocity>) {
    let mut vel = query.single_mut();

    vel.dx = 0.0;
    vel.dy = 0.0;

    if keys.pressed(KeyCode::Left) {
        vel.dx = -100.0;
    }
    if keys.pressed(KeyCode::Right) {
        vel.dx = 100.0;
    }
    if keys.pressed(KeyCode::Up) {
        vel.dy = 100.0;
    }
    if keys.pressed(KeyCode::Down) {
        vel.dy = -100.0;
    }
}

fn movement_system(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Position)>
) {
    for (vel, mut pos) in query.iter_mut() {
        pos.x += vel.dx * time.delta_seconds();
        pos.y += vel.dy * time.delta_seconds();
    }
}
```
                "#.to_string(),
                code_examples: vec![],
                exercises: vec![],
            },
        ];

        self.write_tutorial(&metadata, &chapters)?;
        self.tutorials.insert(metadata.id.clone(), metadata);

        Ok(())
    }

    /// 创建理解ECS教程
    fn create_understanding_ecs_tutorial(&mut self) -> Result<(), String> {
        let metadata = TutorialMetadata {
            id: "understanding_ecs".to_string(),
            title: "深入理解ECS".to_string(),
            description: "深入学习ECS架构的原理和最佳实践".to_string(),
            level: TutorialLevel::Intermediate,
            category: TutorialCategory::Ecs,
            estimated_time: 45,
            prerequisites: vec!["first_game".to_string()],
            learning_objectives: vec![
                "理解ECS的数据导向设计".to_string(),
                "掌握Query的使用".to_string(),
                "学习System的执行顺序".to_string(),
            ],
            tags: vec!["ECS".to_string(), "架构".to_string(), "性能".to_string()],
            author: "Game Engine Team".to_string(),
            created_at: "2026-01-03".to_string(),
            updated_at: "2026-01-03".to_string(),
        };

        let chapters = vec![
            TutorialChapter {
                number: 1,
                title: "ECS数据导向设计".to_string(),
                content: r#"
# ECS数据导向设计

## 传统面向对象 vs ECS

### 传统OOP方式
```rust
// ❌ 不推荐
struct GameObject {
    position: Position,
    velocity: Velocity,
    sprite: Sprite,
    health: Health,
    // ... 大量字段
}
```

### ECS方式
```rust
// ✅ 推荐
// 每个组件都是独立的数据
struct Position { x: f32, y: f32 }
struct Velocity { dx: f32, dy: f32 }
struct Sprite { /* ... */ }
struct Health { value: i32 }

// 实体只是组件的集合
// 系统按组件类型批量处理数据
```

## ECS的优势

1. **性能**: 数据局部性好，缓存友好
2. **灵活性**: 动态组合组件
3. **可维护性**: 系统职责单一
4. **可扩展性**: 易于添加新功能
                "#.to_string(),
                code_examples: vec![],
                exercises: vec![],
            },
        ];

        self.write_tutorial(&metadata, &chapters)?;
        self.tutorials.insert(metadata.id.clone(), metadata);

        Ok(())
    }

    /// 生成ECS教程
    fn generate_ecs_tutorials(&mut self) -> Result<(), String> {
        println!("📖 生成ECS教程...");

        // TODO: 生成更多ECS相关教程

        Ok(())
    }

    /// 生成渲染教程
    fn generate_rendering_tutorials(&mut self) -> Result<(), String> {
        println!("📖 生成渲染教程...");

        // TODO: 生成渲染相关教程

        Ok(())
    }

    /// 生成物理教程
    fn generate_physics_tutorials(&mut self) -> Result<(), String> {
        println!("📖 生成物理教程...");

        // TODO: 生成物理相关教程

        Ok(())
    }

    /// 生成脚本教程
    fn generate_scripting_tutorials(&mut self) -> Result<(), String> {
        println!("📖 生成脚本教程...");

        // TODO: 生成脚本相关教程

        Ok(())
    }

    /// 写入教程文件
    fn write_tutorial(
        &self,
        metadata: &TutorialMetadata,
        chapters: &[TutorialChapter],
    ) -> Result<(), String> {
        let tutorial_dir = self.output_dir.join(&metadata.id);
        fs::create_dir_all(&tutorial_dir)
            .map_err(|e| format!("无法创建教程目录: {}", e))?;

        // 写入README
        let readme_path = tutorial_dir.join("README.md");
        let readme_content = self.generate_tutorial_readme(metadata, chapters)?;
        fs::write(&readme_path, readme_content)
            .map_err(|e| format!("无法写入README: {}", e))?;

        // 写入代码示例
        for (i, chapter) in chapters.iter().enumerate() {
            for (j, example) in chapter.code_examples.iter().enumerate() {
                if example.runnable {
                    let example_path = tutorial_dir.join(format!(
                        "examples/chapter_{}_example_{}.rs",
                        i + 1,
                        j + 1
                    ));
                    fs::write(&example_path, &example.code)
                        .map_err(|e| format!("无法写入示例: {}", e))?;
                }
            }
        }

        Ok(())
    }

    /// 生成教程README
    fn generate_tutorial_readme(
        &self,
        metadata: &TutorialMetadata,
        chapters: &[TutorialChapter],
    ) -> Result<String, String> {
        let mut content = String::new();

        // 标题
        content.push_str(&format!("# {}\n\n", metadata.title));

        // 元数据
        content.push_str("## 教程信息\n\n");
        content.push_str(&format!("- **级别**: {:?}\n", metadata.level));
        content.push_str(&format!("- **分类**: {:?}\n", metadata.category));
        content.push_str(&format!("- **预计时间**: {} 分钟\n", metadata.estimated_time));
        content.push_str(&format!("- **作者**: {}\n", metadata.author));

        // 标签
        if !metadata.tags.is_empty() {
            content.push_str("\n**标签**: ");
            for tag in &metadata.tags {
                content.push_str(&format!("`{}` ", tag));
            }
            content.push('\n');
        }

        // 描述
        content.push_str(&format!("\n## 描述\n\n{}\n", metadata.description));

        // 学习目标
        if !metadata.learning_objectives.is_empty() {
            content.push_str("\n## 学习目标\n\n");
            for objective in &metadata.learning_objectives {
                content.push_str(&format!("- {}\n", objective));
            }
        }

        // 前置条件
        if !metadata.prerequisites.is_empty() {
            content.push_str("\n## 前置教程\n\n");
            for prereq in &metadata.prerequisites {
                content.push_str(&format!("- [{}](../{}/)\n", prereq, prereq));
            }
        }

        // 章节列表
        content.push_str("\n## 章节目录\n\n");
        for chapter in chapters {
            content.push_str(&format!("{}. [{}](#chapter{})\n",
                chapter.number,
                chapter.title,
                chapter.number
            ));
        }

        // 章节内容
        for chapter in chapters {
            content.push_str(&format!("\n---\n\n## Chapter {}: {}\n\n",
                chapter.number,
                chapter.title
            ));
            content.push_str(&chapter.content);
            content.push('\n');
        }

        // 练习答案
        let has_exercises = chapters.iter()
            .any(|c| !c.exercises.is_empty());

        if has_exercises {
            content.push_str("\n---\n\n## 练习参考答案\n\n");

            for chapter in chapters {
                if !chapter.exercises.is_empty() {
                    content.push_str(&format!("### Chapter {}: {}\n\n",
                        chapter.number,
                        chapter.title
                    ));

                    for exercise in &chapter.exercises {
                        content.push_str(&format!("#### {}\n\n", exercise.title));
                        content.push_str(&format!("```\n{}\n```\n\n", exercise.solution));
                    }
                }
            }
        }

        Ok(content)
    }

    /// 生成教程索引
    fn generate_tutorial_index(&self) -> Result<(), String> {
        println!("📋 生成教程索引...");

        let index_path = self.output_dir.join("INDEX.md");
        let mut content = String::from("# 教程索引\n\n");

        // 按分类组织
        let mut by_category: HashMap<TutorialCategory, Vec<&TutorialMetadata>> = HashMap::new();

        for tutorial in self.tutorials.values() {
            by_category
                .entry(tutorial.category.clone())
                .or_insert_with(Vec::new)
                .push(tutorial);
        }

        // 生成分类索引
        content.push_str("## 按分类浏览\n\n");

        for (category, tutorials) in &by_category {
            content.push_str(&format!("### {:?}\n\n", category));

            // 按级别排序
            let mut sorted = tutorials.to_vec();
            sorted.sort_by_key(|t| t.level);

            for tutorial in sorted {
                let level_emoji = match tutorial.level {
                    TutorialLevel::Beginner => "🟢",
                    TutorialLevel::Intermediate => "🟡",
                    TutorialLevel::Advanced => "🟠",
                    TutorialLevel::Expert => "🔴",
                };

                content.push_str(&format!(
                    "- {} [{}]({}/) - {} 分钟 - {}\n",
                    level_emoji,
                    tutorial.title,
                    tutorial.id,
                    tutorial.estimated_time,
                    tutorial.description
                ));
            }

            content.push('\n');
        }

        fs::write(&index_path, content)
            .map_err(|e| format!("无法写入索引: {}", e))?;

        Ok(())
    }

    /// 生成学习路径
    fn generate_learning_paths(&self) -> Result<(), String> {
        println!("🗺️ 生成学习路径...");

        let paths_path = self.output_dir.join("LEARNING_PATHS.md");
        let content = r#"
# 学习路径

## 初学者路径 (0-2周)

适合完全没有游戏开发经验的初学者。

### 第1周：基础入门

1. [Hello World](hello_world/) - 15分钟
   - 理解游戏引擎基本概念
   - 创建第一个窗口程序

2. [你的第一个游戏](first_game/) - 30分钟
   - 学习ECS基础
   - 创建简单移动游戏

3. [深入理解ECS](understanding_ecs/) - 45分钟
   - ECS架构原理
   - 数据导向设计

### 第2周：核心系统

4. [渲染基础](rendering_basics/) - 60分钟
5. [物理系统入门](physics_basics/) - 60分钟
6. [音频系统](audio_basics/) - 30分钟

## 中级路径 (2-4周)

有一定编程基础的开发者。

### 渲染工程师

- [2D渲染进阶](rendering_2d_advanced/)
- [3D渲染入门](rendering_3d_intro/)
- [着色器编程](shader_programming/)

### 游戏逻辑程序员

- [脚本系统进阶](scripting_advanced/)
- [AI基础](ai_basics/)
- [UI系统](ui_system/)

## 高级路径 (1-3个月)

针对有经验的开发者。

1. 自定义渲染管线
2. 性能优化
3. 网络游戏开发
4. 工具开发
        "#;

        fs::write(&paths_path, content)
            .map_err(|e| format!("无法写入学习路径: {}", e))?;

        Ok(())
    }

    /// 获取教程元数据
    pub fn get_tutorial(&self, id: &str) -> Option<&TutorialMetadata> {
        self.tutorials.get(id)
    }

    /// 获取所有教程
    pub fn get_all_tutorials(&self) -> Vec<&TutorialMetadata> {
        self.tutorials.values().collect()
    }

    /// 获取学习进度
    pub fn get_progress(&self, user_id: &str, tutorial_id: &str) -> Option<&LearningProgress> {
        self.progress.get(&format!("{}_{}", user_id, tutorial_id))
    }

    /// 更新学习进度
    pub fn update_progress(&mut self, user_id: &str, progress: LearningProgress) {
        let key = format!("{}_{}", user_id, progress.tutorial_id);
        self.progress.insert(key, progress);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tutorial_metadata() {
        let metadata = TutorialMetadata {
            id: "test".to_string(),
            title: "Test Tutorial".to_string(),
            description: "A test tutorial".to_string(),
            level: TutorialLevel::Beginner,
            category: TutorialCategory::QuickStart,
            estimated_time: 10,
            prerequisites: vec![],
            learning_objectives: vec!["Test objective".to_string()],
            tags: vec!["test".to_string()],
            author: "Test Author".to_string(),
            created_at: "2026-01-03".to_string(),
            updated_at: "2026-01-03".to_string(),
        };

        assert_eq!(metadata.id, "test");
        assert_eq!(metadata.level, TutorialLevel::Beginner);
    }

    #[test]
    fn test_exercise_type() {
        let exercise = Exercise {
            id: "ex1".to_string(),
            title: "Test Exercise".to_string(),
            description: "Test description".to_string(),
            exercise_type: ExerciseType::Coding,
            initial_code: None,
            solution: "solution".to_string(),
            hints: vec![],
            difficulty: 1,
        };

        assert_eq!(exercise.exercise_type, ExerciseType::Coding);
        assert_eq!(exercise.difficulty, 1);
    }
}
