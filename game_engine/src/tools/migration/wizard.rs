//! Unity迁移向导
//!
//! 提供交互式命令行界面，指导用户完成Unity项目迁移。

use crate::tools::migration::{
    APIMappingTable, ComponentMappingRegistry, EngineType, MigrationConfig, MigrationManager,
    MigrationProgress,
};
use std::io::{self, Write};
use std::path::PathBuf;

/// 迁移向导
pub struct MigrationWizard {
    /// 配置
    config: MigrationConfig,
    /// API映射表
    api_mappings: APIMappingTable,
    /// 组件映射表
    component_mappings: ComponentMappingRegistry,
}

impl MigrationWizard {
    /// 创建新的迁移向导
    pub fn new() -> Self {
        Self {
            config: MigrationConfig::default(),
            api_mappings: APIMappingTable::new(),
            component_mappings: ComponentMappingRegistry::new(),
        }
    }

    /// 运行向导
    pub async fn run(&mut self) -> Result<WizardResult, WizardError> {
        println!("╔════════════════════════════════════════════════════════╗");
        println!("║     Unity到游戏引擎迁移向导 v1.0                      ║");
        println!("╚════════════════════════════════════════════════════════╝");
        println!();

        // 步骤1: 选择源引擎
        self.step_1_select_engine()?;

        // 步骤2: 指定项目路径
        self.step_2_project_path()?;

        // 步骤3: 指定输出路径
        self.step_3_output_path()?;

        // 步骤4: 选择迁移选项
        self.step_4_migration_options()?;

        // 步骤5: 确认配置
        self.step_5_confirm()?;

        // 步骤6: 执行迁移
        let result = self.step_6_execute_migration().await?;

        // 步骤7: 显示结果
        self.step_7_show_result(&result);

        Ok(result)
    }

    /// 步骤1: 选择源引擎
    fn step_1_select_engine(&mut self) -> Result<(), WizardError> {
        println!("📋 步骤 1/7: 选择源引擎");
        println!("════════════════════════════════════════════════════════");
        println!();
        println!("支持的引擎:");
        println!("  [1] Unity");
        println!("  [2] Unreal Engine 5");
        println!("  [3] 其他");
        println!();

        print!("请选择源引擎 (1-3): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        self.config.source_engine = match input.trim() {
            "1" => EngineType::Unity,
            "2" => EngineType::Unreal5,
            "3" => EngineType::Other,
            _ => return Err(WizardError::InvalidSelection),
        };

        println!("✓ 已选择: {:?}", self.config.source_engine);
        println!();

        Ok(())
    }

    /// 步骤2: 指定项目路径
    fn step_2_project_path(&mut self) -> Result<(), WizardError> {
        println!("📂 步骤 2/7: 指定Unity项目路径");
        println!("════════════════════════════════════════════════════════");
        println!();

        print!("请输入Unity项目路径: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let path = PathBuf::from(input.trim());

        if !path.exists() {
            return Err(WizardError::PathNotFound(path));
        }

        self.config.project_path = path;
        println!("✓ 项目路径: {}", self.config.project_path.display());
        println!();

        Ok(())
    }

    /// 步骤3: 指定输出路径
    fn step_3_output_path(&mut self) -> Result<(), WizardError> {
        println!("💾 步骤 3/7: 指定输出路径");
        println!("════════════════════════════════════════════════════════");
        println!();

        print!("请输入输出路径 (留空则创建在项目旁): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let path = input.trim();
        self.config.output_path = if path.is_empty() {
            // 在项目路径旁创建output文件夹
            self.config
                .project_path
                .parent()
                .map(|p| p.join("migrated_project"))
                .unwrap_or_else(|| PathBuf::from("./migrated_project"))
        } else {
            PathBuf::from(path)
        };

        println!("✓ 输出路径: {}", self.config.output_path.display());
        println!();

        Ok(())
    }

    /// 步骤4: 选择迁移选项
    fn step_4_migration_options(&mut self) -> Result<(), WizardError> {
        println!("⚙️  步骤 4/7: 选择迁移选项");
        println!("════════════════════════════════════════════════════════");
        println!();

        print!("是否转换纹理? (y/n): ");
        self.config.convert_textures = self.read_yes_no()?;

        print!("是否转换网格? (y/n): ");
        self.config.convert_meshes = self.read_yes_no()?;

        print!("是否转换材质? (y/n): ");
        self.config.convert_materials = self.read_yes_no()?;

        print!("是否转换场景? (y/n): ");
        self.config.convert_scenes = self.read_yes_no()?;

        println!();
        println!("✓ 迁移选项:");
        println!(
            "  - 纹理: {}",
            if self.config.convert_textures {
                "✓"
            } else {
                "✗"
            }
        );
        println!(
            "  - 网格: {}",
            if self.config.convert_meshes {
                "✓"
            } else {
                "✗"
            }
        );
        println!(
            "  - 材质: {}",
            if self.config.convert_materials {
                "✓"
            } else {
                "✗"
            }
        );
        println!(
            "  - 场景: {}",
            if self.config.convert_scenes {
                "✓"
            } else {
                "✗"
            }
        );
        println!();

        Ok(())
    }

    /// 步骤5: 确认配置
    fn step_5_confirm(&mut self) -> Result<(), WizardError> {
        println!("🔍 步骤 5/7: 确认迁移配置");
        println!("════════════════════════════════════════════════════════");
        println!();

        println!("迁移配置摘要:");
        println!("  源引擎: {:?}", self.config.source_engine);
        println!("  项目路径: {}", self.config.project_path.display());
        println!("  输出路径: {}", self.config.output_path.display());
        println!("  转换选项:");
        println!(
            "    - 纹理: {}",
            if self.config.convert_textures {
                "是"
            } else {
                "否"
            }
        );
        println!(
            "    - 网格: {}",
            if self.config.convert_meshes {
                "是"
            } else {
                "否"
            }
        );
        println!(
            "    - 材质: {}",
            if self.config.convert_materials {
                "是"
            } else {
                "否"
            }
        );
        println!(
            "    - 场景: {}",
            if self.config.convert_scenes {
                "是"
            } else {
                "否"
            }
        );
        println!();

        print!("确认开始迁移? (y/n): ");
        io::stdout().flush()?;

        if !self.read_yes_no()? {
            return Err(WizardError::CancelledByUser);
        }

        println!("✓ 配置已确认");
        println!();

        Ok(())
    }

    /// 步骤6: 执行迁移
    async fn step_6_execute_migration(&mut self) -> Result<WizardResult, WizardError> {
        println!("🚀 步骤 6/7: 执行迁移");
        println!("════════════════════════════════════════════════════════");
        println!();

        let mut manager = MigrationManager::new(self.config.clone());

        // 显示进度
        println!("开始迁移...");

        let result = manager.migrate().await;

        match result {
            Ok(migration_result) => {
                println!("✓ 迁移完成!");
                println!();

                Ok(WizardResult {
                    success: true,
                    converted_assets: migration_result.converted_assets,
                    warnings: migration_result.warnings,
                    errors: migration_result.errors,
                })
            }
            Err(e) => {
                println!("✗ 迁移失败: {}", e);
                println!();

                Err(WizardError::MigrationFailed(e.to_string()))
            }
        }
    }

    /// 步骤7: 显示结果
    fn step_7_show_result(&self, result: &WizardResult) {
        println!("📊 步骤 7/7: 迁移结果");
        println!("════════════════════════════════════════════════════════");
        println!();

        if result.success {
            println!("✅ 迁移成功!");
            println!();
            println!("统计信息:");
            println!("  转换的资源: {}", result.converted_assets);
            println!("  警告数量: {}", result.warnings.len());
            println!("  错误数量: {}", result.errors.len());
            println!();

            if !result.warnings.is_empty() {
                println!("⚠️  警告:");
                for warning in &result.warnings {
                    println!("  - {}", warning);
                }
                println!();
            }

            if !result.errors.is_empty() {
                println!("❌ 错误:");
                for error in &result.errors {
                    println!("  - {}", error);
                }
                println!();
            }

            println!("📁 输出位置: {}", self.config.output_path.display());
            println!();

            println!("📝 下一步:");
            println!("  1. 检查转换后的资源");
            println!("  2. 手动修正脚本转换错误");
            println!("  3. 测试游戏功能");
            println!("  4. 优化性能和画质");
            println!();
        } else {
            println!("❌ 迁移失败");
            println!();
        }
    }

    /// 读取yes/no输入
    fn read_yes_no(&mut self) -> Result<bool, WizardError> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => Ok(true),
            "n" | "no" => Ok(false),
            _ => Err(WizardError::InvalidSelection),
        }
    }
}

impl Default for MigrationWizard {
    fn default() -> Self {
        Self::new()
    }
}

/// 向导结果
#[derive(Debug, Clone)]
pub struct WizardResult {
    /// 是否成功
    pub success: bool,
    /// 转换的资源数
    pub converted_assets: u32,
    /// 警告列表
    pub warnings: Vec<String>,
    /// 错误列表
    pub errors: Vec<String>,
}

/// 向导错误
#[derive(Debug, Clone)]
pub enum WizardError {
    /// 无效选择
    InvalidSelection,
    /// 路径不存在
    PathNotFound(PathBuf),
    /// 用户取消
    CancelledByUser,
    /// 迁移失败
    MigrationFailed(String),
    /// IO错误
    IoError(String),
}

impl std::fmt::Display for WizardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WizardError::InvalidSelection => write!(f, "无效的选择"),
            WizardError::PathNotFound(path) => write!(f, "路径不存在: {}", path.display()),
            WizardError::CancelledByUser => write!(f, "用户取消操作"),
            WizardError::MigrationFailed(msg) => write!(f, "迁移失败: {}", msg),
            WizardError::IoError(msg) => write!(f, "IO错误: {}", msg),
        }
    }
}

impl std::error::Error for WizardError {}

impl From<io::Error> for WizardError {
    fn from(error: io::Error) -> Self {
        WizardError::IoError(error.to_string())
    }
}

/// 快速迁移函数
pub async fn quick_migrate(
    project_path: PathBuf,
    output_path: Option<PathBuf>,
) -> Result<WizardResult, WizardError> {
    let output = output_path.unwrap_or_else(|| {
        project_path
            .parent()
            .map(|p| p.join("migrated_project"))
            .unwrap_or_else(|| PathBuf::from("./migrated_project"))
    });

    let config = MigrationConfig {
        source_engine: EngineType::Unity,
        project_path: project_path.clone(),
        output_path: output.clone(),
        convert_textures: true,
        convert_meshes: true,
        convert_materials: true,
        convert_scenes: true,
    };

    let mut manager = MigrationManager::new(config);
    let result = manager.migrate().await;

    match result {
        Ok(migration_result) => Ok(WizardResult {
            success: true,
            converted_assets: migration_result.converted_assets,
            warnings: migration_result.warnings,
            errors: migration_result.errors,
        }),
        Err(e) => Err(WizardError::MigrationFailed(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizard_creation() {
        let wizard = MigrationWizard::new();
        assert_eq!(wizard.config.source_engine, EngineType::Other);
    }

    #[test]
    fn test_wizard_default() {
        let wizard = MigrationWizard::default();
        assert_eq!(wizard.config.source_engine, EngineType::Other);
    }

    #[test]
    fn test_wizard_result() {
        let result = WizardResult {
            success: true,
            converted_assets: 100,
            warnings: vec!["Warning 1".to_string()],
            errors: vec![],
        };

        assert!(result.success);
        assert_eq!(result.converted_assets, 100);
        assert_eq!(result.warnings.len(), 1);
    }
}
