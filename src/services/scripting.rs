//! 脚本服务模块
//!
//! 提供JavaScript/QuickJS脚本执行功能，支持绑定引擎API。

use rquickjs::{Context, Function, Runtime};

/// 脚本服务
///
/// 管理JavaScript运行时和上下文，提供脚本执行和API绑定功能。
///
/// # 使用示例
///
/// ```rust
/// use game_engine::services::scripting::ScriptingService;
///
/// let service = ScriptingService::new();
/// service.bind_core_api();
/// service.execute("print('Hello from script!');");
/// ```
pub struct ScriptingService {
    /// QuickJS运行时
    runtime: Runtime,
    /// JavaScript上下文
    context: Context,
}

impl Default for ScriptingService {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptingService {
    /// 创建新的脚本服务
    ///
    /// # Panics
    ///
    /// 如果无法创建QuickJS运行时或上下文，此方法会panic。
    ///
    /// # 注意
    ///
    /// 在生产环境中，应该处理这些错误而不是panic。
    pub fn new() -> Self {
        let runtime = Runtime::new().unwrap();
        let context = Context::full(&runtime).unwrap();
        Self { runtime, context }
    }

    /// 绑定核心API到JavaScript全局对象
    ///
    /// 当前绑定的API：
    /// - `print(msg: string)` - 日志输出函数
    ///
    /// # 注意
    ///
    /// 可以在此方法中添加更多API绑定（如实体生成等）。
    pub fn bind_core_api(&self) {
        self.context.with(|ctx| {
            let global = ctx.globals();

            // Basic Logging
            global
                .set(
                    "print",
                    Function::new(ctx.clone(), |msg: String| {
                        tracing::info!(target: "scripting", "[JS]: {}", msg);
                    }),
                )
                .unwrap();

            // We can add more bindings here later (Entity spawning, etc.)
        });
    }

    /// 执行JavaScript代码
    ///
    /// # 参数
    ///
    /// * `code` - 要执行的JavaScript代码字符串
    ///
    /// # 错误
    ///
    /// 如果代码执行失败，错误会被记录到日志中，但不会panic。
    pub fn execute(&self, code: &str) {
        self.context.with(|ctx| {
            if let Err(e) = ctx.eval::<(), _>(code) {
                tracing::error!(target: "scripting", "Script Error: {:?}", e);
            }
        });
    }
}
