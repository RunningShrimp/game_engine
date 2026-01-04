//! # C# SDK Generator
//!
//! Generates C# SDK with API bindings, lifecycle hooks, and Unity-style API.

use crate::tools::csharp_sdk::templates;
use std::fs;
use std::path::{Path, PathBuf};

/// C# SDK生成器
pub struct CSharpSdkGenerator {
    output_dir: PathBuf,
    namespace: String,
    api_version: String,
}

impl CSharpSdkGenerator {
    /// 创建新的SDK生成器
    pub fn new(output_dir: impl AsRef<Path>, namespace: impl Into<String>) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
            namespace: namespace.into(),
            api_version: "1.0.0".to_string(),
        }
    }

    /// 设置API版本
    pub fn with_api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = version.into();
        self
    }

    /// 生成完整的C# SDK
    pub fn generate(&self) -> Result<(), String> {
        // 创建输出目录
        fs::create_dir_all(&self.output_dir)
            .map_err(|e| format!("Failed to create output directory: {e}"))?;

        // 生成核心API
        self.generate_core_api()?;

        // 生成生命周期钩子基类
        self.generate_lifecycle_hooks()?;

        // 生成物理API
        self.generate_physics_api()?;

        // 生成音频API
        self.generate_audio_api()?;

        // 生成网络API
        self.generate_network_api()?;

        // 生成输入API
        self.generate_input_api()?;

        // 生成ECS API
        self.generate_ecs_api()?;

        // 生成资源API
        self.generate_resource_api()?;

        // 生成项目文件
        self.generate_project_file()?;

        // 生成README
        self.generate_readme()?;

        Ok(())
    }

    fn generate_core_api(&self) -> Result<(), String> {
        let content = templates::core_api_template(&self.namespace, &self.api_version);
        let path = self.output_dir.join("CoreAPI.cs");
        fs::write(&path, content).map_err(|e| format!("Failed to write CoreAPI.cs: {e}"))?;
        Ok(())
    }

    fn generate_lifecycle_hooks(&self) -> Result<(), String> {
        let content = templates::lifecycle_hooks_template(&self.namespace);
        let path = self.output_dir.join("LifecycleHooks.cs");
        fs::write(&path, content).map_err(|e| format!("Failed to write LifecycleHooks.cs: {e}"))?;
        Ok(())
    }

    fn generate_physics_api(&self) -> Result<(), String> {
        let content = templates::physics_api_template(&self.namespace);
        let path = self.output_dir.join("PhysicsAPI.cs");
        fs::write(&path, content).map_err(|e| format!("Failed to write PhysicsAPI.cs: {e}"))?;
        Ok(())
    }

    fn generate_audio_api(&self) -> Result<(), String> {
        let content = templates::audio_api_template(&self.namespace);
        let path = self.output_dir.join("AudioAPI.cs");
        fs::write(&path, content).map_err(|e| format!("Failed to write AudioAPI.cs: {e}"))?;
        Ok(())
    }

    fn generate_network_api(&self) -> Result<(), String> {
        let content = templates::network_api_template(&self.namespace);
        let path = self.output_dir.join("NetworkAPI.cs");
        fs::write(&path, content).map_err(|e| format!("Failed to write NetworkAPI.cs: {e}"))?;
        Ok(())
    }

    fn generate_input_api(&self) -> Result<(), String> {
        let content = templates::input_api_template(&self.namespace);
        let path = self.output_dir.join("InputAPI.cs");
        fs::write(&path, content).map_err(|e| format!("Failed to write InputAPI.cs: {e}"))?;
        Ok(())
    }

    fn generate_ecs_api(&self) -> Result<(), String> {
        let content = templates::ecs_api_template(&self.namespace);
        let path = self.output_dir.join("ECSAPI.cs");
        fs::write(&path, content).map_err(|e| format!("Failed to write ECSAPI.cs: {e}"))?;
        Ok(())
    }

    fn generate_resource_api(&self) -> Result<(), String> {
        let content = templates::resource_api_template(&self.namespace);
        let path = self.output_dir.join("ResourceAPI.cs");
        fs::write(&path, content).map_err(|e| format!("Failed to write ResourceAPI.cs: {e}"))?;
        Ok(())
    }

    fn generate_project_file(&self) -> Result<(), String> {
        let content = templates::project_file_template(&self.namespace);
        let path = self.output_dir.join("GameEngine.csproj");
        fs::write(&path, content).map_err(|e| format!("Failed to write GameEngine.csproj: {e}"))?;
        Ok(())
    }

    fn generate_readme(&self) -> Result<(), String> {
        let content = templates::readme_template(&self.namespace);
        let path = self.output_dir.join("README.md");
        fs::write(&path, content).map_err(|e| format!("Failed to write README.md: {e}"))?;
        Ok(())
    }
}
