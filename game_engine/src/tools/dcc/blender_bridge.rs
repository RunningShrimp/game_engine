//! Blender实时桥接工具
//!
//! 通过Python API与Blender进行实时数据交换和双向同步。

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncBufReadExtExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout, Command as TokioCommand};

/// Blender桥接配置
#[derive(Debug, Clone)]
pub struct BlenderBridgeConfig {
    /// Blender可执行文件路径
    pub blender_path: PathBuf,

    /// Python脚本路径
    pub script_path: PathBuf,

    /// 通信端口
    pub port: u16,

    /// 是否启用后台模式
    pub background_mode: bool,
}

impl Default for BlenderBridgeConfig {
    fn default() -> Self {
        Self {
            blender_path: PathBuf::from("blender"),
            script_path: PathBuf::from("./blender_bridge.py"),
            port: 9876,
            background_mode: true,
        }
    }
}

/// Blender桥接
pub struct BlenderBridge {
    /// 配置
    config: BlenderBridgeConfig,

    /// Blender进程
    blender_process: Option<BlenderProcess>,

    /// 连接状态
    connected: bool,
}

/// Blender进程包装
struct BlenderProcess {
    /// stdin
    stdin: ChildStdin,

    /// stdout
    stdout: BufReader<ChildStdout>,

    /// 进程句柄
    #[allow(dead_code)]
    child: tokio::process::Child,
}

impl BlenderBridge {
    /// 创建新的Blender桥接
    pub fn new(config: BlenderBridgeConfig) -> Self {
        Self {
            config,
            blender_process: None,
            connected: false,
        }
    }

    /// 连接到Blender
    pub async fn connect(&mut self) -> Result<(), BlenderError> {
        let mut cmd = TokioCommand::new(&self.config.blender_path);

        // 后台模式参数
        if self.config.background_mode {
            cmd.arg("-b");
            cmd.arg("-P");
            cmd.arg(&self.config.script_path);
        } else {
            cmd.arg("-P");
            cmd.arg(&self.config.script_path);
        }

        // 创建进程
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let child = cmd.spawn().await.map_err(|e| BlenderError::ConnectionError(e.to_string()))?;

        let stdin = child
            .stdin
            .ok_or_else(|| BlenderError::ConnectionError("Failed to open stdin".to_string()))?;

        let stdout = child
            .stdout
            .ok_or_else(|| BlenderError::ConnectionError("Failed to open stdout".to_string()))?;

        self.blender_process = Some(BlenderProcess {
            stdin,
            stdout: BufReader::new(stdout),
            child,
        });

        self.connected = true;
        Ok(())
    }

    /// 断开连接
    pub async fn disconnect(&mut self) -> Result<(), BlenderError> {
        if let Some(process) = &mut self.blender_process {
            // 发送退出命令
            let _ = Self::send_command(process, "exit()").await;

            // 关闭进程
            let _ = process.child.kill().await;
        }

        self.blender_process = None;
        self.connected = false;
        Ok(())
    }

    /// 获取场景数据
    pub async fn get_scene_data(&mut self) -> Result<BlenderScene, BlenderError> {
        if !self.connected {
            return Err(BlenderError::NotConnected);
        }

        let process = self.blender_process.as_mut().ok_or(BlenderError::NotConnected)?;

        // 发送获取场景命令
        Self::send_command(process, "get_scene()").await?;

        // 读取响应
        let response = Self::read_response(process).await?;

        // 解析JSON响应
        let scene: BlenderScene =
            serde_json::from_str(&response).map_err(|e| BlenderError::ParseError(e.to_string()))?;

        Ok(scene)
    }

    /// 更新场景数据
    pub async fn update_scene(&mut self, scene: &BlenderScene) -> Result<(), BlenderError> {
        if !self.connected {
            return Err(BlenderError::NotConnected);
        }

        let process = self.blender_process.as_mut().ok_or(BlenderError::NotConnected)?;

        // 序列化场景数据
        let scene_json = serde_json::to_string(scene)
            .map_err(|e| BlenderError::SerializationError(e.to_string()))?;

        // 发送更新命令
        let command = format!("update_scene({})", scene_json);
        Self::send_command(process, &command).await?;

        Ok(())
    }

    /// 执行Python脚本
    pub async fn execute_python(&mut self, script: &str) -> Result<String, BlenderError> {
        if !self.connected {
            return Err(BlenderError::NotConnected);
        }

        let process = self.blender_process.as_mut().ok_or(BlenderError::NotConnected)?;

        Self::send_command(process, script).await?;
        let response = Self::read_response(process).await?;

        Ok(response)
    }

    /// 导出网格
    pub async fn export_mesh(
        &mut self,
        object_name: &str,
        path: &PathBuf,
    ) -> Result<(), BlenderError> {
        let script = format!("export_mesh('{}', '{}')", object_name, path.display());

        self.execute_python(&script).await?;
        Ok(())
    }

    /// 导入网格
    pub async fn import_mesh(&mut self, path: &PathBuf) -> Result<(), BlenderError> {
        let script = format!("import_mesh('{}')", path.display());
        self.execute_python(&script).await?;
        Ok(())
    }

    /// 发送命令到Blender
    async fn send_command(process: &mut BlenderProcess, command: &str) -> Result<(), BlenderError> {
        let command_with_newline = format!("{}\n", command);

        process
            .stdin
            .write_all(command_with_newline.as_bytes())
            .await
            .map_err(|e| BlenderError::IoError(e.to_string()))?;

        process.stdin.flush().await.map_err(|e| BlenderError::IoError(e.to_string()))?;

        Ok(())
    }

    /// 读取Blender响应
    async fn read_response(process: &mut BlenderProcess) -> Result<String, BlenderError> {
        let mut line = String::new();
        process
            .stdout
            .read_line(&mut line)
            .await
            .map_err(|e| BlenderError::IoError(e.to_string()))?;

        Ok(line.trim().to_string())
    }
}

/// Blender场景数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlenderScene {
    /// 场景名称
    pub name: String,

    /// 对象列表
    pub objects: Vec<BlenderObject>,

    /// 网格列表
    pub meshes: Vec<BlenderMesh>,

    /// 材质列表
    pub materials: Vec<BlenderMaterial>,
}

/// Blender对象
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlenderObject {
    /// 对象名称
    pub name: String,

    /// 位置
    pub location: [f32; 3],

    /// 旋转（欧拉角）
    pub rotation: [f32; 3],

    /// 缩放
    pub scale: [f32; 3],

    /// 网格索引
    pub mesh_index: Option<usize>,
}

/// Blender网格
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlenderMesh {
    /// 网格名称
    pub name: String,

    /// 顶点位置
    pub vertices: Vec<[f32; 3]>,

    /// 顶点法线
    pub normals: Vec<[f32; 3]>,

    /// UV坐标
    pub uvs: Vec<[f32; 2]>,

    /// 三角形索引
    pub triangles: Vec<[u32; 3]>,
}

/// Blender材质
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlenderMaterial {
    /// 材质名称
    pub name: String,

    /// 基础颜色
    pub base_color: [f32; 4],

    /// 金属度
    pub metallic: f32,

    /// 粗糙度
    pub roughness: f32,

    /// 发射强度
    pub emission: [f32; 3],
}

/// Blender错误
#[derive(thiserror::Error, Debug)]
pub enum BlenderError {
    #[error("Not connected to Blender")]
    NotConnected,

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),
}

/// Blender桥接管理器
pub struct BlenderBridgeManager {
    /// 活动的桥接
    bridges: Arc<Mutex<Vec<String>>>,
}

impl BlenderBridgeManager {
    pub fn new() -> Self {
        Self {
            bridges: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 创建新的桥接
    pub async fn create_bridge(
        &self,
        id: String,
        config: BlenderBridgeConfig,
    ) -> Result<BlenderBridge, BlenderError> {
        let mut bridge = BlenderBridge::new(config);
        bridge.connect().await?;
        Ok(bridge)
    }
}

impl Default for BlenderBridgeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blender_config_default() {
        let config = BlenderBridgeConfig::default();
        assert_eq!(config.port, 9876);
        assert!(config.background_mode);
    }

    #[test]
    fn test_blender_scene_serialization() {
        let scene = BlenderScene {
            name: "TestScene".to_string(),
            objects: vec![],
            meshes: vec![],
            materials: vec![],
        };

        let json = serde_json::to_string(&scene);
        assert!(json.is_ok());
    }

    #[test]
    fn test_blender_object() {
        let obj = BlenderObject {
            name: "Cube".to_string(),
            location: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            mesh_index: Some(0),
        };

        assert_eq!(obj.name, "Cube");
    }
}
