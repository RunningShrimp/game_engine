//! # 移动平台API使用示例
//!
//! 演示如何在脚本中使用Google Play Games和Game Center功能

use game_engine::{
    ecs::{Entity, World},
    scripting::{
        api::ScriptApi,
        mobile_api::MobileScriptApi,
        system::{JavaScriptContext, ScriptContext, ScriptSystem},
    },
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    println!("=== 移动平台API使用示例 ===\n");

    // 创建ECS世界
    let mut world = World::new();

    // 创建脚本系统
    let mut script_system = ScriptSystem::new();
    let mut js_context = JavaScriptContext::new();

    // 创建移动平台API
    let mobile_api = MobileScriptApi::new();

    // 注册API到脚本系统
    let mut script_api = ScriptApi::new();
    mobile_api.register_api(&mut script_api);
    js_context.register_api(&script_api);

    script_system.register_context(
        game_engine::scripting::system::ScriptLanguage::JavaScript,
        Box::new(js_context.clone()),
    );

    // 示例1: Google Play Games登录
    println!("📱 示例1: Google Play Games登录");
    let gpg_signin_script = r#"
        // 初始化Google Play Games
        gpg_initialize();

        // 尝试登录
        const signed_in = gpg_sign_in();
        if (signed_in) {
            console.log("登录成功！");

            // 获取玩家信息
            const player = gpg_get_player();
            if (player !== null) {
                console.log("玩家ID: " + player.id);
                console.log("玩家名称: " + player.name);
                console.log("玩家等级: " + player.level);
            }
        } else {
            console.log("登录失败或已取消");
        }
    "#;

    match js_context.eval(gpg_signin_script) {
        Ok(result) => println!("执行结果: {:?}", result),
        Err(e) => println!("执行错误: {}", e),
    }

    println!("\n{}", "─".repeat(50));

    // 示例2: 成就系统
    println!("🏆 示例2: 成就系统");
    let achievement_script = r#"
        // 解锁成就
        gpg_unlock_achievement("achievement_first_win");

        // 更新成就进度
        gpg_set_achievement_progress("achievement_play_10_games", 50);

        // 显示所有成就
        gpg_show_achievements();
    "#;

    match js_context.eval(achievement_script) {
        Ok(result) => println!("执行结果: {:?}", result),
        Err(e) => println!("执行错误: {}", e),
    }

    println!("\n{}", "─".repeat(50));

    // 示例3: 排行榜
    println!("🏅 示例3: 排行榜");
    let leaderboard_script = r#"
        // 提交分数
        gpg_submit_score("leaderboard_high_scores", 10000);

        // 显示排行榜
        gpg_show_leaderboard("leaderboard_high_scores");
    "#;

    match js_context.eval(leaderboard_script) {
        Ok(result) => println!("执行结果: {:?}", result),
        Err(e) => println!("执行错误: {}", e),
    }

    println!("\n{}", "─".repeat(50));

    // 示例4: 游戏生命周期集成
    println!("🎮 示例4: 游戏生命周期集成");
    let lifecycle_script = r#"
        // 游戏开始时初始化
        function on_game_start() {
            gpg_initialize();
            gpg_sign_in();
        }

        // 玩家完成关卡
        function on_level_complete(level_id, score) {
            // 提交分数
            gpg_submit_score("level_" + level_id + "_scores", score);

            // 解锁成就
            if (score > 5000) {
                gpg_unlock_achievement("achievement_master");
            }
        }

        // 显示游戏内UI
        function show_leaderboards_button() {
            gpg_show_leaderboard("main_leaderboard");
        }

        function show_achievements_button() {
            gpg_show_achievements();
        }

        // 游戏结束时登出
        function on_game_quit() {
            gpg_sign_out();
        }

        // 调用示例
        on_game_start();
        on_level_complete("1", 7500);
    "#;

    match js_context.eval(lifecycle_script) {
        Ok(result) => println!("执行结果: {:?}", result),
        Err(e) => println!("执行错误: {}", e),
    }

    println!("\n{}", "─".repeat(50));

    // 示例5: Game Center (iOS)
    println!("🍎 示例5: Game Center (iOS)");
    let game_center_script = r#"
        // 初始化Game Center
        gc_initialize();

        // 认证玩家
        const authenticated = gc_authenticate();
        if (authenticated) {
            console.log("Game Center认证成功");

            // 报告成就
            gc_report_achievement("achievement_speed_runner");

            // 提交分数
            gc_submit_score("speedrun_leaderboard", 120);

            // 显示Game Center仪表板
            gc_show_game_center();
        }
    "#;

    match js_context.eval(game_center_script) {
        Ok(result) => println!("执行结果: {:?}", result),
        Err(e) => println!("执行错误: {}", e),
    }

    println!("\n{}", "─".repeat(50));

    // 示例6: 推送通知
    println!("🔔 示例6: 推送通知");
    let notification_script = r#"
        // 初始化推送通知
        push_initialize();

        // 请求通知权限
        const granted = push_request_permission();
        if (granted) {
            console.log("通知权限已授予");

            // 发送本地通知
            push_send_local(
                "每日奖励",
                "登录游戏领取每日奖励！"
            );
        } else {
            console.log("通知权限被拒绝");
        }
    "#;

    match js_context.eval(notification_script) {
        Ok(result) => println!("执行结果: {:?}", result),
        Err(e) => println!("执行错误: {}", e),
    }

    println!("\n{}", "═".repeat(50));
    println!("✅ 所有示例执行完成！");

    Ok(())
}

/// Lua示例（如果启用了mlua feature）
#[cfg(feature = "mlua")]
fn lua_example() -> Result<(), Box<dyn std::error::Error>> {
    use game_engine::scripting::system::LuaContext;

    println!("\n🌙 Lua示例");

    let mut lua_context = LuaContext::new();
    let mobile_api = MobileScriptApi::new();

    let mut script_api = ScriptApi::new();
    mobile_api.register_api(&mut script_api);
    lua_context.register_api(&script_api);

    let lua_script = r#"
        -- Google Play Games登录
        gpg_initialize()
        local signed_in = gpg_sign_in()

        if signed_in then
            print("Lua: 登录成功")

            -- 解锁成就
            gpg_unlock_achievement("achievement_lua_master")

            -- 提交分数
            gpg_submit_score("lua_leaderboard", 9999)
        end
    "#;

    match lua_context.eval(lua_script) {
        Ok(result) => println!("Lua执行结果: {:?}", result),
        Err(e) => println!("Lua执行错误: {}", e),
    }

    Ok(())
}

/// Python示例（如果启用了pyo3 feature）
#[cfg(feature = "pyo3")]
fn python_example() -> Result<(), Box<dyn std::error::Error>> {
    use game_engine::scripting::system::PythonContext;

    println!("\n🐍 Python示例");

    let mut python_context = PythonContext::new();
    let mobile_api = MobileScriptApi::new();

    let mut script_api = ScriptApi::new();
    mobile_api.register_api(&mut script_api);
    python_context.register_api(&script_api);

    let python_script = r#"
# Google Play Games登录
gpg_initialize()
signed_in = gpg_sign_in()

if signed_in:
    print("Python: 登录成功")

    # 解锁成就
    gpg_unlock_achievement("achievement_python_master")

    # 提交分数
    gpg_submit_score("python_leaderboard", 8888)
    "#;

    match python_context.eval(python_script) {
        Ok(result) => println!("Python执行结果: {:?}", result),
        Err(e) => println!("Python执行错误: {}", e),
    }

    Ok(())
}

/// TypeScript示例（如果启用了typescript feature）
#[cfg(feature = "typescript")]
fn typescript_example() -> Result<(), Box<dyn std::error::Error>> {
    use game_engine::scripting::system::TypeScriptContext;

    println!("\n📘 TypeScript示例");

    let mut ts_context = TypeScriptContext::new();
    let mobile_api = MobileScriptApi::new();

    let mut script_api = ScriptApi::new();
    mobile_api.register_api(&mut script_api);
    ts_context.register_api(&script_api);

    let typescript_script = r#"
interface Player {
    id: string;
    name: string;
    level: number;
}

// Google Play Games登录
gpg_initialize();
const signedIn = gpg_sign_in();

if (signedIn) {
    console.log("TypeScript: 登录成功");

    const player: Player = gpg_get_player();
    console.log(`玩家: ${player.name} (等级 ${player.level})`);

    // 解锁成就
    gpg_unlock_achievement("achievement_typescript_master");

    // 提交分数
    gpg_submit_score("typescript_leaderboard", 7777);
}
    "#;

    match ts_context.eval(typescript_script) {
        Ok(result) => println!("TypeScript执行结果: {:?}", result),
        Err(e) => println!("TypeScript执行错误: {}", e),
    }

    Ok(())
}
