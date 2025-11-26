pub mod engine;

pub use engine::*;

use bevy_ecs::prelude::*;
use rquickjs::{Context, Runtime};

pub struct ScriptingRuntime {
    pub runtime: Runtime,
    pub context: Context,
}

impl ScriptingRuntime {
    pub fn new() -> Self {
        let runtime = Runtime::new().unwrap();
        let context = Context::full(&runtime).unwrap();
        
        Self {
            runtime,
            context,
        }
    }

    pub fn run_script(&self, code: &str) -> Result<(), String> {
        self.context.with(|ctx| {
            ctx.eval::<(), _>(code).map_err(|e| e.to_string())
        })
    }
}

pub fn scripting_system() {}
