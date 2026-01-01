// 音频系统脚本API示例
//
// 演示如何在脚本中使用完整的音频系统API，包括2D/3D音效、背景音乐、3D空间音频

use game_engine::{
    ecs::{Entity, World},
    scripting::{
        api::ScriptApi,
        audio_api::AudioScriptApi,
        system::{JavaScriptContext, ScriptContext, ScriptSystem, ScriptValue},
    },
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    println!("=== 音频系统脚本API示例 ===\n");

    // 创建ECS世界
    let mut world = World::new();

    // 创建脚本系统
    let mut script_system = ScriptSystem::new();
    let mut js_context = JavaScriptContext::new();
    script_system.register_context(
        game_engine::scripting::system::ScriptLanguage::JavaScript,
        Box::new(js_context.clone()),
    );

    // 创建并注册音频API
    let audio_api = AudioScriptApi::new();

    // 注册音频API到脚本系统
    let mut api = ScriptApi::new();
    audio_api.register_api(&mut api);

    println!("✓ 音频API已注册到脚本系统");

    // 示例1: 2D音效控制
    println!("\n=== 示例1: 2D音效控制 ===");
    audio_2d_example(&script_system)?;

    // 示例2: 3D空间音频
    println!("\n=== 示例2: 3D空间音频 ===");
    audio_3d_example(&script_system)?;

    // 示例3: 背景音乐控制
    println!("\n=== 示例3: 背景音乐控制 ===");
    music_control_example(&script_system)?;

    // 示例4: 音频资源管理
    println!("\n=== 示例4: 音频资源管理 ===");
    resource_management_example(&script_system)?;

    println!("\n=== 所有示例完成 ===");
    Ok(())
}

/// 示例1: 2D音效控制
fn audio_2d_example(script_system: &ScriptSystem) -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
        // 播放2D音效
        function play2DSound() {
            const clipName = "footstep";
            const volume = 0.8;
            const pitch = 1.0;
            const loop = false;

            const result = audio_play_2d(clipName, volume, pitch, loop);
            Engine.log("Playing 2D sound: " + result);
        }

        // 播放循环音效
        function playLoopingSound() {
            const clipName = "ambient_wind";
            const result = audio_play_2d(clipName, 0.5, 1.0, true);
            Engine.log("Playing looping sound: " + result);
        }

        // 控制音量和音高
        function controlVolumeAndPitch() {
            const sourceId = "source_footstep_1234567890";

            // 设置音量 (0.0-1.0)
            let result = audio_set_volume(sourceId, 0.6);
            Engine.log("Volume set: " + result);

            // 设置音高 (0.1-2.0)
            result = audio_set_pitch(sourceId, 1.2);
            Engine.log("Pitch set: " + result);
        }

        // 暂停和恢复
        function pauseAndResume() {
            const sourceId = "source_footstep_1234567890";

            // 暂停
            let result = audio_pause(sourceId);
            Engine.log("Paused: " + result);

            // 恢复
            result = audio_resume(sourceId);
            Engine.log("Resumed: " + result);
        }

        // 停止音效
        function stopSound() {
            const sourceId = "source_footstep_1234567890";
            const result = audio_stop(sourceId);
            Engine.log("Stopped: " + result);
        }

        // 执行所有操作
        play2DSound();
        playLoopingSound();
        controlVolumeAndPitch();
        pauseAndResume();
        stopSound();
    "#;

    let result = script_system.execute(
        game_engine::scripting::system::ScriptLanguage::JavaScript,
        script,
    )?;

    println!("执行结果: {:?}", result);
    Ok(())
}

/// 示例2: 3D空间音频
fn audio_3d_example(script_system: &ScriptSystem) -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
        // 播放3D音效
        function play3DSound() {
            const clipName = "explosion";
            const x = 5.0;
            const y = 1.0;
            const z = 10.0;
            const volume = 1.0;

            const result = audio_play_3d(clipName, x, y, z, volume);
            Engine.log("Playing 3D sound: " + result);
        }

        // 更新3D音频位置
        function update3DPosition() {
            const sourceId = "3d_source_explosion_1234567890";
            const newX = 8.0;
            const newY = 2.0;
            const newZ = 12.0;

            const result = audio_update_3d_position(sourceId, newX, newY, newZ);
            Engine.log("Position updated: " + result);
        }

        // 设置3D衰减距离
        function setAttenuation() {
            const sourceId = "3d_source_explosion_1234567890";
            const minDistance = 1.0;  // 最近距离（音量不衰减）
            const maxDistance = 50.0; // 最远距离（音量为0）

            const result = audio_set_3d_attenuation(sourceId, minDistance, maxDistance);
            Engine.log("Attenuation set: " + result);
        }

        // 启用多普勒效应
        function enableDoppler() {
            const sourceId = "3d_source_explosion_1234567890";
            const enabled = true;

            const result = audio_enable_doppler(sourceId, enabled);
            Engine.log("Doppler effect enabled: " + result);
        }

        // 模拟移动音源
        function simulateMovingSource() {
            const sourceId = "3d_source_explosion_1234567890";

            // 音源沿X轴移动
            for (let x = 0; x <= 20; x += 2) {
                audio_update_3d_position(sourceId, x, 1.0, 10.0);
                Engine.log("Moving source to x=" + x);
            }
        }

        // 执行所有操作
        play3DSound();
        update3DPosition();
        setAttenuation();
        enableDoppler();
        simulateMovingSource();
    "#;

    let result = script_system.execute(
        game_engine::scripting::system::ScriptLanguage::JavaScript,
        script,
    )?;

    println!("执行结果: {:?}", result);
    Ok(())
}

/// 示例3: 背景音乐控制
fn music_control_example(script_system: &ScriptSystem) -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
        // 播放背景音乐
        function playMusic() {
            const musicName = "background_theme";
            const volume = 0.7;
            const loop = true;
            const fadeDuration = 2.0; // 淡入时长（秒）

            const result = audio_play_music(musicName, volume, loop, fadeDuration);
            Engine.log("Playing music: " + result);
        }

        // 调整音乐音量
        function adjustMusicVolume() {
            const newVolume = 0.5;
            const result = audio_set_music_volume(newVolume);
            Engine.log("Music volume adjusted: " + result);
        }

        // 淡入音乐
        function fadeInMusic() {
            const duration = 3.0; // 秒
            const targetVolume = 0.7;

            const result = audio_fade_in_music(duration, targetVolume);
            Engine.log("Music fading in: " + result);
        }

        // 淡出音乐
        function fadeOutMusic() {
            const duration = 2.0; // 秒

            const result = audio_fade_out_music(duration);
            Engine.log("Music fading out: " + result);
        }

        // 停止音乐（带淡出）
        function stopMusic() {
            const fadeDuration = 1.5;

            const result = audio_stop_music(fadeDuration);
            Engine.log("Music stopped: " + result);
        }

        // 切换音乐（淡出 -> 淡入）
        function crossfadeMusic() {
            // 淡出当前音乐
            audio_fade_out_music(1.0);

            // 等待淡出完成后播放新音乐
            setTimeout(() => {
                audio_play_music("new_theme", 0.7, true, 1.0);
            }, 1000);
        }

        // 执行所有操作
        playMusic();
        adjustMusicVolume();
        fadeInMusic();
        fadeOutMusic();
        crossfadeMusic();
        stopMusic();
    "#;

    let result = script_system.execute(
        game_engine::scripting::system::ScriptLanguage::JavaScript,
        script,
    )?;

    println!("执行结果: {:?}", result);
    Ok(())
}

/// 示例4: 音频资源管理
fn resource_management_example(
    script_system: &ScriptSystem,
) -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
        // 加载音频资源
        function loadAudioClips() {
            // 加载音效
            const clipPath1 = "assets/audio/footstep.wav";
            const clipName1 = "footstep";
            let result = audio_load_clip(clipPath1, clipName1);
            Engine.log("Loaded clip 1: " + result);

            // 加载音乐
            const clipPath2 = "assets/audio/background_theme.mp3";
            const clipName2 = "background_theme";
            result = audio_load_clip(clipPath2, clipName2);
            Engine.log("Loaded clip 2: " + result);

            // 使用路径自动命名（不指定clip_name参数）
            const clipPath3 = "assets/audio/explosion.wav";
            result = audio_load_clip(clipPath3);
            Engine.log("Loaded clip 3: " + result);
        }

        // 获取音频信息
        function getClipInfo() {
            const clipName = "background_theme";
            const result = audio_get_clip_info(clipName);
            Engine.log("Clip info: " + result);

            // 解析信息
            // 返回格式: "Clip info: name=..., duration=...s, channels=..., sample_rate=...Hz"
        }

        // 卸载音频资源
        function unloadClips() {
            const clipName = "footstep";
            const result = audio_unload_clip(clipName);
            Engine.log("Unloaded clip: " + result);
        }

        // 批量加载资源
        function batchLoadResources() {
            const clips = [
                { path: "assets/audio/footstep.wav", name: "footstep" },
                { path: "assets/audio/jump.wav", name: "jump" },
                { path: "assets/audio/explosion.wav", name: "explosion" },
                { path: "assets/audio/pickup.wav", name: "pickup" },
                { path: "assets/audio/background_theme.mp3", name: "background_theme" },
                { path: "assets/audio/battle_theme.mp3", name: "battle_theme" }
            ];

            clips.forEach(clip => {
                const result = audio_load_clip(clip.path, clip.name);
                Engine.log("Batch loaded: " + clip.name + " - " + result);
            });
        }

        // 执行所有操作
        loadAudioClips();
        getClipInfo();
        unloadClips();
        batchLoadResources();
    "#;

    let result = script_system.execute(
        game_engine::scripting::system::ScriptLanguage::JavaScript,
        script,
    )?;

    println!("执行结果: {:?}", result);
    Ok(())
}

// 使用说明
//
// ## 音频系统脚本API完整参考
//
// ### 2D音效 (2D Sound Effects)
//
// **播放2D音效**
// ```javascript
// audio_play_2d(clip_name, volume, pitch, loop)
// ```
// - clip_name: 音频片段名称（需要先加载）
// - volume: 音量 (0.0-1.0)
// - pitch: 音高 (0.1-2.0)
// - loop: 是否循环播放
// - 返回: source_id（用于后续控制）
//
// **控制音频**
// ```javascript
// audio_stop(source_id)              // 停止播放
// audio_pause(source_id)             // 暂停
// audio_resume(source_id)            // 恢复
// audio_set_volume(source_id, vol)   // 设置音量 (0.0-1.0)
// audio_set_pitch(source_id, pitch)  // 设置音高 (0.1-2.0)
// ```
//
// ### 3D空间音频 (3D Spatial Audio)
//
// **播放3D音效**
// ```javascript
// audio_play_3d(clip_name, x, y, z, volume)
// ```
// - x, y, z: 3D世界坐标
// - volume: 音量 (0.0-1.0)
// - 返回: source_id
//
// **更新3D位置**
// ```javascript
// audio_update_3d_position(source_id, x, y, z)
// ```
// 用于移动音源（如：移动的角色、车辆）
//
// **3D音频衰减**
// ```javascript
// audio_set_3d_attenuation(source_id, min_distance, max_distance)
// ```
// - min_distance: 最近距离（音量不衰减）
// - max_distance: 最远距离（音量为0）
//
// **多普勒效应**
// ```javascript
// audio_enable_doppler(source_id, enabled)
// ```
// 启用后，移动音源会产生多普勒频移效果
//
// ### 背景音乐 (Background Music)
//
// **播放音乐**
// ```javascript
// audio_play_music(music_name, volume, loop, fade_duration)
// ```
// - music_name: 音乐名称
// - volume: 音量 (0.0-1.0)
// - loop: 是否循环
// - fade_duration: 淡入时长（秒）
//
// **音乐控制**
// ```javascript
// audio_stop_music(fade_duration)              // 停止（带淡出）
// audio_set_music_volume(volume)               // 设置音量
// audio_fade_in_music(duration, target_volume) // 淡入
// audio_fade_out_music(duration)               // 淡出
// ```
//
// **交叉淡入淡出 (Crossfade)**
// ```javascript
// // 淡出当前音乐
// audio_fade_out_music(1.0);
//
// // 延迟后播放新音乐
// setTimeout(() => {
//     audio_play_music("new_theme", 0.7, true, 1.0);
// }, 1000);
// ```
//
// ### 音频资源管理 (Audio Resources)
//
// **加载音频**
// ```javascript
// audio_load_clip(clip_path, clip_name)
// ```
// - clip_path: 音频文件路径（支持 .wav, .mp3, .ogg 等）
// - clip_name: 自定义名称（可选，默认使用文件名）
//
// **卸载音频**
// ```javascript
// audio_unload_clip(clip_name)
// ```
// 释放内存，卸载后无法播放该音频
//
// **获取音频信息**
// ```javascript
// const info = audio_get_clip_info(clip_name);
// // 返回: "Clip info: name=..., duration=...s, channels=..., sample_rate=...Hz"
// ```
//
// ## Lua示例
// ```lua
// -- 播放2D音效
// audio_play_2d("footstep", 0.8, 1.0, false)
//
// -- 播放3D音效
// audio_play_3d("explosion", 5.0, 1.0, 10.0, 1.0)
//
// -- 播放音乐
// audio_play_music("background_theme", 0.7, true, 2.0)
// ```
//
// ## Python示例
// ```python
// # 播放2D音效
// audio_play_2d("footstep", 0.8, 1.0, False)
//
// # 播放3D音效
// audio_play_3d("explosion", 5.0, 1.0, 10.0, 1.0)
//
// # 播放音乐
// audio_play_music("background_theme", 0.7, True, 2.0)
// ```
//
// ## 实际应用场景
//
// **1. 第一人称射击游戏**
// ```javascript
// function on_update(entity, delta_time) {
//     // 根据玩家位置播放3D脚步声
//     const pos = get_entity_position(entity);
//     audio_update_3d_position(footstep_source, pos.x, pos.y, pos.z);
//
//     // 射击时播放枪声
//     if (is_shooting) {
//         audio_play_3d("gunshot", pos.x, pos.y, pos.z, 1.0);
//     }
// }
// ```
//
// **2. 赛车游戏**
// ```javascript
// function on_update(entity, delta_time) {
//     // 根据车速调整引擎音高
//     const speed = get_vehicle_speed(entity);
//     audio_set_pitch(engine_source, 0.5 + speed / 100.0);
//
//     // 更新引擎位置
//     const pos = get_entity_position(entity);
//     audio_update_3d_position(engine_source, pos.x, pos.y, pos.z);
// }
// ```
//
// **3. 节奏游戏**
// ```javascript
// function on_beat(beat_index) {
//     // 每个节拍播放音效
//     const clip_name = "beat_" + (beat_index % 4);
//     audio_play_2d(clip_name, 1.0, 1.0, false);
// }
//
// function on_game_start() {
//     // 淡入背景音乐
//     audio_play_music("game_music", 0.7, true, 2.0);
// }
//
// function on_game_over() {
//     // 淡出音乐
//     audio_stop_music(3.0);
// }
// ```
//
// ## 性能注意事项
//
// 1. **3D音频开销**: 3D音频计算量较大，限制同时播放的3D音源数量（<32个）
// 2. **资源预加载**: 在场景加载时批量加载音频，避免运行时加载卡顿
// 3. **流式音频**: 背景音乐使用流式播放，减少内存占用
// 4. **音源池管理**: 重用音源对象，减少创建/销毁开销
// 5. **距离剔除**: 远离听者的3D音源应暂停播放
//
// ## 音频格式建议
//
// - **音效**: 使用 WAV（无损、快速解码）或 OGG Vorbis（压缩比高）
// - **音乐**: 使用 MP3（通用性好）或 OGG Vorbis（开源友好）
// - **采样率**: 44.1kHz 或 48kHz（CD质量）
// - **比特深度**: 16-bit（平衡质量和大小）
// - **声道**: 单声道（音效）、立体声（音乐）
//
// ## 生命周期集成
//
// 所有音频API都可以在生命周期钩子中使用:
//
// ```javascript
// function on_enable(entity) {
//     // 预加载音效
//     audio_load_clip("assets/audio/jump.wav", "jump");
//     audio_load_clip("assets/audio/footstep.wav", "footstep");
//
//     // 开始背景音乐
//     audio_play_music("level_theme", 0.7, true, 2.0);
// }
//
// function on_update(entity, delta_time) {
//     // 根据状态播放音效
//     if (is_jumping && !jump_playing) {
//         audio_play_2d("jump", 0.8, 1.0, false);
//         jump_playing = true;
//     }
//
//     // 更新3D音效位置
//     const pos = get_entity_position(entity);
//     audio_update_3d_position(footstep_source, pos.x, pos.y, pos.z);
// }
//
// function on_disable(entity) {
//     // 停止音乐（淡出）
//     audio_stop_music(1.0);
//
//     // 卸载音效
//     audio_unload_clip("jump");
//     audio_unload_clip("footstep");
// }
// ```
