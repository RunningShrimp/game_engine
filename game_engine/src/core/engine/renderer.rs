//  渲染模块
//
//  负责游戏引擎的渲染逻辑，包括：
//  - 场景渲染
//  - 光照处理
//  - 相机管理
//  - 渲染统计更新
//  - 性能监控

use crate::ecs::{Camera, PointLight, Projection, Transform};
use crate::platform::run_sync;
use crate::platform::winit::WinitWindow;
use crate::render::wgpu_utils::{GpuPointLight, WgpuRenderer};
use crate::services::render::RenderService;
use bevy_ecs::prelude::*;
use glam::Mat4;

use crate::core::resources::RenderStats;

/// 渲染帧
///
/// 执行完整的渲染流程，包括：
/// - 编辑器UI渲染
/// - 场景构建和视锥剔除
/// - 光照提取
/// - 相机设置
/// - PBR场景渲染
/// - 材质更新
/// - 渲染统计更新
///
/// # 参数
///
/// * `world` - ECS世界
/// * `renderer` - wgpu渲染器
/// * `editor_ctx` - 编辑器上下文
/// * `render_service` - 渲染服务
/// * `render_cache` - 渲染缓存
/// * `window` - 窗口实例
pub fn render(
    world: &mut World,
    renderer: &mut WgpuRenderer,
    editor_ctx: &mut crate::editor::EditorContext,
    render_service: &mut RenderService,
    render_cache: &mut crate::render::graph::RenderCache,
    window: &WinitWindow,
) {
    let Some(raw_window) = window.raw() else {
        tracing::warn!("Window not initialized, skipping rendering");
        return;
    };

    let entity_count = world.entities().len();
    let _frame_span = crate::performance::tracing_metrics::TracingMetricsManager::frame_span(
        entity_count as usize,
        raw_window.scale_factor(),
    )
    .entered();

    // Editor UI
    editor_ctx.begin_frame(raw_window);
    // TODO: 实现世界检查UI
    // crate::editor::inspect_world_ui(&editor_ctx.context, world);
    let egui_primitives = editor_ctx.end_frame(raw_window);
    let pixels_per_point = raw_window.scale_factor() as f32;

    // Render with frustum culling
    let (layer_tree, culled, total) = crate::render::graph::build_from_world_culled(world);
    render_cache.culled_count = culled;
    render_cache.total_count = total;
    let _instances = render_cache.update(layer_tree);
    // 实例数量已由render_cache.update()内部记录

    // Extract Lights
    let lights = extract_lights(world);
    renderer.set_lights(lights);

    // Build Scene (PBR)
    let scene = render_service.build_pbr_scene(world);

    // Camera setup
    let (view_proj, camera_pos) = setup_camera(world, renderer);

    // Render the scene with egui renderer
    render_pbr_scene(
        world,
        renderer,
        render_service,
        &scene,
        view_proj,
        camera_pos,
        editor_ctx.egui_renderer.as_mut(),
        &egui_primitives,
        pixels_per_point,
    );

    // Update materials
    update_materials(world, renderer);

    // Update render statistics
    update_render_stats(world, renderer, culled, total, window);
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Quat, Vec3};

    #[test]
    fn test_extract_lights_empty() {
        let mut world = World::new();
        let lights = extract_lights(&mut world);
        assert_eq!(lights.len(), 0);
    }

    #[test]
    fn test_extract_lights_with_point_light() {
        let mut world = World::new();

        // 创建一个带点光源的实体
        let entity = world
            .spawn((
                Transform {
                    pos: Vec3::new(1.0, 2.0, 3.0),
                    rot: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
                PointLight {
                    color: [1.0, 1.0, 1.0],
                    intensity: 1.0,
                    radius: 10.0,
                    falloff: 1.0,
                },
            ))
            .id();

        let lights = extract_lights(&mut world);
        assert_eq!(lights.len(), 1);
        assert_eq!(lights[0].pos, [1.0, 2.0]);
    }

    #[test]
    fn test_setup_camera_no_camera() {
        let mut world = World::new();
        // 没有相机时应该返回默认值
        // 注意：这个测试可能需要mock renderer
    }

    #[test]
    fn test_setup_camera_with_camera() {
        let mut world = World::new();

        // 创建一个带相机的实体
        world.spawn((
            Transform {
                pos: Vec3::new(0.0, 0.0, 5.0),
                rot: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            Camera {
                projection: Projection::Perspective {
                    fov: 60.0,
                    aspect: 16.0 / 9.0,
                    near: 0.1,
                    far: 100.0,
                },
                is_active: true,
            },
        ));

        // 注意：这个测试需要mock renderer，暂时跳过
    }
}

/// 提取光源
///
/// 从ECS世界中提取所有点光源并转换为GPU格式。
///
/// # 参数
///
/// * `world` - ECS世界
///
/// # 返回
///
/// GPU格式的点光源列表
fn extract_lights(world: &mut World) -> Vec<GpuPointLight> {
    let mut lights = Vec::new();
    let mut query = world.query::<(&Transform, &PointLight)>();
    for (t, l) in query.iter(world) {
        lights.push(GpuPointLight {
            pos: [t.pos.x, t.pos.y],
            color: l.color,
            radius: l.radius,
            intensity: l.intensity,
            falloff: l.falloff,
            _pad: [0.0, 0.0],
        });
    }
    lights
}

/// 设置相机
///
/// 从ECS世界中提取活跃相机的变换和投影参数，
/// 计算视图-投影矩阵和相机位置。
///
/// # 参数
///
/// * `world` - ECS世界
/// * `renderer` - wgpu渲染器
///
/// # 返回
///
/// (视图-投影矩阵, 相机位置)
fn setup_camera(world: &mut World, renderer: &WgpuRenderer) -> ([[f32; 4]; 4], [f32; 3]) {
    let mut view_proj = glam::Mat4::IDENTITY.to_cols_array_2d();
    let mut camera_pos = [0.0; 3];
    let mut query_cam = world.query::<(&Transform, &Camera)>();

    for (t, c) in query_cam.iter(world) {
        if c.is_active {
            camera_pos = t.pos.to_array();
            let view = glam::Mat4::from_rotation_translation(t.rot, t.pos).inverse();
            let proj = calculate_projection_matrix(c, renderer);
            view_proj = (proj * view).to_cols_array_2d();
            break;
        }
    }

    (view_proj, camera_pos)
}

/// 计算投影矩阵
///
/// 根据相机的投影类型计算相应的投影矩阵。
///
/// # 参数
///
/// * `camera` - 相机组件
/// * `renderer` - wgpu渲染器
///
/// # 返回
///
/// 投影矩阵
fn calculate_projection_matrix(camera: &Camera, renderer: &WgpuRenderer) -> Mat4 {
    match camera.projection {
        Projection::Orthographic { scale, near, far } => {
            let aspect = renderer.config().width as f32 / renderer.config().height as f32;
            glam::Mat4::orthographic_rh(-aspect * scale, aspect * scale, -scale, scale, near, far)
        }
        Projection::Perspective {
            fov,
            aspect,
            near,
            far,
        } => glam::Mat4::perspective_rh(fov, aspect, near, far),
    }
}

/// 渲染PBR场景
///
/// 执行基于物理渲染的场景渲染流程。
///
/// # 参数
///
/// * `world` - ECS世界
/// * `renderer` - wgpu渲染器
/// * `render_service` - 渲染服务
/// * `scene` - PBR场景
/// * `view_proj` - 视图-投影矩阵
/// * `camera_pos` - 相机位置
/// * `egui_renderer` - egui渲染器（可选）
/// * `egui_shapes` - egui形状
/// * `pixels_per_point` - 像素密度
fn render_pbr_scene(
    world: &mut World,
    renderer: &mut WgpuRenderer,
    render_service: &mut RenderService,
    scene: &crate::services::render::PbrScene,
    view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    _egui_renderer: Option<&mut egui_wgpu::Renderer>,
    egui_primitives: &[egui::ClippedPrimitive],
    pixels_per_point: f32,
) {
    let batch_count = world
        .get_resource::<crate::render::instance_batch::BatchManager>()
        .map(|bm| bm.stats.total_batches)
        .unwrap_or(0);
    let _render_span =
        crate::performance::tracing_metrics::TracingMetricsManager::render_submit_span(
            batch_count as usize,
            egui_primitives.len(),
        )
        .entered();
    if let Some(mut bm) = world.get_resource_mut::<crate::render::instance_batch::BatchManager>() {
        renderer.upload_batches(&mut bm);
        if let Err(e) = render_service.paint_pbr(
            renderer,
            &mut bm,
            scene,
            view_proj,
            camera_pos,
            _egui_renderer,
            egui_primitives,
            pixels_per_point,
        ) {
            tracing::warn!(target: "render", "Render error: {}", e);
        }
    }
}

/// 更新材质
///
/// 处理材质的待更新队列，将材质参数更新到GPU。
///
/// # 参数
///
/// * `world` - ECS世界
/// * `renderer` - wgpu渲染器
fn update_materials(world: &mut World, renderer: &mut WgpuRenderer) {
    // Flush material pending updates
    let updates = if let Some(mut pend) =
        world.get_resource_mut::<crate::resources::manager::MaterialPendingUpdates>()
    {
        pend.take_all()
    } else {
        Vec::new()
    };

    if !updates.is_empty()
        && let Some(mut reg) =
            world.get_resource_mut::<crate::resources::manager::MaterialRegistry>()
        && let Some(ref pbr) = renderer.pbr_renderer
    {
        for (id, mat) in updates {
            reg.update_material_params(renderer.device(), renderer.queue(), pbr, id, &mat);
        }
    }
}

/// 更新渲染统计
///
/// 收集和更新各种渲染性能指标，包括：
/// - GPU时间
/// - 绘制调用数
/// - 实例数
/// - 批处理统计
/// - 性能警告
/// - CSV日志记录
///
/// # 参数
///
/// * `world` - ECS世界
/// * `renderer` - wgpu渲染器
/// * `culled` - 剔除的对象数
/// * `total` - 总对象数
/// * `window` - 窗口实例
fn update_render_stats(
    world: &mut World,
    renderer: &WgpuRenderer,
    culled: u32,
    total: u32,
    window: &WinitWindow,
) {
    // Update GPU timing
    if let Some((_t0, dt)) = renderer.gpu_timings_ms()
        && let Some(mut stats) = world.get_resource_mut::<RenderStats>()
    {
        stats.gpu_pass_ms = Some(dt);
    }

    // Update draw call and instance statistics
    let (dc, ic) = renderer.draw_stats();
    let bm_stats = world
        .get_resource::<crate::render::instance_batch::BatchManager>()
        .map(|bm| bm.stats);

    if let Some(mut stats) = world.get_resource_mut::<RenderStats>() {
        stats.draw_calls = dc;
        stats.instances = ic;
        stats.passes = renderer.pass_count();
        stats.culled_objects = culled;
        stats.total_objects = total;

        if let Some(bms) = bm_stats {
            stats.batch_total = bms.total_batches;
            stats.batch_instances = bms.total_instances;
            stats.batch_saved_draw_calls = bms.saved_draw_calls;
            stats.batch_small_draw_calls = bms.small_draw_calls;
            stats.batch_visible_batches = bms.visible_batches;
        }

        // Update timing statistics
        let (upload, main, ui) = renderer.stage_timings_ms();
        stats.upload_ms = upload;
        stats.main_ms = main;
        stats.ui_ms = ui;
        stats.offscreen_ms = renderer.offscreen_timing_ms();

        // Performance warnings
        check_performance_warnings(&mut stats);

        // Write CSV log
        write_render_stats_csv(&stats, window);
    }
}

/// 检查性能警告
///
/// 检查各种渲染性能指标并在超出阈值时记录警告。
///
/// # 参数
///
/// * `stats` - 渲染统计
fn check_performance_warnings(stats: &mut RenderStats) {
    // Upload time warning
    if let Some(u) = stats.upload_ms
        && u > 2.0
    {
        stats.alerts_upload += 1;
        tracing::warn!(target: "render_perf", "Upload time too high: {:.2}ms", u);
    }

    // Main render time warning
    if let Some(m) = stats.main_ms
        && m > 16.7
    {
        stats.alerts_main += 1;
        tracing::warn!(target: "render_perf", "Main render time too high: {:.2}ms", m);
    }

    // UI render time warning
    if let Some(u) = stats.ui_ms
        && u > 4.0
    {
        stats.alerts_ui += 1;
        tracing::warn!(target: "render_perf", "UI render time too high: {:.2}ms", u);
    }

    // Offscreen render time warning
    if let Some(o) = stats.offscreen_ms
        && o > 8.0
    {
        stats.alerts_offscreen += 1;
        tracing::warn!(target: "render_perf", "Offscreen render time too high: {:.2}ms", o);
    }
}

/// 写入渲染统计CSV
///
/// 将渲染统计信息写入CSV文件用于性能分析。
///
/// # 参数
///
/// * `stats` - 渲染统计
/// * `window` - 窗口实例
fn write_render_stats_csv(stats: &RenderStats, window: &WinitWindow) {
    let path = std::env::temp_dir().join("render_stats.csv");
    let scale_factor = window.raw().map(|w| w.scale_factor()).unwrap_or(1.0);
    let _ = {
        let line = format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            stats.draw_calls,
            stats.instances,
            stats.passes,
            stats.upload_ms.unwrap_or(0.0),
            stats.main_ms.unwrap_or(0.0),
            stats.ui_ms.unwrap_or(0.0),
            stats.offscreen_ms.unwrap_or(0.0),
            scale_factor,
            stats.batch_total,
            stats.batch_instances,
            stats.batch_saved_draw_calls,
            stats.batch_small_draw_calls,
            stats.batch_visible_batches
        );

        let path_clone = path.clone();
        let line_clone = line.clone();
        let _ = run_sync(async move {
            use tokio::io::AsyncWriteExt;
            if let Ok(mut f) =
                tokio::fs::OpenOptions::new().create(true).append(true).open(&path_clone).await
            {
                f.write_all(line_clone.as_bytes()).await.ok()
            } else {
                None
            }
        });

        Some(())
    };
}

/// 获取渲染信息
///
/// 返回当前渲染系统的详细信息，用于调试和性能分析。
///
/// # 参数
///
/// * `world` - ECS世界
/// * `renderer` - wgpu渲染器
///
/// # 返回
///
/// 渲染信息字符串
#[allow(dead_code)]
pub fn get_render_info(world: &World, renderer: &WgpuRenderer) -> String {
    let (dc, ic) = renderer.draw_stats();
    let bm_stats = world
        .get_resource::<crate::render::instance_batch::BatchManager>()
        .map(|bm| bm.stats);

    format!(
        "Draw Calls: {}, Instances: {}, Passes: {}, Batches: {}",
        dc,
        ic,
        renderer.pass_count(),
        bm_stats.map(|s| s.total_batches).unwrap_or(0)
    )
}
