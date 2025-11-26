use std::path::{Path, PathBuf};
use std::fs;

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
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();
                
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
                if !self.search_filter.is_empty() && !name.to_lowercase().contains(&self.search_filter.to_lowercase()) {
                    continue;
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
        self.assets.sort_by(|a, b| {
            match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
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
        let parent_path = self.current_directory.parent()
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
        
        // 搜索框
        ui.horizontal(|ui| {
            ui.label("Search:");
            if ui.text_edit_singleline(&mut self.search_filter).changed() {
                self.refresh();
            }
            
            if ui.button("Clear").clicked() {
                self.search_filter.clear();
                self.refresh();
            }
        });
        
        ui.separator();
        
        // 资源列表
        let mut clicked_directory = None;
        let mut clicked_asset = None;
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, asset) in self.assets.iter().enumerate() {
                let is_selected = self.selected_asset == Some(i);
                
                ui.horizontal(|ui| {
                    let icon = if asset.is_directory {
                        "📁"
                    } else {
                        asset.asset_type.icon()
                    };
                    
                    if ui.selectable_label(is_selected, format!("{} {}", icon, asset.name)).clicked() {
                        if asset.is_directory {
                            clicked_directory = Some(asset.path.clone());
                        } else {
                            clicked_asset = Some(i);
                        }
                    }
                });
            }
        });
        
        // 处理点击事件
        if let Some(path) = clicked_directory {
            self.navigate_to(path);
        }
        if let Some(index) = clicked_asset {
            self.selected_asset = Some(index);
        }
        
        ui.separator();
        
        // 选中资源的详细信息
        if let Some(index) = self.selected_asset {
            if let Some(asset) = self.assets.get(index) {
                ui.label("Selected Asset:");
                ui.label(format!("  Name: {}", asset.name));
                ui.label(format!("  Type: {:?}", asset.asset_type));
                ui.label(format!("  Path: {}", asset.path.display()));
                
                // 文件大小
                if let Ok(metadata) = fs::metadata(&asset.path) {
                    let size_kb = metadata.len() as f64 / 1024.0;
                    ui.label(format!("  Size: {:.2} KB", size_kb));
                }
            }
        }
    }
}

impl Default for AssetBrowser {
    fn default() -> Self {
        Self::new("./assets")
    }
}
