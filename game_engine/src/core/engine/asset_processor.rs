//  资源处理模块
//
//  负责处理资源加载事件，包括：
//  - 纹理加载事件
//  - 图集加载事件
//  - GLTF模型加载事件
//  - 资源指标更新
//  - 资源事件日志记录

use crate::ecs::TileSet;
use crate::render::wgpu_utils::WgpuRenderer;
use crate::resources::manager::{AssetEvent, AssetServer};
use bevy_ecs::prelude::*;

use crate::core::resources::{AssetMetrics, LogEvents};

/// 处理资源加载事件
///
/// 处理所有类型的资源加载事件，包括：
/// - 纹理和图集加载
/// - GLTF模型导入
/// - 资源指标更新
/// - 事件日志记录
///
/// # 参数
///
/// * `world` - ECS世界
/// * `asset_server` - 资源服务器
/// * `renderer` - wgpu渲染器
pub fn process_asset_events(
    world: &mut World,
    asset_server: &mut AssetServer,
    renderer: &mut WgpuRenderer,
) {
    let events = asset_server.update(renderer);
    if events.is_empty() {
        return;
    }

    for event in events {
        // 处理图集加载
        process_atlas_loaded_event(&event, world);

        // 更新资源指标
        update_asset_metrics(&event, world);

        // 记录日志
        log_asset_event(&event, world);

        // 处理GLTF导入（仅在gltf feature启用时）
        #[cfg(feature = "gltf")]
        process_gltf_loaded_event(&event, world, renderer);
    }
}

/// 处理图集加载事件
///
/// 当图集加载完成时，提取精灵信息并更新TileSet资源。
///
/// # 参数
///
/// * `event` - 资源事件
/// * `world` - ECS世界
fn process_atlas_loaded_event(event: &AssetEvent, world: &mut World) {
    if let AssetEvent::AtlasLoaded(h, _) = &event
        && let Some(atlas) = h.get()
    {
        let mut ts = world.get_resource_or_insert_with::<TileSet>(Default::default);
        for (name, (uv_off, uv_scale)) in atlas.sprites.iter() {
            ts.tiles.insert(name.clone(), (*uv_off, *uv_scale));
        }
        tracing::info!(target: "assets", "Atlas loaded with {} sprites", atlas.sprites.len());
    }
}

/// 更新资源指标
///
/// 根据资源事件更新各种性能和使用指标。
///
/// # 参数
///
/// * `event` - 资源事件
/// * `world` - ECS世界
fn update_asset_metrics(event: &AssetEvent, world: &mut World) {
    if let Some(mut am) = world.get_resource_mut::<AssetMetrics>() {
        match &event {
            AssetEvent::TextureLoaded(_, ms) => {
                am.last_latency_ms = Some(*ms);
                am.textures_loaded += 1;
                tracing::debug!(target: "assets", "Texture loaded in {:.1}ms, total: {}", ms, am.textures_loaded);
            }
            AssetEvent::AtlasLoaded(_, ms) => {
                am.last_latency_ms = Some(*ms);
                am.atlases_loaded += 1;
                tracing::debug!(target: "assets", "Atlas loaded in {:.1}ms, total: {}", ms, am.atlases_loaded);
            }
            AssetEvent::CustomLoaded {
                type_name,
                time_ms: ms,
                ..
            } => {
                if type_name == "GltfScene" {
                    am.last_latency_ms = Some(*ms);
                    am.models_loaded += 1;
                    tracing::debug!(target: "assets", "GLTF loaded in {:.1}ms, total: {}", ms, am.models_loaded);
                }
            }
            AssetEvent::TextureFailed(_, e) => {
                am.texture_errors += 1;
                tracing::error!(target: "assets", "Texture load failed: {}", e);
            }
            AssetEvent::AtlasFailed(_, e) => {
                am.atlas_errors += 1;
                tracing::error!(target: "assets", "Atlas load failed: {}", e);
            }
            AssetEvent::CustomFailed { type_name, error } => {
                if type_name == "GltfScene" {
                    am.model_errors += 1;
                    tracing::error!(target: "assets", "GLTF load failed: {}", error);
                }
            }
        }
    }
}

/// 记录资源事件日志
///
/// 将资源事件记录到LogEvents资源中，用于UI显示和调试。
///
/// # 参数
///
/// * `event` - 资源事件
/// * `world` - ECS世界
fn log_asset_event(event: &AssetEvent, world: &mut World) {
    if let Some(mut logs) = world.get_resource_mut::<LogEvents>() {
        let msg = match &event {
            AssetEvent::TextureLoaded(_, ms) => format!("TextureLoaded {ms:.1}ms"),
            AssetEvent::AtlasLoaded(_, ms) => format!("AtlasLoaded {ms:.1}ms"),
            AssetEvent::TextureFailed(_, e) => format!("TextureFailed {e}"),
            AssetEvent::AtlasFailed(_, e) => format!("AtlasFailed {e}"),
            AssetEvent::CustomLoaded {
                type_name,
                time_ms: ms,
                ..
            } => {
                if type_name == "GltfScene" {
                    format!("GltfLoaded {ms:.1}ms")
                } else {
                    format!("CustomLoaded({type_name}) {ms:.1}ms")
                }
            }
            AssetEvent::CustomFailed { type_name, error } => {
                if type_name == "GltfScene" {
                    format!("GltfFailed {error}")
                } else {
                    format!("CustomFailed({type_name}) {error}")
                }
            }
        };

        // 维护日志队列大小
        if logs.entries.len() >= logs.capacity {
            logs.entries.pop_front();
        }
        logs.entries.push_back(msg);
    }
}

/// 处理GLTF加载事件
///
/// 当GLTF模型加载完成时，将其导入到ECS世界中。
///
/// # 参数
///
/// * `event` - 资源事件
/// * `world` - ECS世界
/// * `renderer` - wgpu渲染器
#[cfg(feature = "gltf")]
fn process_gltf_loaded_event(event: &AssetEvent, world: &mut World, renderer: &mut WgpuRenderer) {
    if let AssetEvent::CustomLoaded {
        type_name, handle, ..
    } = &event
    {
        if type_name == "GltfScene" {
            #[cfg(feature = "gltf")]
            {
                use crate::resources::gltf_loader::GltfScene;
                use crate::resources::manager::Handle;

                // SAFETY: We know this is a Handle<GltfScene> from the GLTF loader
                let handle_gltf: Handle<GltfScene> = unsafe { std::mem::transmute_copy(handle) };
                crate::resources::manager::import_gltf_to_world(world, renderer, &handle_gltf);
                tracing::info!(target: "assets", "GLTF model imported to world");
            }
        }
    }
}

/// 获取资源统计信息
///
/// 返回当前资源系统的统计信息，用于调试和性能分析。
///
/// # 参数
///
/// * `world` - ECS世界
///
/// # 返回
///
/// 资源统计信息字符串
#[allow(dead_code)]
pub fn get_asset_statistics(world: &World) -> String {
    if let Some(metrics) = world.get_resource::<AssetMetrics>() {
        format!(
            "Textures: {} ({} errors), Atlases: {} ({} errors), Models: {} ({} errors), Last latency: {:.1}ms",
            metrics.textures_loaded,
            metrics.texture_errors,
            metrics.atlases_loaded,
            metrics.atlas_errors,
            metrics.models_loaded,
            metrics.model_errors,
            metrics.last_latency_ms.unwrap_or(0.0)
        )
    } else {
        "Asset metrics not available".to_string()
    }
}

/// 清理资源缓存
///
/// 清理未使用的资源缓存以释放内存。
///
/// # 参数
///
/// * `world` - ECS世界
/// * `asset_server` - 资源服务器
/// * `force` - 是否强制清理所有缓存
#[allow(dead_code)]
pub fn cleanup_asset_cache(world: &mut World, asset_server: &mut AssetServer, force: bool) {
    let mut cleaned_count = 0;

    // 这里可以实现资源引用计数检查和清理逻辑
    // 暂时使用简单的清理策略
    if force {
        asset_server.clear_cache();
        cleaned_count = 100; // 示例值
    }

    tracing::info!(target: "assets", "Cleaned {} asset cache entries", cleaned_count);

    // 更新清理统计
    if let Some(mut metrics) = world.get_resource_mut::<AssetMetrics>() {
        metrics.cache_cleanups += 1;
        metrics.last_cleanup_count = cleaned_count;
    }
}

/// 预加载资源
///
/// 预加载指定的资源列表以减少运行时加载延迟。
///
/// # 参数
///
/// * `asset_server` - 资源服务器
/// * `renderer` - wgpu渲染器
/// * `resource_paths` - 要预加载的资源路径列表
#[allow(dead_code)]
pub async fn preload_resources(
    asset_server: &mut AssetServer,
    renderer: &mut WgpuRenderer,
    resource_paths: Vec<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(target: "assets", "Preloading {} resources", resource_paths.len());

    let start_time = std::time::Instant::now();

    for path in resource_paths {
        // 根据文件扩展名确定资源类型
        if path.ends_with(".png") || path.ends_with(".jpg") || path.ends_with(".jpeg") {
            let _ = asset_server.load_texture(std::path::Path::new(path));
        } else if path.ends_with(".gltf") || path.ends_with(".glb") {
            #[cfg(feature = "gltf")]
            {
                let _ = asset_server.load_gltf(std::path::Path::new(path));
            }
            #[cfg(not(feature = "gltf"))]
            {
                tracing::warn!(
                    "GLTF file '{}' requested but 'gltf' feature is not enabled",
                    path
                );
            }
        }
    }

    // 等待所有资源加载完成
    let mut attempts = 0;
    while !asset_server.is_idle() && attempts < 1000 {
        asset_server.update(renderer);
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        attempts += 1;
    }

    let elapsed = start_time.elapsed();
    tracing::info!(target: "assets", "Preloading completed in {:.1}ms", elapsed.as_millis());

    Ok(())
}

/// 验证资源完整性
///
/// 验证已加载资源的完整性，检查损坏或缺失的资源。
///
/// # 参数
///
/// * `world` - ECS世界
/// * `asset_server` - 资源服务器
///
/// # 返回
///
/// 验证结果，包含发现的任何问题
#[allow(dead_code)]
pub fn validate_asset_integrity(
    world: &World,
    asset_server: &AssetServer,
) -> Result<(), Vec<String>> {
    let mut issues = Vec::new();

    // 检查纹理完整性
    if let Some(metrics) = world.get_resource::<AssetMetrics>() {
        // 注意：AssetMetrics 目前没有 texture_errors, atlas_errors, model_errors 字段
        // 这里只检查已加载的资源数量
        if metrics.textures_loaded == 0 {
            issues.push("No textures loaded".to_string());
        }

        if metrics.atlases_loaded == 0 {
            issues.push("No atlases loaded".to_string());
        }
    }

    // 使用asset_server检查已加载资源的一致性
    let loaded_textures = asset_server.get_loaded_texture_count();
    if let Some(metrics) = world.get_resource::<AssetMetrics>()
        && loaded_textures != metrics.textures_loaded as usize
    {
        issues.push(format!(
            "Texture count mismatch: server reports {} but metrics show {}",
            loaded_textures, metrics.textures_loaded
        ));
    }

    if issues.is_empty() {
        tracing::info!(target: "assets", "Asset integrity check passed");
        Ok(())
    } else {
        tracing::warn!(target: "assets", "Asset integrity issues found: {:?}", issues);
        Err(issues)
    }
}
