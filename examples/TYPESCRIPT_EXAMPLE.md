# TypeScript使用指南

本指南介绍如何在游戏引擎中使用TypeScript编写游戏脚本。

## 快速开始

### 1. 启用TypeScript Feature

在`Cargo.toml`中启用typescript feature：

```toml
[dependencies]
game_engine = { path = "./game_engine", features = ["typescript"] }
```

或者直接运行：

```bash
cargo build --features typescript
```

### 2. 基础示例

```rust
use game_engine::scripting::{ScriptContext, ScriptValue};
use game_engine::scripting::typescript::TypeScriptContext;

fn main() {
    let mut ctx = TypeScriptContext::new();

    // 执行TypeScript代码
    let result = ctx.execute(
        "const x: number = 42; x;",
        None
    );

    match result {
        ScriptResult::Success(ScriptValue::Number(42.0)) => {
            println!("TypeScript执行成功！");
        }
        _ => {}
    }
}
```

## TypeScript特性

### 支持的语言特性

✅ **完整TypeScript语法**:
- 类型注解 (`: number`, `: string`)
- 类和接口
- 箭头函数
- 模板字符串
- 解构赋值
- Spread操作符
- async/await
- 模块导入/导出

✅ **JavaScript ES6+**:
- let/const
- 类语法
- Promise
- Map/Set
- 生成器

### 游戏脚本示例

#### 玩家类

```typescript
class Player {
    private x: number = 100;
    private y: number = 200;
    private health: number = 100;

    constructor(x: number, y: number) {
        this.x = x;
        this.y = y;
    }

    public move(dx: number, dy: number): void {
        this.x += dx;
        this.y += dy;
    }

    public getInfo(): string {
        return `Player at (${this.x}, ${this.y}), HP: ${this.health}`;
    }
}

const player = new Player(100, 200);
player.move(10, 20);
console.log(player.getInfo());
```

#### 游戏循环

```typescript
// 游戏状态
class GameState {
    private score: number = 0;
    private level: number = 1;
    private isGameOver: boolean = false;

    public update(deltaTime: number): void {
        if (this.isGameOver) return;

        // 游戏逻辑
        this.score += Math.floor(deltaTime * 60);

        // 升级
        if (this.score > this.level * 1000) {
            this.level++;
            Engine.log(`升级到 Level ${this.level}!`);
        }
    }
}

// 游戏循环
const game = new GameState();

function gameLoop(deltaTime: number): void {
    game.update(deltaTime);

    if (!game.isGameOver) {
        requestAnimationFrame(gameLoop);
    }
}

// 启动游戏
requestAnimationFrame(gameLoop);
```

## API参考

### ScriptContext Trait

```rust
pub trait ScriptContext: Send + Sync {
    // 执行脚本
    fn execute(&mut self, script: &str, source_code: Option<&str>) -> ScriptResult;

    // 评估表达式
    fn eval(&mut self, expression: &str) -> ScriptResult;

    // 调用函数
    fn call(&mut self, function: &str, args: &[ScriptValue]) -> ScriptResult;

    // 设置全局变量
    fn set_global(&mut self, name: &str, value: ScriptValue) -> ScriptResult;

    // 获取全局变量
    fn get_global(&mut self, name: &str) -> ScriptResult;

    // 重置上下文
    fn reset(&mut self);
}
```

### TypeScriptContext

```rust
use game_engine::scripting::typescript::TypeScriptContext;

// 创建上下文
let mut ctx = TypeScriptContext::new();

// 自动初始化（首次调用时）
ctx.execute("const x = 42;", None);

// 手动初始化（可选）
ctx.runtime.initialize()?;

// 重置运行时
ctx.reset();
```

### ScriptValue枚举

```rust
pub enum ScriptValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(String),
    Array(Vec<ScriptValue>),
    Object(HashMap<String, ScriptValue>),
}
```

### 类型转换

**Rust → TypeScript**:

```rust
// 数字
ctx.set_global("pi", ScriptValue::Number(3.14159));

// 字符串
ctx.set_global("name", ScriptValue::String("Alice".to_string()));

// 布尔值
ctx.set_global("isActive", ScriptValue::Boolean(true));

// 数组
ctx.set_global("numbers", ScriptValue::Array(vec![
    ScriptValue::Number(1.0),
    ScriptValue::Number(2.0),
    ScriptValue::Number(3.0),
]));

// 对象
let mut map = HashMap::new();
map.insert("x".to_string(), ScriptValue::Number(100.0));
map.insert("y".to_string(), ScriptValue::Number(200.0));
ctx.set_global("position", ScriptValue::Object(map));
```

**TypeScript → Rust**:

```rust
// 执行并获取结果
match ctx.execute("return 42;", None) {
    ScriptResult::Success(ScriptValue::Number(n)) => {
        println!("结果: {}", n);
    }
    ScriptResult::Success(ScriptValue::String(s)) => {
        println!("结果: {}", s);
    }
    ScriptResult::Error(e) => {
        eprintln!("错误: {}", e);
    }
    _ => {}
}
```

## 引擎API

### 日志输出

```typescript
// 输出到引擎日志
Engine.log("游戏开始");
Engine.log(`玩家分数: ${score}`);
```

### 实体管理（TODO）

```typescript
// 未来将支持：
const entity = Engine.spawnEntity();
entity.addComponent("Transform", { x: 100, y: 200 });
entity.addComponent("Sprite", { texture: "player.png" });

// 查询实体
const players = Engine.findEntitiesByComponent("PlayerController");
```

## 调试

### VSCode配置

创建`.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "game-engine",
      "request": "launch",
      "name": "Debug TypeScript Script",
      "scriptPath": "${workspaceFolder}/src/main.ts",
      "scriptLanguage": "typescript",
      "cwd": "${workspaceFolder}",
      "stopOnEntry": false
    }
  ]
}
```

### 调试功能

- ✅ 断点设置
- ✅ 单步执行（F10/F11）
- ✅ 变量监视
- ✅ 调用堆栈
- ✅ 表达式求值

## 性能考虑

### QuickJS特性

- **引擎大小**: 210KB（相比V8的50MB）
- **启动时间**: <1ms
- **内存占用**: ~1MB基础
- **执行速度**: JavaScript的~70%

### 优化建议

1. **避免频繁创建对象**: 复用对象和数组
2. **使用类型注解**: 帮助QuickJS优化
3. **减少跨语言调用**: 批量操作数据
4. **缓存脚本结果**: 避免重复计算

```typescript
// ✅ 好 - 类型注解
function add(a: number, b: number): number {
    return a + b;
}

// ❌ 差 - 无类型
function add(a, b) {
    return a + b;
}
```

## 常见问题

### Q: 为什么选择QuickJS而非V8？

**A**:
- ✅ 更小的二进制大小（210KB vs 50MB）
- ✅ 更快的启动时间（<1ms vs ~100ms）
- ✅ 更低的内存占用
- ✅ API稳定，无版本冲突
- ⚠️  执行速度稍慢（但对于游戏脚本足够）

### Q: 支持哪些TypeScript特性？

**A**: 支持完整的TypeScript 4.x语法，包括：
- 类型注解
- 类和接口
- 泛型
- 装饰器
- 模块系统
- async/await
- 等

### Q: 如何调试TypeScript脚本？

**A**:
1. 使用VSCode的调试功能
2. 在代码中设置断点
3. 使用`console.log()`输出
4. 查看"调试控制台"

### Q: 性能如何？

**A**:
- 轻量级脚本: 无明显影响
- 复杂计算: 可能比Rust慢2-3倍
- 建议将性能关键代码用Rust实现

## 最佳实践

### 1. 类型安全

```typescript
// ✅ 使用类型注解
interface Vector2 {
    x: number;
    y: number;
}

function add(v1: Vector2, v2: Vector2): Vector2 {
    return { x: v1.x + v2.x, y: v1.y + v2.y };
}
```

### 2. 错误处理

```typescript
function safeDivide(a: number, b: number): number | null {
    if (b === 0) {
        Engine.log("错误: 除数不能为零");
        return null;
    }
    return a / b;
}
```

### 3. 模块化

```typescript
// player.ts
export class Player {
    // ...
}

// main.ts
import { Player } from './player';
const player = new Player("Alice");
```

### 4. 测试

```typescript
// 简单的测试函数
function testPlayerMovement() {
    const player = new Player(100, 200);
    player.move(10, 20);

    const pos = player.getPosition();
    console.assert(pos.x === 110, "X坐标错误");
    console.assert(pos.y === 220, "Y坐标错误");

    Engine.log("✅ 玩家移动测试通过");
}

testPlayerMovement();
```

## 进阶话题

### 异步脚本

```typescript
async function loadGameData(): Promise<object> {
    const response = await fetch("/api/game-data");
    const data = await response.json();
    return data;
}

loadGameData().then(data => {
    Engine.log("游戏数据加载完成");
    initializeGame(data);
});
```

### 自定义引擎API

```rust
// 在Rust中注册自定义API
context.with(|ctx| {
    let custom_fn = Function::new(ctx.clone(), |msg: String| {
        println!("自定义API: {}", msg);
        Ok::<(), rquickjs::Error>(())
    }).unwrap();

    ctx.globals().set("customAPI", custom_fn).unwrap();
});
```

```typescript
// 在TypeScript中使用
customAPI("来自TypeScript的调用");
```

## 参考资料

- [TypeScript手册](https://www.typescriptlang.org/docs/)
- [QuickJS文档](https://bellard.org/quickjs/)
- [rquickjs文档](https://docs.rs/rquickjs/)
- [游戏引擎文档](../docs/)

## 示例代码

查看更多示例：

- `examples/typescript_example.rs` - 完整示例程序
- `examples/scripts/` - TypeScript脚本示例

## 获取帮助

如有问题，请：

1. 查看文档
2. 搜索现有Issue
3. 创建新Issue并附上最小复现代码

---

**最后更新**: 2025-01-01
**维护者**: Game Engine Development Team
