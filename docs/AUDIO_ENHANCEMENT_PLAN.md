# 音频系统增强计划 (Audio System Enhancement Plan)

**版本**: v0.2.0 → v0.3.0
**创建日期**: 2025-12-30
**优先级**: P2
**预计时间**: 2-3周

---

## 📋 目录

1. [当前状态](#当前状态)
2. [增强目标](#增强目标)
3. [HRTF实现](#hrtf实现)
4. [高级混响](#高级混响)
5. [其他增强](#其他增强)
6. [实施计划](#实施计划)
7. [验证标准](#验证标准)

---

## 当前状态

### ✅ 已实现功能

**基础音频系统**:
- ✅ 3D空间音效（位置、速度、方向）
- ✅ 音频流加载和播放
- ✅ 音量控制和淡入淡出
- ✅ 基础混响（预设）
- ✅ 多声道支持
- ✅ 距离衰减模型

**代码模块**:
```rust
// game_engine/src/audio/
mod spatial_audio;    // 3D空间音效
mod audio_stream;     // 音频流
mod reverb;           // 基础混响
mod audio_manager;    // 音频管理器
```

**性能指标**:
- 混音延迟: < 50ms ✅
- CPU占用: 单核5-10% ✅
- 内存占用: ~100MB (100个并发声音)
- 支持: 最多64个并发3D音源

### ⚠️ 功能缺口

1. **HRTF (Head-Related Transfer Function)** - 缺失
2. **高级混响算法** - 仅预设，无实时计算
3. **音频遮挡/阻塞** - 缺失
4. **多普勒效应** - 缺失
5. **环境音效** - 基础实现
6. **音频压缩** - 无实时压缩

---

## 增强目标

### P1 核心功能 (必须实现)

1. **HRTF支持**
   - 双耳音频渲染
   - 头部相关传递函数
   - 支持标准HRTF数据集（MIT, CIPIC）

2. **实时混响**
   - 基于房间几何的混响计算
   - 动态混响参数
   - 多个混衰区

### P2 增强功能 (推荐实现)

3. **音频遮挡**
   - 基于物理的遮挡计算
   - 材质对音频的影响

4. **多普勒效应**
   - 移动音源和监听者
   - 频率偏移计算

5. **环境音效系统**
   - 动态环境切换
   - 淡入淡出过渡

---

## HRTF实现

### 概述

HRTF (Head-Related Transfer Function) 模拟声音如何被头部、耳朵和躯干反射，实现真实的3D音频定位。

### 技术方案

#### 方案1: 使用现有库 (推荐)

**依赖**: `rodio` + `hrtf` crate

```toml
[dependencies]
hrtf = "0.2"
rodio = { version = "0.21", features = ["hrtf"] }
```

**实现**:
```rust
// game_engine/src/audio/hrtf.rs

use hrtf::{HrtfDataSource, HrtfProcessor};

pub struct HrtfAudioEngine {
    processor: HrtfProcessor,
    enabled: bool,
}

impl HrtfAudioEngine {
    pub fn new() -> Result<Self, AudioError> {
        // 加载HRTF数据集
        let data_source = HrtfDataSource::from_mit_hrtf_dataset()?;

        Ok(Self {
            processor: HrtfProcessor::new(data_source, 44100)?,
            enabled: true,
        })
    }

    pub fn process(&mut self, input: &[f32], position: Vec3) -> Vec<f32> {
        if !self.enabled {
            // 回退到标准立体声
            return input.to_vec();
        }

        self.processor.process(input, position)
    }

    pub fn enable_hrtf(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_hrtf_enabled(&self) -> bool {
        self.enabled
    }
}
```

#### 方案2: 自行实现

**核心算法**:

1. **HRTF数据加载**
   ```rust
   pub struct HrtfDataset {
       // 每个方位角和仰角的HRIR (Head-Related Impulse Response)
       hrir: Vec<Vec<Vec<f32>>>, // [azimuth][elevation][sample]

       // 采样率
       sample_rate: u32,

       // FFT大小
       fft_size: usize,
   }

   impl HrtfDataset {
       pub fn load_sofa_file(path: &Path) -> Result<Self, AudioError> {
           // 解析SOFA文件格式
           // 提取HRIR数据
       }

       pub fn load_mit_dataset() -> Result<Self, AudioError> {
           // 内置MIT HRTF数据集
           // 710个测量点（方位角：-180°到180°，仰角：-40°到90°）
       }
   }
   ```

2. **双耳渲染器**
   ```rust
   pub struct BinauralRenderer {
       dataset: HrtfDataset,
       buffer_size: usize,
       left_ear_fft: Vec<Complex<f32>>,
       right_ear_fft: Vec<Complex<f32>>,
   }

   impl BinauralRenderer {
       pub fn render(
           &mut self,
           input: &[f32],
           source_pos: Vec3,
           listener_pos: Vec3,
           listener_orientation: Quat,
       ) -> (Vec<f32>, Vec<f32>) {
           // 1. 计算相对位置
           let relative_pos = self.compute_relative_position(
               source_pos,
               listener_pos,
               listener_orientation,
           );

           // 2. 转换为球坐标（方位角、仰角、距离）
           let (azimuth, elevation, distance) = self.to_spherical(relative_pos);

           // 3. 查找最近的HRIR
           let (left_hrir, right_hrir) = self.interpolate_hrir(azimuth, elevation);

           // 4. 应用距离衰减
           let attenuation = self.compute_distance_attenuation(distance);

           // 5. 卷积
           let left = self.convolve(input, left_hrir) * attenuation;
           let right = self.convolve(input, right_hrir) * attenuation;

           (left, right)
       }
   }
   ```

3. **HRIR插值**
   ```rust
   impl BinauralRenderer {
       fn interpolate_hrir(
           &self,
           azimuth: f32,
           elevation: f32,
       ) -> (&[f32], &[f32]) {
           // 找到4个最近的测量点
           let (az_idx_low, az_idx_high) = self.find_azimuth_indices(azimuth);
           let (el_idx_low, el_idx_high) = self.find_elevation_indices(elevation);

           // 双线性插值
           let hrir_ll = &self.dataset.hrir[az_idx_low][el_idx_low];
           let hrir_lh = &self.dataset.hrir[az_idx_low][el_idx_high];
           let hrir_hl = &self.dataset.hrir[az_idx_high][el_idx_low];
           let hrir_hh = &self.dataset.hrir[az_idx_high][el_idx_high];

           // ... 执行插值
           (interpolated_left, interpolated_right)
       }
   }
   ```

### 性能优化

1. **FFT加速**
   ```rust
   use rustfft::FftPlanner;

   struct FftConvolver {
       planner: FftPlanner<f32>,
       fft: Arc<dyn Fft<f32>>,
       ifft: Arc<dyn Fft<f32>>,
   }

   impl FftConvolver {
       fn convolve_fft(&mut self, signal: &[f32], kernel: &[f32]) -> Vec<f32> {
           // FFT卷积比直接卷积快约10x
       }
   }
   ```

2. **缓存HRIR FFT**
   ```rust
   struct HrtfCache {
       cached_hrtf: HashMap<(usize, usize), (Vec<Complex<f32>>, Vec<Complex<f32>)>,
   }

   impl HrtfCache {
       fn get_or_compute(&mut self, az_idx: usize, el_idx: usize) -> &(Vec<_>, Vec<_>) {
           self.cached_hrtf.entry((az_idx, el_idx)).or_insert_with(|| {
               let hrir = dataset.get_hrir(az_idx, el_idx);
               (compute_fft(&hrir.left), compute_fft(&hrir.right))
           })
       }
   }
   ```

3. **低延迟模式**
   ```rust
   pub enum HrtfQuality {
       Low,     // 128 taps, <5ms延迟
       Medium,  // 256 taps, <10ms延迟
       High,    // 512 taps, <20ms延迟
       Ultra,   // 1024 taps, <40ms延迟
   }
   ```

### API设计

```rust
// game_engine/src/audio/hrtf.rs

pub struct HrtfConfig {
    pub quality: HrtfQuality,
    pub dataset: HrtfDatasetSource,
    pub crossfade: bool,
}

impl HrtfConfig {
    pub fn low_latency() -> Self {
        Self {
            quality: HrtfQuality::Low,
            dataset: HrtfDatasetSource::BuiltIn,
            crossfade: true,
        }
    }

    pub fn high_quality() -> Self {
        Self {
            quality: HrtfQuality::High,
            dataset: HrtfDatasetSource::Custom(PathBuf::from("data/hrtf.sofa")),
            crossfade: true,
        }
    }
}

// 集成到AudioManager
impl AudioManager {
    pub fn enable_hrtf(&mut self, config: HrtfConfig) -> Result<(), AudioError> {
        self.hrtf_engine = Some(HrtfAudioEngine::new(config)?);
        Ok(())
    }

    pub fn is_hrtf_enabled(&self) -> bool {
        self.hrtf_engine.is_some()
    }
}
```

---

## 高级混响

### 概述

实时混响基于房间几何和材质，提供逼真的环境音效。

### 技术方案

#### 方案1: Image Source Method (ISM)

```rust
// game_engine/src/audio/reverb/advanced.rs

use std::collections::HashMap;

pub struct RoomReverb {
    dimensions: Vec3,        // 房间尺寸
    reflection_order: usize, // 反射阶数
    wall_absorption: HashMap<Wall, f32>, // 墙壁吸音系数
}

impl RoomReverb {
    pub fn new(dimensions: Vec3) -> Self {
        Self {
            dimensions,
            reflection_order: 3, // 计算3阶反射
            wall_absorption: Self::default_materials(),
        }
    }

    pub fn compute_impulse_response(
        &self,
        source_pos: Vec3,
        listener_pos: Vec3,
        sample_rate: u32,
    ) -> Vec<f32> {
        let mut ir = vec![0.0f32; sample_rate as usize * 2]; // 2秒脉冲响应

        // 1. 直达声
        let direct_distance = source_pos.distance(listener_pos);
        let direct_delay = (direct_distance / SPEED_OF_SOUND) * sample_rate as f32;
        let direct_gain = 1.0 / direct_distance;

        ir[direct_delay as usize] += direct_gain;

        // 2. 反射声（Image Source Method）
        for order in 1..=self.reflection_order {
            for image_source in self.generate_image_sources(source_pos, order) {
                let distance = image_source.distance(listener_pos);
                let delay = (distance / SPEED_OF_SOUND) * sample_rate as f32;

                if delay >= ir.len() as f32 {
                    continue;
                }

                // 计算反射增益（考虑墙壁吸音）
                let mut gain = 1.0 / distance;
                for wall in image_source.reflection_walls() {
                    gain *= 1.0 - self.wall_absorption[wall];
                }

                ir[delay as usize] += gain;
            }
        }

        // 3. 后期混响（扩散尾音）
        let rt60 = self.compute_rt60();
        let decay = (-2.0 / rt60).exp();

        for i in (ir.len() / 2)..ir.len() {
            ir[i] *= decay.powi(i as i32);
        }

        ir
    }

    fn compute_rt60(&self) -> f32 {
        // Sabine公式: RT60 = 0.161 * V / A
        let volume = self.dimensions.x * self.dimensions.y * self.dimensions.z;
        let total_absorption = self.wall_absorption.values().cloned().sum::<f32>();

        0.161 * volume / total_absorption
    }
}
```

#### 方案2: FDN (Feedback Delay Network)

```rust
pub struct FdnReverb {
    delay_lines: Vec<DelayLine>,
    feedback_matrix: [[f32; 8]; 8], // 8x8反馈矩阵
    damping: Vec<f32>,
}

impl FdnReverb {
    pub fn new(sample_rate: u32, rt60: f32) -> Self {
        let mut delay_lines = Vec::with_capacity(8);

        // 质数长度延迟线（减少谐振）
        let prime_lengths = [503, 521, 541, 577, 613, 641, 673, 701];
        for &length in &prime_lengths {
            delay_lines.push(DelayLine::new(length));
        }

        // Hadamard反馈矩阵（正交，保证稳定性）
        let feedback_matrix = Self::hadamard_matrix();

        Self {
            delay_lines,
            feedback_matrix,
            damping: vec![0.5; 8],
        }
    }

    pub fn process(&mut self, input: f32) -> [f32; 2] {
        let mut outputs = [0.0f32; 8];

        // 处理每个延迟线
        for i in 0..8 {
            // 读取延迟线
            let delayed = self.delay_lines[i].read();

            // 应用反馈
            let mut feedback = 0.0;
            for j in 0..8 {
                feedback += self.feedback_matrix[i][j] * outputs[j];
            }

            // 应用阻尼
            feedback *= self.damping[i];

            // 写入延迟线
            self.delay_lines[i].write(input + feedback);

            outputs[i] = delayed;
        }

        // 输出（混合所有通道）
        let left = (outputs[0] + outputs[2] + outputs[4] + outputs[6]) * 0.25;
        let right = (outputs[1] + outputs[3] + outputs[5] + outputs[7]) * 0.25;

        [left, right]
    }
}
```

### 材质系统

```rust
// game_engine/src/audio/material.rs

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AcousticMaterial {
    Concrete,
    Wood,
    Carpet,
    Glass,
    Metal,
    Drywall,
    AcousticFoam,
}

impl AcousticMaterial {
    pub fn absorption_coefficient(&self, frequency: f32) -> f32 {
        match self {
            AcousticMaterial::Concrete => {
                // 混凝土：低吸音（尤其是低频）
                if frequency < 250.0 { 0.01 }
                else if frequency < 1000.0 { 0.02 }
                else { 0.05 }
            }
            AcousticMaterial::Wood => {
                // 木材：中等吸音
                if frequency < 250.0 { 0.10 }
                else if frequency < 1000.0 { 0.15 }
                else { 0.20 }
            }
            AcousticMaterial::Carpet => {
                // 地毯：高频吸音好
                if frequency < 250.0 { 0.05 }
                else if frequency < 1000.0 { 0.20 }
                else { 0.60 }
            }
            // ... 其他材质
        }
    }
}
```

### 动态混响

```rust
pub struct DynamicReverb {
    current_room: Option<Room>,
    target_room: Option<Room>,
    crossfade_time: f32,
    crossfade_timer: f32,
}

impl DynamicReverb {
    pub fn transition_to(&mut self, room: Room, duration: Duration) {
        self.target_room = Some(room);
        self.crossfade_time = duration.as_secs_f32();
        self.crossfade_timer = 0.0;
    }

    pub fn update(&mut self, dt: Duration) {
        if self.target_room.is_some() {
            self.crossfade_timer += dt.as_secs_f32();

            if self.crossfade_timer >= self.crossfade_time {
                // 完成过渡
                self.current_room = self.target_room.take();
            }
        }
    }

    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        if let Some(ref target) = self.target_room {
            // 淡合两个房间的混响
            let t = self.crossfade_timer / self.crossfade_time;
            let alpha = smoothstep(t); // S曲线插值

            let current_ir = self.current_room.as_ref()
                .map(|r| r.impulse_response())
                .unwrap_or(&[]);

            let target_ir = target.impulse_response();

            // 混合两个脉冲响应
            let blended_ir: Vec<f32> = current_ir.iter()
                .zip(target_ir.iter())
                .map(|(&c, &t)| c * (1.0 - alpha) + t * alpha)
                .collect();

            convolve(input, &blended_ir)
        } else {
            // 使用当前房间混响
            self.current_room.as_ref()
                .map(|r| r.process(input))
                .unwrap_or_else(|| input.to_vec())
        }
    }
}
```

---

## 其他增强

### 1. 音频遮挡

```rust
pub struct AudioOcclusion {
    physics_world: Arc<PhysicsWorld>,
}

impl AudioOcclusion {
    pub fn compute_occlusion(
        &self,
        source: Vec3,
        listener: Vec3,
    ) -> OcclusionResult {
        // 光线投射检测遮挡
        let ray = Ray::new(source, listener - source);

        let mut occlusion = 0.0;
        let mut transmission_loss = 1.0;

        for hit in self.physics_world.cast_ray(ray) {
            if hit.is_occluding() {
                // 材质影响
                let material = hit.material();
                transmission_loss *= material.audio_transmission();

                occlusion += 1.0;
            }
        }

        OcclusionResult {
            occlusion_factor: occlusion,
            transmission_loss,
        }
    }
}
```

### 2. 多普勒效应

```rust
pub struct DopplerEffect {
    speed_of_sound: f32,
}

impl DopplerEffect {
    pub fn compute_pitch_shift(
        &self,
        source_pos: Vec3,
        source_vel: Vec3,
        listener_pos: Vec3,
        listener_vel: Vec3,
    ) -> f32 {
        // 计算相对速度
        let relative_pos = source_pos - listener_pos;
        let direction = relative_pos.normalize();

        let v_source = source_vel.dot(direction);
        let v_listener = listener_vel.dot(direction);

        // 多普勒公式
        (self.speed_of_sound + v_listener) / (self.speed_of_sound + v_source)
    }
}
```

---

## 实施计划

### Phase 1: HRTF基础 (Week 1-2)

- [ ] 集成`hrtf` crate
- [ ] 实现HrtfAudioEngine
- [ ] 添加HRTF数据集加载
- [ ] 实现双耳渲染
- [ ] 性能优化和基准测试
- [ ] 单元测试

**验收标准**:
- ✅ 可以启用/禁用HRTF
- ✅ CPU开销<15%
- ✅ 定位精度<5°
- ✅ 所有测试通过

### Phase 2: 高级混响 (Week 3-4)

- [ ] 实现RoomReverb (ISM)
- [ ] 实现FdnReverb
- [ ] 材质系统
- [ ] 动态混响过渡
- [ ] 性能优化
- [ ] 集成测试

**验收标准**:
- ✅ RT60计算误差<10%
- ✅ 混响质量主观评分>4/5
- ✅ CPU开销<20%
- ✅ 过渡平滑无爆音

### Phase 3: 遮挡和多普勒 (Week 5)

- [ ] 实现音频遮挡
- [ ] 实现多普勒效应
- [ ] 集成到空间音效
- [ ] 端到端测试

**验收标准**:
- ✅ 遮挡检测准确率>90%
- ✅ 多普勒效应可听
- ✅ 整体CPU开销<25%

### Phase 4: 优化和文档 (Week 6)

- [ ] 性能profiling和优化
- [ ] 内存使用优化
- [ ] API文档完善
- [ ] 使用示例
- [ ] 发布准备

---

## 验证标准

### 功能测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hrtf_localization() {
        let mut hrtf = HrtfAudioEngine::new().unwrap();

        // 测试前方定位
        let front = render_position(&mut hrtf, Vec3::new(0.0, 0.0, -1.0));
        assert!(localization_accuracy(front, 0.0) < 5.0);

        // 测试后方定位
        let back = render_position(&mut hrtf, Vec3::new(0.0, 0.0, 1.0));
        assert!(localization_accuracy(back, 180.0) < 5.0);
    }

    #[test]
    fn test_reverb_rt60() {
        let room = Room::new(Vec3::new(10.0, 5.0, 8.0));
        let computed_rt60 = room.compute_rt60();
        let measured_rt60 = measure_rt60(&room);

        assert!((computed_rt60 - measured_rt60).abs() < measured_rt60 * 0.1);
    }
}
```

### 性能基准

```rust
#[bench]
fn bench_hrtf_render(b: &mut Bencher) {
    let mut hrtf = HrtfAudioEngine::new().unwrap();
    let input = vec![0.0f32; 1024];
    let position = Vec3::new(1.0, 2.0, 3.0);

    b.iter(|| {
        hrtf.process(&input, position)
    });
}

// 目标: <1ms per 1024 samples
```

### 听感测试

- [ ] A/B测试：HRTF开启 vs 关闭
- [ ] 定位精度测试（前/后/左/右/上/下）
- [ ] 混响自然度评分
- [ ] 整体音质评分

---

## 相关资源

### 学术资源
- [MIT HRTF Dataset](https://sound.media.mit.edu/resources/PS-HRTF/)
- [CIPIC HRTF Database](http://interface.cipic.ucdavis.edu/)
- [Listen HRTF Database](https://mediatemple.net/)
- "Perceptual Assessment of HRTFs" - Xie et al.

### 开源库
- [mysofa](https://github.com/hoene/libmysofa) - SOFA文件解析
- [hrtf](https://github.com/mrDIMAS/hrtf) - Rust HRTF实现
- [rodio](https://github.com/RustAudio/rodio) - Rust音频库

### 标准
- [AES69-2020](https://www.aes.org/standards/) - SOFA文件格式
- [ITU-R BS.775-3](https://www.itu.int/) - 多声道音频系统

---

**维护者**: 游戏引擎音频团队
**最后更新**: 2025-12-30
**版本**: v1.0
