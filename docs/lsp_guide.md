# LSP 语言服务器使用指南

**版本**: v0.2.0
**更新日期**: 2026-01-03
**状态**: ✅ 基础功能完成，85%完成度

---

## 📖 目录

1. [概述](#概述)
2. [安装与配置](#安装与配置)
3. [支持的编辑器](#支持的编辑器)
4. [功能说明](#功能说明)
5. [配置选项](#配置选项)
6. [故障排除](#故障排除)
7. [高级用法](#高级用法)
8. [开发者指南](#开发者指南)

---

## 概述

Game Engine LSP (Language Server Protocol) 为游戏引擎项目提供智能代码补全、诊断、导航等功能，提升开发效率。

### 主要特性

- ✅ **智能代码补全**: 基于项目上下文的自动补全
- ✅ **实时诊断**: 语法错误、类型错误、未使用变量警告
- ✅ **悬停信息**: 快速查看类型、文档字符串
- ✅ **转到定义**: 快速导航到符号定义
- ✅ **查找引用**: 查找符号的所有使用位置
- ✅ **代码格式化**: 自动格式化代码
- ✅ **代码动作**: 快速修复、重构建议
- 🚧 **代码片段**: 常用代码模板（计划中）
- 🚧 **语义高亮**: 更精确的语法高亮（计划中）

### 与 Rust Analyzer 的区别

Game Engine LSP 专门为游戏引擎项目优化，提供：
- 游戏引擎特定的 API 补全
- 组件系统智能提示
- 实体-组件系统（ECS）类型安全检查
- 资源管理路径自动补全

---

## 安装与配置

### 前置要求

- Rust 工具链 (rustc 1.70+)
- Cargo
- 支持LSP的代码编辑器

### 安装 LSP 服务器

```bash
# 从源码构建
cd game_engine
cargo install --path game_engine/src/bin/game-engine-lsp

# 或使用 cargo install（已发布时）
cargo install game-engine-lsp
```

### 验证安装

```bash
game-engine-lsp --version
game-engine-lsp --help
```

---

## 支持的编辑器

### VS Code

1. **安装扩展**

创建 `.vsix` 扩展包或从市场安装（未来功能）：

```bash
cd game_engine/src/tools/lsp/vscode
npm install
npm run package
# 生成的 .vsix 文件可以安装到 VS Code
```

2. **配置 VS Code**

创建或编辑 `.vscode/settings.json`：

```json
{
  "languageServerExample.exampleServer.maxNumberOfProblems": 1000,
  "diagnostic.messageTemplate": "engine: {message}"
}
```

3. **客户端配置**

`game_engine/src/tools/lsp/vscode/src/extension.ts`:

```typescript
import * as vscode from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
  const serverOptions: ServerOptions = {
    command: 'game-engine-lsp',
    args: []
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'rust' }],
    synchronize: {
      configurationSection: 'languageServerExample'
    }
  };

  client = new LanguageClient(
    'gameEngineLanguageServer',
    'Game Engine Language Server',
    serverOptions,
    clientOptions
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
```

### Vim/Neovim

#### Neovim (nvim-lspconfig)

添加到 `init.lua`:

```lua
require('lspconfig')['game-engine-lsp'].setup {
  cmd = { 'game-engine-lsp' },
  filetypes = { 'rust' },
  root_dir = require('lspconfig.util').root_pattern('Cargo.toml', '.git'),
  settings = {
    game_engine = {
      maxNumberOfProblems = 100,
    }
  }
}
```

#### Vim (vim-lsp)

添加到 `.vimrc`:

```vim
if executable('game-engine-lsp')
  au User lsp_setup call lsp#register_server({
    \ 'name': 'game-engine-lsp',
    \ 'cmd': {serverroot->..'/game-engine-lsp'},
    \ 'whitelist': ['rust'],
    \ })
endif
```

### Emacs

使用 `lsp-mode`:

```elisp
(use-package lsp-mode
  :config
  (lsp-register-custom-settings
   '(("game-engine-lsp.maxNumberOfProblems" . 100)))
  :hook (rust-mode . lsp))

(use-package lsp-ui
  :after lsp-mode
  :config
  (setq lsp-ui-doc-enable t
        lsp-ui-flycheck-enable t))
```

### 其他编辑器

任何支持 LSP 的编辑器都可以使用 `game-engine-lsp`：

**服务器命令**: `game-engine-lsp`
**支持的语言**: `rust`
**根目录标记**: `Cargo.toml`, `.git`

---

## 功能说明

### 代码补全

当您输入时，LSP 会自动提供补全建议：

```rust
fn main() {
    let entity = world.spawn((
        // 在这里会显示可用的组件类型
        Position::new(0.0, 0.0),
        Velocity::new(1.0, 0.0),
    ));

    // 输入 entity. 会显示可用的方法
    entity.
}
```

**触发方式**:
- 自动触发（输入时）
- 手动触发: `Ctrl+Space` (VS Code)
- 上下文感知: 根据类型提供相关建议

### 诊断信息

实时显示错误和警告：

```rust
fn main() {
    let x: i32 = "string";  // ❌ 类型不匹配错误
    let y = 42;             // ⚠️  未使用变量警告
}
```

**诊断类型**:
- ❌ **错误**: 语法错误、类型错误
- ⚠️ **警告**: 未使用变量、死代码
- ℹ️ **信息**: 代码建议
- 💡 **提示**: 最佳实践

### 悬停信息

将鼠标悬停在符号上查看详细信息：

```rust
Position::new(x, y)
//   ^^^^^^^ 悬停显示: 创建新位置
//            参数: x: f32, y: f32
//            返回: Position
```

**显示内容**:
- 类型签名
- 简短描述
- 参数说明
- 返回值类型

### 转到定义

快速导航到符号定义：

**快捷键**:
- VS Code: `F12`
- Vim/Neovim: `gd`
- Emacs: `M-.`

**示例**:
```rust
fn main() {
    let pos = Position::new(0.0, 0.0);
    //              ^^^^^^^^ 跳转到 Position::new 定义
}
```

### 查找引用

查找符号的所有使用位置：

**快捷键**:
- VS Code: `Shift+F12`
- Vim/Neovim: `gr`
- Emacs: `M-?`

### 代码格式化

自动格式化代码：

**快捷键**:
- VS Code: `Shift+Alt+F`
- Vim/Neovim: `=`
- Emacs: `C-c C-f`

**格式化示例**:
```rust
// 格式化前
fn add(a:i32,b:i32)->i32{return a+b;}

// 格式化后
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}
```

### 代码动作

快速修复和建议：

```rust
fn main() {
    let x = 42;
}
//    ^ 悬停显示: "未使用的变量: x"
//    快速修复: "在变量名前添加下划线"
//            -> "let _x = 42;"
```

---

## 配置选项

### LSP 服务器配置

在项目根目录创建 `game-engine-lsp.toml`:

```toml
# 最大问题数量
max_number_of_problems = 100

# 诊断消息模板
diagnostic_message_template = "engine: {message}"

# 日志级别
log_level = "INFO"

# Cargo 功能
cargo_features = ["all"]

# Rust 工具链路径
rustc_path = "rustc"
cargo_path = "cargo"

# 性能设置
max_inlay_hints = 100
completion_limit = 100

# 实验性功能
experimental_features = true
```

### 编辑器特定配置

#### VS Code (.vscode/settings.json)

```json
{
  "game-engine-lsp.maxNumberOfProblems": 100,
  "game-engine-lsp.diagnosticTemplate": "engine: {message}",
  "game-engine-lsp.cargo.features": ["all"],
  "game-engine-lsp.completion.limit": 100,
  "editor.formatOnSave": true,
  "editor.formatOnType": true,
  "editor.inlayHints.enabled": true
}
```

#### Neovim

```lua
require('lspconfig')['game-engine-lsp'].setup {
  settings = {
    game_engine = {
      maxNumberOfProblems = 100,
      completion = {
        limit = 100
      },
      cargo = {
        features = {"all"}
      }
    }
  }
}
```

---

## 故障排除

### LSP 无法启动

**症状**: 编辑器显示 "LSP server failed to start"

**解决方案**:

1. **检查是否安装**:
   ```bash
   which game-engine-lsp
   ```

2. **手动运行测试**:
   ```bash
   game-engine-lsp --version
   ```

3. **查看日志**:
   - VS Code: `View > Output > Game Engine LSP`
   - Neovim: `:LspLog`

### 补全不工作

**症状**: 输入时没有补全建议

**解决方案**:

1. **检查文件类型**: 确保是 `.rs` 文件
2. **检查项目根目录**: 确保有 `Cargo.toml`
3. **重启 LSP**:
   - VS Code: `Ctrl+Shift+P` > "LSP: Restart Server"
   - Neovim: `:LspRestart`

### 诊断信息不准确

**症状**: 显示错误的错误或警告

**解决方案**:

1. **检查 Rust 工具链版本**:
   ```bash
   rustc --version  # 应该 >= 1.70
   ```

2. **更新依赖**:
   ```bash
   cargo update
   ```

3. **清理缓存**:
   ```bash
   cargo clean
   rm -rf target/
   ```

### 性能问题

**症状**: 编辑器卡顿，补全延迟高

**解决方案**:

1. **限制补全数量**:
   ```toml
   completion_limit = 50  # 减少补全项
   ```

2. **禁用某些功能**:
   ```toml
   enable_inlay_hints = false
   enable_semantic_tokens = false
   ```

3. **增加内存限制**:
   ```bash
   export RUST_MAX_MEMORY=4096  # MB
   ```

---

## 高级用法

### 自定义诊断

创建 `.game-engine-lsp/diagnostics.toml`:

```toml
[[rules]]
name = "no_std_allowed"
pattern = "use std::"
message = "避免使用 std，使用 alloc 替代"
severity = "warning"

[[rules]]
name = "no_panic_in_game_code"
pattern = "panic!"
message = "游戏代码不应使用 panic!"
severity = "error"
```

### 自定义补全

创建 `.game-engine-lsp/completions.toml`:

```toml
[[snippets]]
prefix = "comp"
description = "创建新组件"
body = """
#[derive(Component, Debug)]
pub struct ${1:Name} {
    ${2:field}: ${3:Type},
}
"""

[[snippets]]
prefix = "system"
description = "创建新系统"
body = """
pub fn ${1:function_name}(${2:query}: Query<$3>) {
    for ${4:entity} in ${2:query}.iter() {
        ${5:// 逻辑}
    }
}
"""
```

### 与 CI/CD 集成

```bash
#!/bin/bash
# scripts/lint-with-lsp.sh

# 在 CI 中运行 LSP 诊断
game-engine-lsp check --diagnostics-all > diagnostics.txt

# 检查是否有错误
if grep -q "error" diagnostics.txt; then
    echo "发现错误，构建失败"
    exit 1
fi

echo "LSP 检查通过"
```

---

## 开发者指南

### LSP 架构

```
┌─────────────────────────────────────┐
│         编辑器 (Client)             │
│  VS Code / Vim / Emacs / etc.      │
└──────────────┬──────────────────────┘
               │ LSP Protocol (JSON-RPC)
               │
┌──────────────▼──────────────────────┐
│      Game Engine LSP Server         │
│  ┌────────────────────────────┐    │
│  │   核心引擎                 │    │
│  │  - 语法分析                │    │
│  │  - 类型推断                │    │
│  │  - 代码补全                │    │
│  └────────────────────────────┘    │
│  ┌────────────────────────────┐    │
│  │   游戏引擎扩展             │    │
│  │  - 组件系统补全            │    │
│  │  - ECS 类型检查            │    │
│  │  - 资源路径补全            │    │
│  └────────────────────────────┘    │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│      Rust Analyzer (可选)           │
│  (用于底层 Rust 语言支持)            │
└─────────────────────────────────────┘
```

### 添加新功能

1. **实现 LSP 方法** (`game_engine/src/tools/lsp/server.rs`):

```rust
impl GameEngineLSP {
    // 添加新的 LSP 方法
    async fn handle_custom_request(&self, params: CustomParams) -> Result<CustomResult> {
        // 实现逻辑
        Ok(result)
    }
}
```

2. **注册处理器**:

```rust
#[tower_lsp::async_trait]
impl LanguageServer for GameEngineLSP {
    async fn custom_request(&self, params: CustomParams) -> Result<CustomResult> {
        self.handle_custom_request(params).await
    }
}
```

### 测试

```bash
# 运行 LSP 测试
cargo test --package game_engine --lib lsp

# 运行集成测试
cargo test --test lsp_tests
```

### 调试

```bash
# 启用调试日志
RUST_LOG=debug game-engine-lsp

# 使用 LSP 测试客户端
cargo run --bin lsp-test-client
```

---

## 相关资源

- **LSP 规范**: https://microsoft.github.io/language-server-protocol/
- **Rust Analyzer**: https://rust-analyzer.github.io/
- **VS Code 扩展 API**: https://code.visualstudio.com/api
- **项目仓库**: `/game_engine/src/tools/lsp/`

---

## 更新日志

### v0.2.0 (2026-01-03)

- ✅ 基础 LSP 服务器实现
- ✅ 代码补全
- ✅ 诊断信息
- ✅ 悬停信息
- ✅ 转到定义
- ✅ 代码格式化
- ✅ 测试框架

### 未来计划

- 🚧 VS Code 扩展完整实现
- 🚧 代码片段系统
- 🚧 语义高亮
- 🚧 重构支持
- 🚧 性能优化

---

## 支持

如有问题或建议，请：
1. 查看 FAQ: `/docs/faq.md`
2. 提交 Issue: GitHub Issues
3. 贡献代码: Pull Requests

**祝您开发愉快！** 🚀
