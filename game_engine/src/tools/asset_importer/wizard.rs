//! # 导入向导UI（Import Wizard）
//!
//! 基于egui的图形化资源导入向导。

use crate::tools::asset_importer::{
    AssetFormat, CompressionFormat, PreviewData,
    detector::{AssetDetector, FileAnalysis},
    importer::{AssetImporter, ImportOptions},
    validator::{AssetValidator, ValidationIssue, ValidationResult},
};
use egui::*;
use std::path::PathBuf;

/// 资源导入向导
pub struct AssetImportWizard {
    /// 选中的文件
    files: Vec<PathBuf>,
    /// 导入设置
    import_settings: ImportSettings,
    /// 预览数据
    preview_data: Vec<(PathBuf, PreviewData)>,
    /// 验证结果
    validation_results: Vec<(PathBuf, ValidationResult)>,
    /// 文件分析结果
    file_analyses: Vec<(PathBuf, FileAnalysis)>,
    /// 当前步骤
    current_step: WizardStep,
    /// 是否打开
    is_open: bool,
    /// 拖拽的文件
    dragged_files: Vec<PathBuf>,
    /// 输出目录
    output_directory: PathBuf,
    /// 错误信息
    error_message: Option<String>,
    /// 成功信息
    success_message: Option<String>,
}

impl AssetImportWizard {
    /// 创建新的导入向导
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            import_settings: ImportSettings::default(),
            preview_data: Vec::new(),
            validation_results: Vec::new(),
            file_analyses: Vec::new(),
            current_step: WizardStep::FileSelection,
            is_open: true,
            dragged_files: Vec::new(),
            output_directory: PathBuf::from("assets/imported"),
            error_message: None,
            success_message: None,
        }
    }

    /// 显示向导窗口
    pub fn show(&mut self, ctx: &egui::Context) -> WizardResult {
        if !self.is_open {
            return WizardResult::Closed;
        }

        let mut result = WizardResult::None;

        Window::new("Asset Import Wizard")
            .collapsible(false)
            .resizable(true)
            .default_width(800.0)
            .default_height(600.0)
            .show(ctx, |ui| {
                // 顶部导航
                self.show_header(ui);

                // 分隔线
                ui.separator();

                // 主内容区
                match self.current_step {
                    WizardStep::FileSelection => {
                        if self.show_file_selection(ui) {
                            result = WizardResult::FilesSelected;
                        }
                    }
                    WizardStep::FormatDetection => {
                        if self.show_format_detection(ui) {
                            result = WizardResult::FormatDetected;
                        }
                    }
                    WizardStep::ImportSettings => {
                        if self.show_settings(ui) {
                            result = WizardResult::SettingsConfirmed;
                        }
                    }
                    WizardStep::Preview => {
                        if self.show_preview(ui) {
                            result = WizardResult::PreviewConfirmed;
                        }
                    }
                    WizardStep::Progress => {
                        if self.show_progress(ui) {
                            result = WizardResult::ImportComplete;
                        }
                    }
                    WizardStep::Complete => {
                        if self.show_complete(ui) {
                            result = WizardResult::Closed;
                            self.is_open = false;
                        }
                    }
                }

                // 底部导航按钮
                ui.separator();
                self.show_footer(ui);

                // 显示错误/成功消息
                self.show_messages(ui);
            });

        result
    }

    /// 显示顶部导航
    fn show_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Step:");

            let steps = [
                WizardStep::FileSelection,
                WizardStep::FormatDetection,
                WizardStep::ImportSettings,
                WizardStep::Preview,
                WizardStep::Progress,
                WizardStep::Complete,
            ];

            for (i, step) in steps.iter().enumerate() {
                let is_current =
                    std::mem::discriminant(&self.current_step) == std::mem::discriminant(step);
                let is_past = matches!(
                    (&self.current_step, step),
                    (WizardStep::FormatDetection, WizardStep::FileSelection)
                        | (
                            WizardStep::ImportSettings,
                            WizardStep::FileSelection | WizardStep::FormatDetection
                        )
                        | (
                            WizardStep::Preview,
                            WizardStep::FileSelection
                                | WizardStep::FormatDetection
                                | WizardStep::ImportSettings
                        )
                        | (
                            WizardStep::Progress,
                            WizardStep::FileSelection
                                | WizardStep::FormatDetection
                                | WizardStep::ImportSettings
                                | WizardStep::Preview
                        )
                        | (WizardStep::Complete, _)
                );

                if is_current {
                    ui.colored_label(
                        egui::Color32::LIGHT_BLUE,
                        format!("{} {}", i + 1, self.step_name(step)),
                    );
                } else if is_past {
                    ui.label(format!("{} {}", i + 1, self.step_name(step)));
                } else {
                    ui.label(
                        egui::RichText::new(format!("{} {}", i + 1, self.step_name(step))).weak(),
                    );
                }

                if i < steps.len() - 1 {
                    ui.label("->");
                }
            }
        });
    }

    /// 显示文件选择步骤
    fn show_file_selection(&mut self, ui: &mut egui::Ui) -> bool {
        ui.vertical_centered(|ui| {
            ui.heading("Select Files to Import");
            ui.add_space(10.0);

            ui.label("Drag and drop files here or click to select:");
            ui.add_space(10.0);

            // 拖放区域
            let mut dropped = false;
            let drop_response = ui
                .allocate_response(
                    egui::Vec2::new(ui.available_width(), 200.0),
                    egui::Sense::click_and_drag(),
                )
                .on_hover_cursor(egui::CursorIcon::PointingHand);

            // 绘制拖放区域背景
            let painter = ui.painter_at(drop_response.rect);
            painter.rect_filled(
                drop_response.rect,
                egui::Rounding::same(5),
                egui::Color32::from_gray(50),
            );
            painter.rect_stroke(
                drop_response.rect,
                egui::Rounding::same(5),
                egui::Stroke::new(2.0, egui::Color32::DARK_GRAY),
                egui::StrokeKind::Middle,
            );

            // 检查拖放
            if drop_response.hovered() {
                if let Some(dropped_files) = ui.input(|i| i.raw.dropped_files.clone()) {
                    for file in dropped_files {
                        if let Some(path) = file.path {
                            self.dragged_files.push(path);
                        }
                    }
                    dropped = true;
                }
            }

            // 显示文本
            painter.text(
                drop_response.rect.center(),
                egui::Align2::CENTER_CENTER,
                if self.dragged_files.is_empty() {
                    "Drop files here or click to browse"
                } else {
                    &format!("{} file(s) selected", self.dragged_files.len())
                },
                egui::FontId::default(),
                egui::Color32::LIGHT_GRAY,
            );

            // 点击浏览
            if drop_response.clicked() {
                // 这里需要使用文件对话框
                // 由于egui不内置文件对话框，这里仅作示意
                self.error_message =
                    Some("File browser not implemented. Please use drag and drop.".to_string());
            }

            // 显示已选择的文件列表
            if !self.dragged_files.is_empty() || dropped {
                ui.add_space(10.0);
                ui.group(|ui| {
                    ui.heading("Selected Files:");
                    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        for file in &self.dragged_files {
                            ui.horizontal(|ui| {
                                ui.label(format!("📄 {}", file.display()));
                                if ui.button("×").clicked() {
                                    // TODO: Remove file
                                }
                            });
                        }
                    });
                });

                // 确认按钮
                ui.add_space(10.0);
                if ui.button("Next: Detect Format →").clicked() {
                    self.files = self.dragged_files.clone();
                    self.analyze_files();
                    self.current_step = WizardStep::FormatDetection;
                    return true;
                }
            }

            false
        })
        .inner
    }

    /// 显示格式检测步骤
    fn show_format_detection(&mut self, ui: &mut egui::Ui) -> bool {
        ui.vertical(|ui| {
            ui.heading("Format Detection Results");
            ui.add_space(10.0);

            if self.file_analyses.is_empty() {
                ui.label("No files analyzed.");
                return false;
            }

            egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                for (path, analysis) in &self.file_analyses {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "📁 {}",
                                path.file_name().unwrap().to_str().unwrap()
                            ));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if analysis.is_valid {
                                        ui.colored_label(egui::Color32::GREEN, "✓ Valid");
                                    } else {
                                        ui.colored_label(egui::Color32::RED, "✗ Invalid");
                                    }
                                },
                            );
                        });

                        ui.label(format!("Format: {:?}", analysis.format));
                        ui.label(format!("Size: {} bytes", analysis.size));

                        if !analysis.issues.is_empty() {
                            ui.label("Issues:");
                            for issue in &analysis.issues {
                                ui.colored_label(egui::Color32::YELLOW, format!("• {}", issue));
                            }
                        }
                    });
                    ui.add_space(5.0);
                }
            });

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("← Back").clicked() {
                    self.current_step = WizardStep::FileSelection;
                }

                if ui.button("Next: Settings →").clicked() {
                    self.current_step = WizardStep::ImportSettings;
                    return true;
                }
            });

            false
        })
        .inner
    }

    /// 显示导入设置步骤
    fn show_settings(&mut self, ui: &mut egui::Ui) -> bool {
        ui.vertical(|ui| {
            ui.heading("Import Settings");
            ui.add_space(10.0);

            // 输出目录
            ui.horizontal(|ui| {
                ui.label("Output Directory:");
                ui.text_edit_singleline(&mut self.output_directory.display().to_string());
            });

            ui.add_space(10.0);

            // 压缩格式
            ui.horizontal(|ui| {
                ui.label("Compression:");
                egui::ComboBox::from_id_source("compression")
                    .selected_text(format!("{:?}", self.import_settings.compression))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.import_settings.compression,
                            CompressionFormat::None,
                            "None",
                        );
                        ui.selectable_value(
                            &mut self.import_settings.compression,
                            CompressionFormat::BC1,
                            "BC1 (DXT1)",
                        );
                        ui.selectable_value(
                            &mut self.import_settings.compression,
                            CompressionFormat::BC3,
                            "BC3 (DXT5)",
                        );
                    });
            });

            // 质量设置
            ui.horizontal(|ui| {
                ui.label("Quality:");
                ui.add(egui::Slider::new(
                    &mut self.import_settings.quality,
                    0.0..=1.0,
                ));
                ui.label(format!("{:.0}", self.import_settings.quality * 100.0));
            });

            // Mipmaps
            ui.checkbox(
                &mut self.import_settings.generate_mipmaps,
                "Generate Mipmaps",
            );

            // 归一化法线
            ui.checkbox(
                &mut self.import_settings.normalize_normals,
                "Normalize Normals",
            );

            ui.add_space(10.0);

            // 底部按钮
            ui.horizontal(|ui| {
                if ui.button("← Back").clicked() {
                    self.current_step = WizardStep::FormatDetection;
                }

                if ui.button("Next: Preview →").clicked() {
                    self.generate_previews();
                    self.current_step = WizardStep::Preview;
                    return true;
                }
            });

            false
        })
        .inner
    }

    /// 显示预览步骤
    fn show_preview(&mut self, ui: &mut egui::Ui) -> bool {
        ui.vertical(|ui| {
            ui.heading("Preview");
            ui.add_space(10.0);

            if self.preview_data.is_empty() {
                ui.label("No preview data available.");
                return false;
            }

            egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                for (path, preview) in &self.preview_data {
                    ui.group(|ui| {
                        ui.label(format!(
                            "📄 {}",
                            path.file_name().unwrap().to_str().unwrap()
                        ));
                        self.show_preview_content(ui, preview);
                    });
                    ui.add_space(5.0);
                }
            });

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("← Back").clicked() {
                    self.current_step = WizardStep::ImportSettings;
                }

                if ui.button("Start Import →").clicked() {
                    self.current_step = WizardStep::Progress;
                    self.perform_import();
                    return true;
                }
            });

            false
        })
        .inner
    }

    /// 显示预览内容
    fn show_preview_content(&self, ui: &mut egui::Ui, preview: &PreviewData) {
        match preview {
            PreviewData::Texture {
                width,
                height,
                format,
                size,
            } => {
                ui.label(format!("Texture: {}x{} ({})", width, height, format));
                ui.label(format!("Size: {} bytes", size));
            }
            PreviewData::Model {
                vertices,
                triangles,
                materials,
                animations,
            } => {
                ui.label(format!("Model"));
                ui.label(format!("  Vertices: {}", vertices));
                ui.label(format!("  Triangles: {}", triangles));
                ui.label(format!("  Materials: {}", materials));
                ui.label(format!("  Animations: {}", animations));
            }
            PreviewData::Audio {
                duration,
                channels,
                sample_rate,
                format,
            } => {
                ui.label(format!("Audio: {}", format));
                ui.label(format!("  Duration: {:.2}s", duration));
                ui.label(format!("  Channels: {}", channels));
                ui.label(format!("  Sample Rate: {}Hz", sample_rate));
            }
            PreviewData::Unknown { size, format } => {
                ui.label(format!("Unknown format: {}", format));
                ui.label(format!("Size: {} bytes", size));
            }
        }
    }

    /// 显示进度步骤
    fn show_progress(&mut self, ui: &mut egui::Ui) -> bool {
        ui.vertical(|ui| {
            ui.heading("Importing Assets...");
            ui.add_space(10.0);

            ui.spinner();
            ui.label("Please wait while the assets are being imported.");

            ui.add_space(10.0);

            // 进度条
            let progress = self.files.len() as f32;
            ui.add(egui::ProgressBar::new(progress).show_percentage());

            // 导入完成
            if ui.button("Finish").clicked() {
                self.current_step = WizardStep::Complete;
                return true;
            }

            false
        })
        .inner
    }

    /// 显示完成步骤
    fn show_complete(&mut self, ui: &mut egui::Ui) -> bool {
        ui.vertical_centered(|ui| {
            ui.heading("Import Complete!");
            ui.add_space(20.0);

            ui.label("✓ All assets have been successfully imported.");

            ui.add_space(20.0);

            if ui.button("Close").clicked() {
                return true;
            }

            false
        })
        .inner
    }

    /// 显示底部导航
    fn show_footer(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Cancel").clicked() {
                    self.is_open = false;
                }
            });
        });
    }

    /// 显示消息
    fn show_messages(&mut self, ui: &mut egui::Ui) {
        if let Some(error) = &self.error_message {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
        }

        if let Some(success) = &self.success_message {
            ui.colored_label(egui::Color32::GREEN, format!("Success: {}", success));
        }
    }

    /// 获取步骤名称
    fn step_name(&self, step: &WizardStep) -> &str {
        match step {
            WizardStep::FileSelection => "Files",
            WizardStep::FormatDetection => "Detect",
            WizardStep::ImportSettings => "Settings",
            WizardStep::Preview => "Preview",
            WizardStep::Progress => "Import",
            WizardStep::Complete => "Done",
        }
    }

    /// 分析文件
    fn analyze_files(&mut self) {
        self.file_analyses.clear();
        self.validation_results.clear();

        for file in &self.files {
            if let Ok(analysis) = AssetDetector::analyze_file(file) {
                self.file_analyses.push((file.clone(), analysis));
            }

            let validation = AssetValidator::validate(file);
            self.validation_results.push((file.clone(), validation));
        }
    }

    /// 生成预览
    fn generate_previews(&mut self) {
        self.preview_data.clear();

        for (path, analysis) in &self.file_analyses {
            let preview = match analysis.format {
                AssetFormat::Texture => PreviewData::Texture {
                    width: analysis.metadata.get("width").and_then(|s| s.parse().ok()).unwrap_or(0),
                    height: analysis
                        .metadata
                        .get("height")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0),
                    format: analysis
                        .metadata
                        .get("format")
                        .cloned()
                        .unwrap_or_else(|| "Unknown".to_string()),
                    size: analysis.size as usize,
                },
                AssetFormat::GLTF | AssetFormat::FBX | AssetFormat::OBJ => PreviewData::Model {
                    vertices: analysis
                        .metadata
                        .get("vertices")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0),
                    triangles: analysis
                        .metadata
                        .get("faces")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0),
                    materials: analysis
                        .metadata
                        .get("materials")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0),
                    animations: analysis
                        .metadata
                        .get("animations")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0),
                },
                _ => PreviewData::Unknown {
                    size: analysis.size as usize,
                    format: format!("{:?}", analysis.format),
                },
            };

            self.preview_data.push((path.clone(), preview));
        }
    }

    /// 执行导入
    fn perform_import(&mut self) {
        let importer =
            AssetImporter::new(self.output_directory.clone()).with_options(ImportOptions {
                skip_validation: false,
                generate_mipmaps: self.import_settings.generate_mipmaps,
                normalize_normals: self.import_settings.normalize_normals,
                compression: self.import_settings.compression,
                quality: self.import_settings.quality,
            });

        let mut imported = 0;
        let mut failed = 0;

        for file in &self.files {
            match importer.import(file) {
                Ok(_) => imported += 1,
                Err(e) => {
                    failed += 1;
                    log::error!("Failed to import {:?}: {:?}", file, e);
                }
            }
        }

        if failed == 0 {
            self.success_message = Some(format!("Successfully imported {} files.", imported));
        } else {
            self.error_message = Some(format!("Imported {} files, {} failed.", imported, failed));
        }
    }
}

impl Default for AssetImportWizard {
    fn default() -> Self {
        Self::new()
    }
}

/// 向导步骤
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WizardStep {
    FileSelection,
    FormatDetection,
    ImportSettings,
    Preview,
    Progress,
    Complete,
}

/// 向导结果
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WizardResult {
    None,
    Closed,
    FilesSelected,
    FormatDetected,
    SettingsConfirmed,
    PreviewConfirmed,
    ImportComplete,
}

/// 导入设置
#[derive(Clone, Debug)]
pub struct ImportSettings {
    pub format: AssetFormat,
    pub compression: CompressionFormat,
    pub quality: f32,
    pub generate_mipmaps: bool,
    pub normalize_normals: bool,
}

impl Default for ImportSettings {
    fn default() -> Self {
        Self {
            format: AssetFormat::Unknown,
            compression: CompressionFormat::None,
            quality: 1.0,
            generate_mipmaps: true,
            normalize_normals: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizard_creation() {
        let wizard = AssetImportWizard::new();
        assert!(wizard.is_open);
        assert_eq!(wizard.current_step, WizardStep::FileSelection);
    }

    #[test]
    fn test_import_settings_default() {
        let settings = ImportSettings::default();
        assert!(settings.generate_mipmaps);
        assert!(settings.normalize_normals);
        assert_eq!(settings.quality, 1.0);
    }
}
