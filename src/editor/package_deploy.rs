use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use super::build_tool::BuildTarget;

/// 打包配置
#[derive(Debug, Clone)]
pub struct PackageConfig {
    /// 应用名称
    pub app_name: String,
    /// 版本号
    pub version: String,
    /// 作者
    pub author: String,
    /// 描述
    pub description: String,
    /// 图标路径
    pub icon_path: Option<PathBuf>,
    /// 是否包含资源文件
    pub include_assets: bool,
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            app_name: "Game".to_string(),
            version: "1.0.0".to_string(),
            author: "Developer".to_string(),
            description: "A game made with game_engine".to_string(),
            icon_path: None,
            include_assets: true,
        }
    }
}

/// 部署目标
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeployTarget {
    Local,
    ItchIo,
    Steam,
    GitHub,
}

impl DeployTarget {
    pub fn name(&self) -> &'static str {
        match self {
            DeployTarget::Local => "Local",
            DeployTarget::ItchIo => "Itch.io",
            DeployTarget::Steam => "Steam",
            DeployTarget::GitHub => "GitHub Releases",
        }
    }
}

/// 打包和部署管理器
pub struct PackageDeployManager {
    /// 打包配置
    pub config: PackageConfig,
}

impl PackageDeployManager {
    pub fn new(config: PackageConfig) -> Self {
        Self { config }
    }
    
    /// 打包应用
    pub fn package(
        &self,
        build_path: &Path,
        target: BuildTarget,
        output_dir: &Path,
    ) -> Result<PathBuf, String> {
        match target {
            BuildTarget::Windows => self.package_windows(build_path, output_dir),
            BuildTarget::Linux => self.package_linux(build_path, output_dir),
            BuildTarget::MacOS => self.package_macos(build_path, output_dir),
            BuildTarget::Web => self.package_web(build_path, output_dir),
            BuildTarget::Android => self.package_android(build_path, output_dir),
            BuildTarget::iOS => self.package_ios(build_path, output_dir),
        }
    }
    
    /// 打包Windows应用
    fn package_windows(&self, build_path: &Path, output_dir: &Path) -> Result<PathBuf, String> {
        let package_dir = output_dir.join(format!("{}-windows", self.config.app_name));
        fs::create_dir_all(&package_dir)
            .map_err(|e| format!("Failed to create package directory: {}", e))?;
        
        // 复制可执行文件
        let exe_name = format!("{}.exe", self.config.app_name);
        let src_exe = build_path.join(&exe_name);
        let dst_exe = package_dir.join(&exe_name);
        
        fs::copy(&src_exe, &dst_exe)
            .map_err(|e| format!("Failed to copy executable: {}", e))?;
        
        // 复制资源文件
        if self.config.include_assets {
            self.copy_assets(&package_dir)?;
        }
        
        // 创建ZIP压缩包
        let zip_path = output_dir.join(format!("{}-{}-windows.zip", 
            self.config.app_name, self.config.version));
        
        self.create_zip(&package_dir, &zip_path)?;
        
        Ok(zip_path)
    }
    
    /// 打包Linux应用
    fn package_linux(&self, build_path: &Path, output_dir: &Path) -> Result<PathBuf, String> {
        let package_dir = output_dir.join(format!("{}-linux", self.config.app_name));
        fs::create_dir_all(&package_dir)
            .map_err(|e| format!("Failed to create package directory: {}", e))?;
        
        // 复制可执行文件
        let src_exe = build_path.join(&self.config.app_name);
        let dst_exe = package_dir.join(&self.config.app_name);
        
        fs::copy(&src_exe, &dst_exe)
            .map_err(|e| format!("Failed to copy executable: {}", e))?;
        
        // 设置可执行权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&dst_exe)
                .map_err(|e| format!("Failed to get file metadata: {}", e))?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&dst_exe, perms)
                .map_err(|e| format!("Failed to set executable permissions: {}", e))?;
        }
        
        // 复制资源文件
        if self.config.include_assets {
            self.copy_assets(&package_dir)?;
        }
        
        // 创建tar.gz压缩包
        let tar_path = output_dir.join(format!("{}-{}-linux.tar.gz", 
            self.config.app_name, self.config.version));
        
        self.create_tar_gz(&package_dir, &tar_path)?;
        
        Ok(tar_path)
    }
    
    /// 打包macOS应用
    fn package_macos(&self, build_path: &Path, output_dir: &Path) -> Result<PathBuf, String> {
        let app_bundle = output_dir.join(format!("{}.app", self.config.app_name));
        let contents_dir = app_bundle.join("Contents");
        let macos_dir = contents_dir.join("MacOS");
        let resources_dir = contents_dir.join("Resources");
        
        fs::create_dir_all(&macos_dir)
            .map_err(|e| format!("Failed to create MacOS directory: {}", e))?;
        fs::create_dir_all(&resources_dir)
            .map_err(|e| format!("Failed to create Resources directory: {}", e))?;
        
        // 复制可执行文件
        let src_exe = build_path.join(&self.config.app_name);
        let dst_exe = macos_dir.join(&self.config.app_name);
        
        fs::copy(&src_exe, &dst_exe)
            .map_err(|e| format!("Failed to copy executable: {}", e))?;
        
        // 创建Info.plist
        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>{}</string>
    <key>CFBundleDisplayName</key>
    <string>{}</string>
    <key>CFBundleIdentifier</key>
    <string>com.{}.{}</string>
    <key>CFBundleVersion</key>
    <string>{}</string>
    <key>CFBundleExecutable</key>
    <string>{}</string>
</dict>
</plist>"#,
            self.config.app_name,
            self.config.app_name,
            self.config.author.to_lowercase().replace(" ", ""),
            self.config.app_name.to_lowercase(),
            self.config.version,
            self.config.app_name
        );
        
        let plist_path = contents_dir.join("Info.plist");
        fs::write(&plist_path, plist_content)
            .map_err(|e| format!("Failed to write Info.plist: {}", e))?;
        
        // 复制资源文件
        if self.config.include_assets {
            self.copy_assets(&resources_dir)?;
        }
        
        // 创建DMG镜像
        let dmg_path = output_dir.join(format!("{}-{}-macos.dmg", 
            self.config.app_name, self.config.version));
        
        self.create_dmg(&app_bundle, &dmg_path)?;
        
        Ok(dmg_path)
    }
    
    /// 打包Web应用
    fn package_web(&self, build_path: &Path, output_dir: &Path) -> Result<PathBuf, String> {
        let package_dir = output_dir.join(format!("{}-web", self.config.app_name));
        
        // 复制整个构建目录
        self.copy_dir_all(build_path, &package_dir)?;
        
        // 创建ZIP压缩包
        let zip_path = output_dir.join(format!("{}-{}-web.zip", 
            self.config.app_name, self.config.version));
        
        self.create_zip(&package_dir, &zip_path)?;
        
        Ok(zip_path)
    }
    
    /// 打包Android应用
    fn package_android(&self, build_path: &Path, output_dir: &Path) -> Result<PathBuf, String> {
        // Android打包需要使用Android SDK工具
        // 这里提供简化的实现
        let apk_path = output_dir.join(format!("{}-{}.apk", 
            self.config.app_name, self.config.version));
        
        // 实际实现需要调用Android构建工具
        Err("Android packaging requires Android SDK tools".to_string())
    }
    
    /// 打包iOS应用
    fn package_ios(&self, build_path: &Path, output_dir: &Path) -> Result<PathBuf, String> {
        // iOS打包需要使用Xcode工具
        let ipa_path = output_dir.join(format!("{}-{}.ipa", 
            self.config.app_name, self.config.version));
        
        // 实际实现需要调用Xcode构建工具
        Err("iOS packaging requires Xcode tools".to_string())
    }
    
    /// 复制资源文件
    fn copy_assets(&self, target_dir: &Path) -> Result<(), String> {
        let assets_dir = Path::new("assets");
        if assets_dir.exists() {
            let target_assets = target_dir.join("assets");
            self.copy_dir_all(assets_dir, &target_assets)?;
        }
        Ok(())
    }
    
    /// 递归复制目录
    fn copy_dir_all(&self, src: &Path, dst: &Path) -> Result<(), String> {
        fs::create_dir_all(dst)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
        
        for entry in fs::read_dir(src)
            .map_err(|e| format!("Failed to read directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            let file_name = path.file_name().unwrap();
            let dst_path = dst.join(file_name);
            
            if path.is_dir() {
                self.copy_dir_all(&path, &dst_path)?;
            } else {
                fs::copy(&path, &dst_path)
                    .map_err(|e| format!("Failed to copy file: {}", e))?;
            }
        }
        
        Ok(())
    }
    
    /// 创建ZIP压缩包
    fn create_zip(&self, src_dir: &Path, zip_path: &Path) -> Result<(), String> {
        // 使用zip命令
        let output = Command::new("zip")
            .arg("-r")
            .arg(zip_path)
            .arg(".")
            .current_dir(src_dir)
            .output();
        
        match output {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(format!("Failed to create ZIP: {}", error))
            }
            Err(_) => {
                // 如果zip命令不可用,返回提示信息
                Err("zip command not found. Package created but not compressed.".to_string())
            }
        }
    }
    
    /// 创建tar.gz压缩包
    fn create_tar_gz(&self, src_dir: &Path, tar_path: &Path) -> Result<(), String> {
        let output = Command::new("tar")
            .arg("-czf")
            .arg(tar_path)
            .arg("-C")
            .arg(src_dir.parent().unwrap())
            .arg(src_dir.file_name().unwrap())
            .output();
        
        match output {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(format!("Failed to create tar.gz: {}", error))
            }
            Err(_) => {
                Err("tar command not found. Package created but not compressed.".to_string())
            }
        }
    }
    
    /// 创建DMG镜像
    fn create_dmg(&self, app_bundle: &Path, dmg_path: &Path) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("hdiutil")
                .arg("create")
                .arg("-volname")
                .arg(&self.config.app_name)
                .arg("-srcfolder")
                .arg(app_bundle)
                .arg("-ov")
                .arg("-format")
                .arg("UDZO")
                .arg(dmg_path)
                .output();
            
            match output {
                Ok(output) if output.status.success() => Ok(()),
                Ok(output) => {
                    let error = String::from_utf8_lossy(&output.stderr);
                    Err(format!("Failed to create DMG: {}", error))
                }
                Err(e) => Err(format!("Failed to execute hdiutil: {}", e)),
            }
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            Err("DMG creation is only supported on macOS".to_string())
        }
    }
    
    /// 部署到目标平台
    pub fn deploy(&self, package_path: &Path, target: DeployTarget) -> Result<String, String> {
        match target {
            DeployTarget::Local => self.deploy_local(package_path),
            DeployTarget::ItchIo => self.deploy_itch(package_path),
            DeployTarget::Steam => self.deploy_steam(package_path),
            DeployTarget::GitHub => self.deploy_github(package_path),
        }
    }
    
    /// 部署到本地
    fn deploy_local(&self, package_path: &Path) -> Result<String, String> {
        Ok(format!("Package ready at: {}", package_path.display()))
    }
    
    /// 部署到Itch.io
    fn deploy_itch(&self, package_path: &Path) -> Result<String, String> {
        // 使用butler工具上传到Itch.io
        Err("Itch.io deployment requires butler tool and configuration".to_string())
    }
    
    /// 部署到Steam
    fn deploy_steam(&self, package_path: &Path) -> Result<String, String> {
        // 使用SteamCMD上传到Steam
        Err("Steam deployment requires SteamCMD and Steamworks SDK".to_string())
    }
    
    /// 部署到GitHub Releases
    fn deploy_github(&self, package_path: &Path) -> Result<String, String> {
        // 使用GitHub CLI创建release
        Err("GitHub deployment requires gh CLI and repository configuration".to_string())
    }
}

impl Default for PackageDeployManager {
    fn default() -> Self {
        Self::new(PackageConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_package_config() {
        let config = PackageConfig::default();
        assert_eq!(config.app_name, "Game");
        assert_eq!(config.version, "1.0.0");
    }
    
    #[test]
    fn test_package_deploy_manager() {
        let manager = PackageDeployManager::default();
        assert_eq!(manager.config.app_name, "Game");
    }
}
