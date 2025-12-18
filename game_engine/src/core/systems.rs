//! 核心ECS系统
//!
//! 定义引擎核心运行时使用的ECS系统

pub mod actor;
pub mod error_reporting;

use bevy_ecs::prelude::*;
use glam::Quat;

use crate::ecs::{AiComponent, PreviousTransform, Sprite, Time, Transform};
use crate::platform::{InputBuffer, InputEvent};
use crate::resources::manager::Handle;
use crate::services::audio::{audio_play, audio_set_volume, audio_stop, AudioQueueResource};

use super::resources::Benchmark;
use crate::platform::InputActions;

/// 旋转系统 - 演示用，使所有实体旋转
pub fn rotate_system(mut query: Query<&mut Transform>, time: Res<Time>) {
    for mut t in query.iter_mut() {
        t.rot *= Quat::from_rotation_z(1.0 * time.delta_seconds);
    }
}

/// 纹理句柄应用系统 - 将加载完成的纹理ID应用到Sprite（优化版本：避免Clone）
pub fn apply_texture_handles(mut query: Query<(&Handle<u32>, &mut Sprite)>) {
    for (handle, mut sprite) in query.iter_mut() {
        if let Some(tex_id) = handle.get() {
            // ✅ 修复：使用get()方法代替已移除的get_ref()
            sprite.tex_index = tex_id;
        }
    }
}

/// 纹理句柄应用系统 - 批处理优化版本（更高效的内存访问模式）
/// 注意：此版本改变了查询顺序，通过先访问Sprite再访问Handle来优化内存预取
/// 在大批量数据中可能有更好性能，但需要确保Sprite始终存在
pub fn apply_texture_handles_batch(mut query: Query<(&mut Sprite, &Handle<u32>)>) {
    for (mut sprite, handle) in query.iter_mut() {
        if let Some(tex_id) = handle.get() {
            // ✅ 修复：使用get()方法代替已移除的get_ref()
            sprite.tex_index = tex_id;
        }
    }
}

/// 保存上一帧变换系统 - 用于插值渲染
pub fn save_previous_transform_system(mut query: Query<(&Transform, &mut PreviousTransform)>) {
    for (t, mut pt) in query.iter_mut() {
        pt.pos = t.pos;
        pt.rot = t.rot;
        pt.scale = t.scale;
    }
}

/// 基准测试系统 - 批量生成精灵用于性能测试
pub fn benchmark_system(mut commands: Commands, mut benchmark: ResMut<Benchmark>) {
    if benchmark.enabled && benchmark.sprite_count < 50000 {
        // 每帧生成500个精灵直到达到50000
        for _ in 0..500 {
            commands.spawn((
                Transform {
                    pos: glam::Vec3::new(
                        rand::random::<f32>() * 800.0,
                        rand::random::<f32>() * 600.0,
                        0.0,
                    ),
                    scale: glam::Vec3::new(5.0, 5.0, 1.0),
                    ..Default::default()
                },
                Sprite {
                    color: [rand::random(), rand::random(), rand::random(), 1.0],
                    ..Default::default()
                },
            ));
            benchmark.sprite_count += 1;
        }
    }
}

/// 音频输入系统 - 处理键盘输入控制音频播放
pub fn audio_input_system(input: Res<InputBuffer>, audio: Option<Res<AudioQueueResource>>) {
    if let Some(q) = audio {
        for ev in &input.events {
            if let InputEvent::CharInput(c) = ev {
                match c {
                    'g' | 'G' => {
                        audio_play(&q, "guitar", "assets/guitar.ogg", 0.8, false);
                    }
                    's' | 'S' => {
                        audio_stop(&q, "guitar");
                    }
                    '+' => {
                        audio_set_volume(&q, "guitar", 1.0);
                    }
                    '-' => {
                        audio_set_volume(&q, "guitar", 0.5);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// AI 更新系统 - 更新所有实体的 AI 组件
pub fn ai_update_system(world: &mut World) {
    let time = world.resource::<Time>();
    let delta = time.delta_seconds;
    let mut query = world.query::<&mut AiComponent>();
    for ai in query.iter_mut(world) {
        if let Some(ref bt) = ai.behavior_tree {
            if let Ok(mut bt_guard) = bt.lock() {
                bt_guard.tick();
            }
        }
        if let Some(ref sm) = ai.state_machine {
            if let Ok(mut sm_guard) = sm.lock() {
                sm_guard.update(delta);
            }
        }
    }
}

/// 计算渲染通道数量 (辅助函数)
///
/// # 参数
///
/// * `timing` - 可选的渲染时间元组，包含开始时间和总时间
///
/// # 返回
///
/// 返回渲染通道数量（当前实现返回0）
///
/// # 注意
///
/// 此函数当前为占位符实现，实际渲染通道计算需要根据具体渲染管线实现。
#[allow(dead_code)]
pub fn compute_passes(timing: Option<(f32, f32)>) -> u32 {
    match timing {
        Some((_t0, _total)) => 0,
        None => 0,
    }
}

/// 示例移动系统 - 展示如何使用输入动作
///
/// 此系统演示了如何查询输入动作状态并相应地更新实体。
/// 在实际游戏中，这会被替换为更复杂的角色控制器。
pub fn movement_system(mut query: Query<&mut Transform>, input_actions: Option<Res<InputActions>>, time: Res<Time>) {
    // 如果没有输入动作资源，直接返回
    let actions = match input_actions {
        Some(a) => a,
        None => return,
    };
    
    // 计算移动向量
    let mut move_x = 0.0;
    let mut move_y = 0.0;
    
    if actions.move_forward {
        move_y += 1.0;
    }
    if actions.move_backward {
        move_y -= 1.0;
    }
    if actions.move_left {
        move_x -= 1.0;
    }
    if actions.move_right {
        move_x += 1.0;
    }
    
    // 如果有移动输入，则更新所有实体的位置
    if move_x != 0.0 || move_y != 0.0 {
        // 标准化移动向量
        let len = (move_x * move_x + move_y * move_y).sqrt();
        if len > 0.0 {
            move_x /= len;
            move_y /= len;
        }
        
        // 应用移动速度
        let speed = 200.0; // 像素/秒
        let delta_move_x = move_x * speed * time.delta_seconds;
        let delta_move_y = move_y * speed * time.delta_seconds;
        
        // 更新所有实体的位置
        for mut transform in query.iter_mut() {
            transform.pos.x += delta_move_x;
            transform.pos.y += delta_move_y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_default() {
        let bench = Benchmark::default();
        assert!(!bench.enabled);
        assert_eq!(bench.sprite_count, 0);
    }
}
