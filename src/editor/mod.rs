use bevy_ecs::prelude::*;
use egui_wgpu::Renderer;

pub mod animation_editor;
pub mod asset_browser;
pub mod build_tool;
pub mod config;
pub mod console;
pub mod curve_editor;
pub mod entity_creator;
pub mod hierarchy;
pub mod inspector;
pub mod keyframe_editor;
pub mod material_editor;
pub mod package_deploy;
pub mod particle_editor;
pub mod performance_monitor;
pub mod performance_panel;
pub mod platform_builder;
pub mod project_settings;
pub mod scene_editor;
pub mod scene_editor_enhanced;
pub mod shortcuts;
pub mod terrain_editor;
pub mod transform_gizmo;
pub mod undo_redo;

pub use config::{EditorConfig, EditorConfigManager, EditorTheme};
use egui_winit::State;
pub use hierarchy::HierarchyView;
pub use inspector::Inspector;
pub use shortcuts::{Modifiers, ShortcutAction, ShortcutManager};
pub use undo_redo::{
    Command, CommandError, CommandManager, CompositeCommand, PropertyChangeCommand,
};
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
        let state = State::new(
            context.clone(),
            viewport_id,
            &window,
            Some(window.scale_factor() as f32),
            None,
        );
        let renderer = Renderer::new(device, format, None, 1);
        Self {
            context,
            state,
            renderer,
        }
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
        self.state
            .handle_platform_output(window, output.platform_output);
        let shapes = self
            .context
            .tessellate(output.shapes, output.pixels_per_point);
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
            let fps = if time.delta_seconds > 0.0 {
                1.0 / time.delta_seconds
            } else {
                0.0
            };
            ui.label(format!(
                "Delta: {:.4} s | FPS: {:.1}",
                time.delta_seconds, fps
            ));
        }
        if let Some(stats) = world.get_resource::<crate::core::RenderStats>() {
            if let Some(dt) = stats.gpu_pass_ms {
                ui.label(format!("GPU total: {:.3}ms", dt));
            }
            if let Some(u) = stats.upload_ms {
                let rt = egui::RichText::new(format!("Upload: {:.3}ms", u)).color(if u > 2.0 {
                    egui::Color32::RED
                } else {
                    egui::Color32::WHITE
                });
                ui.label(rt);
            }
            if let Some(m) = stats.main_ms {
                let rt = egui::RichText::new(format!("Main: {:.3}ms", m)).color(if m > 16.7 {
                    egui::Color32::RED
                } else {
                    egui::Color32::WHITE
                });
                ui.label(rt);
            }
            if let Some(u) = stats.ui_ms {
                let rt = egui::RichText::new(format!("UI: {:.3}ms", u)).color(if u > 4.0 {
                    egui::Color32::RED
                } else {
                    egui::Color32::WHITE
                });
                ui.label(rt);
            }
            ui.label(format!(
                "Draw calls: {} | Instances: {} | Passes: {}",
                stats.draw_calls, stats.instances, stats.passes
            ));
            ui.label(format!(
                "Batches: {} | Batch Instances: {}",
                stats.batch_total, stats.batch_instances
            ));
            ui.label(format!(
                "Saved Draw Calls: {}",
                stats.batch_saved_draw_calls
            ));
            ui.label(format!(
                "Small Batch Draw Calls: {}",
                stats.batch_small_draw_calls
            ));
            ui.label(format!(
                "Visible Batches After Culling: {}",
                stats.batch_visible_batches
            ));

            // 显示视锥剔除统计
            if stats.total_objects > 0 {
                let cull_pct =
                    (stats.culled_objects as f32 / stats.total_objects as f32 * 100.0) as u32;
                let cull_text = format!(
                    "Culled: {}/{} ({}%)",
                    stats.culled_objects, stats.total_objects, cull_pct
                );
                let cull_color = if cull_pct > 50 {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::WHITE
                };
                ui.label(egui::RichText::new(cull_text).color(cull_color));
            }

            if let Some(o) = stats.offscreen_ms {
                let rt = egui::RichText::new(format!("Offscreen: {:.3}ms", o)).color(if o > 8.0 {
                    egui::Color32::RED
                } else {
                    egui::Color32::WHITE
                });
                ui.label(rt);
            }
            ui.label(format!(
                "Alerts U:{} M:{} UI:{} O:{}",
                stats.alerts_upload, stats.alerts_main, stats.alerts_ui, stats.alerts_offscreen
            ));
        }
        if let Some(am) = world.get_resource::<crate::core::AssetMetrics>() {
            if let Some(ms) = am.last_latency_ms {
                let rt = egui::RichText::new(format!("Last asset latency: {:.1}ms", ms)).color(
                    if ms > 200.0 {
                        egui::Color32::RED
                    } else {
                        egui::Color32::WHITE
                    },
                );
                ui.label(rt);
            }
            ui.label(format!(
                "Textures: {} | Atlases: {}",
                am.textures_loaded, am.atlases_loaded
            ));
        }
        if let Some(le) = world.get_resource::<crate::core::LogEvents>() {
            ui.separator();
            ui.label("Logs:");
            for s in le.entries.iter() {
                ui.label(s);
            }
        }
    });

    egui::Window::new("Material Editor").show(ctx, |ui| {
        if let Some(reg) = world.get_resource::<crate::resources::manager::MaterialRegistry>() {
            ui.label(format!("Materials: {}", reg.materials.len()));
            ui.separator();
            let mut updates: Vec<(u64, crate::render::pbr::PbrMaterial)> = Vec::new();
            for (id, _entry) in reg.materials.iter() {
                ui.collapsing(format!("Material {}", id), |ui| {
                    let mut mat = crate::render::pbr::PbrMaterial::default();
                    let mut base = [mat.base_color.x, mat.base_color.y, mat.base_color.z, mat.base_color.w];
                    ui.color_edit_button_rgba_unmultiplied(&mut base);
                    mat.base_color = glam::Vec4::new(base[0], base[1], base[2], base[3]);
                    ui.add(egui::Slider::new(&mut mat.metallic, 0.0..=1.0).text("Metallic"));
                    ui.add(egui::Slider::new(&mut mat.roughness, 0.04..=1.0).text("Roughness"));
                    ui.add(egui::Slider::new(&mut mat.normal_scale, 0.0..=4.0).text("Normal Scale"));
                    ui.add(egui::Slider::new(&mut mat.clearcoat, 0.0..=1.0).text("Clearcoat"));
                    ui.add(egui::Slider::new(&mut mat.clearcoat_roughness, 0.04..=1.0).text("Clearcoat Roughness"));
                    ui.add(egui::Slider::new(&mut mat.anisotropy, 0.0..=1.0).text("Anisotropy"));
                    let mut adir = mat.anisotropy_direction;
                    ui.horizontal(|ui| {
                        ui.add(egui::Slider::new(&mut adir[0], -1.0..=1.0).text("Aniso Dir X"));
                        ui.add(egui::Slider::new(&mut adir[1], -1.0..=1.0).text("Aniso Dir Y"));
                    });
                    mat.anisotropy_direction = adir;
                    if ui.button("Apply").clicked() {
                        updates.push((*id, mat));
                    }
                });
            }
            if !updates.is_empty() {
                let mut pend = world.get_resource_or_insert_with::<crate::resources::manager::MaterialPendingUpdates>(Default::default);
                for (id, m) in updates {
                    pend.push(id, m);
                }
            }
        } else {
            ui.label("MaterialRegistry not found");
        }
    });
}
