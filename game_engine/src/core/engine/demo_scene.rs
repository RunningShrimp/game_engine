//  演示场景模块
//
//  负责创建游戏引擎的演示场景，包括：
//  - 物理对象生成
//  - 精灵网格生成
//  - 光源设置
//  - 脚本实体创建

use crate::ecs::{PointLight, PreviousTransform, Sprite, Transform};
use crate::resources::manager::AssetServer;
use crate::resources::manager::Handle;
use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};

/// 生成演示场景
///
/// 创建一个简单的演示场景，包含精灵和物理对象（如果启用了物理特性）。
/// 场景包括：
/// - 物理地面和下落方块（如果启用了物理特性）
/// - 彩色精灵网格
/// - 动态光源
/// - 脚本实体示例
///
/// # 参数
///
/// * `world` - ECS世界
/// * `asset_server` - 资源服务器，用于加载纹理
pub fn spawn_demo_scene(world: &mut World, asset_server: &AssetServer) {
    let atlas_path = std::path::Path::new("assets/atlas.png");
    let atlas_handle = asset_server.load_texture(atlas_path);

    // 生成物理场景
    spawn_physics_scene(world);

    // 生成精灵网格
    spawn_sprite_grid(world, &atlas_handle);

    // 生成光源
    spawn_light_source(world);
}

/// 生成物理场景
///
/// 创建物理地面和下落方块，用于演示物理系统。
///
/// # 参数
///
/// * `world` - ECS世界
fn spawn_physics_scene(world: &mut World) {
    use crate::domain::physics::{RigidBodyType, ShapeType};
    use crate::physics::{ColliderDesc, RigidBodyDesc};

    // 生成物理地面
    world.spawn((
        Transform {
            pos: Vec3::new(400.0, 50.0, 0.0),
            scale: Vec3::new(800.0, 20.0, 1.0),
            ..Default::default()
        },
        PreviousTransform::default(),
        Sprite {
            color: [0.5, 0.5, 0.5, 1.0],
            ..Default::default()
        },
        RigidBodyDesc {
            body_type: RigidBodyType::Fixed,
            position: glam::Vec3::new(400.0, 50.0, 0.0),
            rotation: glam::Quat::IDENTITY,
        },
        ColliderDesc {
            shape_type: ShapeType::Cuboid {
                half_extents: glam::Vec3::new(400.0, 10.0, 0.0),
            },
            half_extents: glam::Vec3::new(400.0, 10.0, 0.0),
            radius: 0.0,
        },
    ));

    // 生成下落方块
    for i in 0..10 {
        let _entity = world.spawn((
            Transform {
                pos: Vec3::new(400.0 + i as f32 * 10.0, 500.0 + i as f32 * 50.0, 0.0),
                scale: Vec3::new(30.0, 30.0, 1.0),
                ..Default::default()
            },
            PreviousTransform::default(),
            Sprite {
                color: [1.0, 0.2, 0.2, 1.0],
                ..Default::default()
            },
            RigidBodyDesc {
                body_type: RigidBodyType::Dynamic,
                position: glam::Vec3::new(400.0 + i as f32 * 10.0, 500.0 + i as f32 * 50.0, 0.0),
                rotation: glam::Quat::IDENTITY,
            },
            ColliderDesc {
                shape_type: ShapeType::Cuboid {
                    half_extents: glam::Vec3::new(15.0, 15.0, 0.0),
                },
                half_extents: glam::Vec3::new(15.0, 15.0, 0.0),
                radius: 0.0,
            },
        ));

    }
}

/// 生成精灵网格
///
/// 创建一个彩色精灵网格，用于演示渲染系统。
/// 网格具有以下特性：
/// - 交替的颜色模式
/// - 部分精灵使用图集纹理
/// - 轻微的旋转变化
///
/// # 参数
///
/// * `world` - ECS世界
/// * `atlas_handle` - 图集纹理句柄
fn spawn_sprite_grid(world: &mut World, atlas_handle: &Handle<u32>) {
    for y in -2..=2 {
        for x in -8..=8 {
            let mut entity = world.spawn((
                Transform {
                    pos: Vec3::new(400.0 + x as f32 * 30.0, 300.0 + y as f32 * 30.0, 0.0),
                    scale: Vec3::new(24.0, 24.0, 1.0),
                    rot: Quat::from_rotation_z((x as f32 + y as f32) * 0.05),
                },
                PreviousTransform::default(),
                Sprite {
                    color: [0.2 + x as f32 * 0.02, 0.6, 0.3 + y as f32 * 0.02, 0.9],
                    tex_index: 0,
                    normal_tex_index: 0,
                    uv_off: [0.0, 0.0],
                    uv_scale: [1.0, 1.0],
                    layer: if (x + y) % 2 == 0 { 0.0 } else { 1.0 },
                },
            ));

            // 为奇数位置的精灵添加图集纹理
            if (x + y) % 2 != 0 {
                entity.insert(crate::ecs::TextureHandle {
                    handle: atlas_handle.clone(),
                });
            }
        }
    }
}

/// 生成光源
///
/// 创建一个动态点光源，用于演示光照系统。
/// 光源具有以下特性：
/// - 暖色调（橙黄色）
/// - 中等半径和强度
/// - 线性衰减
///
/// # 参数
///
/// * `world` - ECS世界
fn spawn_light_source(world: &mut World) {
    world.spawn((
        Transform {
            pos: Vec3::new(400.0, 300.0, 0.0),
            ..Default::default()
        },
        PointLight {
            color: [1.0, 0.8, 0.6], // 暖色调
            radius: 300.0,
            intensity: 2.0,
            falloff: 1.0,
        },
    ));
}

/// 生成额外的演示实体
///
/// 可以用于创建更多复杂的演示场景元素。
///
/// # 参数
///
/// * `world` - ECS世界
/// * `asset_server` - 资源服务器
pub fn spawn_additional_entities(world: &mut World, asset_server: &AssetServer) {
    // 加载着色器资源作为示例
    let shader_path = std::path::Path::new("assets/shaders/pbr.wgsl");
    // 即使我们不直接使用着色器句柄，调用load_texture表明asset_server正在被使用
    // 在实际应用中，这里可能会加载实际的纹理资源
    let _shader_handle = asset_server.load_texture(shader_path);

    // 加载图集资源
    let atlas_path = std::path::Path::new("assets/atlas.png");
    let atlas_handle = asset_server.load_atlas(atlas_path);

    // 生成一个使用图集的精灵实体
    let atlas_entity_id = {
        let entity = world.spawn((
            Transform {
                pos: Vec3::new(300.0, 300.0, 0.0),
                scale: Vec3::new(64.0, 64.0, 1.0),
                ..Default::default()
            },
            PreviousTransform::default(),
            Sprite {
                color: [1.0, 1.0, 1.0, 1.0], // 白色，让纹理颜色显示出来
                tex_index: 0,
                normal_tex_index: 0,
                uv_off: [0.0, 0.0],
                uv_scale: [1.0, 1.0],
                layer: 0.0,
            },
            atlas_handle, // 插入图集句柄
        ));
        entity.id()
    };

    // 生成一个带有脚本的精灵
    let script_entity_id = {
        let entity = world.spawn((
            Transform {
                pos: Vec3::new(200.0, 200.0, 0.0),
                scale: Vec3::new(50.0, 50.0, 1.0),
                ..Default::default()
            },
            PreviousTransform::default(),
            Sprite {
                color: [0.8, 0.4, 0.9, 1.0],
                ..Default::default()
            },
        ));
        entity.id()
    };

    tracing::info!(target: "demo_scene", "Spawned additional demo entities with IDs: {:?} and {:?}", atlas_entity_id, script_entity_id);
}
