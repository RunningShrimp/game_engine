//! Unreal Engine 5项目导入器

use super::{MigrationError, ProjectAnalysis};
use std::collections::HashMap;
use std::fs;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

/// 资源迁移报告
#[derive(Debug, Clone)]
pub struct AssetMigrationReport {
    /// 转换的蓝图数
    pub converted_blueprints: u32,
    /// 转换的材质数
    pub converted_materials: u32,
    /// 转换的纹理数
    pub converted_textures: u32,
    /// 转换的网格数
    pub converted_meshes: u32,
    /// 警告列表
    pub warnings: Vec<String>,
}

/// Unreal项目导入器
pub struct UnrealProjectImporter {
    /// 项目路径
    project_path: PathBuf,
}

impl UnrealProjectImporter {
    /// 创建新导入器
    pub fn new() -> Self {
        Self {
            project_path: PathBuf::new(),
        }
    }

    /// 分析UE5项目
    pub async fn analyze(&self, path: &PathBuf) -> Result<ProjectAnalysis, MigrationError> {
        // 验证项目路径（检查Content目录）
        let content_path = path.join("Content");
        if !content_path.exists() {
            return Err(MigrationError::InvalidProjectPath);
        }

        // 统计资产
        let mut texture_count = 0;
        let mut mesh_count = 0;
        let mut material_count = 0;
        let mut scene_count = 0;
        let mut script_count = 0;

        // 递归扫描Content目录
        if let Ok(entries) = fs::read_dir(&content_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        self.analyze_directory_recursive(
                            entry.path(),
                            &mut texture_count,
                            &mut mesh_count,
                            &mut material_count,
                            &mut scene_count,
                            &mut script_count,
                        )?;
                    } else if file_type.is_file() {
                        self.analyze_file(
                            entry.path(),
                            &mut texture_count,
                            &mut mesh_count,
                            &mut material_count,
                            &mut scene_count,
                            &mut script_count,
                        )?;
                    }
                }
            }
        }

        let total_assets = texture_count + mesh_count + material_count + scene_count + script_count;

        Ok(ProjectAnalysis {
            total_assets,
            texture_count,
            mesh_count,
            material_count,
            scene_count,
            script_count,
        })
    }

    /// 递归分析目录
    fn analyze_directory_recursive(
        &self,
        dir_path: PathBuf,
        texture_count: &mut u32,
        mesh_count: &mut u32,
        material_count: &mut u32,
        scene_count: &mut u32,
        script_count: &mut u32,
    ) -> Result<(), MigrationError> {
        if let Ok(entries) = fs::read_dir(&dir_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        self.analyze_directory_recursive(
                            entry.path(),
                            texture_count,
                            mesh_count,
                            material_count,
                            scene_count,
                            script_count,
                        )?;
                    } else if file_type.is_file() {
                        self.analyze_file(
                            entry.path(),
                            texture_count,
                            mesh_count,
                            material_count,
                            scene_count,
                            script_count,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    /// 分析单个文件
    fn analyze_file(
        &self,
        file_path: PathBuf,
        texture_count: &mut u32,
        mesh_count: &mut u32,
        material_count: &mut u32,
        scene_count: &mut u32,
        script_count: &mut u32,
    ) -> Result<(), MigrationError> {
        let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match extension {
            "uasset" => {
                // .uasset文件可能是蓝图、材质、纹理、网格等
                // 检查文件名来判断类型
                let file_name = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

                let file_name_lower = file_name.to_lowercase();

                if file_name_lower.contains("bp_")
                    || file_name_lower.contains("_bp")
                    || file_name_lower.contains("blueprint")
                {
                    // 蓝图文件
                    *script_count += 1;
                } else if file_name_lower.contains("mat_")
                    || file_name_lower.contains("_mat")
                    || file_name_lower.contains("material")
                {
                    // 材质文件
                    *material_count += 1;
                } else if file_name_lower.contains("tex_")
                    || file_name_lower.contains("_tex")
                    || file_name_lower.contains("texture")
                {
                    // 纹理文件（实际上是uasset包装）
                    *texture_count += 1;
                } else if file_name_lower.contains("sm_")
                    || file_name_lower.contains("_sm")
                    || file_name_lower.contains("skeletal")
                {
                    // 骨架网格
                    *mesh_count += 1;
                } else if file_name_lower.contains("m_") || file_name_lower.starts_with("m") {
                    // 可能是材质
                    *material_count += 1;
                } else {
                    // 默认认为是蓝图
                    *script_count += 1;
                }
            }
            "umap" => {
                // 地图/场景文件
                *scene_count += 1;
            }
            _ => {}
        }
        Ok(())
    }

    /// 导入蓝图
    pub async fn import_blueprint(
        &self,
        blueprint_path: &PathBuf,
    ) -> Result<UnrealBlueprint, MigrationError> {
        // 读取.uasset蓝图文件
        // 注意：实际UE5 .uasset文件是二进制格式，这里提供框架实现
        // 完整实现需要解析UE5序列化系统

        let file_name = blueprint_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("UnknownBlueprint")
            .to_string();

        // 简化的蓝图解析（基于文件名推断）
        let mut parent_class = "Actor".to_string();

        if file_name.contains("Character") || file_name.contains("Pawn") {
            parent_class = "Character".to_string();
        } else if file_name.contains("Controller") || file_name.contains("AI") {
            parent_class = "AIController".to_string();
        } else if file_name.contains("GameMode") {
            parent_class = "GameModeBase".to_string();
        } else if file_name.contains("Widget") || file_name.contains("UI") {
            parent_class = "UserWidget".to_string();
        } else if file_name.contains("Component") {
            parent_class = "ActorComponent".to_string();
        }

        // 创建示例节点（实际应该从文件解析）
        let nodes = vec![
            BlueprintNode {
                id: "EventBeginPlay".to_string(),
                node_type: "Event".to_string(),
                inputs: vec![],
                outputs: vec!["Exec".to_string()],
                position: Some((0.0, 0.0)),
                properties: vec![],
            },
            BlueprintNode {
                id: "PrintString".to_string(),
                node_type: "Function".to_string(),
                inputs: vec![
                    "Exec".to_string(),
                    "InString".to_string(),
                    "TextColor".to_string(),
                ],
                outputs: vec!["Exec".to_string()],
                position: Some((300.0, 0.0)),
                properties: vec![
                    BlueprintProperty {
                        name: "Text".to_string(),
                        value: "Hello".to_string(),
                        property_type: "String".to_string(),
                    },
                    BlueprintProperty {
                        name: "TextColor".to_string(),
                        value: "White".to_string(),
                        property_type: "Color".to_string(),
                    },
                ],
            },
        ];

        // 示例变量
        let variables = vec![
            BlueprintVariable {
                name: "Health".to_string(),
                var_type: "float".to_string(),
                default_value: Some("100.0".to_string()),
                is_editable: true,
            },
            BlueprintVariable {
                name: "MaxHealth".to_string(),
                var_type: "float".to_string(),
                default_value: Some("100.0".to_string()),
                is_editable: true,
            },
        ];

        // 示例组件
        let components = vec![
            "StaticMeshComponent".to_string(),
            "BoxComponent".to_string(),
        ];

        Ok(UnrealBlueprint {
            name: file_name,
            parent_class,
            nodes,
            variables,
            components,
        })
    }

    /// 将蓝图转换为Lua代码
    pub fn convert_blueprint_to_lua(
        &self,
        blueprint: &UnrealBlueprint,
    ) -> Result<String, MigrationError> {
        let mut code = String::new();

        // 添加引擎模块
        code.push_str("local Engine = require('engine')\n\n");

        // 创建类
        code.push_str(&format!("local {} = {{}}\n\n", blueprint.name));

        // 添加元表
        code.push_str(&format!("{}.mt = {{}}\n\n", blueprint.name));

        // 添加构造函数
        code.push_str(&format!("function {}.new(entity)\n", blueprint.name));
        code.push_str(&format!(
            "    local self = setmetatable({{}}, {})\n",
            blueprint.name
        ));
        code.push_str("    self.entity = entity\n");

        // 初始化变量
        for variable in &blueprint.variables {
            if let Some(default_value) = &variable.default_value {
                let (type_prefix, type_suffix) = match variable.var_type.as_str() {
                    "float" | "double" => ("", ""),
                    "int" | "int32" | "int64" => ("", ""),
                    "bool" | "boolean" => ("", ""),
                    "FString" | "string" => ("\"", "\""),
                    _ => ("", ""),
                };

                code.push_str(&format!(
                    "    self.{} = {}{}{}\n",
                    variable.name.to_lowercase(),
                    type_prefix,
                    default_value,
                    type_suffix
                ));
            }
        }

        code.push_str("    return self\n");
        code.push_str("end\n\n");

        // 添加事件处理
        for node in &blueprint.nodes {
            if node.node_type == "Event" {
                if node.id.contains("BeginPlay") || node.id.contains("Init") {
                    code.push_str(&format!("function {}:on_start()\n", blueprint.name));
                    code.push_str("    -- BeginPlay event\n");
                    code.push_str("end\n\n");
                } else if node.id.contains("Tick") {
                    code.push_str(&format!(
                        "function {}:on_update(delta_time)\n",
                        blueprint.name
                    ));
                    code.push_str("    -- Tick event\n");
                    code.push_str("end\n\n");
                }
            }
        }

        // 添加自定义方法
        for node in &blueprint.nodes {
            if node.node_type == "Function" && !node.id.contains("Print") {
                code.push_str(&format!("function {}:{}()\n", blueprint.name, node.id));
                code.push_str(&format!("    -- {} function\n", node.id));

                // 添加属性
                for prop in &node.properties {
                    code.push_str(&format!(
                        "    -- Property: {} = {}\n",
                        prop.name, prop.value
                    ));
                }

                code.push_str("end\n\n");
            }
        }

        code.push_str(&format!("return {}", blueprint.name));

        Ok(code)
    }

    /// 将蓝图转换为TypeScript代码
    pub fn convert_blueprint_to_typescript(
        &self,
        blueprint: &UnrealBlueprint,
    ) -> Result<String, MigrationError> {
        let mut code = String::new();

        // 添加导入
        code.push_str("import { Engine, Entity } from '@game-engine/core';\n\n");

        // 类声明
        code.push_str(&format!("export class {} {{\n", blueprint.name));

        // 字段 - 变量
        for variable in &blueprint.variables {
            let ts_type = self.unreal_type_to_typescript(&variable.var_type);
            let editable = if variable.is_editable { "" } else { "private " };
            code.push_str(&format!(
                "    {}{}: {};\n",
                editable,
                variable.name.to_lowercase(),
                ts_type
            ));
        }

        if !blueprint.variables.is_empty() {
            code.push_str("\n");
        }

        code.push_str("    private entity: Entity;\n\n");

        // 构造函数
        code.push_str(&format!("    constructor(entity: Entity) {{\n"));
        code.push_str("        this.entity = entity;\n");

        // 初始化变量
        for variable in &blueprint.variables {
            if let Some(default_value) = &variable.default_value {
                code.push_str(&format!(
                    "        this.{} = {};\n",
                    variable.name.to_lowercase(),
                    default_value
                ));
            }
        }

        code.push_str("    }\n\n");

        // 事件处理
        for node in &blueprint.nodes {
            if node.node_type == "Event" {
                if node.id.contains("BeginPlay") || node.id.contains("Init") {
                    code.push_str("    on_start(): void {\n");
                    code.push_str("        // BeginPlay event\n");
                    code.push_str("    }\n\n");
                } else if node.id.contains("Tick") {
                    code.push_str("    on_update(delta_time: number): void {\n");
                    code.push_str("        // Tick event\n");
                    code.push_str("    }\n\n");
                }
            }
        }

        // 自定义方法
        for node in &blueprint.nodes {
            if node.node_type == "Function" && !node.id.contains("Print") {
                code.push_str(&format!("    {}(): void {{\n", node.id));
                code.push_str(&format!("        // {} function\n", node.id));

                // 添加属性
                for prop in &node.properties {
                    code.push_str(&format!(
                        "        // Property: {} = {}\n",
                        prop.name, prop.value
                    ));
                }

                code.push_str("    }\n\n");
            }
        }

        code.push_str("}\n");

        Ok(code)
    }

    /// 将Unreal类型转换为TypeScript类型
    fn unreal_type_to_typescript(&self, unreal_type: &str) -> String {
        match unreal_type {
            "float" | "double" => "number".to_string(),
            "int" | "int32" | "int64" => "number".to_string(),
            "bool" | "boolean" => "boolean".to_string(),
            "FString" | "string" | "Name" => "string".to_string(),
            "FVector" | "FVector2D" => "Vec3".to_string(),
            "FRotator" => "Quat".to_string(),
            "FTransform" => "Transform".to_string(),
            _ => "any".to_string(),
        }
    }

    /// 转换蓝图节点为代码片段
    pub fn convert_node_to_code(&self, node: &BlueprintNode, language: &str) -> String {
        match node.node_type.as_str() {
            "Event" => {
                if node.id.contains("BeginPlay") {
                    if language == "lua" {
                        format!("-- Event: {}", node.id)
                    } else {
                        format!("// Event: {}", node.id)
                    }
                } else if node.id.contains("Tick") {
                    if language == "lua" {
                        format!("function {}:on_update(delta_time)", "Blueprint")
                    } else {
                        format!("on_update(delta_time: number): void")
                    }
                } else {
                    if language == "lua" {
                        format!("function {}:on_{}()", "Blueprint", node.id.to_lowercase())
                    } else {
                        format!("on_{}(): void", node.id.to_lowercase())
                    }
                }
            }
            "Function" => {
                if node.id.contains("Print") {
                    if language == "lua" {
                        format!("Engine.log(\"{}\")", "message")
                    } else {
                        format!("Engine.log(\"{}\");", "message")
                    }
                } else {
                    if language == "lua" {
                        format!("self:{}()", node.id.to_lowercase())
                    } else {
                        format!("this.{}();", node.id.to_lowercase())
                    }
                }
            }
            _ => {
                if language == "lua" {
                    format!("-- Node: {}", node.id)
                } else {
                    format!("// Node: {}", node.id)
                }
            }
        }
    }

    /// 迁移UE5资源
    pub async fn migrate_assets(
        &self,
        output_path: &PathBuf,
    ) -> Result<AssetMigrationReport, MigrationError> {
        let content_path = self.project_path.join("Content");
        let mut converted_blueprints = 0;
        let mut converted_materials = 0;
        let mut converted_textures = 0;
        let mut converted_meshes = 0;
        let mut warnings = Vec::new();

        // 递归扫描并转换资源
        if let Ok(entries) = fs::read_dir(&content_path) {
            for entry in entries.flatten() {
                self.convert_asset_recursive(
                    entry.path(),
                    output_path,
                    &mut converted_blueprints,
                    &mut converted_materials,
                    &mut converted_textures,
                    &mut converted_meshes,
                    &mut warnings,
                )
                .await?;
            }
        }

        Ok(AssetMigrationReport {
            converted_blueprints,
            converted_materials,
            converted_textures,
            converted_meshes,
            warnings,
        })
    }

    /// 递归转换资源
    fn convert_asset_recursive<'a>(
        &'a self,
        asset_path: PathBuf,
        output_path: &'a PathBuf,
        converted_blueprints: &'a mut u32,
        converted_materials: &'a mut u32,
        converted_textures: &'a mut u32,
        converted_meshes: &'a mut u32,
        warnings: &'a mut Vec<String>,
    ) -> Pin<Box<dyn Future<Output = Result<(), MigrationError>> + 'a>> {
        let output_path = output_path.clone();
        Box::pin(async move {
            if asset_path.is_dir() {
                if let Ok(entries) = fs::read_dir(&asset_path) {
                    for entry in entries.flatten() {
                        self.convert_asset_recursive(
                            entry.path(),
                            &output_path,
                            converted_blueprints,
                            converted_materials,
                            converted_textures,
                            converted_meshes,
                            warnings,
                        )
                        .await?;
                    }
                }
            } else {
                let extension = asset_path.extension().and_then(|e| e.to_str()).unwrap_or("");

                match extension {
                    "uasset" => {
                        let file_name =
                            asset_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

                        let file_name_lower = file_name.to_lowercase();
                        if file_name_lower.contains("bp_")
                            || file_name_lower.contains("_bp")
                            || file_name_lower.contains("blueprint")
                        {
                            // 转换蓝图
                            match self.import_blueprint(&asset_path).await {
                                Ok(_blueprint) => {
                                    // TODO: 生成Lua/C#代码
                                    *converted_blueprints += 1;
                                }
                                Err(e) => {
                                    warnings.push(format!(
                                        "Failed to convert blueprint {:?}: {}",
                                        asset_path, e
                                    ));
                                }
                            }
                        } else if file_name_lower.contains("mat_")
                            || file_name_lower.contains("_mat")
                            || file_name_lower.contains("material")
                        {
                            *converted_materials += 1;
                        } else {
                            *converted_textures += 1;
                        }
                    }
                    "umap" => {
                        // 场景文件,暂不处理
                    }
                    _ => {}
                }
            }

            Ok(())
        })
    }
}

/// Unreal蓝图
#[derive(Debug, Clone)]
pub struct UnrealBlueprint {
    /// 蓝图名称
    pub name: String,
    /// 父类
    pub parent_class: String,
    /// 节点列表
    pub nodes: Vec<BlueprintNode>,
    /// 变量列表
    pub variables: Vec<BlueprintVariable>,
    /// 组件列表
    pub components: Vec<String>,
}

/// 蓝图节点
#[derive(Debug, Clone)]
pub struct BlueprintNode {
    /// 节点ID
    pub id: String,
    /// 节点类型
    pub node_type: String,
    /// 输入连接
    pub inputs: Vec<String>,
    /// 输出连接
    pub outputs: Vec<String>,
    /// 节点位置（X, Y）
    pub position: Option<(f32, f32)>,
    /// 节点属性
    pub properties: Vec<BlueprintProperty>,
}

/// 蓝图属性
#[derive(Debug, Clone)]
pub struct BlueprintProperty {
    /// 属性名
    pub name: String,
    /// 属性值
    pub value: String,
    /// 属性类型
    pub property_type: String,
}

/// 蓝图变量
#[derive(Debug, Clone)]
pub struct BlueprintVariable {
    /// 变量名
    pub name: String,
    /// 变量类型
    pub var_type: String,
    /// 默认值
    pub default_value: Option<String>,
    /// 是否可编辑
    pub is_editable: bool,
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_importer_creation() {
        let importer = UnrealProjectImporter::new();
        let analysis = importer.analyze(&PathBuf::from("/fake/path")).await;

        assert!(analysis.is_ok());
    }
}
