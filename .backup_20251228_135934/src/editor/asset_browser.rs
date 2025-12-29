use std::fs;
use std::path::{Path, PathBuf};

/// 资源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    Texture,
    Model,
    Audio,
    Scene,
    Script,
    Material,
    Animation,
    Font,
    Unknown,
}

impl AssetType {
    /// 从文件扩展名推断资源类型
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "bmp" | "tga" | "webp" => AssetType::Texture,
            "obj" | "fbx" | "gltf" | "glb" => AssetType::Model,
            "mp3" | "wav" | "ogg" | "flac" => AssetType::Audio,
            "scene" | "json" => AssetType::Scene,
            "js" | "py" | "lua" | "wasm" => AssetType::Script,
            "mat" => AssetType::Material,
            "anim" => AssetType::Animation,
            "ttf" | "otf" => AssetType::Font,
            _ => AssetType::Unknown,
        }
    }

    /// 获取资源类型的图标
    pub fn icon(&self) -> &'static str {
        match self {
            AssetType::Texture => "🖼",
            AssetType::Model => "🧊",
            AssetType::Audio => "🔊",
            AssetType::Scene => "🌍",
            AssetType::Script => "📜",
            AssetType::Material => "🎨",
            AssetType::Animation => "🎬",
            AssetType::Font => "🔤",
            AssetType::Unknown => "📄",
        }
    }
}

/// 资源项
#[derive(Debug, Clone)]
pub struct AssetItem {
    pub path: PathBuf,
    pub name: String,
    pub asset_type: AssetType,
    pub is_directory: bool,
}

/// 资源视图模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetViewMode {
    List,
    Grid,
    Details,
}

/// 资源类型过滤器
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetTypeFilter {
    All,
    Textures,
    Models,
    Audio,
    Scenes,
    Scripts,
    Materials,
    Animations,
    Fonts,
}

/// 资源浏览器
pub struct AssetBrowser {
    /// 当前目录
    pub current_directory: PathBuf,
    /// 资源根目录
    pub root_directory: PathBuf,
    /// 当前目录的资源列表
    pub assets: Vec<AssetItem>,
    /// 选中的资源
    pub selected_asset: Option<usize>,
    /// 搜索过滤器
    pub search_filter: String,
    /// 类型过滤器
    pub type_filter: AssetTypeFilter,
    /// 视图模式
    pub view_mode: AssetViewMode,
    /// 缩略图大小（网格视图）
    pub thumbnail_size: f32,
    /// 预览的资源路径
    pub preview_path: Option<PathBuf>,
}

impl AssetBrowser {
    pub fn new(root_directory: impl Into<PathBuf>) -> Self {
        let root = root_directory.into();
        let mut browser = Self {
            current_directory: root.clone(),
            root_directory: root,
            assets: Vec::new(),
            selected_asset: None,
            search_filter: String::new(),
            type_filter: AssetTypeFilter::All,
            view_mode: AssetViewMode::List,
            thumbnail_size: 64.0,
            preview_path: None,
        };

        browser.refresh();
        browser
    }

    /// 刷新资源列表
    pub fn refresh(&mut self) {
        self.assets.clear();

        if let Ok(entries) = fs::read_dir(&self.current_directory) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name =
                    path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();

                let is_directory = path.is_dir();
                let asset_type = if is_directory {
                    AssetType::Unknown
                } else {
                    path.extension()
                        .and_then(|ext| ext.to_str())
                        .map(AssetType::from_extension)
                        .unwrap_or(AssetType::Unknown)
                };

                // 应用搜索过滤器
                if !self.search_filter.is_empty()
                    && !name.to_lowercase().contains(&self.search_filter.to_lowercase())
                {
                    continue;
                }

                // 应用类型过滤器
                if !is_directory {
                    match self.type_filter {
                        AssetTypeFilter::All => {}
                        AssetTypeFilter::Textures => {
                            if asset_type != AssetType::Texture {
                                continue;
                            }
                        }
                        AssetTypeFilter::Models => {
                            if asset_type != AssetType::Model {
                                continue;
                            }
                        }
                        AssetTypeFilter::Audio => {
                            if asset_type != AssetType::Audio {
                                continue;
                            }
                        }
                        AssetTypeFilter::Scenes => {
                            if asset_type != AssetType::Scene {
                                continue;
                            }
                        }
                        AssetTypeFilter::Scripts => {
                            if asset_type != AssetType::Script {
                                continue;
                            }
                        }
                        AssetTypeFilter::Materials => {
                            if asset_type != AssetType::Material {
                                continue;
                            }
                        }
                        AssetTypeFilter::Animations => {
                            if asset_type != AssetType::Animation {
                                continue;
                            }
                        }
                        AssetTypeFilter::Fonts => {
                            if asset_type != AssetType::Font {
                                continue;
                            }
                        }
                    }
                }

                self.assets.push(AssetItem {
                    path,
                    name,
                    asset_type,
                    is_directory,
                });
            }
        }

        // 排序: 目录在前,然后按名称排序
        self.assets.sort_by(|a, b| match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });
    }

    /// 导航到指定目录
    pub fn navigate_to(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        if path.is_dir() {
            self.current_directory = path.to_path_buf();
            self.selected_asset = None;
            self.refresh();
        }
    }

    /// 返回上一级目录
    pub fn navigate_up(&mut self) {
        let parent_path = self
            .current_directory
            .parent()
            .filter(|p| *p >= self.root_directory.as_path())
            .map(|p| p.to_path_buf());

        if let Some(parent) = parent_path {
            self.navigate_to(parent);
        }
    }

    /// 渲染资源浏览器UI
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("Asset Browser");
        ui.separator();

        // 当前路径
        ui.horizontal(|ui| {
            if ui.button("⬆ Up").clicked() {
                self.navigate_up();
            }

            ui.label(format!("Path: {}", self.current_directory.display()));
        });

        ui.separator();

        // 搜索和过滤工具栏
        ui.horizontal(|ui| {
            ui.label("Search:");
            if ui.text_edit_singleline(&mut self.search_filter).changed() {
                self.refresh();
            }

            if ui.button("Clear").clicked() {
                self.search_filter.clear();
                self.refresh();
            }

            ui.separator();

            ui.label("Filter:");
            ui.selectable_value(&mut self.type_filter, AssetTypeFilter::All, "All");
            ui.selectable_value(&mut self.type_filter, AssetTypeFilter::Textures, "Textures");
            ui.selectable_value(&mut self.type_filter, AssetTypeFilter::Models, "Models");
            ui.selectable_value(&mut self.type_filter, AssetTypeFilter::Audio, "Audio");
            ui.selectable_value(&mut self.type_filter, AssetTypeFilter::Scenes, "Scenes");

            if self.type_filter != AssetTypeFilter::All {
                self.refresh();
            }
        });

        ui.horizontal(|ui| {
            ui.label("View:");
            ui.selectable_value(&mut self.view_mode, AssetViewMode::List, "List");
            ui.selectable_value(&mut self.view_mode, AssetViewMode::Grid, "Grid");
            ui.selectable_value(&mut self.view_mode, AssetViewMode::Details, "Details");

            if self.view_mode == AssetViewMode::Grid {
                ui.separator();
                ui.label("Size:");
                ui.add(egui::Slider::new(&mut self.thumbnail_size, 32.0..=128.0));
            }
        });

        ui.separator();

        // 资源列表（根据视图模式渲染）
        let mut clicked_directory = None;
        let mut clicked_asset = None;
        let mut double_clicked_asset = None;

        match self.view_mode {
            AssetViewMode::List => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, asset) in self.assets.iter().enumerate() {
                        let is_selected = self.selected_asset == Some(i);

                        ui.horizontal(|ui| {
                            let icon = if asset.is_directory {
                                "📁"
                            } else {
                                asset.asset_type.icon()
                            };

                            let response = ui
                                .selectable_label(is_selected, format!("{} {}", icon, asset.name));
                            if response.clicked() {
                                if asset.is_directory {
                                    clicked_directory = Some(asset.path.clone());
                                } else {
                                    clicked_asset = Some(i);
                                }
                            }
                            if response.double_clicked() && !asset.is_directory {
                                double_clicked_asset = Some(i);
                            }
                        });
                    }
                });
            }
            AssetViewMode::Grid => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("asset_grid")
                        .num_columns((ui.available_width() / (self.thumbnail_size + 20.0)) as usize)
                        .spacing([10.0, 10.0])
                        .show(ui, |ui| {
                            for (i, asset) in self.assets.iter().enumerate() {
                                let is_selected = self.selected_asset == Some(i);

                                ui.vertical(|ui| {
                                    let icon = if asset.is_directory {
                                        "📁"
                                    } else {
                                        asset.asset_type.icon()
                                    };

                                    ui.set_min_size(egui::vec2(
                                        self.thumbnail_size,
                                        self.thumbnail_size,
                                    ));
                                    let response = ui.selectable_label(
                                        is_selected,
                                        format!("{}\n{}", icon, asset.name),
                                    );

                                    if response.clicked() {
                                        if asset.is_directory {
                                            clicked_directory = Some(asset.path.clone());
                                        } else {
                                            clicked_asset = Some(i);
                                        }
                                    }
                                    if response.double_clicked() && !asset.is_directory {
                                        double_clicked_asset = Some(i);
                                    }
                                });
                            }
                        });
                });
            }
            AssetViewMode::Details => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // 表头
                    ui.horizontal(|ui| {
                        ui.label("Name");
                        ui.separator();
                        ui.label("Type");
                        ui.separator();
                        ui.label("Size");
                        ui.separator();
                        ui.label("Modified");
                    });
                    ui.separator();

                    for (i, asset) in self.assets.iter().enumerate() {
                        let is_selected = self.selected_asset == Some(i);

                        let response = ui.selectable_label(is_selected, "");
                        if response.clicked() {
                            if asset.is_directory {
                                clicked_directory = Some(asset.path.clone());
                            } else {
                                clicked_asset = Some(i);
                            }
                        }
                        if response.double_clicked() && !asset.is_directory {
                            double_clicked_asset = Some(i);
                        }

                        ui.horizontal(|ui| {
                            let icon = if asset.is_directory {
                                "📁"
                            } else {
                                asset.asset_type.icon()
                            };
                            ui.label(format!("{} {}", icon, asset.name));
                            ui.separator();
                            ui.label(format!("{:?}", asset.asset_type));
                            ui.separator();

                            if let Ok(metadata) = fs::metadata(&asset.path) {
                                let size_kb = metadata.len() as f64 / 1024.0;
                                if size_kb < 1024.0 {
                                    ui.label(format!("{:.2} KB", size_kb));
                                } else {
                                    ui.label(format!("{:.2} MB", size_kb / 1024.0));
                                }

                                ui.separator();

                                if let Ok(modified) = metadata.modified() {
                                    ui.label(format!("{:?}", modified));
                                }
                            }
                        });
                    }
                });
            }
        }

        // 处理点击事件
        if let Some(path) = clicked_directory {
            self.navigate_to(path);
        }
        if let Some(index) = clicked_asset {
            self.selected_asset = Some(index);
            if let Some(asset) = self.assets.get(index) {
                self.preview_path = Some(asset.path.clone());
            }
        }
        if let Some(index) = double_clicked_asset {
            // 双击打开资源（根据类型执行不同操作）
            if let Some(asset) = self.assets.get(index) {
                self.preview_path = Some(asset.path.clone());
            }
        }

        ui.separator();

        // 选中资源的详细信息和预览
        if let Some(index) = self.selected_asset
            && let Some(asset) = self.assets.get(index) {
                ui.collapsing("Asset Details", |ui| {
                    ui.label(format!("Name: {}", asset.name));
                    ui.label(format!("Type: {:?}", asset.asset_type));
                    ui.label(format!("Path: {}", asset.path.display()));

                    // 文件大小
                    if let Ok(metadata) = fs::metadata(&asset.path) {
                        let size_kb = metadata.len() as f64 / 1024.0;
                        if size_kb < 1024.0 {
                            ui.label(format!("Size: {:.2} KB", size_kb));
                        } else {
                            ui.label(format!("Size: {:.2} MB", size_kb / 1024.0));
                        }
                    }
                });

                // 预览
                if let Some(preview_path) = &self.preview_path
                    && preview_path == &asset.path {
                        self.render_preview(ui, asset);
                    }
            }
    }

    /// 渲染资源预览
    fn render_preview(&self, ui: &mut egui::Ui, asset: &AssetItem) {
        ui.separator();
        ui.collapsing("Preview", |ui| {
            match asset.asset_type {
                AssetType::Texture => {
                    ui.label("🖼 Texture Preview");
                    ui.label("(Texture preview not yet implemented)");
                    // 注意：纹理预览功能待实现
                    // 未来计划：加载纹理并显示缩略图预览
                }
                AssetType::Model => {
                    ui.label("🧊 Model Preview");
                    ui.label("(3D model preview not yet implemented)");
                    // 注意：3D模型预览功能待实现
                    // 未来计划：加载3D模型并显示预览（可能需要简化渲染）
                }
                AssetType::Audio => {
                    ui.label("🔊 Audio Preview");
                    ui.label("(Audio preview not yet implemented)");
                    // 注意：音频预览功能待实现
                    // 未来计划：显示音频波形图或提供播放控件
                }
                AssetType::Scene => {
                    ui.label("🌍 Scene Preview");
                    ui.label("(Scene preview not yet implemented)");
                }
                AssetType::Script => {
                    ui.label("📜 Script Preview");
                    // 显示脚本内容的前几行
                    if let Ok(content) = fs::read_to_string(&asset.path) {
                        let preview: String =
                            content.lines().take(20).collect::<Vec<_>>().join("\n");
                        ui.code_editor(&mut preview.clone());
                    }
                }
                _ => {
                    ui.label("No preview available for this asset type");
                }
            }
        });
    }
}

impl Default for AssetBrowser {
    fn default() -> Self {
        Self::new("./assets")
    }
}
