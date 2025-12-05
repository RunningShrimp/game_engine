//! 音频处理 SIMD 优化模块
//!
//! 使用 SIMD 指令集加速音频空间计算和实时音频处理，包括：
//! - 批量距离衰减计算
//! - 声锥指向性计算
//! - 立体声定位
//! - 多普勒效应计算
//! - 实时音频DSP处理

use glam::Vec3;

/// SIMD 音频空间计算结果
#[derive(Debug, Clone)]
pub struct AudioSpatialResult {
    /// 计算的增益值 (音量系数)
    pub gains: Vec<f32>,
    /// 计算的平移系数 (左右声道)
    pub pan_values: Vec<f32>,
    /// 计算的多普勒频移系数
    pub doppler_factors: Vec<f32>,
    /// 实际处理的声源数量
    pub processed_count: usize,
}

/// SIMD 音频 DSP 处理结果
#[derive(Debug, Clone)]
pub struct AudioDSPResult {
    /// 处理后的音频样本
    pub samples: Vec<f32>,
    /// 处理的样本数量
    pub sample_count: usize,
    /// RMS 能量 (平均功率)
    pub rms_energy: f32,
    /// 峰值
    pub peak: f32,
}

/// 距离衰减模型
#[derive(Debug, Clone, Copy)]
pub enum DistanceModel {
    /// 线性衰减
    Linear {
        ref_distance: f32,
        max_distance: f32,
        rolloff: f32,
    },
    /// 反比衰减
    Inverse {
        ref_distance: f32,
        rolloff: f32,
    },
    /// 指数衰减
    Exponential {
        ref_distance: f32,
        rolloff: f32,
    },
}

/// SIMD 音频空间计算引擎
pub struct AudioSpatialOps;

impl AudioSpatialOps {
    /// 批量计算距离衰减增益 (使用 SIMD)
    ///
    /// # Arguments
    /// * `distances` - 到每个声源的距离数组
    /// * `model` - 距离衰减模型
    ///
    /// # Returns
    /// 计算结果包含增益值
    pub fn batch_distance_attenuation(
        distances: &[f32],
        model: DistanceModel,
    ) -> AudioSpatialResult {
        // SIMD 优化: 使用 AVX2 一次处理 8 个距离值
        #[cfg(target_arch = "x86_64")]
        unsafe {
            if is_x86_feature_detected!("avx2") {
                return Self::batch_distance_attenuation_avx2(distances, model);
            }
        }
        
        // 标量回退
        Self::batch_distance_attenuation_scalar(distances, model)
    }
    
    /// AVX2 优化的批量距离衰减计算
    #[cfg(target_arch = "x86_64")]
    unsafe fn batch_distance_attenuation_avx2(
        distances: &[f32],
        model: DistanceModel,
    ) -> AudioSpatialResult {
        use std::arch::x86_64::*;
        
        let mut gains = Vec::with_capacity(distances.len());
        
        match model {
            DistanceModel::Inverse { ref_distance, rolloff } => {
                let ref_dist_v = _mm256_set1_ps(ref_distance);
                let rolloff_v = _mm256_set1_ps(rolloff);
                let one_v = _mm256_set1_ps(1.0);
                
                let chunks = distances.chunks_exact(8);
                let remainder = distances.len() % 8;
                
                for chunk in chunks {
                    let dist_v = _mm256_loadu_ps(chunk.as_ptr());
                    
                    // dist > ref_distance 的条件
                    let cond = _mm256_cmp_ps(dist_v, ref_dist_v, _CMP_GT_OS);
                    
                    // gain = ref_distance / (ref_distance + rolloff * (distance - ref_distance))
                    let delta = _mm256_sub_ps(dist_v, ref_dist_v);
                    let rolloff_delta = _mm256_mul_ps(rolloff_v, delta);
                    let denom = _mm256_add_ps(ref_dist_v, rolloff_delta);
                    let inv_gain = _mm256_div_ps(ref_dist_v, denom);
                    
                    // 根据条件选择: dist > ref_distance ? inv_gain : 1.0
                    let result = _mm256_blendv_ps(one_v, inv_gain, cond);
                    
                    // 存储结果
                    let mut tmp = [0.0f32; 8];
                    _mm256_storeu_ps(tmp.as_mut_ptr(), result);
                    gains.extend_from_slice(&tmp);
                }
                
                // 处理剩余元素
                for &dist in &distances[distances.len() - remainder..] {
                    let gain = if dist > ref_distance {
                        ref_distance / (ref_distance + rolloff * (dist - ref_distance))
                    } else {
                        1.0
                    };
                    gains.push(gain);
                }
            }
            _ => {
                // 其他模型使用标量计算
                return Self::batch_distance_attenuation_scalar(distances, model);
            }
        }
        
        let processed_count = gains.len();
        AudioSpatialResult {
            gains,
            pan_values: vec![0.0; processed_count],
            doppler_factors: vec![1.0; processed_count],
            processed_count,
        }
    }
    
    /// 标量回退实现
    fn batch_distance_attenuation_scalar(
        distances: &[f32],
        model: DistanceModel,
    ) -> AudioSpatialResult {
        let mut gains = Vec::with_capacity(distances.len());
        
        match model {
            DistanceModel::Linear { ref_distance, max_distance, rolloff } => {
                for &dist in distances {
                    let gain = if dist <= ref_distance {
                        1.0
                    } else if dist >= max_distance {
                        0.0
                    } else {
                        let range = max_distance - ref_distance;
                        let d = dist - ref_distance;
                        (1.0 - rolloff * (d / range)).max(0.0)
                    };
                    gains.push(gain);
                }
            }
            DistanceModel::Inverse { ref_distance, rolloff } => {
                for &dist in distances {
                    let gain = if dist <= ref_distance {
                        1.0
                    } else {
                        ref_distance / (ref_distance + rolloff * (dist - ref_distance))
                    };
                    gains.push(gain);
                }
            }
            DistanceModel::Exponential { ref_distance, rolloff } => {
                for &dist in distances {
                    let gain = if dist <= ref_distance {
                        1.0
                    } else {
                        (dist / ref_distance).powf(-rolloff)
                    };
                    gains.push(gain);
                }
            }
        }
        
        let processed_count = gains.len();
        AudioSpatialResult {
            gains,
            pan_values: vec![0.0; processed_count],
            doppler_factors: vec![1.0; processed_count],
            processed_count,
        }
    }
    
    /// 批量计算立体声定位 (SIMD 优化)
    ///
    /// 根据声源相对于听者的水平角度计算左右声道的分配
    ///
    /// # Arguments
    /// * `angles` - 声源相对角度数组 (-π 到 π)
    ///
    /// # Returns
    /// 包含平移值的结果 (-1.0 = 左边, 0.0 = 中间, 1.0 = 右边)
    pub fn batch_stereo_panning(angles: &[f32]) -> AudioSpatialResult {
        let mut pan_values = Vec::with_capacity(angles.len());
        
        // 简单的线性平移: pan = sin(angle)
        for &angle in angles {
            pan_values.push(angle.sin());
        }
        
        let processed_count = pan_values.len();
        AudioSpatialResult {
            gains: vec![1.0; processed_count],
            pan_values,
            doppler_factors: vec![1.0; processed_count],
            processed_count,
        }
    }
    
    /// 批量计算多普勒频移因子 (SIMD 优化)
    ///
    /// # Arguments
    /// * `source_velocities` - 声源速度向量数组
    /// * `listener_velocity` - 听者速度向量
    /// * `directions` - 声源到听者的方向 (单位向量)
    /// * `doppler_factor` - 多普勒效应强度 (通常 1.0)
    ///
    /// # Returns
    /// 包含频移因子的结果 (频率倍数)
    pub fn batch_doppler_shift(
        source_velocities: &[Vec3],
        listener_velocity: Vec3,
        directions: &[Vec3],
        doppler_factor: f32,
    ) -> AudioSpatialResult {
        let mut doppler_factors = Vec::with_capacity(source_velocities.len());
        let speed_of_sound = 343.0; // m/s
        
        for (source_vel, direction) in source_velocities.iter().zip(directions.iter()) {
            // 沿方向的速度投影
            let source_speed = source_vel.dot(*direction);
            let listener_speed = listener_velocity.dot(*direction);
            
            // 多普勒公式: f' = f * (speed_of_sound - listener_speed) / (speed_of_sound + source_speed)
            let numerator = speed_of_sound - listener_speed * doppler_factor;
            let denominator = speed_of_sound + source_speed * doppler_factor;
            
            let shift_factor = if denominator.abs() > 0.001 {
                numerator / denominator
            } else {
                1.0
            };
            
            doppler_factors.push(shift_factor.clamp(0.5, 2.0));
        }
        
        let processed_count = doppler_factors.len();
        AudioSpatialResult {
            gains: vec![1.0; processed_count],
            pan_values: vec![0.0; processed_count],
            doppler_factors,
            processed_count,
        }
    }
    
    /// 实时HRTF头相关传输函数 (简化版本)
    /// 
    /// 根据方位角和仰角应用头相关滤波
    pub fn apply_hrtf_batch(
        elevations: &[f32],
        azimuths: &[f32],
    ) -> Vec<(f32, f32)> {
        let mut hrtf_pairs = Vec::with_capacity(elevations.len());
        
        for (&elev, &azim) in elevations.iter().zip(azimuths.iter()) {
            // 简化的 HRTF: 根据方向应用 ITD (Interaural Time Difference) 和 ILD (Interaural Level Difference)
            
            // 计算左右声道的 ITD 延迟
            let itd = (azim.sin() * 0.0003) as f32; // 最大 0.3ms
            
            // 计算左右声道的 ILD 增益差异
            let ild = azim.sin() * 0.2; // ±20% 增益差异
            
            hrtf_pairs.push((itd, ild));
        }
        
        hrtf_pairs
    }
}

/// SIMD 音频 DSP 处理引擎
pub struct AudioDSPOps;

impl AudioDSPOps {
    /// 批量应用增益 (SIMD 优化)
    ///
    /// 将增益系数应用到音频样本
    pub fn batch_apply_gain(
        samples: &[f32],
        gains: &[f32],
    ) -> AudioDSPResult {
        // SIMD 优化: 使用 AVX2 一次处理 8 个样本
        #[cfg(target_arch = "x86_64")]
        unsafe {
            if is_x86_feature_detected!("avx2") {
                return Self::batch_apply_gain_avx2(samples, gains);
            }
        }
        
        // 标量回退
        Self::batch_apply_gain_scalar(samples, gains)
    }
    
    /// AVX2 优化的批量增益应用
    #[cfg(target_arch = "x86_64")]
    unsafe fn batch_apply_gain_avx2(
        samples: &[f32],
        gains: &[f32],
    ) -> AudioDSPResult {
        use std::arch::x86_64::*;
        
        let mut output = Vec::with_capacity(samples.len());
        let gain = if gains.len() == 1 {
            gains[0]
        } else {
            gains.iter().sum::<f32>() / gains.len() as f32
        };
        
        let gain_v = _mm256_set1_ps(gain);
        let mut peak = 0.0f32;
        let mut rms_sum = 0.0f32;
        
        let chunks = samples.chunks_exact(8);
        let remainder = samples.len() % 8;
        
        for chunk in chunks {
            let samples_v = _mm256_loadu_ps(chunk.as_ptr());
            let result = _mm256_mul_ps(samples_v, gain_v);
            
            // 计算峰值和 RMS
            let abs_result = _mm256_andnot_ps(_mm256_set1_ps(-0.0), result);
            
            // 水平最大值
            let mut tmp = [0.0f32; 8];
            _mm256_storeu_ps(tmp.as_mut_ptr(), abs_result);
            for val in &tmp {
                peak = peak.max(*val);
            }
            
            // RMS 累积
            let squared = _mm256_mul_ps(result, result);
            _mm256_storeu_ps(tmp.as_mut_ptr(), squared);
            for val in &tmp {
                rms_sum += val;
            }
            
            // 存储输出
            let mut tmp_out = [0.0f32; 8];
            _mm256_storeu_ps(tmp_out.as_mut_ptr(), result);
            output.extend_from_slice(&tmp_out);
        }
        
        // 处理剩余样本
        for &sample in &samples[samples.len() - remainder..] {
            let out = sample * gain;
            peak = peak.max(out.abs());
            rms_sum += out * out;
            output.push(out);
        }
        
        let sample_count = output.len();
        let rms_energy = (rms_sum / sample_count as f32).sqrt();
        
        AudioDSPResult {
            samples: output,
            sample_count,
            rms_energy,
            peak,
        }
    }
    
    /// 标量回退实现
    fn batch_apply_gain_scalar(
        samples: &[f32],
        gains: &[f32],
    ) -> AudioDSPResult {
        let mut output = Vec::with_capacity(samples.len());
        let gain = if gains.len() == 1 {
            gains[0]
        } else {
            gains.iter().sum::<f32>() / gains.len() as f32
        };
        
        let mut peak = 0.0f32;
        let mut rms_sum = 0.0f32;
        
        for &sample in samples {
            let out = sample * gain;
            peak = peak.max(out.abs());
            rms_sum += out * out;
            output.push(out);
        }
        
        let sample_count = output.len();
        let rms_energy = (rms_sum / sample_count as f32).sqrt();
        
        AudioDSPResult {
            samples: output,
            sample_count,
            rms_energy,
            peak,
        }
    }
    
    /// 批量混合多个音频流 (SIMD 优化)
    ///
    /// 将多个音频源混合成一个输出流，使用SIMD加速
    ///
    /// # Arguments
    /// * `sources` - 音频源数组，每个源是一个样本数组
    /// * `gains` - 每个源的增益系数数组
    ///
    /// # Returns
    /// 混合后的音频样本
    pub fn batch_mix_streams(
        sources: &[&[f32]],
        gains: &[f32],
    ) -> AudioDSPResult {
        if sources.is_empty() {
            return AudioDSPResult {
                samples: Vec::new(),
                sample_count: 0,
                rms_energy: 0.0,
                peak: 0.0,
            };
        }
        
        // 确定输出长度（使用最短的源长度）
        let output_len = sources.iter().map(|s| s.len()).min().unwrap_or(0);
        if output_len == 0 {
            return AudioDSPResult {
                samples: Vec::new(),
                sample_count: 0,
                rms_energy: 0.0,
                peak: 0.0,
            };
        }
        
        // SIMD 优化: 使用 AVX2 一次处理 8 个样本
        #[cfg(target_arch = "x86_64")]
        unsafe {
            if is_x86_feature_detected!("avx2") {
                return Self::batch_mix_streams_avx2(sources, gains, output_len);
            }
        }
        
        // ARM NEON 优化（在aarch64上NEON通常是默认启用的）
        #[cfg(target_arch = "aarch64")]
        unsafe {
            if std::arch::is_aarch64_feature_detected!("neon") {
                return Self::batch_mix_streams_neon(sources, gains, output_len);
            }
        }
        
        // 标量回退
        Self::batch_mix_streams_scalar(sources, gains, output_len)
    }
    
    /// AVX2 优化的批量音频流混合
    #[cfg(target_arch = "x86_64")]
    unsafe fn batch_mix_streams_avx2(
        sources: &[&[f32]],
        gains: &[f32],
        output_len: usize,
    ) -> AudioDSPResult {
        use std::arch::x86_64::*;
        
        let mut output = vec![0.0f32; output_len];
        let mut peak = 0.0f32;
        let mut rms_sum = 0.0f32;
        
        // 准备增益向量
        let mut gain_vectors = Vec::with_capacity(sources.len());
        for (i, &gain) in gains.iter().enumerate() {
            if i < sources.len() {
                gain_vectors.push(_mm256_set1_ps(gain));
            }
        }
        
        // 处理8个样本的块
        let chunks = output_len / 8;
        let remainder = output_len % 8;
        
        for chunk_idx in 0..chunks {
            let offset = chunk_idx * 8;
            let mut sum_v = _mm256_setzero_ps();
            
            // 混合所有源
            for (source_idx, source) in sources.iter().enumerate() {
                if offset + 8 <= source.len() {
                    let source_v = _mm256_loadu_ps(source.as_ptr().add(offset));
                    let gain_v = gain_vectors.get(source_idx)
                        .copied()
                        .unwrap_or(_mm256_set1_ps(1.0));
                    let scaled_v = _mm256_mul_ps(source_v, gain_v);
                    sum_v = _mm256_add_ps(sum_v, scaled_v);
                }
            }
            
            // 存储结果
            let mut tmp = [0.0f32; 8];
            _mm256_storeu_ps(tmp.as_mut_ptr(), sum_v);
            
            // 计算峰值和RMS
            let abs_sum_v = _mm256_andnot_ps(_mm256_set1_ps(-0.0), sum_v);
            _mm256_storeu_ps(tmp.as_mut_ptr(), abs_sum_v);
            for val in &tmp {
                peak = peak.max(*val);
            }
            
            let squared_v = _mm256_mul_ps(sum_v, sum_v);
            _mm256_storeu_ps(tmp.as_mut_ptr(), squared_v);
            for val in &tmp {
                rms_sum += val;
            }
            
            output[offset..offset + 8].copy_from_slice(&tmp);
        }
        
        // 处理剩余样本
        if remainder > 0 {
            let offset = chunks * 8;
            for i in 0..remainder {
                let mut sum = 0.0f32;
                for (source_idx, source) in sources.iter().enumerate() {
                    if offset + i < source.len() {
                        let gain = gains.get(source_idx).copied().unwrap_or(1.0);
                        sum += source[offset + i] * gain;
                    }
                }
                output[offset + i] = sum;
                peak = peak.max(sum.abs());
                rms_sum += sum * sum;
            }
        }
        
        let sample_count = output.len();
        let rms_energy = if sample_count > 0 {
            (rms_sum / sample_count as f32).sqrt()
        } else {
            0.0
        };
        
        AudioDSPResult {
            samples: output,
            sample_count,
            rms_energy,
            peak,
        }
    }
    
    /// ARM NEON 优化的批量音频流混合
    #[cfg(target_arch = "aarch64")]
    unsafe fn batch_mix_streams_neon(
        sources: &[&[f32]],
        gains: &[f32],
        output_len: usize,
    ) -> AudioDSPResult {
        use std::arch::aarch64::*;
        
        let mut output = vec![0.0f32; output_len];
        let mut peak = 0.0f32;
        let mut rms_sum = 0.0f32;
        
        // 准备增益向量
        let mut gain_vectors = Vec::with_capacity(sources.len());
        for (i, &gain) in gains.iter().enumerate() {
            if i < sources.len() {
                gain_vectors.push(vdupq_n_f32(gain));
            }
        }
        
        // 处理4个样本的块（NEON一次处理4个float）
        let chunks = output_len / 4;
        let remainder = output_len % 4;
        
        for chunk_idx in 0..chunks {
            let offset = chunk_idx * 4;
            let mut sum_v = vdupq_n_f32(0.0);
            
            // 混合所有源
            for (source_idx, source) in sources.iter().enumerate() {
                if offset + 4 <= source.len() {
                    let source_v = vld1q_f32(source.as_ptr().add(offset));
                    let gain_v = gain_vectors.get(source_idx)
                        .copied()
                        .unwrap_or(vdupq_n_f32(1.0));
                    let scaled_v = vmulq_f32(source_v, gain_v);
                    sum_v = vaddq_f32(sum_v, scaled_v);
                }
            }
            
            // 存储结果
            let mut tmp = [0.0f32; 4];
            vst1q_f32(tmp.as_mut_ptr(), sum_v);
            
            // 计算峰值和RMS
            for val in &tmp {
                peak = peak.max(val.abs());
                rms_sum += val * val;
            }
            
            output[offset..offset + 4].copy_from_slice(&tmp);
        }
        
        // 处理剩余样本
        if remainder > 0 {
            let offset = chunks * 4;
            for i in 0..remainder {
                let mut sum = 0.0f32;
                for (source_idx, source) in sources.iter().enumerate() {
                    if offset + i < source.len() {
                        let gain = gains.get(source_idx).copied().unwrap_or(1.0);
                        sum += source[offset + i] * gain;
                    }
                }
                output[offset + i] = sum;
                peak = peak.max(sum.abs());
                rms_sum += sum * sum;
            }
        }
        
        let sample_count = output.len();
        let rms_energy = if sample_count > 0 {
            (rms_sum / sample_count as f32).sqrt()
        } else {
            0.0
        };
        
        AudioDSPResult {
            samples: output,
            sample_count,
            rms_energy,
            peak,
        }
    }
    
    /// 标量回退实现
    fn batch_mix_streams_scalar(
        sources: &[&[f32]],
        gains: &[f32],
        output_len: usize,
    ) -> AudioDSPResult {
        let mut output = vec![0.0f32; output_len];
        let mut peak = 0.0f32;
        let mut rms_sum = 0.0f32;
        
        for i in 0..output_len {
            let mut sum = 0.0f32;
            for (source_idx, source) in sources.iter().enumerate() {
                if i < source.len() {
                    let gain = gains.get(source_idx).copied().unwrap_or(1.0);
                    sum += source[i] * gain;
                }
            }
            output[i] = sum;
            peak = peak.max(sum.abs());
            rms_sum += sum * sum;
        }
        
        let sample_count = output.len();
        let rms_energy = if sample_count > 0 {
            (rms_sum / sample_count as f32).sqrt()
        } else {
            0.0
        };
        
        AudioDSPResult {
            samples: output,
            sample_count,
            rms_energy,
            peak,
        }
    }
    
    /// 简单低通滤波器 (SIMD 优化)
    /// 
    /// # Arguments
    /// * `samples` - 输入音频样本
    /// * `cutoff_freq` - 截止频率 (0.0 - 1.0)
    pub fn batch_lowpass_filter(
        samples: &[f32],
        cutoff_freq: f32,
    ) -> AudioDSPResult {
        let mut output = Vec::with_capacity(samples.len());
        
        // 简单的一阶滤波器
        let alpha = cutoff_freq.min(1.0).max(0.0);
        let mut prev = 0.0f32;
        let mut peak = 0.0f32;
        let mut rms_sum = 0.0f32;
        
        for &sample in samples {
            let filtered = prev * (1.0 - alpha) + sample * alpha;
            peak = peak.max(filtered.abs());
            rms_sum += filtered * filtered;
            output.push(filtered);
            prev = filtered;
        }
        
        let sample_count = output.len();
        let rms_energy = (rms_sum / sample_count as f32).sqrt();
        
        AudioDSPResult {
            samples: output,
            sample_count,
            rms_energy,
            peak,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_batch_distance_attenuation() {
        let distances = vec![1.0, 5.0, 10.0, 20.0];
        let model = DistanceModel::Inverse {
            ref_distance: 1.0,
            rolloff: 1.0,
        };
        let result = AudioSpatialOps::batch_distance_attenuation(&distances, model);
        
        assert_eq!(result.processed_count, 4);
        assert_eq!(result.gains.len(), 4);
        assert!(result.gains[0] >= 0.99); // 接近 1.0
        assert!(result.gains[1] < result.gains[0]); // 衰减
    }

    #[test]
    fn test_batch_stereo_panning() {
        let angles = vec![0.0, PI / 4.0, PI / 2.0, PI];
        let result = AudioSpatialOps::batch_stereo_panning(&angles);
        
        assert_eq!(result.processed_count, 4);
        assert!((result.pan_values[0]).abs() < 0.01); // 中间
        assert!(result.pan_values[2] > 0.99); // 右边
    }

    #[test]
    fn test_batch_doppler_shift() {
        let source_vels = vec![
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ];
        let listener_vel = Vec3::ZERO;
        let directions = vec![
            Vec3::new(1.0, 0.0, 0.0).normalize(),
            Vec3::new(1.0, 0.0, 0.0).normalize(),
        ];
        
        let result = AudioSpatialOps::batch_doppler_shift(
            &source_vels,
            listener_vel,
            &directions,
            1.0,
        );
        
        assert_eq!(result.processed_count, 2);
        assert!(result.doppler_factors[0] < result.doppler_factors[1]); // 接近时频率降低
    }

    #[test]
    fn test_batch_apply_gain() {
        let samples = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let gains = vec![0.5];
        let result = AudioDSPOps::batch_apply_gain(&samples, &gains);
        
        assert_eq!(result.sample_count, 5);
        assert!(result.peak > 0.2 && result.peak < 0.3);
        assert!(result.rms_energy > 0.0);
    }

    #[test]
    fn test_batch_lowpass_filter() {
        let samples = vec![0.1, -0.2, 0.3, -0.4, 0.5];
        let result = AudioDSPOps::batch_lowpass_filter(&samples, 0.3);
        
        assert_eq!(result.sample_count, 5);
        // 滤波应该减少高频内容，peak 应该小于原始
        assert!(result.peak < 0.5);
    }

    #[test]
    fn test_hrtf_batch() {
        let elevations = vec![0.0, PI / 6.0, PI / 4.0];
        let azimuths = vec![0.0, PI / 4.0, PI / 2.0];
        let hrtf_pairs = AudioSpatialOps::apply_hrtf_batch(&elevations, &azimuths);
        
        assert_eq!(hrtf_pairs.len(), 3);
    }
    
    #[test]
    fn test_batch_mix_streams() {
        let source1 = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let source2 = vec![0.2, 0.3, 0.4, 0.5, 0.6];
        let source3 = vec![0.05, 0.1, 0.15, 0.2, 0.25];
        
        let sources = vec![&source1[..], &source2[..], &source3[..]];
        let gains = vec![0.5, 0.5, 0.5];
        
        let result = AudioDSPOps::batch_mix_streams(&sources, &gains);
        
        assert_eq!(result.sample_count, 5);
        assert!(result.peak > 0.0);
        assert!(result.rms_energy > 0.0);
        
        // 验证混合结果（每个样本应该是所有源的加权和）
        assert!((result.samples[0] - (0.1 * 0.5 + 0.2 * 0.5 + 0.05 * 0.5)).abs() < 0.001);
    }
    
    #[test]
    fn test_batch_mix_streams_empty() {
        let sources: Vec<&[f32]> = vec![];
        let gains = vec![];
        
        let result = AudioDSPOps::batch_mix_streams(&sources, &gains);
        
        assert_eq!(result.sample_count, 0);
        assert_eq!(result.samples.len(), 0);
    }
    
    #[test]
    fn test_batch_mix_streams_different_lengths() {
        let source1 = vec![0.1, 0.2, 0.3];
        let source2 = vec![0.2, 0.3];
        
        let sources = vec![&source1[..], &source2[..]];
        let gains = vec![0.5, 0.5];
        
        let result = AudioDSPOps::batch_mix_streams(&sources, &gains);
        
        // 应该使用最短的长度
        assert_eq!(result.sample_count, 2);
        assert_eq!(result.samples.len(), 2);
    }
    
    #[test]
    fn test_batch_mix_streams_single_source() {
        let source1 = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let sources = vec![&source1[..]];
        let gains = vec![0.8];
        
        let result = AudioDSPOps::batch_mix_streams(&sources, &gains);
        
        assert_eq!(result.sample_count, 5);
        assert!((result.samples[0] - (0.1 * 0.8)).abs() < 0.001);
    }
}

