//! 调试器面板
//!
//! 提供基于egui的调试器UI界面，包括断点管理、调用栈查看、变量监视等。

use crate::debug::breakpoints::{BreakpointInfo, BreakpointManager, BreakpointType};
use crate::debug::dap::server::{DapServer, StackFrame, Variable};
use crate::debug::panels::Panel;
use crate::debug::variables::{VariableMonitor, VariableReference};
use bevy_ecs::prelude::*;
use egui::{ScrollArea, Ui};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 调试器面板状态
#[derive(Default)]
pub struct DebuggerPanel {
    /// 断点管理器
    breakpoint_manager: Option<Arc<BreakpointManager>>,
    /// 变量监视器
    variable_monitor: Option<Arc<VariableMonitor>>,
    /// DAP服务器
    dap_server: Option<Arc<DapServer>>,

    /// UI状态
    show_breakpoints: bool,
    show_call_stack: bool,
    show_variables: bool,
    show_watch: bool,

    /// 当前选中的栈帧
    selected_frame: Option<usize>,

    /// 监视表达式
    watch_expressions: Vec<String>,
    new_watch_expression: String,

    /// 断点过滤
    breakpoint_filter: String,

    /// 调试器状态
    debugger_state: DebuggerState,
}

/// 调试器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebuggerState {
    /// 未连接
    Disconnected,
    /// 运行中
    Running,
    /// 暂停
    Paused,
    /// 步进
    Stepping,
}

impl Default for DebuggerState {
    fn default() -> Self {
        Self::Disconnected
    }
}

impl Panel for DebuggerPanel {
    fn show(&mut self, ctx: &egui::Context, _world: &World) {
        egui::Window::new("Debugger").default_size([900.0, 700.0]).resizable(true).show(
            ctx,
            |ui| {
                self.show_ui(ui);
            },
        );
    }
}

impl DebuggerPanel {
    /// 创建新的调试器面板
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置断点管理器
    pub fn with_breakpoint_manager(mut self, manager: Arc<BreakpointManager>) -> Self {
        self.breakpoint_manager = Some(manager);
        self
    }

    /// 设置变量监视器
    pub fn with_variable_monitor(mut self, monitor: Arc<VariableMonitor>) -> Self {
        self.variable_monitor = Some(monitor);
        self
    }

    /// 设置DAP服务器
    pub fn with_dap_server(mut self, server: Arc<DapServer>) -> Self {
        self.dap_server = Some(server);
        self
    }

    /// 显示UI
    fn show_ui(&mut self, ui: &mut Ui) {
        // 顶部工具栏
        self.show_toolbar(ui);
        ui.separator();

        // 主要内容区域
        egui::TopBottomPanel::top("panels").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.checkbox(&mut self.show_breakpoints, "🔴 Breakpoints");
                    ui.checkbox(&mut self.show_call_stack, "📚 Call Stack");
                    ui.checkbox(&mut self.show_variables, "📊 Variables");
                    ui.checkbox(&mut self.show_watch, "👁️ Watch");
                });
            });
        });

        ui.separator();

        // 内容区域
        egui::Splitter::horizontal()
            .ratio(0.3)
            .show(ui, |ui| {
                // 左侧面板
                self.show_left_panel(ui);
            })
            .show(ui, |ui| {
                // 右侧面板
                self.show_right_panel(ui);
            });
    }

    /// 显示工具栏
    fn show_toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // 状态指示器
            let status_text = match self.debugger_state {
                DebuggerState::Disconnected => "⚫ Disconnected",
                DebuggerState::Running => "🟢 Running",
                DebuggerState::Paused => "🟡 Paused",
                DebuggerState::Stepping => "🔵 Stepping",
            };
            ui.label(status_text);

            ui.separator();

            // 控制按钮
            let can_pause = matches!(
                self.debugger_state,
                DebuggerState::Running | DebuggerState::Stepping
            );

            let can_continue = matches!(
                self.debugger_state,
                DebuggerState::Paused | DebuggerState::Stepping
            );

            ui.add_enabled_ui(can_pause, |ui| {
                if ui.button("⏸ Pause").clicked() {
                    self.pause();
                }
            });

            ui.add_enabled_ui(can_continue, |ui| {
                if ui.button("▶ Continue").clicked() {
                    self.continue_execution();
                }
            });

            ui.add_enabled_ui(can_continue, |ui| {
                if ui.button("⏭ Step Over").clicked() {
                    self.step_over();
                }
            });

            ui.add_enabled_ui(can_continue, |ui| {
                if ui.button("⏩ Step Into").clicked() {
                    self.step_into();
                }
            });

            ui.add_enabled_ui(can_continue, |ui| {
                if ui.button("⏪ Step Out").clicked() {
                    self.step_out();
                }
            });

            ui.separator();

            if ui.button("🔄 Refresh").clicked() {
                self.refresh();
            }
        });
    }

    /// 显示左侧面板（断点列表）
    fn show_left_panel(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading("Breakpoints");
            ui.separator();

            // 搜索过滤
            ui.horizontal(|ui| {
                ui.label("Filter:");
                ui.text_edit_singleline(&mut self.breakpoint_filter);
            });

            // 断点列表
            ScrollArea::vertical().show(ui, |ui| {
                if let Some(manager) = &self.breakpoint_manager {
                    let breakpoints = manager.get_all_breakpoints();

                    for (_, bp) in breakpoints.iter() {
                        // 应用过滤
                        if !self.breakpoint_filter.is_empty() {
                            let filter_lower = self.breakpoint_filter.to_lowercase();
                            let source_lower = bp.source_path.to_lowercase();
                            if !source_lower.contains(&filter_lower) {
                                continue;
                            }
                        }

                        self.show_breakpoint_item(ui, bp);
                    }
                } else {
                    ui.label("No breakpoint manager");
                }
            });

            ui.separator();

            // 添加断点按钮
            ui.horizontal(|ui| {
                if ui.button("➕ Add Breakpoint").clicked() {
                    // TODO: 显示添加断点对话框
                }

                if ui.button("🗑 Remove All").clicked() {
                    if let Some(manager) = &self.breakpoint_manager {
                        manager.clear_all();
                    }
                }
            });
        });
    }

    /// 显示断点项
    fn show_breakpoint_item(&mut self, ui: &mut Ui, bp: &BreakpointInfo) {
        let mut is_enabled = bp.is_enabled();
        let original_enabled = is_enabled;

        ui.horizontal(|ui| {
            // 启用/禁用复选框
            if ui.checkbox(&mut is_enabled, "").changed() {
                if is_enabled != original_enabled {
                    if let Some(manager) = &self.breakpoint_manager {
                        if is_enabled {
                            manager.enable_breakpoint(bp.id);
                        } else {
                            manager.disable_breakpoint(bp.id);
                        }
                    }
                }
            }

            // 断点类型图标
            let type_icon = match bp.bp_type {
                BreakpointType::Line => "📍",
                BreakpointType::Function => "🔧",
                BreakpointType::Conditional => "❓",
            };
            ui.label(format!("{}", type_icon));

            // 文件名和行号
            let file_name = bp.source_path.split('/').last().unwrap_or(&bp.source_path);
            ui.label(format!("{}:{}", file_name, bp.line));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 删除按钮
                if ui.button("🗑").clicked() {
                    if let Some(manager) = &self.breakpoint_manager {
                        manager.remove_breakpoint(bp.id);
                    }
                }
            });
        });

        // 显示条件（如果有）
        if let Some(condition) = &bp.condition {
            ui.label(format!("Condition: {}", condition));
        }
    }

    /// 显示右侧面板（调用栈和变量）
    fn show_right_panel(&mut self, ui: &mut Ui) {
        egui::Splitter::vertical()
            .ratio(0.5)
            .show(ui, |ui| {
                // 上半部分：调用栈
                self.show_call_stack_panel(ui);
            })
            .show(ui, |ui| {
                // 下半部分：变量监视
                self.show_variables_panel(ui);
            });
    }

    /// 显示调用栈面板
    fn show_call_stack_panel(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading("Call Stack");
            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                if let Some(server) = &self.dap_server {
                    let stack_frames = server.get_stack_frames();

                    for (i, frame) in stack_frames.iter().enumerate() {
                        let is_selected = self.selected_frame == Some(i);

                        if ui.selectable_label(is_selected, &frame.name).clicked() {
                            self.selected_frame = Some(i);
                        }

                        ui.label(format!("  at {}:{}", frame.source.name, frame.line));
                    }
                } else {
                    ui.label("No debug session");
                }
            });
        });
    }

    /// 显示变量面板
    fn show_variables_panel(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading("Variables");
            ui.separator();

            // 局部变量
            ui.horizontal(|ui| {
                ui.label("📦 Local:");
            });

            ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                if let Some(monitor) = &self.variable_monitor {
                    if let Some(frame_id) = self.selected_frame {
                        // 显示选中栈帧的变量
                        let scope_vars = monitor.get_variables_for_scope(frame_id as i64);

                        for var in scope_vars {
                            self.show_variable(ui, &var, 0);
                        }
                    } else {
                        ui.label("Select a stack frame to view variables");
                    }
                } else {
                    ui.label("No variable monitor");
                }
            });

            ui.separator();

            // 监视表达式
            ui.heading("Watch");
            ui.horizontal(|ui| {
                ui.label("Add watch:");
                ui.text_edit_singleline(&mut self.new_watch_expression);

                if ui.button("Add").clicked() && !self.new_watch_expression.is_empty() {
                    self.watch_expressions.push(self.new_watch_expression.clone());
                    self.new_watch_expression.clear();
                }
            });

            ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                for (i, expr) in self.watch_expressions.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", expr));

                        // 评估表达式
                        if let Some(monitor) = &self.variable_monitor {
                            if let Ok(value) = monitor.evaluate_expression(expr) {
                                ui.label(value);
                            } else {
                                ui.label(egui::RichText::new("Error").color(egui::Color32::RED));
                            }
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("×").clicked() {
                                self.watch_expressions.remove(i);
                            }
                        });
                    });
                }
            });
        });
    }

    /// 显示变量（递归显示子变量）
    fn show_variable(&mut self, ui: &mut Ui, variable: &Variable, depth: usize) {
        let indent = "  ".repeat(depth);

        ui.horizontal(|ui| {
            ui.label(format!("{}{}", indent, variable.name));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!(": {}", variable.value));

                // 如果有子变量，显示展开按钮
                if variable.variables_reference.is_some() {
                    if ui.button("▶").clicked() {
                        // TODO: 展开变量
                    }
                }
            });
        });

        if let Some(type_name) = &variable.type_name {
            ui.label(format!("{}  ({})", indent, type_name));
        }
    }

    /// 暂停执行
    fn pause(&mut self) {
        if let Some(server) = &self.dap_server {
            // TODO: 实现暂停
            log::info!("Debugger: Pause");
        }
    }

    /// 继续执行
    fn continue_execution(&mut self) {
        if let Some(server) = &self.dap_server {
            // TODO: 实现继续
            log::info!("Debugger: Continue");
        }
    }

    /// 单步跳过
    fn step_over(&mut self) {
        if let Some(server) = &self.dap_server {
            // TODO: 实现单步跳过
            log::info!("Debugger: Step Over");
        }
    }

    /// 单步进入
    fn step_into(&mut self) {
        if let Some(server) = &self.dap_server {
            // TODO: 实现单步进入
            log::info!("Debugger: Step Into");
        }
    }

    /// 单步跳出
    fn step_out(&mut self) {
        if let Some(server) = &self.dap_server {
            // TODO: 实现单步跳出
            log::info!("Debugger: Step Out");
        }
    }

    /// 刷新
    fn refresh(&mut self) {
        log::info!("Debugger: Refresh");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugger_panel_creation() {
        let panel = DebuggerPanel::new();
        assert_eq!(panel.debugger_state, DebuggerState::Disconnected);
    }

    #[test]
    fn test_debugger_state_display() {
        assert_eq!(format!("{:?}", DebuggerState::Running), "Running");
    }
}
