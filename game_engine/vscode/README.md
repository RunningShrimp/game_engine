# Game Engine VSCode Extension

为游戏引擎提供完整的VSCode IDE支持，包括语言服务器和调试器。

## 功能特性

### 语言服务器协议 (LSP)

- ✅ **代码补全**: 智能感知引擎API、组件、系统
- ✅ **悬停提示**: 显示类型和文档信息
- ✅ **跳转到定义**: 快速导航到符号定义
- ✅ **实时诊断**: 代码错误检测和警告
- ✅ **多语言支持**: Lua、TypeScript、JavaScript、Python

### 调试适配器协议 (DAP)

- ✅ **断点管理**: 设置和管理断点
- ✅ **步进执行**: 单步执行、单步进入、单步跳出
- ✅ **调用栈**: 查看完整的调用堆栈
- ✅ **变量监视**: 实时查看和修改变量值
- ✅ **表达式求值**: 在调试时求值表达式

## 安装使用

### 方法1: 从VSIX文件安装

1. 构建扩展：
```bash
cd vscode
npm install
npm run vscode:prepublish
```

2. 在VSCode中：
- 按 `F5` 启动扩展开发主机
- 或者从命令面板选择 "Install from VSIX..."

### 方法2: 开发模式

1. 安装依赖：
```bash
cd vscode
npm install
```

2. 按 `F5` 在VSCode中启动扩展开发主机

## 配置

### LSP服务器配置

在 `settings.json` 中配置：

```json
{
  "game-engine.lsp.enabled": true,
  "game-engine.lsp.path": "game-engine-lsp",
  "game-engine.lsp.args": [],
  "game-engine.lsp.trace.server": "off",
  "game-engine.debug.enabled": true,
  "game-engine.debug.port": 4711
}
```

### 调试配置

在 `.vscode/launch.json` 中添加调试配置：

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "game-engine",
      "request": "launch",
      "name": "Debug Lua Script",
      "scriptPath": "${workspaceFolder}/scripts/main.lua",
      "scriptLanguage": "lua",
      "cwd": "${workspaceFolder}",
      "stopOnEntry": false
    },
    {
      "type": "game-engine",
      "request": "launch",
      "name": "Debug TypeScript Script",
      "scriptPath": "${workspaceFolder}/src/main.ts",
      "scriptLanguage": "typescript",
      "cwd": "${workspaceFolder}",
      "stopOnEntry": false
    },
    {
      "type": "game-engine",
      "request": "launch",
      "name": "Debug Python Script",
      "scriptPath": "${workspaceFolder}/scripts/main.py",
      "scriptLanguage": "python",
      "cwd": "${workspaceFolder}",
      "stopOnEntry": false
    }
  ]
}
```

## 使用示例

### Lua脚本调试

1. 创建Lua脚本 (`scripts/main.lua`):
```lua
local player = {
    x = 100,
    y = 200,
    health = 100
}

function update(deltaTime)
    player.x = player.x + 1
    player.y = player.y + 1
    print("Player position:", player.x, player.y)
end

-- 调试时会在这里暂停
update(0.016)
```

2. 在VSCode中：
   - 打开 `main.lua`
   - 在第17行设置断点（点击行号左侧）
   - 按 `F5` 启动调试
   - 使用调试工具栏控制执行

### TypeScript脚本调试

1. 创建TypeScript脚本 (`src/main.ts`):
```typescript
class PlayerController {
    private x: number = 100;
    private y: number = 200;

    update(deltaTime: number): void {
        this.x += 1;
        this.y += 1;
        console.log(`Player position: ${this.x}, ${this.y}`);
    }
}

const player = new PlayerController();
player.update(0.016);
```

2. 在VSCode中：
   - 打开 `main.ts`
   - 设置断点
   - 按 `F5` 启动调试
   - 在调试控制台查看变量

## 键盘快捷键

### 调试快捷键

| 快捷键 | 功能 |
|--------|------|
| `F5` | 启动调试 |
| `Shift+F5` | 停止调试 |
| `F9` | 切换断点 |
| `F10` | 单步跳过 |
| `F11` | 单步进入 |
| `Shift+F11` | 单步跳出 |
| `Ctrl+Shift+F5` | 重启调试 |

## 高级功能

### 条件断点

右键点击断点红点，选择 "Edit Breakpoint..."，然后输入条件：
```lua
player.health > 50
```

### 变量监视

在调试时，可以添加监视表达式：
- `player.x` - 监视player的x坐标
- `deltaTime` - 监视时间增量
- `player.health < 50` - 条件监视

### 调用堆栈

暂停时，查看CALL STACK面板可以看到完整的调用链。

### 日志点

不暂停程序，只输出日志：
1. 右键点击行号
2. 选择 "Add Logpoint..."
3. 输入日志消息，例如：
```
Player position: {player.x}, {player.y}
```

## 故障排查

### LSP服务器无法启动

1. 检查 `game-engine-lsp` 是否在PATH中
2. 查看输出面板 "Game Engine LSP Trace"
3. 确保端口未被占用

### 调试器无法连接

1. 确保DAP端口可用（默认4711）
2. 检查调试配置中的端口号
3. 查看调试控制台输出

### 断点无法命中

1. 确保文件路径正确
2. 检查代码是否已执行到该行
3. 确保脚本语言配置正确

## 开发

### 构建扩展

```bash
npm install
npm run compile
```

### 运行测试

```bash
npm test
```

### 打包扩展

```bash
npm run vscode:prepublish
vsce package
```

## 支持的语言

| 语言 | 扩展名 | LSP | DAP |
|------|--------|-----|-----|
| Lua | `.lua` | ✅ | ✅ |
| TypeScript | `.ts`, `.tsx` | ✅ | ✅ |
| JavaScript | `.js`, `.jsx` | ✅ | ✅ |
| Python | `.py` | ✅ | ✅ |
| Rust | `.rs` | ✅ | ❌ |

## 贡献

欢迎提交Issue和Pull Request！

## 许可证

MIT License
