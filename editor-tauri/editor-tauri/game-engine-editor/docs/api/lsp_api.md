# LSP API 文档
# Language Server Protocol API Reference

**版本**: v0.3.0
**协议**: LSP 3.17
**最后更新**: 2026-01-03

---

## 📋 目录

1. [概述](#概述)
2. [初始化](#初始化)
3. [代码补全](#代码补全)
4. [悬停提示](#悬停提示)
5. [跳转定义](#跳转定义)
6. [查找引用](#查找引用)
7. [符号搜索](#符号搜索)
8. [代码格式化](#代码格式化)
9. [诊断信息](#诊断信息)
10. [代码重构](#代码重构)

---

## 概述

游戏引擎LSP服务器为Rust游戏引擎代码提供全面的代码智能支持，包括代码补全、悬停提示、跳转定义等功能。

### 启动服务器

```bash
# 标准输入/输出模式
game-engine-lsp

# TCP模式
game-engine-lsp --tcp --port 9000

# Socket模式
game-engine-lsp --socket /tmp/game-engine-lsp.sock
```

### VS Code集成

```typescript
import * as vscode from 'vscode';
import * as path from 'path';

const serverOptions = {
    command: path.join('path', 'to', 'game-engine-lsp'),
    args: ['--tcp', '--port', '9000'],
};

const clientOptions = {
    documentSelector: [{ scheme: 'file', language: 'rust' }],
};

const client = new vscode.LanguageClient(
    'gameEngine',
    'Game Engine Language Server',
    serverOptions,
    clientOptions
);
```

---

## 初始化

### initialize请求

客户端发送的第一个请求，用于初始化服务器。

**请求**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "processId": 12345,
    "rootUri": "file:///path/to/project",
    "capabilities": {
      "textDocument": {
        "completion": {
          "completionItem": {
            "snippetSupport": true,
            "commitCharactersSupport": true
          }
        },
        "hover": {
          "contentFormat": ["markdown", "plaintext"]
        }
      }
    }
  }
}
```

**响应**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "capabilities": {
      "textDocumentSync": 2,
      "completionProvider": {
        "resolveProvider": true,
        "triggerCharacters": [".", ":", " "]
      },
      "hoverProvider": true,
      "definitionProvider": true,
      "referencesProvider": true,
      "documentSymbolProvider": true,
      "workspaceSymbolProvider": true,
      "documentFormattingProvider": true,
      "codeActionProvider": true,
      "renameProvider": true
    }
  }
}
```

---

## 代码补全

### completion请求

在输入位置获取代码补全建议。

**请求参数**:
```typescript
interface CompletionParams {
    textDocument: TextDocumentIdentifier;
    position: Position;
    context?: CompletionContext;
}
```

**示例**:
```rust
// 用户输入:
let entity = Entity::new("Player");
entity.add_co

// LSP补全建议:
[
    {
        "label": "add_component",
        "kind": 2,  // Method
        "detail": "fn add_component<T: Component>(&mut self, component: T)",
        "documentation": {
            "kind": "markdown",
            "value": "向实体添加组件\n\n```rust\nentity.add_component(Transform::default());\n```"
        },
        "sortText": "0add_component",
        "filterText": "add_component",
        "insertText": "add_component(${1:component})",
        "insertTextFormat": 2  // Snippet
    },
    {
        "label": "add_child",
        "kind": 2,
        "detail": "fn add_child(&mut self, child: Entity)",
        "documentation": "添加子实体"
    }
]
```

**补全项类型**:
- `1`: Text (文本)
- `2`: Method (方法)
- `3`: Function (函数)
- `4`: Constructor (构造函数)
- `5`: Field (字段)
- `6`: Variable (变量)
- `7`: Class (类/结构体)
- `8`: Interface (接口/Trait)
- `9`: Module (模块)
- `12`: Keyword (关键字)

**性能目标**:
- 响应时间: <50ms
- 补全项数量: 10-20项
- 相关性排序: >90%准确率

---

## 悬停提示

### hover请求

显示符号的类型和文档信息。

**请求参数**:
```typescript
interface HoverParams {
    textDocument: TextDocumentIdentifier;
    position: Position;
}
```

**示例**:
```rust
// 用户悬停在:
Entity::new("Player")
     ^^^^

// LSP响应:
{
    "contents": {
        "kind": "markdown",
        "value": "```rust\npub fn new(name: impl Into<String>) -> Self\n```\n\n创建新实体\n\n**参数:**\n- `name`: 实体名称\n\n**返回值:**\n新创建的实体实例\n\n**示例:**\n```rust\nlet entity = Entity::new(\"Player\");\n```"
    },
    "range": {
        "start": { "line": 0, "character": 8 },
        "end": { "line": 0, "character": 12 }
    }
}
```

**支持的悬停类型**:
- 函数和方法
- 结构体和枚举
- 变量和字段
- Trait和类型别名
- 宏

**性能目标**:
- 响应时间: <25ms
- 文档完整性: >95%

---

## 跳转定义

### definition请求

跳转到符号的定义位置。

**请求参数**:
```typescript
interface DefinitionParams {
    textDocument: TextDocumentIdentifier;
    position: Position;
}
```

**示例**:
```rust
// 用户请求跳转到:
entity.add_component(transform)
                ^^^^^^^^^^^^^

// LSP响应:
[
    {
        "uri": "file:///path/to/game_engine/src/entity.rs",
        "range": {
            "start": { "line": 45, "character": 0 },
            "end": { "line": 50, "character": 1 }
        }
    }
]
```

**跳转类型**:
- 同文件内定义
- 跨文件定义
- 标准库定义
- 第三方库定义
- 宏展开定义

**性能目标**:
- 响应时间: <15ms
- 准确率: >98%

---

## 查找引用

### references请求

查找符号的所有引用位置。

**请求参数**:
```typescript
interface ReferencesParams {
    textDocument: TextDocumentIdentifier;
    position: Position;
    context: {
        includeDeclaration: boolean;
    };
}
```

**示例**:
```rust
// 用户查找引用:
fn update_player(entity: &Entity) {
    entity.set_position(Vector3::ZERO);
    //    ^^^^^^^^^^^^
}

let player = Entity::new("Player");
player.set_position(Vector3::ONE);
//  ^^^^^^^^^^^^^^

// LSP响应:
[
    {
        "uri": "file:///path/to/game.rs",
        "range": {
            "start": { "line": 10, "character": 4 },
            "end": { "line": 10, "character": 17 }
        }
    },
    {
        "uri": "file:///path/to/game.rs",
        "range": {
            "start": { "line": 15, "character": 1 },
            "end": { "line": 15, "character": 14 }
        }
    }
]
```

**性能目标**:
- 响应时间: <100ms
- 覆盖率: >95%

---

## 符号搜索

### documentSymbol请求

获取文档中的所有符号。

**示例**:
```rust
// LSP响应:
[
    {
        "name": "PlayerController",
        "kind": 5,  // Struct
        "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 50, "character": 1 } },
        "children": [
            {
                "name": "new",
                "kind": 3,  // Function
                "range": { "start": { "line": 5, "character": 4 }, "end": { "line": 8, "character": 5 } }
            },
            {
                "name": "update",
                "kind": 3,
                "range": { "start": { "line": 10, "character": 4 }, "end": { "line": 15, "character": 5 } }
            }
        ]
    }
]
```

### workspaceSymbol请求

在整个工作空间搜索符号。

**请求参数**:
```typescript
interface WorkspaceSymbolParams {
    query: string;
}
```

**示例**:
```typescript
// 搜索: "Entity"

// LSP响应:
[
    {
        "name": "Entity",
        "kind": 5,
        "location": {
            "uri": "file:///path/to/game_engine/src/entity.rs",
            "range": { "start": { "line": 10, "character": 0 }, "end": { "line": 100, "character": 1 } }
        }
    },
    {
        "name": "EntityManager",
        "kind": 5,
        "location": {
            "uri": "file:///path/to/game_engine/src/entity_manager.rs",
            "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 200, "character": 1 } }
        }
    }
]
```

**性能目标**:
- 响应时间: <200ms
- 相关性: >90%

---

## 代码格式化

### formatting请求

格式化整个文档。

**请求参数**:
```typescript
interface DocumentFormattingParams {
    textDocument: TextDocumentIdentifier;
    options: {
        tabSize: number;
        insertSpaces: boolean;
    };
}
```

**示例**:
```rust
// 格式化前:
fn foo(x:i32,y:i32)->i32{x+y}

// 格式化后:
fn foo(x: i32, y: i32) -> i32 {
    x + y
}
```

**格式化规则**:
- 缩进: 4空格
- 最大行宽: 100字符
- 函数大括号位置: 行尾
- 结构体字段对齐: 是
- 尾随逗号: 是

**性能目标**:
- 响应时间: <500ms
- 准确率: >99%

---

## 诊断信息

### diagnostics通知

服务器主动发送的诊断信息。

**示例**:
```rust
// LSP发送诊断:
{
    "jsonrpc": "2.0",
    "method": "textDocument/publishDiagnostics",
    "params": {
        "uri": "file:///path/to/game.rs",
        "diagnostics": [
            {
                "range": {
                    "start": { "line": 10, "character": 8 },
                    "end": { "line": 10, "character": 12 }
                },
                "severity": 1,  // Error
                "message": "未找到方法 `add_componet`\n\n你是否想说的是 `add_component`？",
                "code": "E0599",
                "source": "rustc",
                "relatedInformation": [
                    {
                        "message": "相似方法定义在此",
                        "location": {
                            "uri": "file:///path/to/entity.rs",
                            "range": {
                                "start": { "line": 45, "character": 0 },
                                "end": { "line": 50, "character": 1 }
                            }
                        }
                    }
                ]
            },
            {
                "range": {
                    "start": { "line": 15, "character": 0 },
                    "end": { "line": 15, "character": 20 }
                },
                "severity": 2,  // Warning
                "message": "未使用的变量 `x`",
                "code": "unused_variables",
                "source": "clippy"
            }
        ]
    }
}
```

**诊断级别**:
- `1`: Error (错误)
- `2`: Warning (警告)
- `3`: Information (信息)
- `4`: Hint (提示)

---

## 代码重构

### rename请求

重命名符号。

**请求参数**:
```typescript
interface RenameParams {
    textDocument: TextDocumentIdentifier;
    position: Position;
    newName: string;
}
```

**示例**:
```rust
// 用户重命名:
let entity = Entity::new("Player");
    //^^^^^
// 新名称: "game_entity"

// LSP响应:
{
    "documentChanges": [
        {
            "textDocument": {
                "uri": "file:///path/to/game.rs"
            },
            "edits": [
                {
                    "range": {
                        "start": { "line": 0, "character": 4 },
                        "end": { "line": 0, "character": 9 }
                    },
                    "newText": "game_entity"
                },
                {
                    "range": {
                        "start": { "line": 5, "character": 8 },
                        "end": { "line": 5, "character": 13 }
                    },
                    "newText": "game_entity"
                }
            ]
        }
    ]
}
```

**重构能力**:
- 跨文件重命名
- 作用域感知
- 注释和文档更新
- 字符串字面量更新

**性能目标**:
- 响应时间: <1s
- 准确率: >95%

---

## 高级功能

### 代码动作 (Code Actions)

提供可执行的代码修复建议。

**示例**:
```rust
// LSP响应:
{
    "title": "导入 `Entity`",
    "kind": "quickfix",
    "edit": {
        "documentChanges": [
            {
                "textDocument": { "uri": "..." },
                "edits": [
                    {
                        "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } },
                        "newText": "use game_engine::ecs::Entity;\n\n"
                    }
                ]
            }
        ]
    }
}
```

### 语义高亮

提供更精确的语法高亮。

**示例**:
```rust
// 不同符号类型使用不同颜色:
let entity: Entity = Entity::new("Player");
//  ^^^^     ^^^^^^   ^^^^^^ ^^^^ ^^^^^^^^^^
//  变量      类型      函数    函数  字符串字面量
```

### 内联提示

显示参数名称等信息。

**示例**:
```rust
entity.set_position(
    //  ^^^^^^^^^^^ position
    Vector3::new(x, y, z)
);
```

---

## 性能优化

### 索引缓存

服务器缓存解析结果，避免重复解析。

**配置**:
```json
{
    "cacheEnabled": true,
    "cacheSize": 1000,
    "cacheTTL": 3600
}
```

### 增量解析

只重新解析修改的文件。

**配置**:
```json
{
    "incrementalParsing": true,
    "fileWatcherEnabled": true
}
```

### 并行处理

使用多线程并行处理请求。

**配置**:
```json
{
    "parallelism": 4,
    "asyncProcessing": true
}
```

---

## 故障排除

### 常见问题

**Q: LSP服务器无法启动**
- A: 检查端口是否被占用，确保防火墙允许连接

**Q: 补全建议不显示**
- A: 检查文件是否保存，确保项目索引完成

**Q: 诊断信息不准确**
- A: 清理缓存并重新索引项目

### 日志调试

启用详细日志：
```bash
RUST_LOG=debug game-engine-lsp
```

---

## 参考资料

- [LSP规范](https://microsoft.github.io/language-server-protocol/)
- [tower-lsp文档](https://docs.rs/tower-lsp/)
- [VS Code扩展API](https://code.visualstudio.com/api)

---

**文档版本**: v1.0
**最后更新**: 2026-01-03
**维护者**: Game Engine Team
