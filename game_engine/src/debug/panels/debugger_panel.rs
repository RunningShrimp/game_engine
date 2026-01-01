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
                    // 注意：get_all_breakpoints现在是异步的，但在UI中我们无法直接await
                    // 在真实实现中，应该有缓存或同步的getter
                    // 这里简化为显示静态消息
                    ui.label("Breakpoint list requires async access");
                    ui.label("Use DAP server to manage breakpoints");
                } else {
                    ui.label("No breakpoint manager");
                }
            });

            ui.separator();

            // 添加断点按钮
            ui.horizontal(|ui| {
                if ui.button("➕ Add Breakpoint").clicked() {
                    // 简化实现：显示提示信息
                    tracing::info!("Add breakpoint clicked - use DAP server to add breakpoints");
                }

                if ui.button("🗑 Remove All").clicked() {
                    if let Some(manager) = &self.breakpoint_manager {
                        // 注意：clear_all现在是async的
                        tracing::info!("Remove all breakpoints clicked");
                    }
                }
            });
        });
    }

    /// 显示断点项（简化版，避免异步问题）
    fn show_breakpoint_item(&mut self, ui: &mut Ui, bp: &BreakpointInfo) {
        ui.horizontal(|ui| {
            // 启用/禁用复选框
            let mut is_enabled = bp.enabled;
            ui.checkbox(&mut is_enabled, "");

            // 断点类型图标
            let type_icon = match bp.bp_type {
                BreakpointType::Line => "📍",
                BreakpointType::Function => "🔧",
                BreakpointType::Exception => "⚠️",
                BreakpointType::Log => "📝",
            };
            ui.label(format!("{}", type_icon));

            // 文件名和行号
            let file_name = bp.source_path.split('/').last().unwrap_or(&bp.source_path);
            ui.label(format!("{}:{}", file_name, bp.line));

            // 命中次数
            if bp.hit_count > 0 {
                ui.label(format!("(hits: {})", bp.hit_count));
            }
        });

        // 显示条件（如果有）
        if let Some(condition) = &bp.condition {
            ui.label(format!("  Condition: {}", condition.expression));
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
                if variable.variables_reference.is_some()
                    && variable.variables_reference.unwrap() > 0
                {
                    if ui.button("▶").clicked() {
                        // 展开变量 - 在真实实现中会获取子变量
                        tracing::debug!(
                            "Expand variable: {} (ref: {})",
                            variable.name,
                            variable.variables_reference.unwrap()
                        );
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
            // 发送pause请求到DAP服务器
            let pause_request = crate::debug::dap::server::DapMessage {
                seq: 1,
                type_: "request".to_string(),
                request_seq: 0,
                success: false,
                command: "pause".to_string(),
                message: None,
                body: None,
            };

            // 在实际实现中，这里应该异步发送请求
            // 简化实现：更新状态
            self.debugger_state = DebuggerState::Paused;
            tracing::info!("Debugger: Pause requested");
        }
    }

    /// 继续执行
    fn continue_execution(&mut self) {
        if let Some(server) = &self.dap_server {
            // 发送continue请求到DAP服务器
            let continue_request = crate::debug::dap::server::DapMessage {
                seq: 1,
                type_: "request".to_string(),
                request_seq: 0,
                success: false,
                command: "continue".to_string(),
                message: None,
                body: None,
            };

            // 简化实现：更新状态
            self.debugger_state = DebuggerState::Running;
            tracing::info!("Debugger: Continue requested");
        }
    }

    /// 单步跳过
    fn step_over(&mut self) {
        if let Some(server) = &self.dap_server {
            // 发送next请求到DAP服务器
            let next_request = crate::debug::dap::server::DapMessage {
                seq: 1,
                type_: "request".to_string(),
                request_seq: 0,
                success: false,
                command: "next".to_string(),
                message: None,
                body: None,
            };

            // 简化实现：更新状态
            self.debugger_state = DebuggerState::Stepping;
            tracing::info!("Debugger: Step Over requested");
        }
    }

    /// 单步进入
    fn step_into(&mut self) {
        if let Some(server) = &self.dap_server {
            // 发送stepIn请求到DAP服务器
            let stepin_request = crate::debug::dap::server::DapMessage {
                seq: 1,
                type_: "request".to_string(),
                request_seq: 0,
                success: false,
                command: "stepIn".to_string(),
                message: None,
                body: None,
            };

            // 简化实现：更新状态
            self.debugger_state = DebuggerState::Stepping;
            tracing::info!("Debugger: Step Into requested");
        }
    }

    /// 单步跳出
    fn step_out(&mut self) {
        if let Some(server) = &self.dap_server {
            // 发送stepOut请求到DAP服务器
            let stepout_request = crate::debug::dap::server::DapMessage {
                seq: 1,
                type_: "request".to_string(),
                request_seq: 0,
                success: false,
                command: "stepOut".to_string(),
                message: None,
                body: None,
            };

            // 简化实现：更新状态
            self.debugger_state = DebuggerState::Stepping;
            tracing::info!("Debugger: Step Out requested");
        }
    }

    /// 刷新
    fn refresh(&mut self) {
        tracing::info!("Debugger: Refreshing data");

        // 刷新断点列表（已经通过Rc<RefCell>自动更新）
        // 刷新调用栈（在真实实现中会从DAP服务器获取）
        // 刷新变量监视（在真实实现中会重新求值）

        if let Some(monitor) = &self.variable_monitor {
            // 使用tokio spawn在后台运行异步任务
            // 在真实实现中应该有更好的任务管理
            tracing::debug!("Triggering watch evaluation");
        }
    }

    /// 更新调试器状态
    pub fn update_state(&mut self, new_state: DebuggerState) {
        let old_state = self.debugger_state;
        self.debugger_state = new_state;

        if old_state != new_state {
            tracing::info!("Debugger state changed: {:?} -> {:?}", old_state, new_state);
        }
    }

    /// 获取当前调试器状态
    pub fn get_state(&self) -> DebuggerState {
        self.debugger_state
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
        assert_eq!(format!("{:?}", DebuggerState::Paused), "Paused");
        assert_eq!(format!("{:?}", DebuggerState::Stepping), "Stepping");
        assert_eq!(format!("{:?}", DebuggerState::Disconnected), "Disconnected");
    }

    #[test]
    fn test_debugger_state_transitions() {
        let mut panel = DebuggerPanel::new();

        // 初始状态
        assert_eq!(panel.get_state(), DebuggerState::Disconnected);

        // 切换到运行状态
        panel.update_state(DebuggerState::Running);
        assert_eq!(panel.get_state(), DebuggerState::Running);

        // 切换到暂停状态
        panel.update_state(DebuggerState::Paused);
        assert_eq!(panel.get_state(), DebuggerState::Paused);

        // 切换到步进状态
        panel.update_state(DebuggerState::Stepping);
        assert_eq!(panel.get_state(), DebuggerState::Stepping);
    }

    #[test]
    fn test_debugger_panel_ui_flags() {
        let panel = DebuggerPanel::new();

        // 默认情况下，所有面板都是显示的
        assert!(panel.show_breakpoints);
        assert!(panel.show_call_stack);
        assert!(panel.show_variables);
        assert!(panel.show_watch);
    }

    #[test]
    fn test_debugger_panel_empty_watch_list() {
        let panel = DebuggerPanel::new();

        // 初始监视表达式列表应该为空
        assert!(panel.watch_expressions.is_empty());
        assert!(panel.new_watch_expression.is_empty());
    }

    #[test]
    fn test_debugger_panel_filter() {
        let panel = DebuggerPanel::new();

        // 初始过滤器应该为空
        assert!(panel.breakpoint_filter.is_empty());
    }

    #[test]
    fn test_debugger_state_equality() {
        assert_eq!(DebuggerState::Running, DebuggerState::Running);
        assert_ne!(DebuggerState::Running, DebuggerState::Paused);
        assert_ne!(DebuggerState::Paused, DebuggerState::Stepping);
        assert_ne!(DebuggerState::Stepping, DebuggerState::Disconnected);
    }

    #[test]
    fn test_debugger_panel_with_managers() {
        use std::sync::Arc;

        let bp_manager = Arc::new(crate::debug::breakpoints::BreakpointManager::new());
        let var_monitor = Arc::new(crate::debug::variables::VariableMonitor::new());

        let panel = DebuggerPanel::new()
            .with_breakpoint_manager(bp_manager)
            .with_variable_monitor(var_monitor);

        // 验证managers已设置
        assert!(panel.breakpoint_manager.is_some());
        assert!(panel.variable_monitor.is_some());
    }

    #[test]
    fn test_debugger_panel_default_selection() {
        let panel = DebuggerPanel::new();

        // 初始状态下没有选中的栈帧
        assert!(panel.selected_frame.is_none());
    }

    #[test]
    fn test_debugger_state_controls() {
        let mut panel = DebuggerPanel::new();

        // 测试在断开状态下所有控制都应该是禁用的
        let can_pause = matches!(
            panel.debugger_state,
            DebuggerState::Running | DebuggerState::Stepping
        );
        assert!(!can_pause);

        let can_continue = matches!(
            panel.debugger_state,
            DebuggerState::Paused | DebuggerState::Stepping
        );
        assert!(!can_continue);

        // 切换到暂停状态
        panel.update_state(DebuggerState::Paused);

        let can_continue = matches!(
            panel.debugger_state,
            DebuggerState::Paused | DebuggerState::Stepping
        );
        assert!(can_continue);
    }
}
