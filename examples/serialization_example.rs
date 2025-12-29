// 序列化系统示例
//
// 演示游戏引擎的序列化功能，包括：
// - 游戏状态保存/加载
// - 场景序列化
// - 资源元数据管理
// - 多种格式支持（RON, Bincode, JSON）

use game_engine::ecs::Transform;
use game_engine::serialization::{
    GameTime, GameState, GameStateMetadata, PlayerProgress, ResourceIndex, ResourceMetadata,
    ResourceType, SerializationFormat,
};
use game_engine::scene::SerializedScene;
use bevy_ecs::prelude::World;
use glam::{Quat, Vec3};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 游戏引擎序列化系统演示 ===\n");

    // 1. 游戏状态序列化示例
    println!("1. 游戏状态序列化示例");
    game_state_example()?;

    // 2. 场景序列化示例
    println!("\n2. 场景序列化示例");
    scene_serialization_example()?;

    // 3. 资源元数据示例
    println!("\n3. 资源元数据示例");
    resource_metadata_example()?;

    // 4. 版本管理示例
    println!("\n4. 版本管理示例");
    version_management_example()?;

    // 5. 多格式序列化对比
    println!("\n5. 多格式序列化对比");
    format_comparison_example()?;

    println!("\n=== 序列化系统演示完成 ===");
    Ok(())
}

/// 游戏状态序列化示例
fn game_state_example() -> Result<(), Box<dyn std::error::Error>> {
    // 创建World
    let mut world = World::new();

    // 添加一些实体
    world.spawn(Transform {
        pos: Vec3::new(1.0, 2.0, 3.0),
        rot: Quat::IDENTITY,
        scale: Vec3::ONE,
    });

    world.spawn(Transform {
        pos: Vec3::new(4.0, 5.0, 6.0),
        rot: Quat::IDENTITY,
        scale: Vec3::new(2.0, 2.0, 2.0),
    });

    // 创建游戏状态
    let mut game_state = GameState::from_world(&mut world, "示例存档");

    // 设置玩家进度
    let progress = PlayerProgress {
        current_level: "关卡3".to_string(),
        unlocked_levels: vec![
            "关卡1".to_string(),
            "关卡2".to_string(),
            "关卡3".to_string(),
        ],
        score: 12500,
        playtime_seconds: 7200,
    };
    game_state.set_progress(progress);

    // 设置全局变量
    game_state.set_global_variable("难度".to_string(), "困难".to_string());
    game_state.set_global_variable("语言".to_string(), "中文".to_string());

    println!("  - 创建游戏状态，包含 {} 个实体", game_state.scenes.len());
    println!("  - 玩家进度: {} 分，时长: {} 秒", progress.score, progress.playtime_seconds);

    // 保存为不同格式
    let save_dir = "/tmp/game_engine_saves";
    std::fs::create_dir_all(save_dir)?;

    let ron_path = format!("{}/save.ron", save_dir);
    game_state.save_to_file(&ron_path, SerializationFormat::Ron)?;
    println!("  - 保存为 RON 格式: {}", ron_path);

    let bin_path = format!("{}/save.bin", save_dir);
    game_state.save_to_file(&bin_path, SerializationFormat::Bincode)?;
    println!("  - 保存为 Bincode 格式: {}", bin_path);

    let json_path = format!("{}/save.json", save_dir);
    game_state.save_to_file(&json_path, SerializationFormat::Json)?;
    println!("  - 保存为 JSON 格式: {}", json_path);

    // 加载游戏状态
    let loaded_state = GameState::load_from_file(&ron_path, SerializationFormat::Ron)?;
    println!("  - 成功加载游戏状态");
    println!("  - 存档名称: {}", loaded_state.metadata.save_name);
    println!(
        "  - 玩家等级: {}",
        loaded_state.get_progress().current_level
    );

    Ok(())
}

/// 场景序列化示例
fn scene_serialization_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut world = World::new();

    // 创建场景实体
    let player = world.spawn(Transform {
        pos: Vec3::new(0.0, 0.0, 0.0),
        rot: Quat::IDENTITY,
        scale: Vec3::ONE,
    });

    let enemy = world.spawn(Transform {
        pos: Vec3::new(10.0, 0.0, 5.0),
        rot: Quat::IDENTITY,
        scale: Vec3::ONE,
    });

    let camera = world.spawn(Transform {
        pos: Vec3::new(0.0, 5.0, 10.0),
        rot: Quat::IDENTITY,
        scale: Vec3::ONE,
    });

    println!("  - 创建场景，包含 3 个实体");

    // 序列化场景
    let scene = SerializedScene::from_world(&mut world, "示例场景");
    println!("  - 序列化 {} 个实体", scene.entities.len());

    // 保存场景
    let scene_path = "/tmp/scene_example.ron";
    scene.save_to_file(scene_path)?;
    println!("  - 保存场景: {}", scene_path);

    // 加载场景
    let loaded_scene = SerializedScene::load_from_file(scene_path)?;
    println!("  - 加载场景: {}", loaded_scene.name);

    // 反序列化到新World
    let mut new_world = World::new();
    let entity_map = loaded_scene.to_world(&mut new_world);
    println!("  - 反序列化 {} 个实体到新World", entity_map.len());

    Ok(())
}

/// 资源元数据示例
fn resource_metadata_example() -> Result<(), Box<dyn std::error::Error>> {
    // 创建资源索引
    let mut index = ResourceIndex::new();

    // 添加纹理资源
    let mut player_texture = ResourceMetadata::new("tex_player", ResourceType::Texture, "assets/textures/player.png");
    player_texture
        .add_tag("角色")
        .add_tag("主角")
        .add_tag("512x512")
        .with_size(512 * 512 * 4) // RGBA
        .set_property("压缩格式", "PNG");

    let mut enemy_texture = ResourceMetadata::new("tex_enemy", ResourceType::Texture, "assets/textures/enemy.png");
    enemy_texture
        .add_tag("角色")
        .add_tag("敌人")
        .with_size(256 * 256 * 4);

    // 添加音频资源
    let mut bgm = ResourceMetadata::new("bgm_main", ResourceType::Audio, "assets/audio/bgm.mp3");
    bgm.add_tag("音乐").add_tag("背景").with_size(5_000_000);

    // 添加到索引
    index.add(player_texture);
    index.add(enemy_texture);
    index.add(bgm);

    println!("  - 资源索引包含 {} 个资源", index.count());

    // 查询资源
    let character_resources = index.find_by_tag("角色");
    println!("  - 找到 {} 个'角色'标签的资源", character_resources.len());

    let texture_resources = index.find_by_type(&ResourceType::Texture);
    println!("  - 找到 {} 个纹理资源", texture_resources.len());

    // 按ID查找
    if let Some(resource) = index.find_by_id("tex_player") {
        println!("  - 找到资源: {} ({})", resource.name, resource.id);
        println!("    大小: {} bytes", resource.size_bytes);
        println!("    标签: {:?}", resource.tags);
    }

    Ok(())
}

/// 版本管理示例
fn version_management_example() -> Result<(), Box<dyn std::error::Error>> {
    use game_engine::serialization::{SemanticVersion, VersionManager};

    // 语义化版本
    let v1 = SemanticVersion::new(1, 2, 3);
    let v2 = SemanticVersion::new(1, 3, 0);
    let v3 = SemanticVersion::new(2, 0, 0);

    println!("  - 版本比较:");
    println!("    {} < {} ? {}", v1, v2, v1 < v2);
    println!("    {} 兼容 {} ? {}", v1, v2, v1.is_compatible(&v2));
    println!("    {} 兼容 {} ? {}", v1, v3, v1.is_compatible(&v3));

    // 版本升级
    println!("  - 版本升级:");
    println!("    {} bump_patch -> {}", v1, v1.bump_patch());
    println!("    {} bump_minor -> {}", v1, v1.bump_minor());
    println!("    {} bump_major -> {}", v1, v1.bump_major());

    // 版本管理器
    let mut manager = VersionManager::new(3);
    println!("  - 当前版本: {}", manager.current_version());

    Ok(())
}

/// 多格式序列化对比
fn format_comparison_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut world = World::new();

    // 创建测试数据
    for i in 0..10 {
        world.spawn(Transform {
            pos: Vec3::new(i as f32, i as f32, i as f32),
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        });
    }

    let game_state = GameState::from_world(&mut world, "性能测试");

    // 对比不同格式的大小
    let ron_size = game_state.estimate_size(SerializationFormat::Ron);
    let bin_size = game_state.estimate_size(SerializationFormat::Bincode);
    let json_size = game_state.estimate_size(SerializationFormat::Json);

    println!("  - 序列化大小对比:");
    println!("    RON:     {} bytes", ron_size);
    println!("    Bincode: {} bytes (最小)", bin_size);
    println!("    JSON:    {} bytes", json_size);

    println!("  - 压缩率:");
    println!("    Bincode vs JSON: {:.1}%", (bin_size as f64 / json_size as f64) * 100.0);
    println!("    RON vs JSON: {:.1}%", (ron_size as f64 / json_size as f64) * 100.0);

    Ok(())
}
