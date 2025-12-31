//! 调试UI核心结构
//!
//! 提供调试UI的主要接口和状态管理。

use crate::debug::{
    panels::{ComponentPanel, ConsolePanel, EntityPanel, PerformancePanel, ResourcePanel},
    DebugConfig,
};
use bevy_ecs::prelude::*;
use std::time::Instant;

/// 调试UI主结构
///
/// 管理所有调试面板的显示和交互。
pub struct DebugUI {
    /// 配置
    config: DebugConfig,
    /// 实体面板
    entity_panel: EntityPanel,
    /// 组件面板
    component_panel: ComponentPanel,
    /// 性能面板
    performance_panel: PerformancePanel,
    /// 控制台面板
    console_panel: ConsolePanel,
    /// 资源面板
    resource_panel: ResourcePanel,
    /// UI状态
    ui_state: DebugUIState,
    /// 创建时间
    creation_time: Instant,
}

/// 调试UI内部状态
#[derive(Debug)]
struct DebugUIState {
    /// 是否显示菜单栏
    show_menu: bool,
    /// 当前帧时间
    frame_time: f32,
    /// 帧计数器
    frame_count: u64,
    /// 上次更新时间
    last_update: Instant,
}

impl DebugUI {
    /// 创建新的调试UI实例
    ///
    /// # Examples
    ///
    /// ```rust
    /// use game_engine::debug::DebugUI;
    ///
    /// let debug_ui = DebugUI::new();
    /// ```
    pub fn new() -> Self {
        Self::with_config(DebugConfig::default())
    }

    /// 使用自定义配置创建调试UI
    ///
    /// # Arguments
    ///
    /// * `config` - 调试UI配置
    pub fn with_config(config: DebugConfig) -> Self {
        let performance_panel = PerformancePanel::with_history_size(config.performance_history_size);
        let console_panel = ConsolePanel::with_max_lines(config.console_max_lines);

        Self {
            config,
            entity_panel: EntityPanel::new(),
            component_panel: ComponentPanel::new(),
            performance_panel,
            console_panel,
            resource_panel: ResourcePanel::new(),
            ui_state: DebugUIState {
                show_menu: true,
                frame_time: 0.0,
                frame_count: 0,
                last_update: Instant::now(),
            },
            creation_time: Instant::now(),
        }
    }

    /// 渲染调试UI
    ///
    /// # Arguments
    ///
    /// * `ctx` - egui上下文
    /// * `world` - ECS世界引用
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// debug_ui.render(&egui_ctx, &world);
    /// ```
    pub fn render(&mut self, ctx: &egui::Context, world: &World) {
        if !self.config.enabled {
            return;
        }

        // 更新帧时间
        self.update_frame_time();

        // 渲染菜单栏
        if self.ui_state.show_menu {
            self.render_menu_bar(ctx);
        }

        // 渲染各个面板
        if self.config.show_entities {
            self.entity_panel.show(ctx, world, &mut self.component_panel);
        }

        if self.config.show_components {
            self.component_panel.show(ctx, world);
        }

        if self.config.show_performance {
            self.performance_panel.show(ctx, &self.ui_state);
        }

        if self.config.show_console {
            self.console_panel.show(ctx);
        }

        if self.config.show_resources {
            self.resource_panel.show(ctx, world);
        }

        // 更新帧计数
        self.ui_state.frame_count += 1;
    }

    /// 切换面板可见性
    ///
    /// # Arguments
    ///
    /// * `panel` - 面板名称
    pub fn toggle_panel(&mut self, panel: &str) {
        match panel {
            "entities" => self.config.show_entities = !self.config.show_entities,
            "components" => self.config.show_components = !self.config.show_components,
            "performance" => self.config.show_performance = !self.config.show_performance,
            "console" => self.config.show_console = !self.config.show_console,
            "resources" => self.config.show_resources = !self.config.show_resources,
            _ => {}
        }
    }

    /// 添加日志消息到控制台
    ///
    /// # Arguments
    ///
    /// * `message` - 日志消息
    pub fn log(&mut self, message: String) {
        self.console_panel.add_log(message);
    }

    /// 添加错误消息到控制台
    ///
    /// # Arguments
    ///
    /// * `error` - 错误消息
    pub fn log_error(&mut self, error: String) {
        self.console_panel.add_error(error);
    }

    /// 获取性能面板引用
    pub fn performance_panel(&mut self) -> &mut PerformancePanel {
        &mut self.performance_panel
    }

    /// 获取控制台面板引用
    pub fn console_panel(&mut self) -> &mut ConsolePanel {
        &mut self.console_panel
    }

    /// 更新帧时间
    fn update_frame_time(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.ui_state.last_update);
        self.ui_state.frame_time = delta.as_secs_f32();
        self.ui_state.last_update = now;

        // 更新性能面板
        self.performance_panel.update_metrics(
            self.ui_state.frame_time,
            self.ui_state.frame_count,
        );
    }

    /// 渲染菜单栏
    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("debug_menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Debug", |ui| {
                    if ui.button("Entities").clicked() {
                        self.config.show_entities = !self.config.show_entities;
                        ui.close_menu();
                    }
                    if ui.button("Components").clicked() {
                        self.config.show_components = !self.config.show_components;
                        ui.close_menu();
                    }
                    if ui.button("Performance").clicked() {
                        self.config.show_performance = !self.config.show_performance;
                        ui.close_menu();
                    }
                    if ui.button("Console").clicked() {
                        self.config.show_console = !self.config.show_console;
                        ui.close_menu();
                    }
                    if ui.button("Resources").clicked() {
                        self.config.show_resources = !self.config.show_resources;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Close All").clicked() {
                        self.config.show_entities = false;
                        self.config.show_components = false;
                        self.config.show_performance = false;
                        self.config.show_console = false;
                        self.config.show_resources = false;
                        ui.close_menu();
                    }
                    if ui.button("Open All").clicked() {
                        self.config.show_entities = true;
                        self.config.show_components = true;
                        self.config.show_performance = true;
                        self.config.show_console = true;
                        self.config.show_resources = true;
                        ui.close_menu();
                    }
                });

                ui.separator();

                // 显示FPS
                if let Some(fps) = self.performance_panel.current_fps() {
                    ui.label(format!("FPS: {:.1}", fps));
                }

                // 显示运行时间
                let uptime = self.creation_time.elapsed().as_secs();
                ui.label(format!("Uptime: {}s", uptime));
            });
        });
    }
}

impl Default for DebugUI {
    fn default() -> Self {
        Self::new()
    }
}
