# 课程元数据示例

**用途**: 为课程管理系统提供标准化的元数据格式示例

---

## 示例1: 入门课程元数据

```json
{
  "id": "B01-introduction",
  "version": "1.0.0",
  "title": {
    "zh": "游戏引擎介绍",
    "en": "Game Engine Introduction"
  },
  "description": {
    "zh": "了解游戏引擎的核心功能、应用场景和学习路线",
    "en": "Understand the core features, use cases, and learning path of the game engine"
  },
  "level": "beginner",
  "category": "快速开始",
  "duration": 30,
  "language": ["zh", "en"],
  "tags": ["入门", "引擎概览", "介绍", "Introduction"],
  "prerequisites": [],
  "learningObjectives": [
    {
      "zh": "理解游戏引擎的价值和作用",
      "en": "Understand the value and role of game engines"
    },
    {
      "zh": "了解引擎核心功能和特性",
      "en": "Learn about core engine features and capabilities"
    },
    {
      "zh": "制定个人学习计划",
      "en": "Create a personal learning plan"
    }
  ],
  "instructor": {
    "id": "INS001",
    "name": "张三",
    "title": "高级游戏开发工程师",
    "bio": "10年游戏开发经验,参与多款AAA游戏项目开发",
    "avatar": "https://cdn.gameengine.dev/instructors/ins001.jpg"
  },
  "thumbnail": {
    "zh": "https://cdn.gameengine.dev/courses/B01/thumbnail-zh.jpg",
    "en": "https://cdn.gameengine.dev/courses/B01/thumbnail-en.jpg"
  },
  "previewVideo": {
    "zh": "https://cdn.gameengine.dev/courses/B01/preview-zh.mp4",
    "en": "https://cdn.gameengine.dev/courses/B01/preview-en.mp4"
  },
  "lessons": [
    {
      "id": "B01-01",
      "order": 1,
      "title": {
        "zh": "什么是游戏引擎",
        "en": "What is a Game Engine"
      },
      "duration": 300,
      "videoUrl": "https://cdn.gameengine.dev/courses/B01/videos/B01-01.mp4",
      "freePreview": true
    },
    {
      "id": "B01-02",
      "order": 2,
      "title": {
        "zh": "引擎核心功能",
        "en": "Core Engine Features"
      },
      "duration": 600,
      "videoUrl": "https://cdn.gameengine.dev/courses/B01/videos/B01-02.mp4",
      "freePreview": true
    },
    {
      "id": "B01-03",
      "order": 3,
      "title": {
        "zh": "应用场景展示",
        "en": "Use Cases Showcase"
      },
      "duration": 600,
      "videoUrl": "https://cdn.gameengine.dev/courses/B01/videos/B01-03.mp4",
      "freePreview": false
    },
    {
      "id": "B01-04",
      "order": 4,
      "title": {
        "zh": "学习路线规划",
        "en": "Learning Path Planning"
      },
      "duration": 300,
      "videoUrl": "https://cdn.gameengine.dev/courses/B01/videos/B01-04.mp4",
      "freePreview": false
    }
  ],
  "resources": [
    {
      "id": "R01-01",
      "type": "slide",
      "title": {
        "zh": "课程讲义",
        "en": "Course Slides"
      },
      "url": "https://cdn.gameengine.dev/courses/B01/resources/slides.pdf",
      "size": 5242880,
      "format": "pdf"
    },
    {
      "id": "R01-02",
      "type": "document",
      "title": {
        "zh": "学习路线图",
        "en": "Learning Roadmap"
      },
      "url": "https://cdn.gameengine.dev/courses/B01/resources/roadmap.pdf",
      "size": 1048576,
      "format": "pdf"
    }
  ],
  "quizzes": [
    {
      "id": "Q01-01",
      "lessonId": "B01-01",
      "title": {
        "zh": "游戏引擎基础测试",
        "en": "Game Engine Basics Quiz"
      },
      "questions": 5,
      "passingScore": 80
    }
  ],
  "pricing": {
    "free": true,
    "regularPrice": 0,
    "discountedPrice": 0,
    "currency": "CNY"
  },
  "stats": {
    "enrolledCount": 1523,
    "completedCount": 1245,
    "averageRating": 4.8,
    "ratingCount": 892
  },
  "publishedAt": "2026-01-15T10:00:00Z",
  "updatedAt": "2026-01-15T10:00:00Z",
  "status": "published"
}
```

---

## 示例2: 进阶课程元数据

```json
{
  "id": "I01-scripting",
  "version": "1.0.0",
  "title": {
    "zh": "脚本系统",
    "en": "Scripting System"
  },
  "description": {
    "zh": "掌握C#脚本编程,实现复杂的游戏逻辑和交互",
    "en": "Master C# scripting to implement complex game logic and interactions"
  },
  "level": "intermediate",
  "category": "编程开发",
  "duration": 60,
  "language": ["zh", "en"],
  "tags": ["脚本", "C#", "编程", "Scripting", "CSharp"],
  "prerequisites": ["B01-introduction", "B03-first-scene"],
  "learningObjectives": [
    {
      "zh": "编写和运行C#脚本",
      "en": "Write and execute C# scripts"
    },
    {
      "zh": "理解脚本生命周期",
      "en": "Understand script lifecycle"
    },
    {
      "zh": "使用引擎API实现游戏逻辑",
      "en": "Use engine API to implement game logic"
    },
    {
      "zh": "掌握事件系统和消息处理",
      "en": "Master event system and message handling"
    }
  ],
  "instructor": {
    "id": "INS002",
    "name": "李四",
    "title": "资深技术专家",
    "bio": "8年游戏引擎开发经验,精通脚本系统和架构设计",
    "avatar": "https://cdn.gameengine.dev/instructors/ins002.jpg"
  },
  "thumbnail": {
    "zh": "https://cdn.gameengine.dev/courses/I01/thumbnail-zh.jpg",
    "en": "https://cdn.gameengine.dev/courses/I01/thumbnail-en.jpg"
  },
  "previewVideo": {
    "zh": "https://cdn.gameengine.dev/courses/I01/preview-zh.mp4",
    "en": "https://cdn.gameengine.dev/courses/I01/preview-en.mp4"
  },
  "lessons": [
    {
      "id": "I01-01",
      "order": 1,
      "title": {
        "zh": "脚本系统架构",
        "en": "Scripting System Architecture"
      },
      "duration": 600,
      "videoUrl": "https://cdn.gameengine.dev/courses/I01/videos/I01-01.mp4",
      "freePreview": true
    },
    {
      "id": "I01-02",
      "order": 2,
      "title": {
        "zh": "C#脚本基础",
        "en": "C# Scripting Basics"
      },
      "duration": 900,
      "videoUrl": "https://cdn.gameengine.dev/courses/I01/videos/I01-02.mp4",
      "freePreview": false
    },
    {
      "id": "I01-03",
      "order": 3,
      "title": {
        "zh": "生命周期函数",
        "en": "Lifecycle Functions"
      },
      "duration": 600,
      "videoUrl": "https://cdn.gameengine.dev/courses/I01/videos/I01-03.mp4",
      "freePreview": false
    },
    {
      "id": "I01-04",
      "order": 4,
      "title": {
        "zh": "常用API介绍",
        "en": "Common APIs Overview"
      },
      "duration": 900,
      "videoUrl": "https://cdn.gameengine.dev/courses/I01/videos/I01-04.mp4",
      "freePreview": false
    },
    {
      "id": "I01-05",
      "order": 5,
      "title": {
        "zh": "实战示例",
        "en": "Practical Examples"
      },
      "duration": 600,
      "videoUrl": "https://cdn.gameengine.dev/courses/I01/videos/I01-05.mp4",
      "freePreview": false
    }
  ],
  "resources": [
    {
      "id": "R01-03",
      "type": "slide",
      "title": {
        "zh": "课程讲义",
        "en": "Course Slides"
      },
      "url": "https://cdn.gameengine.dev/courses/I01/resources/slides.pdf",
      "size": 8388608,
      "format": "pdf"
    },
    {
      "id": "R01-04",
      "type": "code",
      "title": {
        "zh": "代码示例库",
        "en": "Code Examples"
      },
      "url": "https://cdn.gameengine.dev/courses/I01/resources/code-examples.zip",
      "size": 2097152,
      "format": "zip"
    },
    {
      "id": "R01-05",
      "type": "document",
      "title": {
        "zh": "API参考手册",
        "en": "API Reference"
      },
      "url": "https://cdn.gameengine.dev/courses/I01/resources/api-reference.pdf",
      "size": 3145728,
      "format": "pdf"
    }
  ],
  "quizzes": [
    {
      "id": "Q01-02",
      "lessonId": "I01-02",
      "title": {
        "zh": "C#基础测试",
        "en": "C# Basics Quiz"
      },
      "questions": 10,
      "passingScore": 80
    },
    {
      "id": "Q01-03",
      "lessonId": "I01-05",
      "title": {
        "zh": "综合测试",
        "en": "Comprehensive Quiz"
      },
      "questions": 15,
      "passingScore": 75
    }
  ],
  "pricing": {
    "free": false,
    "regularPrice": 19900,
    "discountedPrice": 14900,
    "currency": "CNY",
    "discount": {
      "type": "percentage",
      "value": 25,
      "validUntil": "2026-02-28T23:59:59Z"
    }
  },
  "stats": {
    "enrolledCount": 856,
    "completedCount": 623,
    "averageRating": 4.7,
    "ratingCount": 512
  },
  "publishedAt": "2026-02-01T10:00:00Z",
  "updatedAt": "2026-02-01T10:00:00Z",
  "status": "published"
}
```

---

## 示例3: 实战项目元数据

```json
{
  "id": "P01-2d-platformer",
  "version": "1.0.0",
  "title": {
    "zh": "2D平台游戏",
    "en": "2D Platformer Game"
  },
  "description": {
    "zh": "从零开始开发一个完整的2D横版平台跳跃游戏,包含角色控制、关卡设计、敌人AI等核心功能",
    "en": "Build a complete 2D side-scrolling platformer game from scratch, including character control, level design, enemy AI, and more"
  },
  "level": "intermediate",
  "category": "实战项目",
  "projectType": "完整游戏",
  "duration": 240,
  "language": ["zh", "en"],
  "tags": ["2D", "平台游戏", "实战", "项目", "Platformer", "Project"],
  "prerequisites": [
    "B01-introduction",
    "B04-object-manipulation",
    "I01-scripting",
    "I03-physics"
  ],
  "learningObjectives": [
    {
      "zh": "掌握2D游戏开发完整流程",
      "en": "Master the complete 2D game development workflow"
    },
    {
      "zh": "实现角色控制器和物理交互",
      "en": "Implement character controller and physics interactions"
    },
    {
      "zh": "设计并实现游戏关卡",
      "en": "Design and implement game levels"
    },
    {
      "zh": "创建敌人AI和战斗系统",
      "en": "Create enemy AI and combat system"
    },
    {
      "zh": "添加音效、UI和游戏打磨",
      "en": "Add audio, UI, and game polish"
    }
  ],
  "instructor": {
    "id": "INS003",
    "name": "王五",
    "title": "游戏开发专家",
    "bio": "独立游戏开发者,发布过多款热门游戏,擅长游戏设计和开发",
    "avatar": "https://cdn.gameengine.dev/instructors/ins003.jpg"
  },
  "thumbnail": {
    "zh": "https://cdn.gameengine.dev/courses/P01/thumbnail-zh.jpg",
    "en": "https://cdn.gameengine.dev/courses/P01/thumbnail-en.jpg"
  },
  "previewVideo": {
    "zh": "https://cdn.gameengine.dev/courses/P01/gameplay-zh.mp4",
    "en": "https://cdn.gameengine.dev/courses/P01/gameplay-en.mp4"
  },
  "lessons": [
    {
      "id": "P01-01",
      "order": 1,
      "title": {
        "zh": "项目概述和设置",
        "en": "Project Overview and Setup"
      },
      "duration": 1200,
      "videoUrl": "https://cdn.gameengine.dev/courses/P01/videos/P01-01.mp4",
      "freePreview": true
    },
    {
      "id": "P01-02",
      "order": 2,
      "title": {
        "zh": "角色控制器",
        "en": "Character Controller"
      },
      "duration": 1800,
      "videoUrl": "https://cdn.gameengine.dev/courses/P01/videos/P01-02.mp4",
      "freePreview": false
    },
    {
      "id": "P01-03",
      "order": 3,
      "title": {
        "zh": "物理和碰撞",
        "en": "Physics and Collisions"
      },
      "duration": 1800,
      "videoUrl": "https://cdn.gameengine.dev/courses/P01/videos/P01-03.mp4",
      "freePreview": false
    },
    {
      "id": "P01-04",
      "order": 4,
      "title": {
        "zh": "关卡设计",
        "en": "Level Design"
      },
      "duration": 2400,
      "videoUrl": "https://cdn.gameengine.dev/courses/P01/videos/P01-04.mp4",
      "freePreview": false
    },
    {
      "id": "P01-05",
      "order": 5,
      "title": {
        "zh": "敌人AI",
        "en": "Enemy AI"
      },
      "duration": 1800,
      "videoUrl": "https://cdn.gameengine.dev/courses/P01/videos/P01-05.mp4",
      "freePreview": false
    },
    {
      "id": "P01-06",
      "order": 6,
      "title": {
        "zh": "UI和音效",
        "en": "UI and Audio"
      },
      "duration": 1800,
      "videoUrl": "https://cdn.gameengine.dev/courses/P01/videos/P01-06.mp4",
      "freePreview": false
    },
    {
      "id": "P01-07",
      "order": 7,
      "title": {
        "zh": "游戏打磨",
        "en": "Game Polish"
      },
      "duration": 1800,
      "videoUrl": "https://cdn.gameengine.dev/courses/P01/videos/P01-07.mp4",
      "freePreview": false
    },
    {
      "id": "P01-08",
      "order": 8,
      "title": {
        "zh": "构建和发布",
        "en": "Build and Release"
      },
      "duration": 1200,
      "videoUrl": "https://cdn.gameengine.dev/courses/P01/videos/P01-08.mp4",
      "freePreview": false
    }
  ],
  "resources": [
    {
      "id": "R01-06",
      "type": "project",
      "title": {
        "zh": "完整项目源码",
        "en": "Complete Project Source"
      },
      "url": "https://cdn.gameengine.dev/courses/P01/resources/project-source.zip",
      "size": 52428800,
      "format": "zip"
    },
    {
      "id": "R01-07",
      "type": "assets",
      "title": {
        "zh": "美术资源包",
        "en": "Art Assets Pack"
      },
      "url": "https://cdn.gameengine.dev/courses/P01/resources/art-assets.zip",
      "size": 104857600,
      "format": "zip"
    },
    {
      "id": "R01-08",
      "type": "audio",
      "title": {
        "zh": "音效素材库",
        "en": "Audio Effects Library"
      },
      "url": "https://cdn.gameengine.dev/courses/P01/resources/audio.zip",
      "size": 20971520,
      "format": "zip"
    },
    {
      "id": "R01-09",
      "type": "document",
      "title": {
        "zh": "设计文档",
        "en": "Design Document"
      },
      "url": "https://cdn.gameengine.dev/courses/P01/resources/design-doc.pdf",
      "size": 5242880,
      "format": "pdf"
    }
  ],
  "quizzes": [
    {
      "id": "Q01-04",
      "lessonId": "P01-02",
      "title": {
        "zh": "角色控制测试",
        "en": "Character Control Quiz"
      },
      "questions": 8,
      "passingScore": 75
    },
    {
      "id": "Q01-05",
      "lessonId": "P01-05",
      "title": {
        "zh": "AI系统测试",
        "en": "AI System Quiz"
      },
      "questions": 10,
      "passingScore": 70
    },
    {
      "id": "Q01-06",
      "lessonId": null,
      "title": {
        "zh": "项目综合测试",
        "en": "Project Comprehensive Quiz"
      },
      "questions": 20,
      "passingScore": 70
    }
  ],
  "pricing": {
    "free": false,
    "regularPrice": 49900,
    "discountedPrice": 39900,
    "currency": "CNY",
    "discount": {
      "type": "fixed",
      "value": 10000,
      "validUntil": "2026-03-31T23:59:59Z"
    }
  },
  "certificate": {
    "enabled": true,
    "template": "project-completion",
    "requirements": {
      "completionRate": 100,
      "quizPassing": true
    }
  },
  "stats": {
    "enrolledCount": 423,
    "completedCount": 287,
    "averageRating": 4.9,
    "ratingCount": 356
  },
  "publishedAt": "2026-03-01T10:00:00Z",
  "updatedAt": "2026-03-01T10:00:00Z",
  "status": "published"
}
```

---

## 课时元数据示例

```json
{
  "id": "B01-01",
  "version": "1.0.0",
  "title": {
    "zh": "什么是游戏引擎",
    "en": "What is a Game Engine"
  },
  "courseId": "B01-introduction",
  "order": 1,
  "duration": 300,
  "video": {
    "url": "https://cdn.gameengine.dev/courses/B01/videos/B01-01.mp4",
    "qualities": [
      {
        "label": "1080p",
        "width": 1920,
        "height": 1080,
        "bitrate": 8000000,
        "url": "https://cdn.gameengine.dev/courses/B01/videos/B01-01-1080p.mp4"
      },
      {
        "label": "720p",
        "width": 1280,
        "height": 720,
        "bitrate": 5000000,
        "url": "https://cdn.gameengine.dev/courses/B01/videos/B01-01-720p.mp4"
      },
      {
        "label": "480p",
        "width": 854,
        "height": 480,
        "bitrate": 2500000,
        "url": "https://cdn.gameengine.dev/courses/B01/videos/B01-01-480p.mp4"
      }
    ],
    "thumbnails": [
      {
        "time": 0,
        "url": "https://cdn.gameengine.dev/courses/B01/thumbnails/B01-01-0.jpg"
      },
      {
        "time": 150,
        "url": "https://cdn.gameengine.dev/courses/B01/thumbnails/B01-01-150.jpg"
      },
      {
        "time": 300,
        "url": "https://cdn.gameengine.dev/courses/B01/thumbnails/B01-01-300.jpg"
      }
    ]
  },
  "subtitles": [
    {
      "language": "zh",
      "label": "中文",
      "url": "https://cdn.gameengine.dev/courses/B01/subtitles/B01-01-zh.vtt",
      "format": "vtt",
      "default": true
    },
    {
      "language": "en",
      "label": "English",
      "url": "https://cdn.gameengine.dev/courses/B01/subtitles/B01-01-en.vtt",
      "format": "vtt",
      "default": false
    }
  ],
  "description": {
    "zh": "介绍游戏引擎的基本概念、发展历史和核心价值",
    "en": "Introduce the basic concepts, history, and core value of game engines"
  },
  "keyPoints": [
    {
      "time": 30,
      "title": {
        "zh": "游戏引擎定义",
        "en": "Game Engine Definition"
      }
    },
    {
      "time": 120,
      "title": {
        "zh": "引擎发展历史",
        "en": "Engine History"
      }
    },
    {
      "time": 200,
      "title": {
        "zh": "现代引擎特性",
        "en": "Modern Engine Features"
      }
    }
  ],
  "resources": [
    {
      "type": "slide",
      "time": 0,
      "title": {
        "zh": "本节课件",
        "en": "Lesson Slides"
      },
      "url": "https://cdn.gameengine.dev/courses/B01/resources/B01-01-slides.pdf"
    }
  ],
  "quiz": {
    "id": "Q01-01",
    "time": 280,
    "duration": 20,
    "questions": 5,
    "passingScore": 80
  },
  "freePreview": true,
  "stats": {
    "views": 5423,
    "completions": 4892,
    "averageWatchTime": 285
  },
  "publishedAt": "2026-01-15T10:00:00Z",
  "updatedAt": "2026-01-15T10:00:00Z"
}
```

---

## 测验元数据示例

```json
{
  "id": "Q01-01",
  "version": "1.0.0",
  "title": {
    "zh": "游戏引擎基础测试",
    "en": "Game Engine Basics Quiz"
  },
  "lessonId": "B01-01",
  "courseId": "B01-introduction",
  "type": "lesson",
  "duration": 300,
  "questions": [
    {
      "id": "Q1",
      "type": "single-choice",
      "question": {
        "zh": "游戏引擎的主要作用是什么?",
        "en": "What is the main purpose of a game engine?"
      },
      "options": [
        {
          "id": "A",
          "text": {
            "zh": "提供游戏开发的基础框架和工具",
            "en": "Provide basic framework and tools for game development"
          },
          "correct": true
        },
        {
          "id": "B",
          "text": {
            "zh": "仅用于3D建模",
            "en": "Only for 3D modeling"
          },
          "correct": false
        },
        {
          "id": "C",
          "text": {
            "zh": "替代编程语言",
            "en": "Replace programming languages"
          },
          "correct": false
        },
        {
          "id": "D",
          "text": {
            "zh": "只用于音频处理",
            "en": "Only for audio processing"
          },
          "correct": false
        }
      ],
      "explanation": {
        "zh": "游戏引擎为游戏开发提供完整的开发框架,包括渲染、物理、音频等核心系统",
        "en": "Game engines provide a complete development framework for game development, including rendering, physics, audio and other core systems"
      },
      "points": 10
    },
    {
      "id": "Q2",
      "type": "multiple-choice",
      "question": {
        "zh": "以下哪些是现代游戏引擎的核心功能?(多选)",
        "en": "Which of the following are core features of modern game engines? (Multiple)"
      },
      "options": [
        {
          "id": "A",
          "text": {
            "zh": "渲染引擎",
            "en": "Rendering Engine"
          },
          "correct": true
        },
        {
          "id": "B",
          "text": {
            "zh": "物理引擎",
            "en": "Physics Engine"
          },
          "correct": true
        },
        {
          "id": "C",
          "text": {
            "zh": "音频系统",
            "en": "Audio System"
          },
          "correct": true
        },
        {
          "id": "D",
          "text": {
            "zh": "脚本系统",
            "en": "Scripting System"
          },
          "correct": true
        }
      ],
      "explanation": {
        "zh": "现代游戏引擎包含渲染、物理、音频、脚本等多个核心系统",
        "en": "Modern game engines include multiple core systems such as rendering, physics, audio, and scripting"
      },
      "points": 15
    },
    {
      "id": "Q3",
      "type": "true-false",
      "question": {
        "zh": "游戏引擎可以用于开发2D和3D游戏",
        "en": "Game engines can be used to develop both 2D and 3D games"
      },
      "options": [
        {
          "id": "A",
          "text": {
            "zh": "正确",
            "en": "True"
          },
          "correct": true
        },
        {
          "id": "B",
          "text": {
            "zh": "错误",
            "en": "False"
          },
          "correct": false
        }
      ],
      "explanation": {
        "zh": "现代游戏引擎同时支持2D和3D游戏开发",
        "en": "Modern game engines support both 2D and 3D game development"
      },
      "points": 10
    }
  ],
  "passingScore": 80,
  "totalPoints": 100,
  "timeLimit": 600,
  "shuffleQuestions": true,
  "shuffleOptions": true,
  "showAnswers": true,
  "allowRetake": true,
  "maxRetakes": 3,
  "stats": {
    "attempts": 1234,
    "passes": 1156,
    "averageScore": 87.5,
    "averageTime": 420
  }
}
```

---

## 用户进度元数据示例

```json
{
  "id": "PROG001",
  "userId": "U123456",
  "courseId": "B01-introduction",
  "enrolledAt": "2026-01-16T10:30:00Z",
  "startedAt": "2026-01-16T10:35:00Z",
  "completedAt": null,
  "lastAccessedAt": "2026-01-18T15:20:00Z",
  "progress": {
    "totalLessons": 7,
    "completedLessons": 3,
    "currentLesson": "B01-04",
    "percentage": 42.86,
    "timeSpent": 2850
  },
  "lessons": [
    {
      "lessonId": "B01-01",
      "status": "completed",
      "startedAt": "2026-01-16T10:35:00Z",
      "completedAt": "2026-01-16T11:05:00Z",
      "timeSpent": 1800,
      "watchTime": 1750,
      "quiz": {
        "attempted": true,
        "passed": true,
        "score": 90,
        "attempts": 1
      }
    },
    {
      "lessonId": "B01-02",
      "status": "completed",
      "startedAt": "2026-01-17T14:00:00Z",
      "completedAt": "2026-01-17T14:52:00Z",
      "timeSpent": 3120,
      "watchTime": 3000,
      "quiz": {
        "attempted": true,
        "passed": true,
        "score": 85,
        "attempts": 1
      }
    },
    {
      "lessonId": "B01-03",
      "status": "completed",
      "startedAt": "2026-01-18T10:00:00Z",
      "completedAt": "2026-01-18T10:45:00Z",
      "timeSpent": 2700,
      "watchTime": 2650,
      "quiz": {
        "attempted": true,
        "passed": true,
        "score": 95,
        "attempts": 1
      }
    },
    {
      "lessonId": "B01-04",
      "status": "in_progress",
      "startedAt": "2026-01-18T15:20:00Z",
      "timeSpent": 450,
      "watchTime": 420,
      "watchPosition": 150
    }
  ],
  "bookmarks": [
    {
      "lessonId": "B01-02",
      "time": 240,
      "note": "重要的概念讲解",
      "createdAt": "2026-01-17T14:25:00Z"
    }
  ],
  "notes": [
    {
      "lessonId": "B01-01",
      "time": 180,
      "content": "游戏引擎定义的核心要点",
      "createdAt": "2026-01-16T11:00:00Z"
    }
  ],
  "certificate": null,
  "achievements": [
    {
      "id": "ACH001",
      "type": "first-lesson",
      "earnedAt": "2026-01-16T11:05:00Z"
    },
    {
      "id": "ACH002",
      "type": "perfect-score",
      "earnedAt": "2026-01-18T10:45:00Z"
    }
  ]
}
```

---

## 证书元数据示例

```json
{
  "id": "CERT001",
  "userId": "U123456",
  "courseId": "B01-introduction",
  "type": "completion",
  "issuedAt": "2026-01-20T10:00:00Z",
  "completedAt": "2026-01-20T09:30:00Z",
  "student": {
    "name": "张三",
    "email": "zhangsan@example.com"
  },
  "course": {
    "title": "游戏引擎介绍",
    "level": "beginner",
    "duration": 30
  },
  "instructor": {
    "name": "李四",
    "title": "高级游戏开发工程师"
  },
  "score": 92,
  "verificationUrl": "https://gameengine.dev/certificates/verify/CERT001",
  "certificateUrl": "https://cdn.gameengine.dev/certificates/CERT001.pdf",
  "shareUrl": "https://gameengine.dev/certificates/CERT001",
  "template": {
    "id": "TPL001",
    "name": "standard-completion",
    "design": {
      "background": "#ffffff",
      "primaryColor": "#2563eb",
      "secondaryColor": "#1e40af",
      "font": "Noto Sans SC"
    }
  },
  "metadata": {
    "issuer": "Game Engine Dev",
    "issuerUrl": "https://gameengine.dev",
    "issuerLogo": "https://cdn.gameengine.dev/logo.png",
    "signature": "digital-signature-hash",
    "blockchainTxId": null
  }
}
```

---

**文档版本**: v1.0
**最后更新**: 2026-01-02
**维护者**: 技术团队

这些示例提供了课程管理系统中各种实体的标准化元数据格式,可以作为数据库设计和API开发的参考。
