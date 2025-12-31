//! Render Plugin
//!
//! A rendering plugin that demonstrates custom renderers and post-processing.

use game_engine::plugins::api::*;
use game_engine::ecs::{Component, ComponentRegistry, System, SystemContext, SystemScheduler};
use game_engine::error::Error;
use std::any::Any;

/// Mesh component
#[derive(Component, Debug, Clone)]
pub struct Mesh {
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub bounds: (f32, f32, f32, f32, f32, f32), // min_x, max_x, min_y, max_y, min_z, max_z
}

impl Mesh {
    pub fn new(vertex_count: usize, triangle_count: usize) -> Self {
        Self {
            vertex_count,
            triangle_count,
            bounds: (0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        }
    }
}

/// Material component
#[derive(Component, Debug, Clone)]
pub struct Material {
    pub shader: String,
    pub albedo: (f32, f32, f32),  // RGB color
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: (f32, f32, f32),
}

impl Material {
    pub fn new(shader: String) -> Self {
        Self {
            shader,
            albedo: (1.0, 1.0, 1.0),
            metallic: 0.0,
            roughness: 0.5,
            emissive: (0.0, 0.0, 0.0),
        }
    }
}

/// Light component
#[derive(Component, Debug, Clone)]
pub struct Light {
    pub light_type: LightType,
    pub color: (f32, f32, f32),
    pub intensity: f32,
    pub range: f32,
}

/// Light type
#[derive(Debug, Clone, Copy)]
pub enum LightType {
    Directional,
    Point,
    Spot,
}

/// Camera component
#[derive(Component, Debug, Clone)]
pub struct Camera {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub clear_color: (f32, f32, f32, f32),
}

impl Camera {
    pub fn new(fov: f32, near: f32, far: f32) -> Self {
        Self {
            fov,
            near,
            far,
            clear_color: (0.1, 0.1, 0.1, 1.0),
        }
    }
}

/// Post-processing effect component
#[derive(Component, Debug, Clone)]
pub struct PostProcessing {
    pub bloom_enabled: bool,
    pub bloom_threshold: f32,
    pub bloom_intensity: f32,
    pub tone_mapping: ToneMappingType,
    pub vignette_intensity: f32,
}

/// Tone mapping type
#[derive(Debug, Clone, Copy)]
pub enum ToneMappingType {
    None,
    ACES,
    Reinhard,
    Unreal,
}

/// Rendering system
pub struct RenderSystem {
    frame_count: usize,
    draw_calls: usize,
}

impl RenderSystem {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            draw_calls: 0,
        }
    }
}

impl Default for RenderSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for RenderSystem {
    fn name(&self) -> &str {
        "RenderSystem"
    }

    fn run(&mut self, _context: &SystemContext, _delta: f32) {
        self.frame_count += 1;
        self.draw_calls = 0;

        // In a real implementation, this would:
        // 1. Cull objects not in view
        // 2. Sort by material/depth
        // 3. Submit draw calls
        // 4. Apply post-processing

        self.draw_calls = 42; // Simulated draw calls

        if self.frame_count % 60 == 0 {
            println!("[RenderSystem] Frame: {}, Draw calls: {}", self.frame_count, self.draw_calls);
        }
    }
}

/// Render Plugin
pub struct RenderPlugin {
    metadata: PluginMetadata,
    renderer_type: String,
}

impl RenderPlugin {
    /// Create a new Render plugin
    pub fn new() -> Self {
        let metadata = PluginMetadata::new("render".to_string(), "0.1.0".to_string())
            .with_author("Game Engine Team".to_string())
            .with_description("Rendering and post-processing plugin".to_string())
            .with_website("https://example.com".to_string())
            .with_license("MIT".to_string());

        Self {
            metadata,
            renderer_type: "ForwardRenderer".to_string(),
        }
    }

    /// Set renderer type
    pub fn with_renderer(mut self, renderer: String) -> Self {
        self.renderer_type = renderer;
        self
    }
}

impl Default for RenderPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for RenderPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&mut self, context: &PluginContext) -> Result<(), Error> {
        println!("[RenderPlugin] Loaded successfully!");
        println!("[RenderPlugin] Renderer type: {}", self.renderer_type);
        println!("[RenderPlugin] Engine version: {}", context.engine_version());
        println!("[RenderPlugin] Hot-reload: {}", context.is_hot_reload());

        Ok(())
    }

    fn on_unload(&mut self, _context: &PluginContext) -> Result<(), Error> {
        println!("[RenderPlugin] Unloaded successfully!");
        Ok(())
    }

    fn on_update(&mut self, _context: &PluginContext, _delta: f32) {
        // Rendering is handled by the render system
    }

    fn on_event(&mut self, _context: &PluginContext, event: &PluginEvent) {
        match event {
            PluginEvent::SceneLoaded(name) => {
                println!("[RenderPlugin] Scene loaded: {}, preparing renderer...", name);
            }
            PluginEvent::SceneUnloading(name) => {
                println!("[RenderPlugin] Scene unloading: {}, cleaning up...", name);
            }
            _ => {}
        }
    }

    fn register_components(&self, registry: &mut ComponentRegistry) {
        println!("[RenderPlugin] Registering render components...");

        registry.register::<Mesh>("Mesh");
        registry.register::<Material>("Material");
        registry.register::<Light>("Light");
        registry.register::<Camera>("Camera");
        registry.register::<PostProcessing>("PostProcessing");

        println!("[RenderPlugin] Registered 5 component types");
    }

    fn register_systems(&self, scheduler: &mut SystemScheduler) {
        println!("[RenderPlugin] Registering render systems...");

        let render_system = RenderSystem::new();
        scheduler.register_system(Box::new(render_system));

        println!("[RenderPlugin] Registered render system");
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Plugin creation function (exported for dynamic loading)
#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(RenderPlugin::new())
}
