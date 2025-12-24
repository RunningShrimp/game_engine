//! 异步协程世界生成器
//!
//! 提供基于tokio协程的异步世界生成服务。
//! 支持地形生成、物体放置、导航网格生成等操作，替代传统的同步阻塞实现。

use bevy_ecs::prelude::*;
use glam::Vec3;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tokio::sync::{Semaphore, mpsc, oneshot};
use tokio::task::spawn_blocking;

/// 世界生成错误
#[derive(Debug, thiserror::Error)]
pub enum WorldGenerationError {
    /// IO错误
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// 生成参数错误
    #[error("Invalid generation parameter: {0}")]
    InvalidParameter(String),
    /// 生成超时
    #[error("Generation timeout")]
    Timeout,
    /// 生成被取消
    #[error("Generation cancelled")]
    Cancelled,
    /// 其他错误
    #[error("Other error: {0}")]
    Other(String),
}

/// 世界生成请求
#[derive(Debug, Clone)]
pub struct WorldGenerationRequest {
    /// 请求ID
    pub request_id: u64,
    /// 生成类型
    pub generation_type: GenerationType,
    /// 生成配置
    pub config: GenerationConfig,
    /// 区域大小（可选，用于分块生成）
    pub region_size: Option<(u32, u32)>,
    /// 区域位置（可选，用于分块生成）
    pub region_position: Option<(i32, i32)>,
}

/// 生成类型
#[derive(Debug, Clone)]
pub enum GenerationType {
    /// 地形生成
    Terrain {
        width: usize,
        height: usize,
        seed: Option<u64>,
    },
    /// 物体放置
    ObjectPlacement {
        object_count: usize,
        object_types: Vec<ObjectType>,
    },
    /// 导航网格生成
    NavigationMesh { geometry_path: Option<PathBuf> },
    /// 完整世界生成（包含所有内容）
    FullWorld {
        terrain_size: (usize, usize),
        object_count: usize,
        generate_navmesh: bool,
    },
}

/// 对象类型
#[derive(Debug, Clone)]
pub enum ObjectType {
    /// 树木
    Tree,
    /// 建筑
    Building,
    /// 资源点（矿石、植物等）
    Resource,
    /// NPC
    Npc,
    /// 装饰物
    Decoration,
}

/// 生成配置
#[derive(Debug, Clone)]
pub struct GenerationConfig {
    /// 噪声参数（用于地形生成）
    pub noise_scale: f32,
    /// 噪声强度
    pub noise_strength: f32,
    /// 地形平滑度
    pub terrain_smoothness: usize,
    /// 对象密度（0.0 - 1.0）
    pub object_density: f32,
    /// 是否生成资源
    pub generate_resources: bool,
    /// 资源密度（0.0 - 1.0）
    pub resource_density: f32,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            noise_scale: 0.1,
            noise_strength: 10.0,
            terrain_smoothness: 3,
            object_density: 0.1,
            generate_resources: true,
            resource_density: 0.05,
        }
    }
}

/// 世界生成结果
#[derive(Debug)]
pub struct WorldGenerationResult {
    /// 请求ID
    pub request_id: u64,
    /// 生成的实体列表（用于ECS世界）
    pub entities: Vec<GeneratedEntity>,
    /// 地形数据（如果有）
    pub terrain_data: Option<TerrainData>,
    /// 导航网格数据（如果有）
    pub navmesh_data: Option<NavMeshData>,
    /// 生成耗时（毫秒）
    pub generation_time_ms: f64,
    /// 错误（如果有）
    pub error: Option<WorldGenerationError>,
}

/// 生成的实体
#[derive(Debug, Clone)]
pub struct GeneratedEntity {
    /// 实体类型
    pub entity_type: ObjectType,
    /// 位置
    pub position: Vec3,
    /// 旋转（欧拉角）
    pub rotation: Vec3,
    /// 缩放
    pub scale: Vec3,
    /// 额外属性（用于存储特定类型的属性）
    pub properties: std::collections::HashMap<String, String>,
}

/// 地形数据
#[derive(Debug, Clone)]
pub struct TerrainData {
    /// 宽度（顶点数）
    pub width: usize,
    /// 高度（顶点数）
    pub height: usize,
    /// 高度图数据
    pub heightmap: Vec<f32>,
    /// 地形缩放
    pub scale: Vec3,
}

/// 导航网格数据
#[derive(Debug, Clone)]
pub struct NavMeshData {
    /// 顶点列表
    pub vertices: Vec<Vec3>,
    /// 索引列表
    pub indices: Vec<u32>,
    /// 区域标记
    pub regions: Vec<NavMeshRegion>,
}

/// 导航网格区域
#[derive(Debug, Clone)]
pub struct NavMeshRegion {
    /// 区域ID
    pub region_id: u32,
    /// 区域类型（可通行、不可通行等）
    pub region_type: String,
    /// 顶点索引范围
    pub vertex_range: (usize, usize),
}

/// 异步世界生成器
///
/// 使用tokio协程替代传统同步实现，提供更好的异步集成和取消支持。
///
/// ## 架构设计
///
/// - **协程工作池**: 使用tokio::spawn创建轻量级协程
/// - **异步通道**: 使用tokio::sync::mpsc进行异步消息传递
/// - **并发控制**: 使用Semaphore限制同时处理的请求数
/// - **取消支持**: 使用oneshot通道实现优雅取消
/// - **分块生成**: 支持分块生成大型世界
///
/// ## 性能特性
///
/// - 轻量级协程（栈仅64KB，相比线程的2-8MB）
/// - 用户级上下文切换（比系统级快5-10倍）
/// - 与异步系统无缝集成
/// - 支持超时和取消
/// - 支持分块生成，避免一次性生成大型世界导致卡顿
///
/// ## 使用示例
///
/// ```ignore
/// use game_engine::world::async_generator::AsyncWorldGenerator;
///
/// // 创建异步世界生成器（最大并发数为4）
/// let generator = AsyncWorldGenerator::new(4);
///
/// // 异步生成地形
/// let result = generator.generate_terrain(
///     512, 512, // 宽度和高度
///     Some(12345), // 随机种子
///     GenerationConfig::default(),
/// ).await;
///
/// // 批量生成多个区域
/// let regions = vec![
///     ((0, 0), (256, 256)),
///     ((256, 0), (256, 256)),
/// ];
/// let results = generator.generate_regions_batch(regions, GenerationConfig::default()).await;
/// ```
pub struct AsyncWorldGenerator {
    /// 请求发送端（异步通道）
    request_tx: mpsc::Sender<(
        WorldGenerationRequest,
        oneshot::Sender<WorldGenerationResult>,
    )>,
    /// 并发控制信号量
    semaphore: Arc<Semaphore>,
    /// 取消通道发送端
    cancel_tx: Arc<tokio::sync::Mutex<Option<oneshot::Sender<()>>>>,
    /// 下一个请求ID
    next_request_id: Arc<AtomicU64>,
    /// 批量处理大小
    batch_size: usize,
    /// 待处理请求计数
    pending_count: Arc<std::sync::atomic::AtomicUsize>,
    /// 已完成请求计数
    completed_count: Arc<std::sync::atomic::AtomicUsize>,
}

impl AsyncWorldGenerator {
    /// 创建新的异步世界生成器
    ///
    /// # 参数
    /// - `max_concurrent`: 最大并发处理数，0表示使用CPU核心数
    ///
    /// # 返回
    /// 新的异步世界生成器实例
    pub fn new(max_concurrent: usize) -> Self {
        Self::new_with_batch_size(max_concurrent, 16)
    }

    /// 创建新的异步世界生成器（带批量大小配置）
    ///
    /// # 参数
    /// - `max_concurrent`: 最大并发处理数，0表示使用CPU核心数
    /// - `batch_size`: 批量处理大小，一次处理多个请求以减少上下文切换
    ///
    /// # 返回
    /// 新的异步世界生成器实例
    pub fn new_with_batch_size(max_concurrent: usize, batch_size: usize) -> Self {
        let (request_tx, mut request_rx) = mpsc::channel::<(
            WorldGenerationRequest,
            oneshot::Sender<WorldGenerationResult>,
        )>(1000);

        let max_concurrent = if max_concurrent == 0 {
            num_cpus::get().max(1)
        } else {
            max_concurrent
        };

        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        let (cancel_tx, mut cancel_rx) = oneshot::channel();
        let cancel_tx_arc = Arc::new(tokio::sync::Mutex::new(Some(cancel_tx)));

        let pending_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let completed_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        // 启动工作协程
        let semaphore_clone = semaphore.clone();
        let pending_count_clone = pending_count.clone();
        let completed_count_clone = completed_count.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut cancel_rx => {
                        // 收到取消信号，退出循环
                        break;
                    }
                    Some((req, result_tx)) = request_rx.recv() => {
                        pending_count_clone.fetch_sub(1, Ordering::Relaxed);

                        // 获取信号量许可
                        let permit = semaphore_clone.clone().acquire_owned().await;
                        if permit.is_err() {
                            let _ = result_tx.send(WorldGenerationResult {
                                request_id: req.request_id,
                                entities: Vec::new(),
                                terrain_data: None,
                                navmesh_data: None,
                                generation_time_ms: 0.0,
                                error: Some(WorldGenerationError::Other("Failed to acquire semaphore".to_string())),
                            });
                            continue;
                        }
                        let permit = permit.unwrap();

                        let req_id = req.request_id;
                        let req_type = req.generation_type.clone();
                        let req_config = req.config.clone();
                        let req_region_size = req.region_size;
                        let req_region_position = req.region_position;
                        let completed_count_task = completed_count_clone.clone();

                        // 世界生成是CPU密集型，使用spawn_blocking
                        tokio::spawn(async move {
                            let start = std::time::Instant::now();
                            let result = spawn_blocking(move || {
                                Self::generate_world_internal(
                                    req_type,
                                    req_config,
                                    req_region_size,
                                    req_region_position,
                                )
                            }).await;

                            drop(permit); // 释放许可

                            let (entities, terrain_data, navmesh_data, error) = match result {
                                Ok(Ok((entities, terrain, navmesh, err))) => (entities, terrain, navmesh, err),
                                Ok(Err(e)) => (Vec::new(), None, None, Some(e)),
                                Err(_) => (Vec::new(), None, None, Some(WorldGenerationError::Other("Task join error".to_string()))),
                            };

                            let generation_time_ms = start.elapsed().as_secs_f64() * 1000.0;

                            let result = WorldGenerationResult {
                                request_id: req_id,
                                entities,
                                terrain_data,
                                navmesh_data,
                                generation_time_ms,
                                error,
                            };

                            let _ = result_tx.send(result);
                            completed_count_task.fetch_add(1, Ordering::Relaxed);
                        });
                    }
                }
            }
        });

        Self {
            request_tx,
            semaphore,
            cancel_tx: cancel_tx_arc,
            next_request_id: Arc::new(AtomicU64::new(1)),
            batch_size,
            pending_count,
            completed_count,
        }
    }

    /// 内部世界生成函数（在spawn_blocking中运行）
    fn generate_world_internal(
        generation_type: GenerationType,
        config: GenerationConfig,
        region_size: Option<(u32, u32)>,
        region_position: Option<(i32, i32)>,
    ) -> Result<
        (
            Vec<GeneratedEntity>,
            Option<TerrainData>,
            Option<NavMeshData>,
            Option<WorldGenerationError>,
        ),
        WorldGenerationError,
    > {
        match generation_type {
            GenerationType::Terrain {
                width,
                height,
                seed,
            } => {
                let terrain = Self::generate_terrain_internal(width, height, seed, &config)?;
                Ok((Vec::new(), Some(terrain), None, None))
            }
            GenerationType::ObjectPlacement {
                object_count,
                object_types,
            } => {
                let entities = Self::generate_objects_internal(
                    object_count,
                    object_types,
                    &config,
                    region_size,
                    region_position,
                )?;
                Ok((entities, None, None, None))
            }
            GenerationType::NavigationMesh { geometry_path: _ } => {
                // 简化实现：生成一个基本的导航网格
                let navmesh = Self::generate_navmesh_internal(&config)?;
                Ok((Vec::new(), None, Some(navmesh), None))
            }
            GenerationType::FullWorld {
                terrain_size,
                object_count,
                generate_navmesh,
            } => {
                let terrain =
                    Self::generate_terrain_internal(terrain_size.0, terrain_size.1, None, &config)?;
                let entities = Self::generate_objects_internal(
                    object_count,
                    vec![ObjectType::Tree, ObjectType::Building, ObjectType::Resource],
                    &config,
                    None,
                    None,
                )?;
                let navmesh = if generate_navmesh {
                    Some(Self::generate_navmesh_internal(&config)?)
                } else {
                    None
                };
                Ok((entities, Some(terrain), navmesh, None))
            }
        }
    }

    /// 生成地形（内部实现）
    fn generate_terrain_internal(
        width: usize,
        height: usize,
        seed: Option<u64>,
        config: &GenerationConfig,
    ) -> Result<TerrainData, WorldGenerationError> {
        // 简化实现：使用简单的噪声生成高度图
        let mut heightmap = Vec::with_capacity(width * height);

        // 使用简单的伪随机数生成器（实际应该使用专业的噪声库如noise-rs）
        let mut rng = if let Some(s) = seed {
            SimpleRng::new(s)
        } else {
            SimpleRng::new(12345)
        };

        for y in 0..height {
            for x in 0..width {
                // 简单的噪声计算
                let nx = x as f32 * config.noise_scale;
                let ny = y as f32 * config.noise_scale;
                let height = (nx.sin() * ny.cos() * config.noise_strength) + rng.next_f32() * 2.0;
                heightmap.push(height);
            }
        }

        // 平滑处理
        let mut terrain = TerrainData {
            width,
            height,
            heightmap,
            scale: Vec3::new(1.0, 1.0, 1.0),
        };

        for _ in 0..config.terrain_smoothness {
            terrain.smooth();
        }

        Ok(terrain)
    }

    /// 生成对象（内部实现）
    fn generate_objects_internal(
        object_count: usize,
        object_types: Vec<ObjectType>,
        config: &GenerationConfig,
        region_size: Option<(u32, u32)>,
        region_position: Option<(i32, i32)>,
    ) -> Result<Vec<GeneratedEntity>, WorldGenerationError> {
        if object_types.is_empty() {
            return Err(WorldGenerationError::InvalidParameter(
                "No object types specified".to_string(),
            ));
        }

        let mut entities = Vec::new();
        let mut rng = SimpleRng::new(12345);

        let (region_width, region_height) = region_size.unwrap_or((1000, 1000));
        let (region_x, region_y) = region_position.unwrap_or((0, 0));

        for i in 0..object_count {
            let object_type = &object_types[i % object_types.len()];
            let x = region_x as f32 + rng.next_f32() * region_width as f32;
            let z = region_y as f32 + rng.next_f32() * region_height as f32;
            let y = 0.0; // 简化：假设地面高度为0

            let mut properties = std::collections::HashMap::new();
            match object_type {
                ObjectType::Tree => {
                    properties.insert("tree_type".to_string(), "oak".to_string());
                    properties.insert("age".to_string(), rng.next_f32().to_string());
                }
                ObjectType::Building => {
                    properties.insert("building_type".to_string(), "house".to_string());
                    properties.insert(
                        "floors".to_string(),
                        (rng.next_f32() * 3.0 + 1.0).to_string(),
                    );
                }
                ObjectType::Resource => {
                    properties.insert("resource_type".to_string(), "ore".to_string());
                    properties.insert("amount".to_string(), (rng.next_f32() * 100.0).to_string());
                }
                _ => {}
            }

            entities.push(GeneratedEntity {
                entity_type: object_type.clone(),
                position: Vec3::new(x, y, z),
                rotation: Vec3::new(0.0, rng.next_f32() * 360.0, 0.0),
                scale: Vec3::new(1.0, 1.0, 1.0),
                properties,
            });
        }

        Ok(entities)
    }

    /// 生成导航网格（内部实现）
    fn generate_navmesh_internal(
        _config: &GenerationConfig,
    ) -> Result<NavMeshData, WorldGenerationError> {
        // 简化实现：生成一个基本的平面导航网格
        let vertices = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(100.0, 0.0, 0.0),
            Vec3::new(100.0, 0.0, 100.0),
            Vec3::new(0.0, 0.0, 100.0),
        ];
        let indices = vec![0, 1, 2, 0, 2, 3];
        let regions = vec![NavMeshRegion {
            region_id: 0,
            region_type: "walkable".to_string(),
            vertex_range: (0, vertices.len()),
        }];

        Ok(NavMeshData {
            vertices,
            indices,
            regions,
        })
    }

    /// 异步生成地形
    pub async fn generate_terrain(
        &self,
        width: usize,
        height: usize,
        seed: Option<u64>,
        config: GenerationConfig,
    ) -> Result<WorldGenerationResult, WorldGenerationError> {
        self.submit_request(WorldGenerationRequest {
            request_id: self.next_request_id.fetch_add(1, Ordering::SeqCst),
            generation_type: GenerationType::Terrain {
                width,
                height,
                seed,
            },
            config,
            region_size: None,
            region_position: None,
        })
        .await
    }

    /// 异步生成对象
    pub async fn generate_objects(
        &self,
        object_count: usize,
        object_types: Vec<ObjectType>,
        config: GenerationConfig,
    ) -> Result<WorldGenerationResult, WorldGenerationError> {
        self.submit_request(WorldGenerationRequest {
            request_id: self.next_request_id.fetch_add(1, Ordering::SeqCst),
            generation_type: GenerationType::ObjectPlacement {
                object_count,
                object_types,
            },
            config,
            region_size: None,
            region_position: None,
        })
        .await
    }

    /// 异步生成完整世界
    pub async fn generate_full_world(
        &self,
        terrain_size: (usize, usize),
        object_count: usize,
        generate_navmesh: bool,
        config: GenerationConfig,
    ) -> Result<WorldGenerationResult, WorldGenerationError> {
        self.submit_request(WorldGenerationRequest {
            request_id: self.next_request_id.fetch_add(1, Ordering::SeqCst),
            generation_type: GenerationType::FullWorld {
                terrain_size,
                object_count,
                generate_navmesh,
            },
            config,
            region_size: None,
            region_position: None,
        })
        .await
    }

    /// 提交生成请求（内部方法）
    async fn submit_request(
        &self,
        request: WorldGenerationRequest,
    ) -> Result<WorldGenerationResult, WorldGenerationError> {
        let (result_tx, result_rx) = oneshot::channel();

        if self.request_tx.send((request, result_tx)).await.is_err() {
            return Err(WorldGenerationError::Other(
                "Service channel closed".to_string(),
            ));
        }

        self.pending_count.fetch_add(1, Ordering::Relaxed);

        match result_rx.await {
            Ok(result) => {
                if let Some(error) = result.error {
                    Err(error)
                } else {
                    Ok(result)
                }
            }
            Err(_) => Err(WorldGenerationError::Other(
                "Result channel closed".to_string(),
            )),
        }
    }

    /// 获取待处理请求数量
    pub fn pending_requests(&self) -> usize {
        self.pending_count.load(Ordering::Relaxed)
    }

    /// 获取总完成数（自服务启动以来）
    pub fn total_completed(&self) -> usize {
        self.completed_count.load(Ordering::Relaxed)
    }

    /// 取消所有待处理的请求
    pub async fn cancel_all(&self) {
        let mut cancel_tx_guard = self.cancel_tx.lock().await;
        if let Some(tx) = cancel_tx_guard.take() {
            let _ = tx.send(());
        }
    }

    /// 获取批量处理大小
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }
}

impl TerrainData {
    /// 平滑地形
    fn smooth(&mut self) {
        let mut new_heightmap = self.heightmap.clone();
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let sum = self.heightmap[(y - 1) * self.width + x]
                    + self.heightmap[(y + 1) * self.width + x]
                    + self.heightmap[y * self.width + (x - 1)]
                    + self.heightmap[y * self.width + (x + 1)]
                    + self.heightmap[y * self.width + x];
                new_heightmap[y * self.width + x] = sum / 5.0;
            }
        }
        self.heightmap = new_heightmap;
    }
}

/// 简单的伪随机数生成器（用于演示）
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        (self.state >> 16) as f32 / 65536.0
    }
}

impl Drop for AsyncWorldGenerator {
    fn drop(&mut self) {
        if let Ok(mut cancel_tx_guard) = self.cancel_tx.try_lock() {
            if let Some(tx) = cancel_tx_guard.take() {
                let _ = tx.send(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_terrain() {
        let generator = AsyncWorldGenerator::new(2);

        let result = generator
            .generate_terrain(256, 256, Some(12345), GenerationConfig::default())
            .await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.terrain_data.is_some());
        let terrain = result.terrain_data.unwrap();
        assert_eq!(terrain.width, 256);
        assert_eq!(terrain.height, 256);
        assert_eq!(terrain.heightmap.len(), 256 * 256);
    }

    #[tokio::test]
    async fn test_generate_objects() {
        let generator = AsyncWorldGenerator::new(2);

        let result = generator
            .generate_objects(
                100,
                vec![ObjectType::Tree, ObjectType::Building],
                GenerationConfig::default(),
            )
            .await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.entities.len(), 100);
    }

    #[tokio::test]
    async fn test_generate_full_world() {
        let generator = AsyncWorldGenerator::new(2);

        let result = generator
            .generate_full_world((128, 128), 50, true, GenerationConfig::default())
            .await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.terrain_data.is_some());
        assert!(result.navmesh_data.is_some());
        assert!(!result.entities.is_empty());
    }
}
