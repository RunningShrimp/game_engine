# Discord 服务器设置指南

本指南帮助您为游戏引擎项目设置和管理Discord社区服务器。

## 目录

- [服务器创建](#服务器创建)
- [频道结构](#频道结构)
- [角色和权限](#角色和权限)
- [机器人集成](#机器人集成)
- [管理工具](#管理工具)
- [社区增长](#社区增长)

---

## 服务器创建

### 1. 初始设置

1. **创建服务器**
   - 访问 https://discord.com/
   - 点击"+" → "创建服务器"
   - 选择服务器名称: `Rust Game Engine` 或 `游戏引擎社区`
   - 选择服务器区域: 选择离主要用户最近的区域

2. **基础配置**
   - 上传服务器图标 (512x512px PNG)
   - 设置服务器描述
   - 选择合适的邀请等级

### 2. 验证级别

建议设置验证级别以防止垃圾用户：
- **设置** → **审核** → **验证级别**
- 推荐设置: **中等** (需要在服务器上注册10分钟)

---

## 频道结构

### 信息类别 (`ℹ️ Information`)

```
#rules          - 社区规则和行为准则
#announcements  - 重要公告和更新通知
#introduction   - 新成员自我介绍
#resources      - 重要资源和链接
```

**权限设置:**
- `@everyone`: 只读
- 管理团队: 发送消息/管理

### 一般讨论 (`💬 General`)

```
#general        - 一般讨论和聊天
#help           - 技术支持和问题求助
#showcase       - 作品展示和项目分享
#off-topic      - 休闲话题和非技术讨论
```

**权限设置:**
- `@everyone`: 发送消息/添加反应
- `@verified`: 附加权限(上传文件、添加链接)

### 技术讨论 (`⚙️ Development`)

```
#development    - 引擎开发讨论
#ecs            - ECS架构相关讨论
#rendering      - 渲染系统讨论
#physics        - 物理引擎讨论
#ai             - AI系统讨论
#performance    - 性能优化讨论
```

**权限设置:**
- `@everyone`: 只读
- `@developer`: 发送消息
- `@contributor`: 附加权限

### 文档 (`📚 Documentation`)

```
#docs           - 文档讨论和建议
#tutorials      - 教程和指南讨论
#examples       - 示例代码讨论
#api-reference  - API参考讨论
```

**权限设置:**
- `@everyone`: 发送消息
- `@verified`: 附加权限

### 语言频道 (`🌍 Languages`)

```
#chinese        - 中文讨论
#japanese       - 日文讨论
#korean         - 韩文讨论
#spanish        - 西班牙语讨论
#other-languages - 其他语言
```

**权限设置:**
- `@everyone`: 发送消息(对应语言)

### 语音频道 (`🎙️ Voice`)

```
📢 General Voice    - 一般语音聊天
🎮 Gaming           - 游戏语音
💻 Dev Corner       - 开发协作
🎵 Music            - 音乐分享
```

---

## 角色和权限

### 角色层级

#### 1. **@everyone** (默认角色)
- **权限:**
  - 读取指定频道
  - 发送消息到一般频道
  - 添加反应
  - 连接语音频道

#### 2. **@verified** (已验证成员)
- **获取方式:** 在 `#introduction` 发布自我介绍
- **额外权限:**
  - 上传文件
  - 添加外部链接
  - 使用表情
  - 优先发言

#### 3. **@contributor** (贡献者)
- **获取方式:** 提交有效的PR或Issue
- **额外权限:**
  - 发送到技术频道
  - 创建邀请
  - 优先发言

#### 4. **@developer** (开发团队成员)
- **获取方式:** 成为项目维护者
- **额外权限:**
  - 管理消息
  - 审查成员
  - 创建活动

#### 5. **@moderator** (版主)
- **获取方式:** 由管理员任命
- **权限:**
  - 踢出成员
  - 封禁成员
  - 管理频道
  - 审查消息

#### 6. **@admin** (管理员)
- **权限:** 所有权限

### 角色颜色

```yaml
@everyone:       #99AAB5 (灰色)
@verified:       #95C5DE (浅蓝)
@contributor:    #3498DB (蓝色)
@developer:      #9B59B6 (紫色)
@moderator:      #E67E22 (橙色)
@admin:          #E74C3C (红色)
```

---

## 机器人集成

### 推荐机器人

#### 1. **Carl-bot** (管理)

**功能:**
- 自动角色分配
- 反应角色
- 日志记录
- 自动审核

**设置:**
```
?setup
```

**配置示例:**
```yaml
# 在 #introduction 的自动角色
欢迎消息:
  "欢迎 {user}! 请阅读 #rules，然后在 #introduction 介绍你自己。"

反应角色:
  "📚 文档": @documenter
  "💻 开发": @developer
  "🎨 设计": @designer
```

#### 2. **MEE6** (等级和积分)

**功能:**
- 用户等级系统
- 积分奖励
- 自定义命令
- 音乐播放

**设置:**
```
!setup
```

#### 3. **Dyno** (管理)

**功能:**
- 自动管理
- 自定义命令
- 消息清理
- 提醒功能

#### 4. **GitHub Bot** (开发集成)

**功能:**
- Issue通知
- PR通知
- Release通知
- 仓库更新

**Webhook配置:**
```json
{
  "github": {
    "webhook_url": "your-webhook-url",
    "events": [
      "issues",
      "pull_request",
      "push",
      "release"
    ]
  }
}
```

### 自定义机器人

创建自己的Discord机器人用于特定功能:

**技术栈:**
- Rust: `serenity` 或 `twilight`
- Python: `discord.py` 或 `nextcord`

**示例功能:**
```rust
// 自动回复文档链接
async fn docs_command(ctx: &Context, msg: &Message) {
    let query = extract_query(msg);
    let docs_url = format!("https://docs.gameengine.dev/?q={}", query);

    msg.channel_id.send_message(&ctx.http, |m| {
        m.content(format!("文档链接: {}", docs_url))
    }).await?;
}
```

---

## 管理工具

### 1. **审核日志**

**关键事件:**
- 成员加入/离开
- 消息删除
- 角色变更
- 频道创建/删除

**设置:**
```
设置 → 审核日志 → 启用审核日志
```

### 2. **自动审核**

**规则示例:**
```yaml
垃圾信息:
  - 链接发送: >5/分钟
  - 相同消息: >3/分钟
  - 提及所有人: 0

关键词过滤:
  - 垃圾词汇: 自动删除
  - 冒犯性内容: 警告 + 删除
  - 链接短网址: 要求验证
```

### 3. **欢迎系统**

**配置:**
```yaml
欢迎频道: #welcome
欢迎消息: |
  欢迎 {user_mention} 来到游戏引擎社区!

  请务必:
  📖 阅读 #rules
  🎯 查看 #announcements
  👋 在 #introduction 介绍你自己
  💬 加入 #general 开始讨论

  获取验证角色后即可访问所有频道!
```

### 4. **离开消息**

```yaml
离开频道: #general
离开消息: "{user} 离开了服务器。再见!"
```

---

## 社区增长

### 1. **推广策略**

- **GitHub README** 添加Discord邀请链接
- **文档网站** 添加Discord组件
- **Reddit** 在 r/rust 和 r/gamedev 分享
- **Twitter** 定期分享社区亮点
- **YouTube** 教程视频提及社区

### 2. **内容策略**

**每周活动:**
```
周一:   #monday-motivation - 分享本周目标
周三:   #work-wednesday - 分享工作进展
周五:   #showcase-friday - 展示本周作品
```

**每月活动:**
```
第一个周二:  社区会议
第三个周五:  代码审查会
最后一个周末: 游戏开发挑战
```

### 3. **激励计划**

- **月度贡献者:** 在公告中表彰
- **优秀项目:** 在 #showcase 置顶
- **教程作者:** 颁发 @contributor 角色
- **Bug猎手:** 特殊徽章和积分

### 4. **参与指标**

**追踪指标:**
- 日活跃用户 (DAU)
- 周活跃用户 (WAU)
- 月活跃用户 (MAU)
- 消息量
- 新成员增长率
- 留存率

**工具:**
- Discord Server Insights (内置)
- Statbot / Server Insights (第三方)

---

## 最佳实践

### 1. **响应时间**

- **紧急问题:** 24小时内
- **一般咨询:** 48小时内
- **功能请求:** 1周内回复

### 2. **冲突解决**

1. 私下联系相关方
2. 倾听双方意见
3. 引用社区规则
4. 寻求共识
5. 必要时升级处理

### 3. **文档优先**

鼓励用户:
1. 先搜索现有文档
2. 查看 FAQ
3. 搜索历史消息
4. 再提问

### 4. **积极文化**

- 欢迎新成员
- 鼓励提问
- 认可贡献
- 建设性反馈
- 分享知识

---

## 资源链接

- **Discord文档:** https://discord.com/developers/docs
- **Discordify:** https://discord.gg/discord-developers
- **Awesome Discord:** https://github.com/meew0/discord-bot-awesome

---

**最后更新:** 2024-12-31
**维护者:** 游戏引擎团队
