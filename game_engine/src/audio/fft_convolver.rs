// FFT卷积器 - 高效音频处理
//
// 使用FFT加速线性卷积运算

use std::f32::consts::PI;

/// FFT卷积器
pub struct FftConvolver {
    /// FFT大小 (必须是2的幂)
    fft_size: usize,
    /// 输入缓冲区
    input_buffer: Vec<f32>,
    /// 频域滤波器 (HRIR)
    filter_freq: Vec<Vec<Complex>>,
    /// 输出缓冲区
    output_buffer: Vec<f32>,
    /// 处理位置
    process_pos: usize,
}

/// 复数
#[derive(Clone, Copy, Debug)]
pub struct Complex {
    pub re: f32,
    pub im: f32,
}

impl Complex {
    pub fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }

    pub fn from_polar(r: f32, theta: f32) -> Self {
        Self {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    pub fn magnitude(&self) -> f32 {
        (self.re * self.re + self.im * self.im).sqrt()
    }

    pub fn phase(&self) -> f32 {
        self.im.atan2(self.re)
    }
}

impl std::ops::Add for Complex {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }
}

impl std::ops::Mul for Complex {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        // (a + bi)(c + di) = (ac - bd) + (ad + bc)i
        Self {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }
}

impl std::ops::Sub for Complex {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            re: self.re - other.re,
            im: self.im - other.im,
        }
    }
}

impl FftConvolver {
    /// 创建新的FFT卷积器
    pub fn new(fft_size: usize) -> Self {
        assert!(fft_size.is_power_of_two(), "FFT size must be a power of 2");

        Self {
            fft_size,
            input_buffer: vec![0.0; fft_size],
            filter_freq: Vec::new(),
            output_buffer: vec![0.0; fft_size],
            process_pos: 0,
        }
    }

    /// 设置滤波器 (HRIR)
    pub fn set_filter(&mut self, filter: &[f32]) {
        // 准备FFT缓冲区
        let mut fft_buffer = vec![Complex::new(0.0, 0.0); self.fft_size];

        // 复制滤波器并补零
        let len = filter.len().min(self.fft_size);
        for i in 0..len {
            fft_buffer[i] = Complex::new(filter[i], 0.0);
        }

        // 执行FFT
        self.fft_in_place(&mut fft_buffer);

        // 存储频域滤波器
        self.filter_freq = vec![fft_buffer.clone(), fft_buffer];
    }

    /// 处理音频块
    pub fn process(&mut self, input: &[f32], output_left: &mut [f32], output_right: &mut [f32]) {
        let block_size = input.len().min(self.fft_size / 2);

        // 复制输入到缓冲区
        let start = self.process_pos % self.fft_size;
        for i in 0..block_size {
            self.input_buffer[(start + i) % self.fft_size] = input[i];
        }

        // 处理左右声道
        if self.filter_freq.len() >= 2 {
            // 准备FFT输入
            let mut fft_input = vec![Complex::new(0.0, 0.0); self.fft_size];
            for i in 0..self.fft_size {
                fft_input[i] = Complex::new(self.input_buffer[i], 0.0);
            }

            // 执行FFT
            self.fft_in_place(&mut fft_input);

            // 频域相乘 (左声道)
            let mut fft_left = fft_input.clone();
            for i in 0..self.fft_size {
                fft_left[i] = fft_left[i] * self.filter_freq[0][i];
            }

            // 频域相乘 (右声道)
            let mut fft_right = fft_input;
            for i in 0..self.fft_size {
                fft_right[i] = fft_right[i] * self.filter_freq[1][i];
            }

            // IFFT回时域
            self.ifft_in_place(&mut fft_left);
            self.ifft_in_place(&mut fft_right);

            // 复制输出
            for i in 0..block_size {
                if i < output_left.len() {
                    output_left[i] = fft_left[i].re / self.fft_size as f32;
                }
                if i < output_right.len() {
                    output_right[i] = fft_right[i].re / self.fft_size as f32;
                }
            }
        }

        self.process_pos += block_size;
    }

    /// Cooley-Tukey FFT算法 (原地)
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
            let step = PI / half as f32;

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

    /// IFFT (原地)
    fn ifft_in_place(&self, data: &mut [Complex]) {
        // 共轭
        for sample in data.iter_mut() {
            sample.im = -sample.im;
        }

        // FFT
        self.fft_in_place(data);

        // 共轭并缩放
        let scale = 1.0 / data.len() as f32;
        for sample in data.iter_mut() {
            sample.im = -sample.im;
            sample.re *= scale;
            sample.im *= scale;
        }
    }

    /// 重置状态
    pub fn reset(&mut self) {
        self.input_buffer.fill(0.0);
        self.output_buffer.fill(0.0);
        self.process_pos = 0;
    }
}

impl Default for FftConvolver {
    fn default() -> Self {
        Self::new(2048) // 默认2048点FFT
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_operations() {
        let a = Complex::new(3.0, 4.0);
        let b = Complex::new(1.0, 2.0);

        let sum = a + b;
        assert!((sum.re - 4.0).abs() < 0.001);
        assert!((sum.im - 6.0).abs() < 0.001);

        let product = a * b;
        assert!((product.re - (-5.0)).abs() < 0.001);
        assert!((product.im - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_complex_polar() {
        let c = Complex::from_polar(5.0, 0.927295); // magnitude 5, phase ~53.13°
        assert!((c.magnitude() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_fft_convolver_creation() {
        let convolver = FftConvolver::new(1024);
        assert_eq!(convolver.fft_size(), 1024);
    }

    #[test]
    fn test_filter_setting() {
        let mut convolver = FftConvolver::new(256);
        let filter = vec![1.0, 0.5, 0.25, 0.125];

        convolver.set_filter(&filter);

        assert!(!convolver.filter_freq.is_empty());
        assert_eq!(convolver.filter_freq[0].len(), 256);
    }

    #[test]
    fn test_fft_round_trip() {
        let mut convolver = FftConvolver::new(256);

        // 创建测试信号
        let mut signal = vec![Complex::new(0.0, 0.0); 256];
        for i in 0..256 {
            let t = i as f32 / 256.0;
            signal[i] = Complex::new((2.0 * PI * 10.0 * t).sin(), 0.0);
        }

        // 保存原始信号
        let original = signal.clone();

        // FFT -> IFFT
        convolver.fft_in_place(&mut signal);
        convolver.ifft_in_place(&mut signal);

        // 验证恢复
        for i in 0..256 {
            assert!((signal[i].re - original[i].re).abs() < 0.01);
            assert!(signal[i].im.abs() < 0.01);
        }
    }
}
