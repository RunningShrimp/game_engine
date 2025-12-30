//! # 音频系统示例
//!
//! 此示例展示如何使用音频系统播放3D空间音频。
//!
//! ## 运行
//!
//! ```bash
//! cargo run --example audio
//! ```

use game_engine::audio::SpatialAudioSource;
use game_engine::audio::{AudioDomainService, AudioSource, AudioSourceId, AudioSourceState};
use game_engine::domain::audio::SpatialAudioSource as RichSpatialAudioSource;
use glam::Vec3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Audio System Example ===\n");

    // 创建音频服务
    let mut audio_service = AudioDomainService::new();

    println!("1. Creating audio listener (camera)...");
    audio_service.set_listener_position(Vec3::new(0.0, 0.0, 5.0));
    audio_service.set_listener_orientation(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 1.0, 0.0));
    println!("   Listener position: (0.0, 0.0, 5.0)");

    println!("\n2. Creating spatial audio sources...");

    // 创建背景音乐（2D，无位置衰减）
    let music_id = AudioSourceId::new(1);
    let music_source = AudioSource::new(
        music_id,
        "background_music.ogg".to_string(),
        AudioSourceState::Stopped,
    );
    audio_service.create_source(music_source)?;
    println!("   Created background music source (ID: {})", music_id);

    // 创建3D音效（有位置衰减）
    let sfx_id = AudioSourceId::new(2);
    let sfx_source = RichSpatialAudioSource::new(
        sfx_id,
        "explosion.ogg".to_string(),
        AudioSourceState::Stopped,
        Vec3::new(10.0, 0.0, 0.0),
        10.0, // max_distance
    );
    audio_service.create_spatial_source(sfx_source)?;
    println!(
        "   Created spatial SFX source at (10.0, 0.0, 0.0) (ID: {})",
        sfx_id
    );

    // 创建移动音效（如飞行物体）
    let flyby_id = AudioSourceId::new(3);
    let mut flyby_source = RichSpatialAudioSource::new(
        flyby_id,
        "flyby.ogg".to_string(),
        AudioSourceState::Playing,
        Vec3::new(-20.0, 5.0, 0.0),
        30.0,
    );
    flyby_source.set_velocity(Vec3::new(15.0, 0.0, 0.0)); // 移动速度，用于多普勒效应
    audio_service.create_spatial_source(flyby_source)?;
    println!("   Created flyby sound source at (-20.0, 5.0, 0.0) with velocity (15.0, 0.0, 0.0)");

    println!("\n3. Simulating audio playback...");

    // 模拟几个帧的音频更新
    for frame in 0..10 {
        println!("\n--- Frame {} ---", frame);

        // 更新音源位置（模拟移动）
        if let Ok(source) = audio_service.get_spatial_source(flyby_id) {
            let new_pos = source.position() + Vec3::new(1.5, 0.0, 0.0);
            audio_service.set_source_position(flyby_id, new_pos)?;
            println!(
                "Flyby sound position: ({:.1}, {:.1}, {:.1})",
                new_pos.x, new_pos.y, new_pos.z
            );

            // 计算到听者的距离
            let listener_pos = Vec3::new(0.0, 0.0, 5.0);
            let distance = new_pos.distance(listener_pos);
            println!("Distance to listener: {:.1} units", distance);

            // 模拟距离衰减（简化版）
            if distance < 30.0 {
                let volume = 1.0 - (distance / 30.0);
                println!("Attenuated volume: {:.2}", volume);
            } else {
                println!("Sound is too quiet to hear");
            }
        }

        // 模拟音频处理
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("\n4. Audio system features demonstrated:");
    println!("   ✓ 2D background music (no attenuation)");
    println!("   ✓ 3D spatial audio with distance attenuation");
    println!("   ✓ Moving sound sources with velocity");
    println!("   ✓ Doppler effect (via velocity)");
    println!("   ✓ Listener positioning");

    println!("\n=== Example Complete ===");

    Ok(())
}

/// 更多音频功能示例（伪代码，展示API）
fn _advanced_audio_features() {
    use game_engine::audio::{EffectChain, EqualizerConfig, ReverbConfig};

    println!("Advanced Audio Features:");
    println!("  - Audio effects chain (reverb, EQ, delay, compressor)");
    println!("  - HRTF (Head-Related Transfer Function) for 3D positioning");
    println!("  - Streaming for large audio files");
    println!("  - Multiple distance models (linear, inverse, exponential)");

    // 伪代码展示特效链使用
    let _effect_chain = EffectChain::new()
        .with_reverb(ReverbConfig::room())
        .with_equalizer(EqualizerConfig::bass_boost());

    // 伪代码展示流式音频
    // let stream = AudioStream::load("large_music.ogg")?;
    // stream.play()?;
}
