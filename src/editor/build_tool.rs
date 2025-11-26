use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};

/// 构建目标平台
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildTarget {
    Windows,
    Linux,
    MacOS,
    Web,
    Android,
    #[allow(non_camel_case_types)]
    iOS,
}

impl BuildTarget {
    pub fn name(&self) -> &'static str {
        match self {
            BuildTarget::Windows => "Windows",
            BuildTarget::Linux => "Linux",
            BuildTarget::MacOS => "macOS",
            BuildTarget::Web => "Web (WASM)",
            BuildTarget::Android => "Android",
            BuildTarget::iOS => "iOS",
        }
    }
    
    pub fn rust_target(&self) -> &'static str {
        match self {
            BuildTarget::Windows => "x86_64-pc-windows-msvc",
            BuildTarget::Linux => "x86_64-unknown-linux-gnu",
            BuildTarget::MacOS => "x86_64-apple-darwin",
            BuildTarget::Web => "wasm32-unknown-unknown",
            BuildTarget::Android => "aarch64-linux-android",
            BuildTarget::iOS => "aarch64-apple-ios",
        }
    }
}

/// 构建配置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildProfile {
    Debug,
    Release,
}

impl BuildProfile {
    pub fn name(&self) -> &'static str {
        match self {
            BuildProfile::Debug => "Debug",
            BuildProfile::Release => "Release",
        }
    }
}

/// 构建选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOptions {
    /// 目标平台
    pub target: BuildTarget,
    /// 构建配置
    pub profile: BuildProfile,
    /// 输出目录
    pub output_dir: PathBuf,
    /// 是否优化大小
    pub optimize_size: bool,
    /// 是否包含调试信息
    pub include_debug_info: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            target: BuildTarget::Linux,
            profile: BuildProfile::Debug,
            output_dir: PathBuf::from("./build"),
            optimize_size: false,
            include_debug_info: true,
        }
    }
}

/// 构建结果
#[derive(Debug, Clone)]
pub enum BuildResult {
    Success { output_path: PathBuf, duration: std::time::Duration },
    Failed { error: String },
}

/// 构建工具
pub struct BuildTool {
    /// 当前构建选项
    pub options: BuildOptions,
    /// 是否正在构建
    pub is_building: bool,
    /// 最后的构建结果
    pub last_result: Option<BuildResult>,
    /// 构建日志
    pub build_log: Vec<String>,
}

impl BuildTool {
    pub fn new() -> Self {
        Self {
            options: BuildOptions::default(),
            is_building: false,
            last_result: None,
            build_log: Vec::new(),
        }
    }
    
    /// 开始构建
    pub fn build(&mut self, project_path: &Path) -> Result<(), String> {
        if self.is_building {
            return Err("Build already in progress".to_string());
        }
        
        self.is_building = true;
        self.build_log.clear();
        self.build_log.push(format!("Starting build for {} ({})...", 
            self.options.target.name(), 
            self.options.profile.name()));
        
        let start_time = std::time::Instant::now();
        
        // 构建cargo命令
        let mut cmd = Command::new("cargo");
        cmd.current_dir(project_path);
        cmd.arg("build");
        
        // 添加目标平台
        cmd.arg("--target");
        cmd.arg(self.options.target.rust_target());
        
        // 添加构建配置
        if matches!(self.options.profile, BuildProfile::Release) {
            cmd.arg("--release");
        }
        
        // 执行构建
        self.build_log.push("Running cargo build...".to_string());
        
        match cmd.output() {
            Ok(output) => {
                let duration = start_time.elapsed();
                
                // 记录输出
                if !output.stdout.is_empty() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    self.build_log.push(stdout.to_string());
                }
                
                if !output.stderr.is_empty() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    self.build_log.push(stderr.to_string());
                }
                
                if output.status.success() {
                    let output_path = self.get_output_path(project_path);
                    self.build_log.push(format!("Build succeeded in {:.2}s", duration.as_secs_f32()));
                    self.build_log.push(format!("Output: {:?}", output_path));
                    
                    self.last_result = Some(BuildResult::Success { output_path, duration });
                    self.is_building = false;
                    Ok(())
                } else {
                    let error = format!("Build failed with exit code: {:?}", output.status.code());
                    self.build_log.push(error.clone());
                    self.last_result = Some(BuildResult::Failed { error });
                    self.is_building = false;
                    Err("Build failed".to_string())
                }
            }
            Err(e) => {
                let error = format!("Failed to execute cargo: {}", e);
                self.build_log.push(error.clone());
                self.last_result = Some(BuildResult::Failed { error });
                self.is_building = false;
                Err("Failed to execute cargo".to_string())
            }
        }
    }
    
    /// 获取输出路径
    fn get_output_path(&self, project_path: &Path) -> PathBuf {
        let mut path = project_path.to_path_buf();
        path.push("target");
        path.push(self.options.target.rust_target());
        
        match self.options.profile {
            BuildProfile::Debug => path.push("debug"),
            BuildProfile::Release => path.push("release"),
        }
        
        path
    }
    
    /// 渲染构建工具UI
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("Build Tool");
        ui.separator();
        
        // 构建选项
        ui.collapsing("Build Options", |ui| {
            ui.horizontal(|ui| {
                ui.label("Target:");
                egui::ComboBox::from_label("")
                    .selected_text(self.options.target.name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.options.target, BuildTarget::Windows, BuildTarget::Windows.name());
                        ui.selectable_value(&mut self.options.target, BuildTarget::Linux, BuildTarget::Linux.name());
                        ui.selectable_value(&mut self.options.target, BuildTarget::MacOS, BuildTarget::MacOS.name());
                        ui.selectable_value(&mut self.options.target, BuildTarget::Web, BuildTarget::Web.name());
                        ui.selectable_value(&mut self.options.target, BuildTarget::Android, BuildTarget::Android.name());
                        ui.selectable_value(&mut self.options.target, BuildTarget::iOS, BuildTarget::iOS.name());
                    });
            });
            
            ui.horizontal(|ui| {
                ui.label("Profile:");
                ui.selectable_value(&mut self.options.profile, BuildProfile::Debug, "Debug");
                ui.selectable_value(&mut self.options.profile, BuildProfile::Release, "Release");
            });
            
            ui.checkbox(&mut self.options.optimize_size, "Optimize for size");
            ui.checkbox(&mut self.options.include_debug_info, "Include debug info");
        });
        
        ui.separator();
        
        // 构建按钮
        ui.horizontal(|ui| {
            if ui.add_enabled(!self.is_building, egui::Button::new("Build")).clicked() {
                // 实际构建需要在后台线程中执行
                self.build_log.push("Build button clicked (actual build requires background thread)".to_string());
            }
            
            if self.is_building {
                ui.spinner();
                ui.label("Building...");
            }
        });
        
        ui.separator();
        
        // 构建结果
        if let Some(result) = &self.last_result {
            match result {
                BuildResult::Success { output_path, duration } => {
                    ui.colored_label(egui::Color32::GREEN, "✓ Build succeeded");
                    ui.label(format!("Duration: {:.2}s", duration.as_secs_f32()));
                    ui.label(format!("Output: {:?}", output_path));
                }
                BuildResult::Failed { error } => {
                    ui.colored_label(egui::Color32::RED, "✗ Build failed");
                    ui.label(error);
                }
            }
        }
        
        ui.separator();
        
        // 构建日志
        ui.collapsing("Build Log", |ui| {
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for log_line in &self.build_log {
                        ui.label(log_line);
                    }
                });
        });
    }
}

impl Default for BuildTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_build_target() {
        assert_eq!(BuildTarget::Linux.name(), "Linux");
        assert_eq!(BuildTarget::Linux.rust_target(), "x86_64-unknown-linux-gnu");
    }
    
    #[test]
    fn test_build_options() {
        let options = BuildOptions::default();
        assert_eq!(options.target, BuildTarget::Linux);
        assert_eq!(options.profile, BuildProfile::Debug);
    }
}
