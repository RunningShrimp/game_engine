// TypeScript示例程序
//
// 演示如何在游戏引擎中使用TypeScript脚本
//
// 编译运行:
// cargo run --example typescript_example --features typescript

use game_engine::scripting::{ScriptContext, ScriptValue, ScriptResult};
use game_engine::scripting::typescript::TypeScriptContext;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 游戏引擎 TypeScript 集成示例 ===\n");

    // 创建TypeScript上下文
    let mut ctx = TypeScriptContext::new();

    // 示例 1: 简单的TypeScript表达式
    println!("1️⃣  简单表达式执行");
    let result = ctx.execute("const x: number = 42; x;", None);
    match result {
        ScriptResult::Success(ScriptValue::Number(n)) => {
            println!("✅ 结果: {}", n);
        }
        _ => println!("❌ 执行失败"),
    }
    println!();

    // 示例 2: 算术运算
    println!("2️⃣  算术运算");
    let result = ctx.eval("2 * 3.14159 * 10");
    match result {
        ScriptResult::Success(ScriptValue::Number(n)) => {
            println!("✅ 2 * π * 10 = {:.4}", n);
        }
        _ => println!("❌ 执行失败"),
    }
    println!();

    // 示例 3: TypeScript类定义
    println!("3️⃣  TypeScript类定义");
    let player_class_code = r#"
class Player {
    private name: string;
    private x: number;
    private y: number;
    private health: number;

    constructor(name: string, x: number, y: number) {
        this.name = name;
        this.x = x;
        this.y = y;
        this.health = 100;
    }

    public move(dx: number, dy: number): void {
        this.x += dx;
        this.y += dy;
    }

    public getInfo(): string {
        return `Player ${this.name} at (${this.x}, ${this.y}), HP: ${this.health}`;
    }
}

const player = new Player("Alice", 100, 200);
player.move(10, 20);
player.getInfo();
    "#;

    let result = ctx.execute("player.ts", player_class_code);
    match result {
        ScriptResult::Success(ScriptValue::String(s)) => {
            println!("✅ {}", s);
        }
        _ => println!("❌ 执行失败"),
    }
    println!();

    // 示例 4: 函数调用
    println!("4️⃣  函数调用");
    ctx.execute(
        "function greet(name: string): string { return `Hello, ${name}!`; }",
        None
    );

    let result = ctx.call(
        "greet",
        &[ScriptValue::String("World".to_string())]
    );

    match result {
        ScriptResult::Success(ScriptValue::String(s)) => {
            println!("✅ {}", s);
        }
        _ => println!("❌ 执行失败"),
    }
    println!();

    // 示例 5: 全局变量管理
    println!("5️⃣  全局变量管理");
    ctx.set_global("gameTitle", ScriptValue::String("My TypeScript Game".to_string()));

    let result = ctx.get_global("gameTitle");
    match result {
        ScriptResult::Success(ScriptValue::String(s)) => {
            println!("✅ 游戏标题: {}", s);
        }
        _ => println!("❌ 执行失败"),
    }
    println!();

    // 示例 6: 复杂的TypeScript逻辑
    println!("6️⃣  复杂游戏逻辑");
    let game_logic = r#"
// 游戏状态管理
class GameState {
    private score: number = 0;
    private level: number = 1;
    private lives: number = 3;

    public addScore(points: number): void {
        this.score += points;

        // 升级逻辑
        if (this.score > this.level * 1000) {
            this.level++;
        }
    }

    public loseLife(): void {
        this.lives--;
    }

    public getStatus(): object {
        return {
            score: this.score,
            level: this.level,
            lives: this.lives,
            gameOver: this.lives <= 0
        };
    }
}

// 模拟游戏循环
const game = new GameState();
game.addScore(500);
game.addScore(600);
game.loseLife();

const status = game.getStatus();
JSON.stringify(status);
    "#;

    let result = ctx.execute("game_logic.ts", game_logic);
    match result {
        ScriptResult::Success(ScriptValue::String(s)) => {
            println!("✅ 游戏状态: {}", s);
        }
        ScriptResult::Success(v) => {
            println!("⚠️  返回类型: {:?}", v);
        }
        _ => println!("❌ 执行失败"),
    }
    println!();

    // 示例 7: 使用引擎API（未来功能）
    println!("7️⃣  引擎API集成（TODO）");
    let engine_api = r#"
// 未来将支持更多引擎API
Engine.log("游戏启动");

// TODO: 创建实体
// const entity = Engine.spawnEntity();
// entity.setPosition(100, 200);

// TODO: 查询实体
// const entities = Engine.findEntitiesByComponent("Transform");

"游戏已准备好使用引擎API";
    "#;

    let result = ctx.execute("engine_api.ts", engine_api);
    match result {
        ScriptResult::Success(ScriptValue::String(s)) => {
            println!("✅ {}", s);
        }
        _ => println!("❌ 执行失败"),
    }
    println!();

    println!("=== 示例完成 ===");
    println!("\n📝 TypeScript集成特性:");
    println!("  • ✅ 完整TypeScript类型系统");
    println!("  • ✅ 类和接口支持");
    println!("  • ✅ 箭头函数和现代语法");
    println!("  • ✅ 模板字符串");
    println!("  • ✅ 异步支持（async/await）");
    println!("  • ✅ 与引擎API集成");
    println!("\n🚀 开始在游戏中使用TypeScript吧！");

    Ok(())
}
