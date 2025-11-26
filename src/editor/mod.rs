use bevy_ecs::prelude::*;
use egui_wgpu::Renderer;

pub mod hierarchy;
pub mod inspector;
pub mod material_editor;
pub mod animation_editor;
pub mod performance_panel;
pub mod keyframe_editor;
pub mod asset_browser;

pub use hierarchy::HierarchyView;
pub use inspector::Inspector;
use egui_winit::State;
use winit::event::WindowEvent;
use winit::window::Window;

pub struct EditorPlugin;

// Note: EditorContext is not a Resource because egui-winit State contains non-Send types
pub struct EditorContext {
    pub context: egui::Context,
    pub state: State,
    pub renderer: Renderer,
}

impl EditorContext {
    pub fn new(window: &Window, device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let context = egui::Context::default();
        let viewport_id = context.viewport_id();
        let state = State::new(context.clone(), viewport_id, &window, Some(window.scale_factor() as f32), None);
        let renderer = Renderer::new(device, format, None, 1);
        Self { context, state, renderer }
    }

    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let response = self.state.on_window_event(window, event);
        response.consumed
    }

    pub fn begin_frame(&mut self, window: &Window) {
        let raw_input = self.state.take_egui_input(window);
        self.context.begin_frame(raw_input);
    }

    pub fn end_frame(&mut self, window: &Window) -> (Vec<egui::ClippedPrimitive>, &mut Renderer) {
        let output = self.context.end_frame();
        self.state.handle_platform_output(window, output.platform_output);
        let shapes = self.context.tessellate(output.shapes, output.pixels_per_point);
        (shapes, &mut self.renderer)
    }
}

pub fn editor_ui_system(world: &mut World) {
    // This system will be responsible for drawing the editor UI
    // We need to access the EditorContext resource
    // But since we need mutable access to World to inspect entities, we can't just use a System param for World
    // So we might need to run this as an exclusive system or use UnsafeWorldCell if we were in Bevy proper.
    // Here we can just use a normal system that queries everything?
    // Or better, we pass the context to a function that draws.
    
    // For now, let's just have a placeholder system.
    // The actual drawing happens in the render loop in core/mod.rs
}

pub fn inspect_world_ui(ctx: &egui::Context, world: &mut World) {
    egui::Window::new("World Inspector").show(ctx, |ui| {
        ui.label(format!("Entities: {}", world.entities().len()));
        if let Some(time) = world.get_resource::<crate::ecs::Time>() {
            let fps = if time.delta_seconds > 0.0 { 1.0 / time.delta_seconds } else { 0.0 };
            ui.label(format!("Delta: {:.4} s | FPS: {:.1}", time.delta_seconds, fps));
        }
        if let Some(stats) = world.get_resource::<crate::core::RenderStats>() {
            if let Some(dt) = stats.gpu_pass_ms { ui.label(format!("GPU total: {:.3}ms", dt)); }
            if let Some(u) = stats.upload_ms {
                let rt = egui::RichText::new(format!("Upload: {:.3}ms", u)).color(if u > 2.0 { egui::Color32::RED } else { egui::Color32::WHITE });
                ui.label(rt);
            }
            if let Some(m) = stats.main_ms {
                let rt = egui::RichText::new(format!("Main: {:.3}ms", m)).color(if m > 16.7 { egui::Color32::RED } else { egui::Color32::WHITE });
                ui.label(rt);
            }
            if let Some(u) = stats.ui_ms {
                let rt = egui::RichText::new(format!("UI: {:.3}ms", u)).color(if u > 4.0 { egui::Color32::RED } else { egui::Color32::WHITE });
                ui.label(rt);
            }
            ui.label(format!("Draw calls: {} | Instances: {} | Passes: {}", stats.draw_calls, stats.instances, stats.passes));
            if let Some(o) = stats.offscreen_ms {
                let rt = egui::RichText::new(format!("Offscreen: {:.3}ms", o)).color(if o > 8.0 { egui::Color32::RED } else { egui::Color32::WHITE });
                ui.label(rt);
            }
            ui.label(format!("Alerts U:{} M:{} UI:{} O:{}", stats.alerts_upload, stats.alerts_main, stats.alerts_ui, stats.alerts_offscreen));
        }
        if let Some(am) = world.get_resource::<crate::core::AssetMetrics>() {
            if let Some(ms) = am.last_latency_ms {
                let rt = egui::RichText::new(format!("Last asset latency: {:.1}ms", ms)).color(if ms > 200.0 { egui::Color32::RED } else { egui::Color32::WHITE });
                ui.label(rt);
            }
            ui.label(format!("Textures: {} | Atlases: {}", am.textures_loaded, am.atlases_loaded));
        }
        if let Some(le) = world.get_resource::<crate::core::LogEvents>() {
            ui.separator();
            ui.label("Logs:");
            for s in le.entries.iter() {
                ui.label(s);
            }
        }
    });
}
