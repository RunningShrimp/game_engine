use std::path::{Path, PathBuf};
use std::process::Command;
use super::build_tool::{BuildTarget, BuildProfile, BuildResult};

/// 平台特定构建器
pub struct PlatformBuilder;

impl PlatformBuilder {
    /// 为Web平台构建
    pub fn build_web(
        project_path: &Path,
        profile: BuildProfile,
        output_dir: &Path,
    ) -> Result<BuildResult, String> {
        let start_time = std::time::Instant::now();
        
        // 1. 检查wasm-pack是否安装
        let wasm_pack_check = Command::new("wasm-pack")
            .arg("--version")
            .output();
        
        if wasm_pack_check.is_err() {
            return Err("wasm-pack not found. Please install it with: cargo install wasm-pack".to_string());
        }
        
        // 2. 使用wasm-pack构建
        let mut cmd = Command::new("wasm-pack");
        cmd.current_dir(project_path);
        cmd.arg("build");
        cmd.arg("--target");
        cmd.arg("web");
        
        if matches!(profile, BuildProfile::Release) {
            cmd.arg("--release");
        } else {
            cmd.arg("--dev");
        }
        
        cmd.arg("--out-dir");
        cmd.arg(output_dir);
        
        match cmd.output() {
            Ok(output) => {
                let duration = start_time.elapsed();
                
                if output.status.success() {
                    // 生成HTML模板
                    Self::generate_web_template(output_dir)?;
                    
                    Ok(BuildResult::Success {
                        output_path: output_dir.to_path_buf(),
                        duration,
                    })
                } else {
                    let error = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(format!("Web build failed: {}", error))
                }
            }
            Err(e) => Err(format!("Failed to execute wasm-pack: {}", e)),
        }
    }
    
    /// 生成Web HTML模板
    fn generate_web_template(output_dir: &Path) -> Result<(), String> {
        let html_content = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Game Engine - Web Build</title>
    <style>
        body {
            margin: 0;
            padding: 0;
            overflow: hidden;
            background: #000;
        }
        canvas {
            display: block;
            width: 100vw;
            height: 100vh;
        }
    </style>
</head>
<body>
    <canvas id="game-canvas"></canvas>
    <script type="module">
        import init from './game_engine.js';
        
        async function run() {
            await init();
            console.log('Game engine initialized');
        }
        
        run();
    </script>
</body>
</html>"#;
        
        let html_path = output_dir.join("index.html");
        std::fs::write(html_path, html_content)
            .map_err(|e| format!("Failed to write HTML template: {}", e))?;
        
        Ok(())
    }
    
    /// 为Android平台构建
    pub fn build_android(
        project_path: &Path,
        profile: BuildProfile,
        output_dir: &Path,
    ) -> Result<BuildResult, String> {
        let start_time = std::time::Instant::now();
        
        // 1. 检查cargo-ndk是否安装
        let cargo_ndk_check = Command::new("cargo")
            .arg("ndk")
            .arg("--version")
            .output();
        
        if cargo_ndk_check.is_err() {
            return Err("cargo-ndk not found. Please install it with: cargo install cargo-ndk".to_string());
        }
        
        // 2. 使用cargo-ndk构建
        let mut cmd = Command::new("cargo");
        cmd.current_dir(project_path);
        cmd.arg("ndk");
        cmd.arg("--target");
        cmd.arg("aarch64-linux-android");
        cmd.arg("--platform");
        cmd.arg("29"); // Android API level 29
        cmd.arg("build");
        
        if matches!(profile, BuildProfile::Release) {
            cmd.arg("--release");
        }
        
        match cmd.output() {
            Ok(output) => {
                let duration = start_time.elapsed();
                
                if output.status.success() {
                    // 复制输出到指定目录
                    let target_dir = project_path.join("target/aarch64-linux-android");
                    let profile_dir = match profile {
                        BuildProfile::Debug => target_dir.join("debug"),
                        BuildProfile::Release => target_dir.join("release"),
                    };
                    
                    Ok(BuildResult::Success {
                        output_path: profile_dir,
                        duration,
                    })
                } else {
                    let error = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(format!("Android build failed: {}", error))
                }
            }
            Err(e) => Err(format!("Failed to execute cargo-ndk: {}", e)),
        }
    }
    
    /// 为iOS平台构建
    pub fn build_ios(
        project_path: &Path,
        profile: BuildProfile,
        _output_dir: &Path,
    ) -> Result<BuildResult, String> {
        let start_time = std::time::Instant::now();
        
        // 1. 使用cargo构建iOS目标
        let mut cmd = Command::new("cargo");
        cmd.current_dir(project_path);
        cmd.arg("build");
        cmd.arg("--target");
        cmd.arg("aarch64-apple-ios");
        
        if matches!(profile, BuildProfile::Release) {
            cmd.arg("--release");
        }
        
        match cmd.output() {
            Ok(output) => {
                let duration = start_time.elapsed();
                
                if output.status.success() {
                    let target_dir = project_path.join("target/aarch64-apple-ios");
                    let profile_dir = match profile {
                        BuildProfile::Debug => target_dir.join("debug"),
                        BuildProfile::Release => target_dir.join("release"),
                    };
                    
                    Ok(BuildResult::Success {
                        output_path: profile_dir,
                        duration,
                    })
                } else {
                    let error = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(format!("iOS build failed: {}", error))
                }
            }
            Err(e) => Err(format!("Failed to execute cargo: {}", e)),
        }
    }
    
    /// 根据目标平台选择合适的构建方法
    pub fn build_for_target(
        target: BuildTarget,
        project_path: &Path,
        profile: BuildProfile,
        output_dir: &Path,
    ) -> Result<BuildResult, String> {
        match target {
            BuildTarget::Web => Self::build_web(project_path, profile, output_dir),
            BuildTarget::Android => Self::build_android(project_path, profile, output_dir),
            BuildTarget::iOS => Self::build_ios(project_path, profile, output_dir),
            _ => {
                // 对于其他平台,使用标准的cargo build
                Self::build_standard(target, project_path, profile)
            }
        }
    }
    
    /// 标准构建流程
    fn build_standard(
        target: BuildTarget,
        project_path: &Path,
        profile: BuildProfile,
    ) -> Result<BuildResult, String> {
        let start_time = std::time::Instant::now();
        
        let mut cmd = Command::new("cargo");
        cmd.current_dir(project_path);
        cmd.arg("build");
        cmd.arg("--target");
        cmd.arg(target.rust_target());
        
        if matches!(profile, BuildProfile::Release) {
            cmd.arg("--release");
        }
        
        match cmd.output() {
            Ok(output) => {
                let duration = start_time.elapsed();
                
                if output.status.success() {
                    let target_dir = project_path.join(format!("target/{}", target.rust_target()));
                    let profile_dir = match profile {
                        BuildProfile::Debug => target_dir.join("debug"),
                        BuildProfile::Release => target_dir.join("release"),
                    };
                    
                    Ok(BuildResult::Success {
                        output_path: profile_dir,
                        duration,
                    })
                } else {
                    let error = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(format!("Build failed: {}", error))
                }
            }
            Err(e) => Err(format!("Failed to execute cargo: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_platform_builder() {
        // 这个测试只验证API的存在性,不实际执行构建
        let target = BuildTarget::Web;
        let profile = BuildProfile::Debug;
        
        assert_eq!(target.rust_target(), "wasm32-unknown-unknown");
        assert_eq!(profile.name(), "Debug");
    }
}
