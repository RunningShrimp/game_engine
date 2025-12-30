// HRTF音频处理器
//
// 实现双耳3D音频渲染

use crate::audio::hrtf::HrtfConfig;
use std::f32::consts::PI;

/// HRTF音频处理器
pub struct HrtfProcessor {
    config: HrtfConfig,
    /// HRIR数据 (Head-Related Impulse Response)
    hrir_data: HrirDataset,
    /// FFT卷积器
    convolver: Option<FftConvolver>,
}

/// HRIR数据集
pub struct HrirDataset {
    /// HRIR滤波器 - 扁平化存储以提高性能
    /// 索引: (azimuth * elevation_steps + elevation) * 2 + channel
    pub hrir: Vec<Vec<f32>>,
    /// 采样率
    pub sample_rate: f32,
    /// 方位角分辨率（度）
    pub azimuth_resolution: f32,
    /// 仰角分辨率（度）
    pub elevation_resolution: f32,
    /// 方位角步数
    pub azimuth_steps: usize,
    /// 仰角步数
    pub elevation_steps: usize,
}

/// FFT卷积器（简化版）
struct FftConvolver {
    buffer_size: usize,
}

impl HrtfProcessor {
    /// 创建新的HRTF处理器
    pub fn new(config: HrtfConfig) -> Result<Self, String> {
        let hrir_data = HrirDataset::load_builtin()?;

        Ok(Self {
            config,
            hrir_data,
            convolver: None, // TODO: 初始化FFT卷积器
        })
    }

    /// 处理单声道音频，生成立体声双耳输出
    pub fn process(
        &mut self,
        input: &[f32],
        source_pos: (f32, f32, f32),
        listener_pos: (f32, f32, f32),
        listener_orientation: (f32, f32, f32, f32), // 四元数 (w, x, y, z)
    ) -> Vec<[f32; 2]> {
        // 1. 计算相对位置
        let relative_pos = self.compute_relative_position(source_pos, listener_pos);

        // 2. 转换为球坐标
        let (azimuth, elevation, distance) = self.to_spherical(relative_pos, listener_orientation);

        // 3. 获取HRIR
        let (left_hrir, right_hrir) = self.get_hrir(azimuth, elevation);

        // 4. 应用距离衰减
        let attenuation = self.compute_distance_attenuation(distance);

        // 5. 卷积

        if self.convolver.is_some() {
            // 简化版：不使用FFT卷积
            self.convolve_direct(input, &left_hrir, &right_hrir, attenuation)
        } else {
            self.convolve_direct(input, &left_hrir, &right_hrir, attenuation)
        }
    }

    /// 计算相对位置
    fn compute_relative_position(
        &self,
        source_pos: (f32, f32, f32),
        listener_pos: (f32, f32, f32),
    ) -> (f32, f32, f32) {
        (
            source_pos.0 - listener_pos.0,
            source_pos.1 - listener_pos.1,
            source_pos.2 - listener_pos.2,
        )
    }

    /// 转换为球坐标（方位角、仰角、距离）
    fn to_spherical(
        &self,
        pos: (f32, f32, f32),
        listener_orientation: (f32, f32, f32, f32),
    ) -> (f32, f32, f32) {
        let x = pos.0;
        let y = pos.1;
        let z = pos.2;

        let distance = (x * x + y * y + z * z).sqrt();

        // 方位角（水平方向，-180到180度）
        let azimuth = (z.atan2(x) * 180.0 / PI).to_degrees();

        // 仰角（垂直方向，-90到90度）
        let elevation = (y.atan2((x * x + z * z).sqrt()) * 180.0 / PI).to_degrees();

        (azimuth, elevation, distance)
    }

    /// 获取HRIR滤波器
    fn get_hrir(&self, azimuth: f32, elevation: f32) -> (Vec<f32>, Vec<f32>) {
        // 将角度转换为索引
        let az_idx = (((azimuth + 180.0) / self.hrir_data.azimuth_resolution) as usize)
            .min(self.hrir_data.azimuth_steps - 1);

        let el_idx = (((elevation + 90.0) / self.hrir_data.elevation_resolution) as usize)
            .min(self.hrir_data.elevation_steps - 1);

        // 计算扁平化索引
        let left_idx = (az_idx * self.hrir_data.elevation_steps + el_idx) * 2;
        let right_idx = left_idx + 1;

        let hrir_length = self.hrir_data.hrir[left_idx].len();

        // TODO: 实现双线性插值
        (
            self.hrir_data.hrir[left_idx].clone(),
            self.hrir_data.hrir[right_idx].clone(),
        )
    }

    /// 计算距离衰减
    fn compute_distance_attenuation(&self, distance: f32) -> f32 {
        // 简单的反比衰减
        if distance < 1.0 { 1.0 } else { 1.0 / distance }
    }

    /// 直接卷积（慢但简单）
    fn convolve_direct(
        &self,
        input: &[f32],
        left_hrir: &[f32],
        right_hrir: &[f32],
        attenuation: f32,
    ) -> Vec<[f32; 2]> {
        let mut output = Vec::with_capacity(input.len());

        for i in 0..input.len() {
            let mut left_sample = 0.0;
            let mut right_sample = 0.0;

            // 卷积
            for (j, &hrir_sample) in left_hrir.iter().enumerate() {
                if i >= j {
                    left_sample += input[i - j] * hrir_sample;
                }
            }

            for (j, &hrir_sample) in right_hrir.iter().enumerate() {
                if i >= j {
                    right_sample += input[i - j] * hrir_sample;
                }
            }

            output.push([left_sample * attenuation, right_sample * attenuation]);
        }

        output
    }

    /// FFT加速卷积（快速但复杂）
    fn convolve_fft(
        &self,
        _convolver: &FftConvolver,
        _input: &[f32],
        _left_hrir: &[f32],
        _right_hrir: &[f32],
        _attenuation: f32,
    ) -> Vec<[f32; 2]> {
        // TODO: 实现FFT卷积
        vec![]
    }

    /// 启用/禁用HRTF
    pub fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            self.convolver = None;
        }
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.convolver.is_some() || true // 简化版总是启用
    }
}

impl HrirDataset {
    /// 加载内置HRIR数据集
    fn load_builtin() -> Result<Self, String> {
        // 简化版：使用模拟的HRIR数据
        // 实际应用中应该加载真实的HRTF数据集（如MIT HRTF）

        let azimuth_steps = 72; // 5度间隔
        let elevation_steps = 18; // 10度间隔
        let hrir_length = 256; // HRIR长度

        let total_channels = azimuth_steps * elevation_steps * 2;
        let mut hrir = vec![vec![0.0; hrir_length]; total_channels];

        // 生成简化的HRIR（实际应该从文件加载）
        for az in 0..azimuth_steps {
            for el in 0..elevation_steps {
                let base_idx = (az * elevation_steps + el) * 2;

                // 使用split_at_mut来避免多个可变引用
                let (left_hrirs, right_hrirs) = hrir.split_at_mut(base_idx + 1);
                let left_hrir = &mut left_hrirs[base_idx];
                let right_hrir = &mut right_hrirs[0];

                // 模拟HRIR数据
                let azimuth_angle = az as f32 * 5.0 - 180.0;
                let interaural_delay = (azimuth_angle / 180.0) * 10.0; // 最大10个样本延迟

                for i in 0..hrir_length {
                    let t = i as f32 / hrir_length as f32;

                    // 简化的HRIR形状
                    let sample = (-t * 10.0).exp() * (0.5 + 0.5 * (2.0 * PI * t * 10.0).sin());

                    // 应用耳间时间差
                    if azimuth_angle < 0.0 {
                        // 声源在左侧，左耳先听到
                        left_hrir[i] = sample;
                        right_hrir[i] = if i as f32 >= interaural_delay.abs() {
                            sample * 0.8
                        } else {
                            0.0
                        };
                    } else {
                        // 声源在右侧，右耳先听到
                        left_hrir[i] = if i as f32 >= interaural_delay {
                            sample * 0.8
                        } else {
                            0.0
                        };
                        right_hrir[i] = sample;
                    }
                }
            }
        }

        Ok(Self {
            hrir,
            sample_rate: 44100.0,
            azimuth_resolution: 5.0,
            elevation_resolution: 10.0,
            azimuth_steps,
            elevation_steps,
        })
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hrtf_processor_creation() {
        let config = crate::audio::hrtf::HrtfConfig::default();
        let processor = HrtfProcessor::new(config);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_process_stereo() {
        let config = crate::audio::hrtf::HrtfConfig::default();
        let mut processor = HrtfProcessor::new(config).unwrap();

        // 生成测试音频（1秒正弦波）
        let sample_rate = 44100;
        let frequency = 440.0;
        let input: Vec<f32> = (0..sample_rate)
            .map(|i| (2.0 * PI * frequency * i as f32 / sample_rate as f32).sin())
            .collect();

        // 声源在正前方
        let source_pos = (0.0, 0.0, -1.0);
        let listener_pos = (0.0, 0.0, 0.0);
        let listener_orientation = (1.0, 0.0, 0.0, 0.0); // 单位四元数

        let output = processor.process(&input, source_pos, listener_pos, listener_orientation);

        // 验证输出是立体声
        assert_eq!(output.len(), input.len());

        // 验证左右声道相似（正前方的声源）
        let left_rms: f32 = output.iter().map(|s| s[0] * s[0]).sum::<f32>().sqrt();
        let right_rms: f32 = output.iter().map(|s| s[1] * s[1]).sum::<f32>().sqrt();

        let ratio = left_rms / right_rms;
        assert!(ratio > 0.9 && ratio < 1.1); // 允许10%误差
    }

    #[test]
    fn test_left_right_distinction() {
        let config = crate::audio::hrtf::HrtfConfig::default();
        let mut processor = HrtfProcessor::new(config).unwrap();

        let input = vec![1.0; 1000];

        // 测试左侧声源
        let source_left = (-1.0, 0.0, 0.0);
        let output_left =
            processor.process(&input, source_left, (0.0, 0.0, 0.0), (1.0, 0.0, 0.0, 0.0));

        // 测试右侧声源
        let source_right = (1.0, 0.0, 0.0);
        let output_right =
            processor.process(&input, source_right, (0.0, 0.0, 0.0), (1.0, 0.0, 0.0, 0.0));

        // 左侧声源应该在左耳更响
        let left_left = output_left.iter().map(|s| s[0].abs()).sum::<f32>();
        let left_right = output_left.iter().map(|s| s[1].abs()).sum::<f32>();

        // 右侧声源应该在右耳更响
        let right_left = output_right.iter().map(|s| s[0].abs()).sum::<f32>();
        let right_right = output_right.iter().map(|s| s[1].abs()).sum::<f32>();

        assert!(left_left > left_right);
        assert!(right_right > right_left);
    }
}
