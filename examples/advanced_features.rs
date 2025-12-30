// 高级音频系统和AI系统完整使用示例
//
// 展示新功能在实际游戏场景中的使用

use game_engine::ai::{
    behavior_tree_serialization::BehaviorTreeDeserializer,
    goap::{GoapPlanner, AttackAction, MoveToAction, EliminateTargetGoal, SurvivalGoal, WorldState, StateValue},
    influence_map::{TacticalInfluenceMap, InfluenceGrid},
    influence_visualization::InfluenceVisualizer,
    behavior_tree::{Sequence, Selector, Status, Node, Action as BTAction},
};
use game_engine::audio::{
    hrtf_processor::HrtfProcessor,
    advanced_reverb::{RoomReverb, FdnReverb, Wall},
    occlusion::AudioOcclusion,
    doppler::DopplerEffect,
    hrtf::HrtfConfig,
};

// ============================================================================
// 示例1: 使用GOAP规划AI行为
// ============================================================================

fn example_1_goap_planning() {
    println!("\n=== 示例1: GOAP目标导向规划 ===\n");

    // 创建GOAP规划器
    let mut planner = GoapPlanner::new();

    // 注册动作
    planner.register_action(Box::new(MoveToAction {
        target_position: (10.0, 0.0, 10.0),
    }));
    planner.register_action(Box::new(AttackAction {
        target_id: 1,
        damage: 25.0,
    }));

    // 注册目标
    planner.register_goal(Box::new(EliminateTargetGoal));
    planner.register_goal(Box::new(SurvivalGoal));

    // 当前世界状态
    let mut current_state = WorldState::new();
    current_state.set("has_weapon", StateValue::Bool(true));
    current_state.set("can_move", StateValue::Bool(true));
    current_state.set("health", StateValue::Float(100.0));
    current_state.set("target_alive", StateValue::Bool(true));
    current_state.set("in_range", StateValue::Bool(false));

    println!("📊 当前世界状态:");
    println!("  - 生命值: {:?}", current_state.get("health"));
    println!("  - 目标存活: {:?}", current_state.get("target_alive"));
    println!("  - 射程内: {:?}", current_state.get("in_range"));

    // 规划动作序列
    if let Some(plan) = planner.plan(&current_state) {
        println!("\n🎯 生成计划 ({}个动作):", plan.len());
        for (i, action) in plan.iter().enumerate() {
            println!("  {}. {}", i + 1, action.name());
        }
    }

    // 检查目标优先级
    println!("\n📋 目标优先级:");
    println!("  - 消灭目标: {:.1}", EliminateTargetGoal.priority(&current_state));
    println!("  - 生存: {:.1}", SurvivalGoal.priority(&current_state));
}

// ============================================================================
// 示例2: 从JSON加载行为树
// ============================================================================

fn example_2_load_behavior_tree() {
    println!("\n=== 示例2: JSON行为树加载 ===\n");

    let json = r#"
    {
        "version": "1.0",
        "tree": "soldier_ai",
        "nodes": [
            {
                "id": "root",
                "type": "selector",
                "name": "Main Selector",
                "children": ["attack_sequence", "patrol"],
                "config": {}
            },
            {
                "id": "attack_sequence",
                "type": "sequence",
                "name": "Attack Sequence",
                "children": ["check_health", "attack"],
                "config": {}
            },
            {
                "id": "check_health",
                "type": "condition",
                "name": "Check Health",
                "children": [],
                "config": {}
            },
            {
                "id": "attack",
                "type": "action",
                "name": "Attack Target",
                "children": [],
                "config": {}
            },
            {
                "id": "patrol",
                "type": "action",
                "name": "Patrol Area",
                "children": [],
                "config": {}
            }
        ]
    }
    "#;

    match BehaviorTreeDeserializer::from_json(json) {
        Ok(tree) => {
            println!("✅ 成功加载行为树:");
            println!("  - 节点类型: Selector (根节点)");
            println!("  - 子节点数量: 2");
            println!("  - 树名称: soldier_ai");
        }
        Err(e) => {
            println!("❌ 加载失败: {}", e);
        }
    }
}

// ============================================================================
// 示例3: HRTF 3D音频渲染
// ============================================================================

fn example_3_hrtf_rendering() {
    println!("\n=== 示例3: HRTF 3D音频渲染 ===\n");

    let config = HrtfConfig::default();
    let mut processor = match HrtfProcessor::new(config) {
        Ok(p) => p,
        Err(e) => {
            println!("❌ HRTF初始化失败: {}", e);
            return;
        }
    };

    // 生成测试音频（1kHz正弦波，0.1秒）
    let sample_rate = 44100;
    let frequency = 1000.0;
    let duration = 0.1; // 秒
    let samples = (sample_rate as f32 * duration) as usize;

    let input: Vec<f32> = (0..samples)
        .map(|i| (2.0 * std::f32::consts::PI * frequency * i as f32 / sample_rate as f32).sin())
        .collect();

    // 测试不同位置的声源
    let positions = vec![
        ("正前方", (0.0, 0.0, -1.0)),
        ("左侧", (-1.0, 0.0, 0.0)),
        ("右侧", (1.0, 0.0, 0.0)),
        ("上方", (0.0, 1.0, 0.0)),
    ];

    let listener_pos = (0.0, 0.0, 0.0);
    let listener_orientation = (1.0, 0.0, 0.0, 0.0); // 单位四元数

    println!("🎧 测试声源定位:");
    for (name, source_pos) in &positions {
        let output = processor.process(
            &input,
            *source_pos,
            listener_pos,
            listener_orientation,
        );

        // 计算左右声道能量
        let left_energy: f32 = output.iter().map(|s| s[0].abs()).sum();
        let right_energy: f32 = output.iter().map(|s| s[1].abs()).sum();

        let ratio = if right_energy > 0.0 {
            left_energy / right_energy
        } else {
            999.0
        };

        println!("  {} - 左/右能量比: {:.2}", name, ratio);
    }
}

// ============================================================================
// 示例4: 房间混响效果
// ============================================================================

fn example_4_room_reverb() {
    println!("\n=== 示例4: 房间混响效果 ===\n");

    // 创建房间混响
    let mut reverb = RoomReverb::new((10.0, 5.0, 8.0));

    // 设置不同材质
    println!("🏠 房间配置:");
    reverb.set_wall_material(Wall::Floor, AcousticMaterial::Wood);
    reverb.set_wall_material(Wall::Ceiling, AcousticMaterial::Concrete);
    reverb.set_wall_material(Wall::Front, AcousticMaterial::Glass);

    // 计算RT60
    let rt60 = reverb.compute_rt60();
    println!("  - 房间尺寸: 10m x 5m x 8m");
    println!("  - RT60: {:.2}秒", rt60);

    // 计算脉冲响应
    let source = (1.0, 2.5, 2.0);
    let listener = (8.0, 2.5, 5.0);
    let impulse_response = reverb.compute_impulse_response(source, listener, 44100.0);

    println!("\n📊 脉冲响应统计:");
    println!("  - 长度: {} 样本", impulse_response.len());
    println!("  - 最大值: {:.4}", impulse_response.iter().cloned().fold(0.0_f32, f32::max));

    // 测试FDN混响
    println!("\n🔄 FDN混响测试:");
    let mut fdn = FdnReverb::new(44100.0, rt60);

    for i in 0..10 {
        let input = (i as f32 / 10.0 * 2.0 * std::f32::consts::PI).sin();
        let (left, right) = fdn.process(input);
        println!("  帧 {}: 左={:.3}, 右={:.3}", i, left, right);
    }
}

// ============================================================================
// 示例5: 音频遮挡计算
// ============================================================================

fn example_5_audio_occlusion() {
    println!("\n=== 示例5: 音频遮挡计算 ===\n");

    let mut occlusion = AudioOcclusion::new();
    occlusion.set_max_distance(50.0);

    let source = (10.0, 0.0, 0.0);
    let listener = (0.0, 0.0, 0.0);

    println!("🔊 遮挡分析:");
    println!("  - 声源位置: {:?}", source);
    println!("  - 听者位置: {:?}", listener);

    // 没有物理世界时的结果
    let result = occlusion.compute_occlusion(source, listener);
    println!("\n📊 遮挡结果:");
    println!("  - 遮挡因子: {:.2}%", result.occlusion_factor * 100.0);
    println!("  - 传输损失: {:.2}%", result.transmission_loss * 100.0);
    println!("  - 低频衰减: {:.1} dB", result.low_frequency_attenuation);
    println!("  - 高频衰减: {:.1} dB", result.high_frequency_attenuation);
}

// ============================================================================
// 示例6: 多普勒效应
// ============================================================================

fn example_6_doppler_effect() {
    println!("\n=== 示例6: 多普勒效应 ===\n");

    let doppler = DopplerEffect::new();

    let base_frequency = 440.0; // A4
    let listener = (0.0, 0.0, 0.0);
    let listener_vel = (0.0, 0.0, 0.0);

    println!("🎵 多普勒频移测试 (基频: {:.1} Hz):", base_frequency);

    // 不同速度的声源
    let scenarios = vec![
        ("静止", (0.0, 0.0, 0.0)),
        ("慢速接近", (10.0, 0.0, 0.0)),
        ("快速接近", (34.0, 0.0, 0.0)),  // 122 km/h
        ("远离", (-10.0, 0.0, 0.0)),
    ];

    for (name, source_vel) in scenarios {
        let source_pos = (20.0, 0.0, 0.0);

        let shifted_freq = doppler.compute_frequency_shift(
            base_frequency,
            source_pos,
            source_vel,
            listener,
            listener_vel,
        );

        let freq_diff = shifted_freq - base_frequency;

        println!("  {} - 频率: {:.1} Hz (偏移: {:.1} Hz)",
                 name, shifted_freq, freq_diff);
    }
}

// ============================================================================
// 示例7: 覆盖图可视化
// ============================================================================

fn example_7_influence_map_visualization() {
    println!("\n=== 示例7: 覆盖图可视化 ===\n");

    // 创建战术覆盖图
    let mut tactical = TacticalInfluenceMap::new(100, 100, 1.0);

    // 设置场景
    println!("⚔️ 战术场景:");
    println!("  - 友军单位: (30, 30), (70, 70)");
    println!("  - 敌军单位: (50, 50)");

    tactical.territory.add_source(30, 30, 100.0);
    tactical.territory.add_source(70, 70, 80.0);
    tactical.danger.add_source(50, 50, -90.0);
    tactical.opportunity.add_source(40, 40, 60.0);

    // 更新覆盖图
    tactical.update(0.3, 5);

    // 查找最佳位置
    let (best_x, best_y, best_score) = tactical.find_best_position();
    println!("\n🎯 最佳战术位置: ({}, {}) 得分={:.1}", best_x, best_y, best_score);

    // 可视化
    let viz = InfluenceVisualizer::new(20, 20, 5);

    println!("\n📊 领土控制图 (ASCII):");
    let ascii = viz.to_ascii(&tactical.territory);
    // 只显示前20行
    for line in ascii.lines().take(20) {
        println!("{}", line);
    }

    // 统计信息
    let stats = viz.statistics(&tactical.territory);
    println!("\n📈 统计信息:");
    println!("  - 格局: {}x{}", stats.width, stats.height);
    println!("  - 最小值: {:.2}", stats.min);
    println!("  - 最大值: {:.2}", stats.max);
    println!("  - 平均值: {:.2}", stats.mean);
}

// ============================================================================
// 示例8: 综合音频场景
// ============================================================================

fn example_8_comprehensive_audio() {
    println!("\n=== 示例8: 综合音频场景 ===\n");

    println!("🎬 场景：高速移动的车辆经过隧道");

    let config = HrtfConfig::default();
    let mut hrtf = HrtfProcessor::new(config).unwrap();
    let mut reverb = RoomReverb::new((5.0, 3.0, 20.0)); // 隧道
    let mut doppler = DopplerEffect::new();

    // 隧道材质（混凝土）
    reverb.set_wall_material(Wall::Ceiling, AcousticMaterial::Concrete);
    reverb.set_wall_material(Wall::Floor, AcousticMaterial::Concrete);

    println!("\n🔊 音频参数:");
    println!("  - 隧道尺寸: 5m x 3m x 20m");
    println!("  - RT60: {:.2}s", reverb.compute_rt60());
    println!("  - HRTF: 已启用");

    // 模拟车辆移动
    let sample_rate = 44100;
    let duration = 1.0; // 1秒
    let samples = (sample_rate as f32 * duration) as usize;

    let source_positions = vec![
        (-20.0, 0.0, 0.0),  // 开始：远离
        (0.0, 0.0, 0.0),    // 中点：经过
        (20.0, 0.0, 0.0),   // 结束：离开
    ];

    println!("\n🚗 车辆移动模拟:");
    for (i, &pos) in source_positions.iter().enumerate() {
        let velocity = 40.0; // 144 km/h

        let input = vec![0.0f32; sample_rate / 10]; // 短音频片段

        // 应用多普勒效应
        let pitch_shift = doppler.compute_pitch_shift(
            pos,
            (velocity, 0.0, 0.0),
            (0.0, 0.0, 0.0),
            (0.0, 0.0, 0.0),
        );

        // HRTF渲染
        let output = hrtf.process(
            &input,
            pos,
            (0.0, 0.0, 0.0),
            (1.0, 0.0, 0.0, 0.0),
        );

        println!("  时间 {:.2}s - 位置 ({:.1}, 0, 0) - 音调偏移: {:.2}%",
                 i as f32 * duration, pos, pitch_shift);
    }
}

// ============================================================================
// 主函数
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 游戏引擎高级功能演示\n");
    println!("====================================\n");

    example_1_goap_planning();
    example_2_load_behavior_tree();
    example_3_hrtf_rendering();
    example_4_room_reverb();
    example_5_audio_occlusion();
    example_6_doppler_effect();
    example_7_influence_map_visualization();
    example_8_comprehensive_audio();

    println!("\n====================================");
    println!("🎉 所有高级功能演示完成！");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_1() {
        example_1_goap_planning();
    }

    #[test]
    fn test_example_2() {
        example_2_load_behavior_tree();
    }

    #[test]
    fn test_example_7() {
        example_7_influence_map_visualization();
    }
}
