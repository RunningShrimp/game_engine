// 环境音效系统
//
// 生成和处理自然环境音效 (风、雨、雷、雪等)

use std::f32::consts::PI;

/// 环境音效生成器
pub struct EnvironmentalAudioGenerator {
    sample_rate: f32,
}

/// 环境音效类型
#[derive(Debug, Clone, Copy)]
pub enum EnvironmentType {
    Wind,
    Rain,
    Thunder,
    Snow,
    Fire,
    Ocean,
    Forest,
    Urban,
}

impl EnvironmentalAudioGenerator {
    /// 创建新的环境音效生成器
    pub fn new(sample_rate: f32) -> Self {
        Self { sample_rate }
    }

    /// 生成风声
    pub fn generate_wind(
        &self,
        duration: f32,
        intensity: f32, // 0.0-1.0
        gustiness: f32, // 阵风强度 0.0-1.0
    ) -> Vec<f32> {
        let samples = (duration * self.sample_rate) as usize;
        let mut output = Vec::with_capacity(samples);

        // 多层噪声叠加
        for i in 0..samples {
            let t = i as f32 / self.sample_rate;

            // 基础风声 (粉红噪声)
            let base = self.pink_noise(i);

            // 阵风调制
            let gust = if gustiness > 0.0 {
                let gust_freq = 0.1 + gustiness * 0.3;
                let gust_mod = (2.0 * PI * gust_freq * t).sin() * 0.5 + 0.5;
                1.0 + gust_mod * gustiness
            } else {
                1.0
            };

            // 低频起伏
            let low_freq = (2.0 * PI * 0.05 * t).sin() * 0.3;

            let sample = base * intensity * gust * (1.0 + low_freq);
            output.push(sample);
        }

        output
    }

    /// 生成雨声
    pub fn generate_rain(
        &self,
        duration: f32,
        intensity: f32,    // 0.0-1.0
        droplet_size: f32, // 0.0 (细雨) - 1.0 (大雨)
    ) -> Vec<f32> {
        let samples = (duration * self.sample_rate) as usize;
        let mut output = vec![0.0; samples];

        // 生成雨滴撞击
        let num_droplets = (intensity * 1000.0) as usize;
        let droplet_duration = ((0.01 + droplet_size * 0.05) * self.sample_rate) as usize;

        for _ in 0..num_droplets {
            let start = (rand_random() * samples as f32) as usize;
            if start + droplet_duration > samples {
                continue;
            }

            // 雨滴撞击声音
            for j in 0..droplet_duration.min(samples - start) {
                let t = j as f32 / droplet_duration as f32;
                let envelope = (-t * 5.0).exp(); // 快速衰减
                let noise = (rand_random() - 0.5) * 2.0;
                output[start + j] += noise * envelope * intensity * (0.3 + droplet_size * 0.7);
            }
        }

        // 添加持续背景噪声
        for (i, output_sample) in output.iter_mut().enumerate().take(samples) {
            let noise = self.white_noise(i);
            *output_sample += noise * intensity * 0.1;
        }

        output
    }

    /// 生成雷声
    pub fn generate_thunder(
        &self,
        duration: f32,
        intensity: f32,
        distance: f32, // 0.0 (近) - 1.0 (远)
    ) -> Vec<f32> {
        let samples = (duration * self.sample_rate) as usize;
        let mut output = Vec::with_capacity(samples);

        // 距离衰减
        let distance_attenuation = 1.0 / (1.0 + distance * 10.0);

        // 多层滚雷
        for i in 0..samples {
            let t = i as f32 / self.sample_rate;

            // 低频轰鸣
            let rumble = (2.0 * PI * 40.0 * t).sin()
                + (2.0 * PI * 60.0 * t).sin() * 0.7
                + (2.0 * PI * 80.0 * t).sin() * 0.5;

            // 随机调制
            let modulation = self.pink_noise(i) * 0.5;

            // 包络 (冲击 + 衰减)
            let envelope = if t < 0.1 {
                t / 0.1 // 快速上升
            } else {
                (-(t - 0.1) * 2.0).exp() // 缓慢衰减
            };

            let sample = (rumble + modulation) * envelope * intensity * distance_attenuation;
            output.push(sample);
        }

        output
    }

    /// 生成雪地行走声
    pub fn generate_snow_crunch(&self, duration: f32, step_frequency: f32) -> Vec<f32> {
        let samples = (duration * self.sample_rate) as usize;
        let mut output = vec![0.0; samples];

        let num_steps = (duration * step_frequency) as usize;

        for step in 0..num_steps {
            let step_time = step as f32 / step_frequency;
            let step_start = (step_time * self.sample_rate) as usize;

            // 每步的嘎吱声
            let crunch_duration = (0.05 * self.sample_rate) as usize;

            for j in 0..crunch_duration {
                if step_start + j >= samples {
                    break;
                }

                let t = j as f32 / crunch_duration as f32;
                let envelope = (-t * 10.0).exp();

                // 高频噪声模拟压雪声
                let noise = self.white_noise(step_start + j);
                output[step_start + j] += noise * envelope * 0.3;
            }
        }

        output
    }

    /// 生成火焰声
    pub fn generate_fire(
        &self,
        duration: f32,
        intensity: f32,
        crackling: f32, // 劈啪声强度
    ) -> Vec<f32> {
        let samples = (duration * self.sample_rate) as usize;
        let mut output = Vec::with_capacity(samples);

        for i in 0..samples {
            let t = i as f32 / self.sample_rate;

            // 基础火焰声 (滤波噪声)
            let base = self.pink_noise(i);

            // 低通滤波效果
            let lpf = base * 0.7 + self.pink_noise(i - 1) * 0.3;

            // 随机劈啪声
            let crackle = if crackling > 0.0 && rand_random() < crackling * 0.001 {
                (rand_random() - 0.5) * 2.0 * 0.5
            } else {
                0.0
            };

            let sample = (lpf * 0.3 + crackle) * intensity;
            output.push(sample);
        }

        output
    }

    /// 生成海浪声
    pub fn generate_ocean(&self, duration: f32, wave_height: f32) -> Vec<f32> {
        let samples = (duration * self.sample_rate) as usize;
        let mut output = Vec::with_capacity(samples);

        for i in 0..samples {
            let t = i as f32 / self.sample_rate;

            // 基础波浪噪声
            let base_noise = self.pink_noise(i);

            // 周期性波浪
            let wave = (2.0 * PI * 0.1 * t).sin() * 0.5 + 0.5;
            let wave2 = (2.0 * PI * 0.15 * t).sin() * 0.3;

            // 泡沫声 (调制噪声)
            let foam = base_noise * (wave + wave2);

            let sample = foam * wave_height;
            output.push(sample);
        }

        output
    }

    /// 生成森林环境音
    pub fn generate_forest(
        &self,
        duration: f32,
        bird_activity: f32,
        wind_intensity: f32,
    ) -> Vec<f32> {
        let samples = (duration * self.sample_rate) as usize;
        let mut output = vec![0.0; samples];

        // 风声 (树叶)
        let wind = self.generate_wind(duration, wind_intensity, 0.3);
        for i in 0..samples {
            output[i] += wind[i] * 0.5;
        }

        // 鸟鸣
        if bird_activity > 0.0 {
            let num_birds = (bird_activity * 5.0) as usize;

            for _ in 0..num_birds {
                let start = (rand_random() * samples as f32) as usize;
                let bird_duration = ((0.5 + rand_random() * 2.0) * self.sample_rate) as usize;

                for j in 0..bird_duration.min(samples - start) {
                    let t = j as f32 / self.sample_rate;
                    let freq = 2000.0 + rand_random() * 2000.0;
                    let warble =
                        (2.0 * PI * freq * t).sin() + (2.0 * PI * (freq * 1.5) * t).sin() * 0.5;

                    let envelope = if t < 0.1 {
                        t / 0.1
                    } else {
                        (-(t - 0.1) * 3.0).exp()
                    };

                    output[start + j] += warble * envelope * bird_activity * 0.1;
                }
            }
        }

        output
    }

    /// 生成城市环境音
    pub fn generate_urban(
        &self,
        duration: f32,
        traffic_density: f32,
        time_of_day: f32, // 0.0 (深夜) - 1.0 (白天)
    ) -> Vec<f32> {
        let samples = (duration * self.sample_rate) as usize;
        let mut output = vec![0.0; samples];

        // 背景嗡嗡声
        for (i, output_sample) in output.iter_mut().enumerate().take(samples) {
            let base = self.pink_noise(i) * 0.05;
            *output_sample = base;
        }

        // 车辆经过声
        let num_vehicles = (traffic_density * 20.0) as usize;

        for _ in 0..num_vehicles {
            let start = (rand_random() * samples as f32) as usize;
            let vehicle_duration = ((1.0 + rand_random() * 3.0) * self.sample_rate) as usize;

            for j in 0..vehicle_duration.min(samples - start) {
                let t = j as f32 / self.sample_rate;
                let rel_t = t / vehicle_duration as f32;

                // 多普勒效应模拟
                let doppler_shift = 1.0 + (rel_t - 0.5) * 0.3;
                let engine_freq = 100.0 * doppler_shift;

                let engine = (2.0 * PI * engine_freq * t).sin() * 0.1;
                let tire_noise = self.white_noise(start + j) * 0.05;

                output[start + j] += (engine + tire_noise) * (1.0 - time_of_day * 0.5);
            }
        }

        output
    }

    // === 辅助函数 ===

    /// 白噪声生成
    fn white_noise(&self, _index: usize) -> f32 {
        (rand_random() - 0.5) * 2.0
    }

    /// 粉红噪声生成 (1/f噪声)
    fn pink_noise(&self, index: usize) -> f32 {
        // 简化的粉红噪声近似
        let n = index as f32;
        let mut sum = 0.0;
        for octave in 0u32..8 {
            let freq_mult = 2f32.powi(octave as i32);
            sum += (2.0 * PI * n * freq_mult / self.sample_rate).sin();
        }
        sum / 8.0
    }
}

impl Default for EnvironmentalAudioGenerator {
    fn default() -> Self {
        Self::new(44100.0)
    }
}

/// 简单随机数生成器 (线性同余)
fn rand_random() -> f32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static SEED: AtomicU32 = AtomicU32::new(12345);

    let seed = SEED.fetch_add(1, Ordering::Relaxed);
    let a = 1664525u32;
    let c = 1013904223u32;
    let m = 2u32.pow(32);

    ((seed.wrapping_mul(a).wrapping_add(c)) % m) as f32 / m as f32
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wind_generation() {
        let generator = EnvironmentalAudioGenerator::new(44100.0);
        let wind = generator.generate_wind(1.0, 0.5, 0.3);

        assert_eq!(wind.len(), 44100);
        assert!(wind.iter().any(|&x| x.abs() > 0.001)); // 应该有非零样本
    }

    #[test]
    fn test_rain_generation() {
        let generator = EnvironmentalAudioGenerator::new(44100.0);
        let rain = generator.generate_rain(1.0, 0.7, 0.5);

        assert_eq!(rain.len(), 44100);
    }

    #[test]
    fn test_thunder_generation() {
        let generator = EnvironmentalAudioGenerator::new(44100.0);
        let thunder = generator.generate_thunder(2.0, 0.8, 0.3);

        assert_eq!(thunder.len(), 88200);
    }

    #[test]
    fn test_fire_generation() {
        let generator = EnvironmentalAudioGenerator::new(44100.0);
        let fire = generator.generate_fire(1.0, 0.6, 0.4);

        assert_eq!(fire.len(), 44100);
    }
}
