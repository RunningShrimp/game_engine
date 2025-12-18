use std::collections::VecDeque;
use std::time::{Duration, SystemTime};

/// 帧处理阶段的性能数据
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseMetrics {
    pub phase_name: String,
    pub duration: Duration,
    pub memory_allocated: usize,
    pub memory_freed: usize,
    pub gpu_time: Option<Duration>,
}

impl PhaseMetrics {
    pub fn new(name: impl Into<String>, duration: Duration) -> Self {
        Self {
            phase_name: name.into(),
            duration,
            memory_allocated: 0,
            memory_freed: 0,
            gpu_time: None,
        }
    }

    pub fn with_memory(mut self, allocated: usize, freed: usize) -> Self {
        self.memory_allocated = allocated;
        self.memory_freed = freed;
        self
    }

    pub fn with_gpu_time(mut self, gpu_time: Duration) -> Self {
        self.gpu_time = Some(gpu_time);
        self
    }

    pub fn net_memory(&self) -> isize {
        self.memory_allocated as isize - self.memory_freed as isize
    }

    pub fn total_time(&self) -> Duration {
        let cpu = self.duration.as_micros();
        let gpu = self.gpu_time.map(|d| d.as_micros()).unwrap_or(0);
        Duration::from_micros((cpu + gpu) as u64)
    }
}

/// 单帧的完整性能快照
#[derive(Debug, Clone)]
pub struct FrameSnapshot {
    pub frame_number: u64,
    pub timestamp: SystemTime,
    pub frame_duration: Duration,
    pub fps: f64,
    pub phases: Vec<PhaseMetrics>,
    pub total_memory_delta: isize,
}

impl FrameSnapshot {
    pub fn new(frame_number: u64, frame_duration: Duration) -> Self {
        let fps = if frame_duration.as_secs_f64() > 0.0 {
            1.0 / frame_duration.as_secs_f64()
        } else {
            0.0
        };

        Self {
            frame_number,
            timestamp: SystemTime::now(),
            frame_duration,
            fps,
            phases: Vec::new(),
            total_memory_delta: 0,
        }
    }

    pub fn add_phase(&mut self, phase: PhaseMetrics) {
        self.total_memory_delta += phase.net_memory();
        self.phases.push(phase);
    }

    /// 获取最耗时的阶段
    pub fn get_bottleneck_phase(&self) -> Option<&PhaseMetrics> {
        self.phases.iter().max_by_key(|p| p.duration.as_micros())
    }

    /// 计算阶段占比（CPU时间）
    pub fn phase_percentage(&self, phase_name: &str) -> Option<f64> {
        let total_cpu: u128 = self.phases.iter().map(|p| p.duration.as_micros()).sum();
        if total_cpu == 0 {
            return Some(0.0);
        }

        self.phases
            .iter()
            .find(|p| p.phase_name == phase_name)
            .map(|p| (p.duration.as_micros() as f64 / total_cpu as f64) * 100.0)
    }

    /// 计算GPU/CPU时间比
    pub fn gpu_cpu_ratio(&self) -> Option<f64> {
        let total_cpu: u128 = self.phases.iter().map(|p| p.duration.as_micros()).sum();
        let total_gpu: u128 = self
            .phases
            .iter()
            .filter_map(|p| p.gpu_time.map(|d| d.as_micros()))
            .sum();

        if total_cpu == 0 {
            return Some(0.0);
        }

        Some(total_gpu as f64 / total_cpu as f64)
    }
}

/// 帧分析仪 - 收集并分析多帧性能数据
pub struct FrameAnalyzer {
    frame_buffer: VecDeque<FrameSnapshot>,
    max_frames: usize,
    current_frame: Option<FrameSnapshot>,
}

impl FrameAnalyzer {
    pub fn new(max_frames: usize) -> Self {
        Self {
            frame_buffer: VecDeque::with_capacity(max_frames),
            max_frames,
            current_frame: None,
        }
    }

    /// 开始记录新的一帧
    pub fn start_frame(&mut self, frame_number: u64, frame_duration: Duration) {
        if let Some(snapshot) = self.current_frame.take() {
            self.add_frame_snapshot(snapshot);
        }
        self.current_frame = Some(FrameSnapshot::new(frame_number, frame_duration));
    }

    /// 为当前帧添加阶段性能数据
    pub fn add_phase(&mut self, phase: PhaseMetrics) -> Result<(), &'static str> {
        if let Some(frame) = &mut self.current_frame {
            frame.add_phase(phase);
            Ok(())
        } else {
            Err("No frame started")
        }
    }

    /// 结束当前帧的记录
    pub fn end_frame(&mut self) -> Result<(), &'static str> {
        if let Some(snapshot) = self.current_frame.take() {
            self.add_frame_snapshot(snapshot);
            Ok(())
        } else {
            Err("No frame started")
        }
    }

    fn add_frame_snapshot(&mut self, snapshot: FrameSnapshot) {
        if self.frame_buffer.len() >= self.max_frames {
            self.frame_buffer.pop_front();
        }
        self.frame_buffer.push_back(snapshot);
    }

    /// 获取所有帧的平均FPS
    pub fn average_fps(&self) -> f64 {
        if self.frame_buffer.is_empty() {
            return 0.0;
        }

        let total_fps: f64 = self.frame_buffer.iter().map(|f| f.fps).sum();
        total_fps / self.frame_buffer.len() as f64
    }

    /// 获取所有帧的最小/最大FPS
    pub fn fps_range(&self) -> Option<(f64, f64)> {
        if self.frame_buffer.is_empty() {
            return None;
        }

        let min_fps = self
            .frame_buffer
            .iter()
            .map(|f| f.fps as u64)
            .min()
            .unwrap_or(0) as f64;
        let max_fps = self
            .frame_buffer
            .iter()
            .map(|f| f.fps as u64)
            .max()
            .unwrap_or(0) as f64;

        Some((min_fps, max_fps))
    }

    /// 计算帧时间的95分位数
    pub fn frame_time_percentile_95(&self) -> Option<Duration> {
        if self.frame_buffer.is_empty() {
            return None;
        }

        let mut durations: Vec<Duration> =
            self.frame_buffer.iter().map(|f| f.frame_duration).collect();
        durations.sort();

        let index = (durations.len() as f64 * 0.95) as usize;
        durations.get(index).copied()
    }

    /// 获取特定阶段的平均耗时
    pub fn average_phase_time(&self, phase_name: &str) -> Option<Duration> {
        let mut total_duration = Duration::ZERO;
        let mut count = 0;

        for frame in &self.frame_buffer {
            if let Some(phase) = frame.phases.iter().find(|p| p.phase_name == phase_name) {
                total_duration += phase.duration;
                count += 1;
            }
        }

        if count == 0 {
            None
        } else {
            Some(Duration::from_micros(
                total_duration.as_micros() as u64 / count as u64,
            ))
        }
    }

    /// 获取阶段的变异系数（检测不稳定性）
    pub fn phase_variation_coefficient(&self, phase_name: &str) -> Option<f64> {
        let mut durations = Vec::new();

        for frame in &self.frame_buffer {
            if let Some(phase) = frame.phases.iter().find(|p| p.phase_name == phase_name) {
                durations.push(phase.duration.as_micros() as f64);
            }
        }

        if durations.len() < 2 {
            return None;
        }

        let mean = durations.iter().sum::<f64>() / durations.len() as f64;
        let variance =
            durations.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / durations.len() as f64;
        let std_dev = variance.sqrt();

        if mean == 0.0 {
            Some(0.0)
        } else {
            Some(std_dev / mean)
        }
    }

    /// 获取所有阶段名称（去重）
    pub fn get_phase_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for frame in &self.frame_buffer {
            for phase in &frame.phases {
                if !names.contains(&phase.phase_name) {
                    names.push(phase.phase_name.clone());
                }
            }
        }
        names
    }

    /// 获取帧缓冲区大小
    pub fn get_frame_count(&self) -> usize {
        self.frame_buffer.len()
    }

    /// 获取特定帧的快照
    pub fn get_frame(&self, index: usize) -> Option<&FrameSnapshot> {
        self.frame_buffer.get(index)
    }

    /// 获取最后一帧
    pub fn get_last_frame(&self) -> Option<&FrameSnapshot> {
        self.frame_buffer.back()
    }

    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.frame_buffer.clear();
        self.current_frame = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_metrics() {
        let phase = PhaseMetrics::new("render", Duration::from_micros(1000))
            .with_memory(1024, 512)
            .with_gpu_time(Duration::from_micros(500));

        assert_eq!(phase.phase_name, "render");
        assert_eq!(phase.duration.as_micros(), 1000);
        assert_eq!(phase.net_memory(), 512);
        assert_eq!(phase.gpu_time.unwrap().as_micros(), 500);
        assert_eq!(phase.total_time().as_micros(), 1500);
    }

    #[test]
    fn test_frame_snapshot() {
        let mut snapshot = FrameSnapshot::new(0, Duration::from_millis(16));
        snapshot.add_phase(PhaseMetrics::new("physics", Duration::from_micros(5000)));
        snapshot.add_phase(PhaseMetrics::new("render", Duration::from_micros(10000)));

        assert_eq!(snapshot.phases.len(), 2);
        assert!((snapshot.fps - 62.5).abs() < 0.1);

        assert_eq!(snapshot.phase_percentage("physics").unwrap() as i32, 33);
        assert_eq!(snapshot.phase_percentage("render").unwrap() as i32, 66);
    }

    #[test]
    fn test_frame_analyzer() {
        let mut analyzer = FrameAnalyzer::new(10);

        analyzer.start_frame(0, Duration::from_millis(16));
        analyzer.add_phase(PhaseMetrics::new("physics", Duration::from_micros(3000)));
        analyzer.add_phase(PhaseMetrics::new("render", Duration::from_micros(12000)));
        analyzer.end_frame().unwrap();

        analyzer.start_frame(1, Duration::from_millis(17));
        analyzer.add_phase(PhaseMetrics::new("physics", Duration::from_micros(3100)));
        analyzer.add_phase(PhaseMetrics::new("render", Duration::from_micros(13000)));
        analyzer.end_frame().unwrap();

        assert_eq!(analyzer.get_frame_count(), 2);
        assert!((analyzer.average_fps() - 60.0).abs() < 5.0);

        let physics_avg = analyzer.average_phase_time("physics").unwrap();
        assert!(physics_avg.as_micros() > 3000 && physics_avg.as_micros() < 3100);
    }

    #[test]
    fn test_bottleneck_detection() {
        let mut snapshot = FrameSnapshot::new(0, Duration::from_millis(16));
        snapshot.add_phase(PhaseMetrics::new("physics", Duration::from_micros(2000)));
        snapshot.add_phase(PhaseMetrics::new("render", Duration::from_micros(10000)));
        snapshot.add_phase(PhaseMetrics::new("io", Duration::from_micros(500)));

        let bottleneck = snapshot.get_bottleneck_phase().unwrap();
        assert_eq!(bottleneck.phase_name, "render");
    }

    #[test]
    fn test_gpu_cpu_ratio() {
        let mut snapshot = FrameSnapshot::new(0, Duration::from_millis(16));
        snapshot.add_phase(
            PhaseMetrics::new("compute", Duration::from_micros(1000))
                .with_gpu_time(Duration::from_micros(4000)),
        );
        snapshot.add_phase(PhaseMetrics::new("render", Duration::from_micros(14000)));

        let ratio = snapshot.gpu_cpu_ratio().unwrap();
        assert!((ratio - 0.266).abs() < 0.01);
    }

    #[test]
    fn test_variation_coefficient() {
        let mut analyzer = FrameAnalyzer::new(10);

        // 添加帧，render时间有变异
        for i in 0..5 {
            analyzer.start_frame(i, Duration::from_millis(16));
            let render_time = 10000 + (i as u128 * 1000);
            analyzer.add_phase(PhaseMetrics::new(
                "render",
                Duration::from_micros(render_time as u64),
            ));
            analyzer.end_frame().unwrap();
        }

        let cv = analyzer.phase_variation_coefficient("render");
        assert!(cv.is_some());
        assert!(cv.unwrap() > 0.0);
    }
}
