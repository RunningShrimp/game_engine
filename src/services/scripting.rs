use rquickjs::{Context, Runtime, Function};

pub struct ScriptingService {
    runtime: Runtime,
    context: Context,
}

impl Default for ScriptingService {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptingService {
    pub fn new() -> Self {
        let runtime = Runtime::new().unwrap();
        let context = Context::full(&runtime).unwrap();
        Self { runtime, context }
    }

    pub fn bind_core_api(&self) {
        self.context.with(|ctx| {
            let global = ctx.globals();
            
            // Basic Logging
            global.set("print", Function::new(ctx.clone(), |msg: String| {
                println!("[JS]: {}", msg);
            })).unwrap();

            // We can add more bindings here later (Entity spawning, etc.)
        });
    }

    pub fn execute(&self, code: &str) {
        self.context.with(|ctx| {
            if let Err(e) = ctx.eval::<(), _>(code) {
                println!("Script Error: {:?}", e);
            }
        });
    }
}
