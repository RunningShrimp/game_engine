// 增强音频系统示例
//
// 展示如何使用P2-2阶段的音频增强特性：
// - HRTF双线性插值
// - 高级混响系统（高阶反射和预设）
// - 音频遮挡（低通滤波器和过渡）
// - 音频分析工具

use game_engine::audio::{
    analysis::{LatencyMeter, SpectrumAnalyzer, VolumeMeter},
    advanced_reverb::{ReverbPreset, RoomReverb},
    hrtf_processor::{HrtfProcessor, HrirDataset},
    hrir_interpolation::InterpolationMethod,
    occlusion::{AudioOcclusion, AcousticMaterial},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 增强音频系统示例 ===\n");

    // ========================================================================
    // 1. HRTF双线性插值示例
    // ========================================================================
    println!("1. HRTF双线性插值示例");
    println!("-------------------");

    let config = game_engine::audio::hrtf::HrtfConfig::default();
    let mut hrtf_processor = HrtfProcessor::new(config)?;

    // 生成测试音频
    let sample_rate = 44100;
    let duration_sec = 1;
    let mut audio: Vec<f32> = Vec::with_capacity(sample_rate * duration_sec);

    for i in 0..(sample_rate * duration_sec) {
        let t = i as f32 / sample_rate as f32;
        // 440Hz正弦波
        audio.push((2.0 * std::f32::consts::PI * 440.0 * t).sin());
    }

    // 测试不同位置的音频处理
    let positions = [
        ((0.0, 0.0, -1.0), "正前方"),
        ((-1.0, 0.0, 0.0), "左侧"),
        ((1.0, 0.0, 0.0), "右侧"),
        ((0.0, 1.0, 0.0), "上方"),
    ];

    for (pos, description) in &positions {
        let output = hrtf_processor.process(
            &audio,
            *pos,
            (0.0, 0.0, 0.0), // 听者在原点
            (1.0, 0.0, 0.0, 0.0), // 单位四元数
        );

        // 计算左右声道的RMS能量
        let left_rms: f32 = output.iter().map(|s| s[0] * s[0]).sum::<f32>().sqrt();
        let right_rms: f32 = output.iter().map(|s| s[1] * s[1]).sum::<f32>().sqrt();

        println!("位置 {}: 左RMS={:.4}, 右RMS={:.4}, 比率={:.2}",
            description, left_rms, right_rms, left_rms / right_rms);
    }

    // 切换插值方法
    println!("\n切换插值方法:");
    hrtf_processor.set_interpolation_method(InterpolationMethod::Trilinear);
    println!("当前插值方法: {:?}", hrtf_processor.get_interpolation_method());

    println!();

    // ========================================================================
    // 2. 高级混响系统示例
    // ========================================================================
    println!("2. 高级混响系统示例");
    println!("-------------------");

    // 创建不同类型的房间
    let rooms = [
        ((5.0, 3.0, 4.0), ReverbPreset::SmallRoom, "小房间"),
        ((10.0, 5.0, 8.0), ReverbPreset::MediumRoom, "中型房间"),
        ((30.0, 15.0, 40.0), ReverbPreset::LargeHall, "大厅"),
        ((20.0, 30.0, 50.0), ReverbPreset::Cathedral, "教堂"),
        ((25.0, 20.0, 35.0), ReverbPreset::ConcertHall, "音乐厅"),
    ];

    for (dimensions, preset, name) in &rooms {
        let reverb = RoomReverb::from_preset(*dimensions, *preset);
        let rt60 = reverb.compute_rt60();
        let (preset_rt60, density, _, _) = preset.get_parameters();

        println!("{}: 尺寸={:?}x{:?}x{:?}m, RT60={:.2}s, 密度={:.2}",
            name, dimensions.0, dimensions.1, dimensions.2, rt60, density);
    }

    // 计算脉冲响应
    println!("\n计算脉冲响应:");
    let reverb = RoomReverb::from_preset((10.0, 5.0, 8.0), ReverbPreset::MediumRoom);
    let source_pos = (2.0, 1.5, 3.0);
    let listener_pos = (5.0, 2.0, 4.0);

    let impulse_response = reverb.compute_impulse_response(source_pos, listener_pos, 44100.0);
    let direct_distance = ((source_pos.0 - listener_pos.0).powi(2) +
        (source_pos.1 - listener_pos.1).powi(2) +
        (source_pos.2 - listener_pos.2).powi(2)).sqrt();
    let direct_delay_samples = (direct_distance / 343.0 * 44100.0) as usize;

    println!("源位置: {:?}", source_pos);
    println!("听者位置: {:?}", listener_pos);
    println!("直达声距离: {:.2}m", direct_distance);
    println!("直达声延迟: {} 样本 ({:.2}ms)",
        direct_delay_samples, direct_delay_samples as f32 / 44100.0 * 1000.0);
    println!("脉冲响应长度: {} 样本 ({:.2}ms)",
        impulse_response.len(), impulse_response.len() as f32 / 44100.0 * 1000.0);

    println!();

    // ========================================================================
    // 3. 音频遮挡系统示例
    // ========================================================================
    println!("3. 音频遮挡系统示例");
    println!("-------------------");

    let mut occlusion = AudioOcclusion::new(44100.0);

    // 测试不同材质的遮挡效果
    let materials = [
        (AcousticMaterial::air(), "空气（无遮挡）"),
        (AcousticMaterial::wood(), "木材"),
        (AcousticMaterial::glass(), "玻璃"),
        (AcousticMaterial::concrete(), "混凝土"),
        (AcousticMaterial::metal(), "金属"),
    ];

    println!("材质特性:");
    for (material, name) in &materials {
        println!("  {}: 传输系数={:.3}, 吸收系数={:.3}, 频率依赖={:.2}",
            name,
            material.transmission_coefficient,
            material.absorption_coefficient,
            material.frequency_dependency,
        );
    }

    // 模拟遮挡过渡
    println!("\n遮挡过渡效果:");
    for i in 0..10 {
        // 模拟遮挡因子从0变化到1
        let occlusion_factor = i as f32 / 10.0;

        // 创建一个简单的测试缓冲区
        let mut buffer = vec![1.0; 441]; // 10ms @ 44.1kHz

        // 应用衰减
        let gain = 1.0 - occlusion_factor * 0.5; // 最大衰减50%
        for sample in buffer.iter_mut() {
            *sample *= gain;
        }

        let result_level = buffer.iter().sum::<f32>() / buffer.len() as f32;
        println!("  遮挡={:.1}: 输出电平={:.3}", occlusion_factor, result_level);
    }

    println!();

    // ========================================================================
    // 4. 音频分析工具示例
    // ========================================================================
    println!("4. 音频分析工具示例");
    println!("-------------------");

    // 4.1 频谱分析
    println!("频谱分析:");
    let mut spectrum_analyzer = SpectrumAnalyzer::new(1024, 44100.0);

    // 创建混合频率的测试信号
    let mut test_signal = vec![0.0f32; 1024];
    for i in 0..1024 {
        let t = i as f32 / 44100.0;
        // 混合低频(100Hz)、中频(1000Hz)、高频(5000Hz)
        test_signal[i] = 0.3 * (2.0 * std::f32::consts::PI * 100.0 * t).sin() +
                        0.5 * (2.0 * std::f32::consts::PI * 1000.0 * t).sin() +
                        0.2 * (2.0 * std::f32::consts::PI * 5000.0 * t).sin();
    }

    spectrum_analyzer.analyze(&test_signal);

    println!("  频率分辨率: {:.2} Hz", spectrum_analyzer.frequency_resolution());
    println!("  低频能量 (20-250Hz): {:.4}", spectrum_analyzer.get_bass_energy());
    println!("  中频能量 (250-4000Hz): {:.4}", spectrum_analyzer.get_mid_energy());
    println!("  高频能量 (4000-20000Hz): {:.4}", spectrum_analyzer.get_high_energy());

    // 4.2 音量计量
    println!("\n音量计量:");
    let mut volume_meter = VolumeMeter::new(4800); // 100ms @ 48kHz

    // 模拟音频信号
    for i in 0..10000 {
        let t = i as f32 / 44100.0;
        let amplitude = (i as f32 / 10000.0) as f32; // 逐渐增大
        let sample = amplitude * (2.0 * std::f32::consts::PI * 440.0 * t).sin();
        volume_meter.process(sample);
    }

    println!("  RMS音量: {:.4} ({:.1} dBFS)", volume_meter.level(), volume_meter.get_level_db());
    println!("  峰值音量: {:.4} ({:.1} dBFS)", volume_meter.peak_level(), volume_meter.get_peak_db());
    println!("  峰值保持: {:.4} ({:.1} dBFS)", volume_meter.peak_hold_level(),
        20.0 * volume_meter.peak_hold_level().log10().max(-100.0));

    // 4.3 延迟测量
    println!("\n延迟测量:");
    let mut latency_meter = LatencyMeter::new(1000);

    // 模拟音频处理延迟
    let base_latency = 10.0; // 10ms基础延迟
    for i in 0..100 {
        // 添加一些随机抖动
        let jitter = (rand::random::<f32>() - 0.5) * 5.0; // ±2.5ms抖动
        let latency = base_latency + jitter;
        latency_meter.record_latency(latency);
    }

    println!("  平均延迟: {:.2} ms", latency_meter.average_latency());
    println!("  最小延迟: {:.2} ms", latency_meter.min_latency());
    println!("  最大延迟: {:.2} ms", latency_meter.max_latency());
    println!("  P50延迟: {:.2} ms", latency_meter.percentile_latency(50.0));
    println!("  P95延迟: {:.2} ms", latency_meter.percentile_latency(95.0));
    println!("  P99延迟: {:.2} ms", latency_meter.percentile_latency(99.0));
    println!("  抖动: {:.2} ms", latency_meter.jitter());

    println!();

    // ========================================================================
    // 5. 综合示例
    // ========================================================================
    println!("5. 综合示例：音频处理管道");
    println!("-------------------------");

    let pipeline_latency = &mut latency_meter;
    let pipeline_volume = &mut volume_meter;
    let pipeline_spectrum = &mut spectrum_analyzer;

    println!("音频处理管道:");
    println!("  输入 -> HRTF处理 -> 混响 -> 遮挡 -> 分析 -> 输出");

    let test_audio = vec![0.5; 1024];

    // 模拟处理管道
    let start = std::time::Instant::now();

    // 1. HRTF处理
    let _hrtf_output = hrtf_processor.process(
        &test_audio,
        (0.0, 0.0, -1.0),
        (0.0, 0.0, 0.0),
        (1.0, 0.0, 0.0, 0.0),
    );

    // 2. 混响
    let _reverb_output = test_audio.clone(); // 简化

    // 3. 遮挡
    let mut occluded_audio = test_audio.clone();
    // occlusion.apply_occlusion(&mut occluded_audio, &occlusion_result); // 简化

    // 4. 分析
    pipeline_spectrum.analyze(&occluded_audio);
    pipeline_volume.process_buffer(&occluded_audio);

    let processing_time = start.elapsed();

    pipeline_latency.record_latency(processing_time.as_secs_f64() as f32 * 1000.0);

    println!("  处理时间: {:.3} ms", processing_time.as_secs_f64() * 1000.0);
    println!("  输出RMS: {:.4}", pipeline_volume.level());
    println!("  低频能量: {:.4}", pipeline_spectrum.get_bass_energy());
    println!("  中频能量: {:.4}", pipeline_spectrum.get_mid_energy());
    println!("  高频能量: {:.4}", pipeline_spectrum.get_high_energy());

    println!("\n=== 示例完成 ===");

    Ok(())
}
