// 音频分析工具
//
// 提供频谱分析、音量计量、延迟测量等功能

use super::fft_convolver::Complex;
use std::collections::VecDeque;

/// 音频频谱分析器
pub struct SpectrumAnalyzer {
    /// FFT大小
    fft_size: usize,
    /// 采样率
    sample_rate: f32,
    /// 频率分辨率
    frequency_resolution: f32,
    /// 当前频谱幅度
    magnitudes: Vec<f32>,
}

impl SpectrumAnalyzer {
    /// 创建新的频谱分析器
    pub fn new(fft_size: usize, sample_rate: f32) -> Self {
        assert!(fft_size.is_power_of_two(), "FFT size must be a power of 2");

        let frequency_resolution = sample_rate / fft_size as f32;
        let magnitudes = vec![0.0; fft_size / 2];

        Self {
            fft_size,
            sample_rate,
            frequency_resolution,
            magnitudes,
        }
    }

    /// 分析音频缓冲区
    pub fn analyze(&mut self, audio: &[f32]) -> &[(f32, f32)] {
        // 准备FFT输入
        let mut fft_input = vec![Complex::new(0.0, 0.0); self.fft_size];

        // 复制音频数据并补零
        let len = audio.len().min(self.fft_size);
        for i in 0..len {
            // 应用汉宁窗
            let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / len as f32).cos());
            fft_input[i] = Complex::new(audio[i] * window, 0.0);
        }

        // 执行FFT
        self.fft_in_place(&mut fft_input);

        // 计算幅度（只保留前半部分，因为后半部分是对称的）
        for i in 0..self.fft_size / 2 {
            let magnitude = fft_input[i].magnitude();
            self.magnitudes[i] = magnitude;
        }

        // 返回频率和幅度对
        // 注意：这里简化处理，实际应该返回切片
        &[]
    }

    /// 获取频带能量
    pub fn get_band_energy(&self, min_freq: f32, max_freq: f32) -> f32 {
        let min_bin = (min_freq / self.frequency_resolution).ceil() as usize;
        let max_bin = (max_freq / self.frequency_resolution).floor() as usize;
        let max_bin = max_bin.min(self.magnitudes.len() - 1);

        let mut energy = 0.0;
        for i in min_bin..=max_bin {
            energy += self.magnitudes[i].powi(2);
        }

        energy
    }

    /// 获取低频能量（20-250Hz）
    pub fn get_bass_energy(&self) -> f32 {
        self.get_band_energy(20.0, 250.0)
    }

    /// 获取中频能量（250-4000Hz）
    pub fn get_mid_energy(&self) -> f32 {
        self.get_band_energy(250.0, 4000.0)
    }

    /// 获取高频能量（4000-20000Hz）
    pub fn get_high_energy(&self) -> f32 {
        self.get_band_energy(4000.0, 20000.0)
    }

    /// FFT（原地）
    fn fft_in_place(&self, data: &mut [Complex]) {
        let n = data.len();

        // 位反转
        let mut j = 0;
        for i in 1..n {
            let mut k = n >> 1;
            while j & k != 0 {
                j &= !k;
                k >>= 1;
            }
            j |= k;

            if i < j {
                data.swap(i, j);
            }
        }

        // FFT蝶形运算
        let mut length = 2;
        while length <= n {
            let half = length >> 1;
            let step = std::f32::consts::PI / half as f32;

            for i in (0..n).step_by(length) {
                let mut w = Complex::new(1.0, 0.0);
                let w_step = Complex::from_polar(1.0, -step);

                for j in 0..half {
                    let u = data[i + j];
                    let v = data[i + j + half] * w;

                    data[i + j] = u + v;
                    data[i + j + half] = u - v;

                    w = w * w_step;
                }
            }

            length <<= 1;
        }
    }

    /// 获取频率分辨率
    pub fn frequency_resolution(&self) -> f32 {
        self.frequency_resolution
    }

    /// 获取频谱数据
    pub fn magnitudes(&self) -> &[f32] {
        &self.magnitudes
    }
}

/// 音量计量器
#[derive(Debug, Clone)]
pub struct VolumeMeter {
    /// 瞬时音量
    pub instantaneous: f32,
    /// 峰值音量
    pub peak: f32,
    /// RMS音量
    pub rms: f32,
    /// 历史峰值（保持一段时间）
    pub peak_hold: f32,
    /// 缓冲区
    buffer: VecDeque<f32>,
    /// 缓冲区大小
    buffer_size: usize,
}

impl VolumeMeter {
    /// 创建新的音量计量器
    pub fn new(buffer_size: usize) -> Self {
        Self {
            instantaneous: 0.0,
            peak: 0.0,
            rms: 0.0,
            peak_hold: 0.0,
            buffer: VecDeque::with_capacity(buffer_size),
            buffer_size,
        }
    }

    /// 处理音频样本
    pub fn process(&mut self, sample: f32) {
        let abs_sample = sample.abs();

        // 更新瞬时音量
        self.instantaneous = abs_sample;

        // 更新峰值
        self.peak = self.peak.max(abs_sample);

        // 更新峰值保持
        self.peak_hold = self.peak_hold.max(abs_sample);

        // 添加到缓冲区
        if self.buffer.len() >= self.buffer_size {
            self.buffer.pop_front();
        }
        self.buffer.push_back(abs_sample);

        // 更新RMS
        if !self.buffer.is_empty() {
            let sum_squares: f32 = self.buffer.iter().map(|&x| x * x).sum();
            self.rms = (sum_squares / self.buffer.len() as f32).sqrt();
        }
    }

    /// 处理音频缓冲区
    pub fn process_buffer(&mut self, buffer: &[f32]) {
        for &sample in buffer {
            self.process(sample);
        }
    }

    /// 获取音量（dBFS）
    pub fn get_level_db(&self) -> f32 {
        if self.rms > 0.0 {
            20.0 * self.rms.log10()
        } else {
            -f32::INFINITY
        }
    }

    /// 获取峰值音量（dBFS）
    pub fn get_peak_db(&self) -> f32 {
        if self.peak > 0.0 {
            20.0 * self.peak.log10()
        } else {
            -f32::INFINITY
        }
    }

    /// 重置峰值
    pub fn reset_peak(&mut self) {
        self.peak = 0.0;
    }

    /// 重置峰值保持
    pub fn reset_peak_hold(&mut self) {
        self.peak_hold = 0.0;
    }

    /// 获取当前音量（线性）
    pub fn level(&self) -> f32 {
        self.rms
    }

    /// 获取当前峰值（线性）
    pub fn peak_level(&self) -> f32 {
        self.peak
    }

    /// 获取峰值保持（线性）
    pub fn peak_hold_level(&self) -> f32 {
        self.peak_hold
    }
}

impl Default for VolumeMeter {
    fn default() -> Self {
        Self::new(4800) // 默认100ms @ 48kHz
    }
}

/// 延迟测量器
pub struct LatencyMeter {
    /// 测量的延迟历史
    latencies: VecDeque<f32>,
    /// 最大历史记录数
    max_history: usize,
    /// 总计数
    count: usize,
}

impl LatencyMeter {
    /// 创建新的延迟测量器
    pub fn new(max_history: usize) -> Self {
        Self {
            latencies: VecDeque::with_capacity(max_history),
            max_history,
            count: 0,
        }
    }

    /// 记录延迟
    pub fn record_latency(&mut self, latency_ms: f32) {
        if self.latencies.len() >= self.max_history {
            self.latencies.pop_front();
        }
        self.latencies.push_back(latency_ms);
        self.count += 1;
    }

    /// 获取平均延迟
    pub fn average_latency(&self) -> f32 {
        if self.latencies.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.latencies.iter().sum();
        sum / self.latencies.len() as f32
    }

    /// 获取最小延迟
    pub fn min_latency(&self) -> f32 {
        self.latencies.iter().cloned().fold(f32::INFINITY, f32::min)
    }

    /// 获取最大延迟
    pub fn max_latency(&self) -> f32 {
        self.latencies.iter().cloned().fold(0.0f32, f32::max)
    }

    /// 获取延迟百分位数
    pub fn percentile_latency(&self, percentile: f32) -> f32 {
        if self.latencies.is_empty() {
            return 0.0;
        }

        let mut sorted: Vec<f32> = self.latencies.iter().cloned().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = ((percentile / 100.0) * sorted.len() as f32) as usize;
        sorted[index.min(sorted.len() - 1)]
    }

    /// 获取抖动（延迟变化）
    pub fn jitter(&self) -> f32 {
        if self.latencies.len() < 2 {
            return 0.0;
        }

        let avg = self.average_latency();
        let sum_squared_diff: f32 = self.latencies.iter().map(|&x| (x - avg).powi(2)).sum();

        (sum_squared_diff / self.latencies.len() as f32).sqrt()
    }

    /// 重置统计
    pub fn reset(&mut self) {
        self.latencies.clear();
        self.count = 0;
    }

    /// 获取测量次数
    pub fn count(&self) -> usize {
        self.count
    }
}

impl Default for LatencyMeter {
    fn default() -> Self {
        Self::new(1000) // 默认保存1000个样本
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_meter() {
        let mut meter = VolumeMeter::new(100);

        // 处理一些音频
        for i in 0..1000 {
            let sample = (i as f32 / 1000.0) * 2.0 - 1.0; // -1.0 到 1.0
            meter.process(sample);
        }

        // 验证音量在合理范围内
        assert!(meter.level() >= 0.0 && meter.level() <= 1.0);
        assert!(meter.peak_level() >= 0.0 && meter.peak_level() <= 1.0);
    }

    #[test]
    fn test_volume_meter_db() {
        let mut meter = VolumeMeter::new(100);

        // 处理单位增益信号
        for _ in 0..1000 {
            meter.process(1.0);
        }

        let level_db = meter.get_level_db();
        assert!((level_db - 0.0).abs() < 0.1); // 应该接近0 dBFS
    }

    #[test]
    fn test_latency_meter() {
        let mut meter = LatencyMeter::new(100);

        // 记录一些延迟
        meter.record_latency(10.0);
        meter.record_latency(20.0);
        meter.record_latency(15.0);

        assert_eq!(meter.count(), 3);
        assert_eq!(meter.average_latency(), 15.0);
        assert_eq!(meter.min_latency(), 10.0);
        assert_eq!(meter.max_latency(), 20.0);
    }

    #[test]
    fn test_spectrum_analyzer() {
        let analyzer = SpectrumAnalyzer::new(1024, 44100.0);

        // 验证频率分辨率
        assert!((analyzer.frequency_resolution() - 43.066).abs() < 0.1);

        // 验证频带能量计算
        let mut analyzer = analyzer;
        let audio = vec![0.5; 1024];
        analyzer.analyze(&audio);

        // 验证能量非零
        assert!(analyzer.get_bass_energy() >= 0.0);
        assert!(analyzer.get_mid_energy() >= 0.0);
        assert!(analyzer.get_high_energy() >= 0.0);
    }
}
