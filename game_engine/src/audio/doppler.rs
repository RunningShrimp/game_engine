// 多普勒效应系统
//
// 模拟移动物体和听者的音调变化

use std::f32::consts::PI;

/// 多普勒效应处理器
pub struct DopplerEffect {
    /// 声音速度 (m/s)
    speed_of_sound: f32,
}

impl DopplerEffect {
    /// 创建新的多普勒效应处理器
    pub fn new() -> Self {
        Self {
            speed_of_sound: 343.0, // 20°C时的声速
        }
    }

    /// 设置声速（用于不同环境）
    pub fn set_speed_of_sound(&mut self, speed: f32) {
        self.speed_of_sound = speed;
    }

    /// 计算音调偏移（多普勒频移）
    pub fn compute_pitch_shift(
        &self,
        source_pos: (f32, f32, f32),
        source_vel: (f32, f32, f32),
        listener_pos: (f32, f32, f32),
        listener_vel: (f32, f32, f32),
    ) -> f32 {
        // 1. 计算相对位置向量
        let relative_pos = (
            source_pos.0 - listener_pos.0,
            source_pos.1 - listener_pos.1,
            source_pos.2 - listener_pos.2,
        );

        let distance = (relative_pos.0 * relative_pos.0
            + relative_pos.1 * relative_pos.1
            + relative_pos.2 * relative_pos.2)
            .sqrt();

        if distance < 0.001 {
            return 1.0; // 避免除零
        }

        // 2. 归一化方向
        let direction = (
            relative_pos.0 / distance,
            relative_pos.1 / distance,
            relative_pos.2 / distance,
        );

        // 3. 计算径向速度分量
        let v_source_radial =
            source_vel.0 * direction.0 + source_vel.1 * direction.1 + source_vel.2 * direction.2;

        let v_listener_radial = listener_vel.0 * direction.0
            + listener_vel.1 * direction.1
            + listener_vel.2 * direction.2;

        // 4. 多普勒公式：f' = f * (c + vr) / (c + vs)
        // 其中：
        //   c = 声速
        //   vr = 听者径向速度
        //   vs = 声源径向速度
        let pitch_shift =
            (self.speed_of_sound + v_listener_radial) / (self.speed_of_sound + v_source_radial);

        pitch_shift
    }

    /// 计算频率偏移（Hz）
    pub fn compute_frequency_shift(
        &self,
        base_frequency: f32,
        source_pos: (f32, f32, f32),
        source_vel: (f32, f32, f32),
        listener_pos: (f32, f32, f32),
        listener_vel: (f32, f32, f32),
    ) -> f32 {
        let pitch_shift =
            self.compute_pitch_shift(source_pos, source_vel, listener_pos, listener_vel);

        base_frequency * pitch_shift
    }

    /// 应用多普勒效应到音频缓冲区
    pub fn apply_to_buffer(&self, buffer: &mut [f32], sample_rate: f32, pitch_shift: f32) {
        // 简化版：仅调整音调
        // 实际应用中应使用resampling算法
        if (pitch_shift - 1.0).abs() < 0.01 {
            return; // 变化太小，忽略
        }

        // 简单的重采样（线性插值）
        let mut output = Vec::with_capacity(buffer.len());

        for i in 0..buffer.len() {
            let src_idx = (i as f32 * pitch_shift) as usize;
            if src_idx < buffer.len() - 1 {
                let frac = (i as f32 * pitch_shift) - src_idx as f32;
                let sample = buffer[src_idx] * (1.0 - frac) + buffer[src_idx + 1] * frac;
                output.push(sample);
            } else {
                output.push(0.0);
            }
        }

        // 复制回原缓冲区
        for (i, &sample) in output.iter().enumerate() {
            if i < buffer.len() {
                buffer[i] = sample;
            }
        }
    }
}

impl Default for DopplerEffect {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_shift_static() {
        let doppler = DopplerEffect::new();

        // 静止情况：无偏移
        let shift = doppler.compute_pitch_shift(
            (0.0, 0.0, 0.0),
            (0.0, 0.0, 0.0),
            (10.0, 0.0, 0.0),
            (0.0, 0.0, 0.0),
        );

        assert!((shift - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_pitch_shift_moving_source() {
        let doppler = DopplerEffect::new();

        // 声源向听者移动
        let shift = doppler.compute_pitch_shift(
            (10.0, 0.0, 0.0), // 声源在x=10
            (-5.0, 0.0, 0.0), // 声源向负x移动
            (0.0, 0.0, 0.0),  // 听者在原点
            (0.0, 0.0, 0.0),  // 听者静止
        );

        // 声源靠近，频率应该升高
        assert!(shift > 1.0);
    }

    #[test]
    fn test_frequency_shift() {
        let doppler = DopplerEffect::new();

        let base_freq = 440.0; // A4音符
        let shifted = doppler.compute_frequency_shift(
            base_freq,
            (10.0, 0.0, 0.0),
            (34.0, 0.0, 0.0), // 声源以34m/s移动(约122km/h)
            (0.0, 0.0, 0.0),
            (0.0, 0.0, 0.0),
        );

        // 频率应该改变
        assert!((shifted - base_freq).abs() > 0.1);
    }

    #[test]
    fn test_apply_to_buffer() {
        let doppler = DopplerEffect::new();

        let mut buffer = vec![1.0f32; 100];
        let original = buffer.clone();

        // 应用2倍音调偏移
        doppler.apply_to_buffer(&mut buffer, 44100.0, 2.0);

        // 缓冲区应该改变
        assert_ne!(buffer, original);

        // 长度应该保持
        assert_eq!(buffer.len(), 100);
    }
}
