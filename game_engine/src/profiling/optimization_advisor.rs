//! 性能优化建议生成器
//!
//! 自动分析性能数据并生成具体的优化建议，包括问题描述、影响评估、优化方案和预估收益。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 优化建议类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationType {
    /// 降低画质
    ReduceQuality,
    /// 批量处理
    Batching,
    /// 对象池
    ObjectPooling,
    /// 纹理压缩
    TextureCompression,
    /// 几何体简化
    GeometrySimplification,
    /// 代码优化
    CodeOptimization,
    /// 内存优化
    MemoryOptimization,
    /// 着色器优化
    ShaderOptimization,
    /// 其他
    Other(String),
}

/// 优化优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OptimizationPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl OptimizationPriority {
    pub fn as_str(&self) -> &str {
        match self {
            OptimizationPriority::Low => "低",
            OptimizationPriority::Medium => "中",
            OptimizationPriority::High => "高",
            OptimizationPriority::Critical => "紧急",
        }
    }
}

/// 性能问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceIssue {
    /// 问题ID
    pub id: String,
    /// 问题标题
    pub title: String,
    /// 问题描述
    pub description: String,
    /// 问题类别
    pub category: IssueCategory,
    /// 严重程度
    pub severity: OptimizationPriority,
    /// 影响的指标
    pub affected_metrics: Vec<String>,
    /// 当前值
    pub current_value: f64,
    /// 目标值
    pub target_value: f64,
    /// 影响评估 (0-100)
    pub impact_score: u32,
}

/// 问题类别
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueCategory {
    CPU,
    GPU,
    Memory,
    IO,
    Network,
    Other,
}

/// 优化建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    /// 建议ID
    pub id: String,
    /// 关联的问题ID
    pub issue_id: String,
    /// 建议类型
    pub opt_type: OptimizationType,
    /// 建议标题
    pub title: String,
    /// 详细描述
    pub description: String,
    /// 实施步骤
    pub implementation_steps: Vec<String>,
    /// 预估性能提升 (百分比)
    pub estimated_improvement: f32,
    /// 实施难度 (1-10)
    pub difficulty: u32,
    /// 风险等级 (1-10)
    pub risk_level: u32,
    /// 优先级
    pub priority: OptimizationPriority,
    /// 预估耗时 (小时)
    pub estimated_time_hours: f32,
    /// 代码示例
    pub code_example: Option<String>,
    /// 相关文档链接
    pub doc_links: Vec<String>,
}

/// 优化计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationPlan {
    /// 建议列表
    pub suggestions: Vec<OptimizationSuggestion>,
    /// 总预估性能提升
    pub total_estimated_improvement: f32,
    /// 总预估耗时
    pub total_estimated_time_hours: f32,
    /// 高优先级建议数量
    pub high_priority_count: usize,
    /// 生成的建议数量
    pub suggestion_count: usize,
}

/// 优化建议生成器
pub struct OptimizationAdvisor {
    /// 建议历史
    suggestion_history: HashMap<String, OptimizationSuggestion>,
    /// 配置
    config: AdvisorConfig,
}

/// 顾问配置
#[derive(Debug, Clone)]
pub struct AdvisorConfig {
    /// 是否启用详细分析
    pub detailed_analysis: bool,
    /// 最小影响分数阈值
    pub min_impact_threshold: u32,
    /// 最大建议数量
    pub max_suggestions: usize,
    /// 是否包含代码示例
    pub include_code_examples: bool,
}

impl Default for AdvisorConfig {
    fn default() -> Self {
        Self {
            detailed_analysis: true,
            min_impact_threshold: 10,
            max_suggestions: 20,
            include_code_examples: true,
        }
    }
}

impl OptimizationAdvisor {
    /// 创建新的优化建议生成器
    pub fn new() -> Self {
        Self {
            suggestion_history: HashMap::new(),
            config: AdvisorConfig::default(),
        }
    }

    /// 设置配置
    pub fn with_config(mut self, config: AdvisorConfig) -> Self {
        self.config = config;
        self
    }

    /// 分析性能数据并生成优化建议
    pub fn analyze_and_suggest(&mut self, metrics: &PerformanceMetrics) -> OptimizationPlan {
        let mut issues = self.detect_issues(metrics);
        let mut suggestions = Vec::new();

        // 根据问题生成优化建议
        for issue in &issues {
            let issue_suggestions = self.generate_suggestions_for_issue(issue, metrics);
            suggestions.extend(issue_suggestions);
        }

        // 根据配置过滤建议
        suggestions = self.filter_suggestions(suggestions);

        // 计算总体统计
        let total_estimated_improvement = suggestions
            .iter()
            .map(|s| s.estimated_improvement)
            .sum::<f32>()
            .min(95.0); // 最高不超过95%

        let total_estimated_time_hours = suggestions
            .iter()
            .map(|s| s.estimated_time_hours)
            .sum();

        let high_priority_count = suggestions
            .iter()
            .filter(|s| s.priority >= OptimizationPriority::High)
            .count();

        let suggestion_count = suggestions.len();

        // 保存到历史记录
        for suggestion in &suggestions {
            self.suggestion_history
                .insert(suggestion.id.clone(), suggestion.clone());
        }

        OptimizationPlan {
            suggestions,
            total_estimated_improvement,
            total_estimated_time_hours,
            high_priority_count,
            suggestion_count,
        }
    }

    /// 检测性能问题
    fn detect_issues(&self, metrics: &PerformanceMetrics) -> Vec<PerformanceIssue> {
        let mut issues = Vec::new();

        // 检测帧率问题
        if metrics.fps < 30.0 {
            issues.push(PerformanceIssue {
                id: "low_fps".to_string(),
                title: "帧率过低".to_string(),
                description: format!(
                    "当前帧率为 {:.1} FPS，低于最低要求 30 FPS，严重影响游戏体验。",
                    metrics.fps
                ),
                category: IssueCategory::CPU,
                severity: OptimizationPriority::Critical,
                affected_metrics: vec!["fps".to_string(), "frame_time".to_string()],
                current_value: metrics.fps as f64,
                target_value: 60.0,
                impact_score: 90,
            });
        } else if metrics.fps < 60.0 {
            issues.push(PerformanceIssue {
                id: "suboptimal_fps".to_string(),
                title: "帧率未达标".to_string(),
                description: format!(
                    "当前帧率为 {:.1} FPS，未达到目标 60 FPS，可能影响游戏流畅度。",
                    metrics.fps
                ),
                category: IssueCategory::CPU,
                severity: OptimizationPriority::High,
                affected_metrics: vec!["fps".to_string()],
                current_value: metrics.fps as f64,
                target_value: 60.0,
                impact_score: 60,
            });
        }

        // 检测Draw Calls问题
        if metrics.draw_calls > 100 {
            issues.push(PerformanceIssue {
                id: "high_draw_calls".to_string(),
                title: "Draw Calls过多".to_string(),
                description: format!(
                    "当前Draw Calls为 {}，远超推荐值 (<100)，严重影响CPU-GPU通信性能。",
                    metrics.draw_calls
                ),
                category: IssueCategory::GPU,
                severity: OptimizationPriority::High,
                affected_metrics: vec!["draw_calls".to_string(), "cpu_time".to_string()],
                current_value: metrics.draw_calls as f64,
                target_value: 100.0,
                impact_score: 75,
            });
        }

        // 检测三角形数量问题
        if metrics.triangles > 100000 {
            issues.push(PerformanceIssue {
                id: "high_triangle_count".to_string(),
                title: "几何体复杂度过高".to_string(),
                description: format!(
                    "场景包含 {} 个三角形，超过推荐值 (<100K)，建议优化模型或使用LOD。",
                    metrics.triangles
                ),
                category: IssueCategory::GPU,
                severity: OptimizationPriority::Medium,
                affected_metrics: vec!["triangles".to_string(), "gpu_time".to_string()],
                current_value: metrics.triangles as f64,
                target_value: 100000.0,
                impact_score: 50,
            });
        }

        // 检测内存使用问题
        if metrics.memory_mb > 500.0 {
            issues.push(PerformanceIssue {
                id: "high_memory_usage".to_string(),
                title: "内存使用过高".to_string(),
                description: format!(
                    "当前内存使用为 {:.1} MB，超过推荐值 (<500MB)。",
                    metrics.memory_mb
                ),
                category: IssueCategory::Memory,
                severity: OptimizationPriority::Medium,
                affected_metrics: vec!["memory_mb".to_string()],
                current_value: metrics.memory_mb as f64,
                target_value: 500.0,
                impact_score: 55,
            });
        }

        // 检测GC时间问题
        if metrics.gc_time_ms > 5.0 {
            issues.push(PerformanceIssue {
                id: "high_gc_time".to_string(),
                title: "GC时间过长".to_string(),
                description: format!(
                    "每帧GC时间为 {:.2} ms，超过推荐值 (<5ms)，建议减少内存分配。",
                    metrics.gc_time_ms
                ),
                category: IssueCategory::Memory,
                severity: OptimizationPriority::High,
                affected_metrics: vec!["gc_time_ms".to_string(), "frame_time".to_string()],
                current_value: metrics.gc_time_ms as f64,
                target_value: 5.0,
                impact_score: 70,
            });
        }

        // 检测纹理内存问题
        if metrics.texture_memory_mb > 200.0 {
            issues.push(PerformanceIssue {
                id: "high_texture_memory".to_string(),
                title: "纹理内存占用过高".to_string(),
                description: format!(
                    "纹理内存占用为 {:.1} MB，建议使用纹理压缩或降低分辨率。",
                    metrics.texture_memory_mb
                ),
                category: IssueCategory::Memory,
                severity: OptimizationPriority::Medium,
                affected_metrics: vec!["texture_memory_mb".to_string()],
                current_value: metrics.texture_memory_mb as f64,
                target_value: 200.0,
                impact_score: 45,
            });
        }

        issues
    }

    /// 为特定问题生成优化建议
    fn generate_suggestions_for_issue(
        &self,
        issue: &PerformanceIssue,
        metrics: &PerformanceMetrics,
    ) -> Vec<OptimizationSuggestion> {
        match issue.id.as_str() {
            "low_fps" | "suboptimal_fps" => {
                vec![
                    self.create_fps_optimization_suggestion(issue),
                    self.create_draw_call_reduction_suggestion(issue),
                    self.create_lod_suggestion(issue),
                ]
            }
            "high_draw_calls" => {
                vec![
                    self.create_draw_call_reduction_suggestion(issue),
                    self.create_dynamic_batching_suggestion(issue),
                ]
            }
            "high_triangle_count" => {
                vec![
                    self.create_lod_suggestion(issue),
                    self.create_mesh_simplification_suggestion(issue),
                ]
            }
            "high_memory_usage" => {
                vec![
                    self.create_object_pooling_suggestion(issue),
                    self.create_asset_streaming_suggestion(issue),
                ]
            }
            "high_gc_time" => {
                vec![
                    self.create_object_pooling_suggestion(issue),
                    self.create_allocation_reduction_suggestion(issue),
                ]
            }
            "high_texture_memory" => {
                vec![
                    self.create_texture_compression_suggestion(issue),
                    self.create_mipmap_suggestion(issue),
                ]
            }
            _ => Vec::new(),
        }
    }

    /// 创建FPS优化建议
    fn create_fps_optimization_suggestion(&self, issue: &PerformanceIssue) -> OptimizationSuggestion {
        OptimizationSuggestion {
            id: format!("{}_fps_opt", issue.id),
            issue_id: issue.id.clone(),
            opt_type: OptimizationType::ReduceQuality,
            title: "降低渲染质量以提升帧率".to_string(),
            description: "通过降低阴影质量、禁用后处理效果、减少粒子数量等方式快速提升帧率。".to_string(),
            implementation_steps: vec![
                "在质量设置中将阴影质量降低到Medium或Low".to_string(),
                "禁用或降低后处理效果（如Bloom、Motion Blur）".to_string(),
                "减少粒子系统的最大粒子数量".to_string(),
                "降低水面反射质量".to_string(),
                "减少实时光源数量，使用烘焙光照".to_string(),
            ],
            estimated_improvement: 25.0,
            difficulty: 2,
            risk_level: 1,
            priority: OptimizationPriority::High,
            estimated_time_hours: 1.0,
            code_example: self.config.include_code_examples.then(|| {
                r#"
// 在渲染配置中降低质量
RenderConfig {
    shadow_quality: ShadowQuality::Medium,
    post_processing: false,
    particle_count: 1000,
    // ...
}
"#.to_string()
            }),
            doc_links: vec![
                "https://docs.example.com/render-quality".to_string(),
            ],
        }
    }

    /// 创建Draw Call优化建议
    fn create_draw_call_reduction_suggestion(&self, issue: &PerformanceIssue) -> OptimizationSuggestion {
        OptimizationSuggestion {
            id: format!("{}_draw_call_opt", issue.id),
            issue_id: issue.id.clone(),
            opt_type: OptimizationType::Batching,
            title: "减少Draw Calls".to_string(),
            description: "通过批处理相同材质的对象、使用GPU Instancing、合并网格等方式减少Draw Calls。".to_string(),
            implementation_steps: vec![
                "识别使用相同材质的对象".to_string(),
                "实现静态批处理（Static Batching）".to_string(),
                "实现动态批处理（Dynamic Batching）".to_string(),
                "使用GPU Instancing渲染重复对象".to_string(),
                "合并小网格为单个大网格".to_string(),
            ],
            estimated_improvement: 30.0,
            difficulty: 5,
            risk_level: 2,
            priority: OptimizationPriority::High,
            estimated_time_hours: 8.0,
            code_example: self.config.include_code_examples.then(|| {
                r#"
// GPU Instancing示例
fn render_instanced_meshes(meshes: &[Mesh]) {
    let instances: Vec<Mat4> = meshes.iter()
        .map(|m| m.transform)
        .collect();

    render_command.draw_instanced(&mesh_data, &instances);
}

// 静态批处理
fn batch_static_objects(objects: &[GameObject]) {
    let batched_mesh = merge_meshes(objects);
    render_batched(&batched_mesh);
}
"#.to_string()
            }),
            doc_links: vec![
                "https://docs.example.com/batching".to_string(),
                "https://docs.example.com/gpu-instancing".to_string(),
            ],
        }
    }

    /// 创建LOD优化建议
    fn create_lod_suggestion(&self, issue: &PerformanceIssue) -> OptimizationSuggestion {
        OptimizationSuggestion {
            id: format!("{}_lod_opt", issue.id),
            issue_id: issue.id.clone(),
            opt_type: OptimizationType::GeometrySimplification,
            title: "实现LOD (Level of Detail) 系统".to_string(),
            description: "根据摄像机距离使用不同细节级别的模型，远处的对象使用简化模型。".to_string(),
            implementation_steps: vec![
                "为重要模型创建多个LOD级别".to_string(),
                "设置LOD切换距离阈值".to_string(),
                "实现LOD组管理器".to_string(),
                "添加LOD过渡效果（可选）".to_string(),
            ],
            estimated_improvement: 20.0,
            difficulty: 6,
            risk_level: 2,
            priority: OptimizationPriority::Medium,
            estimated_time_hours: 12.0,
            code_example: self.config.include_code_examples.then(|| {
                r#"
// LOD配置
struct LODConfig {
    levels: Vec<LODLevel>,
}

struct LODLevel {
    mesh: Mesh,
    max_distance: f32,
    transition_size: f32,
}

// LOD选择
fn select_lod(config: &LODConfig, distance: f32) -> &Mesh {
    for level in &config.levels {
        if distance < level.max_distance {
            return &level.mesh;
        }
    }
    config.levels.last().unwrap()
}
"#.to_string()
            }),
            doc_links: vec![
                "https://docs.example.com/lod-system".to_string(),
            ],
        }
    }

    /// 创建对象池优化建议
    fn create_object_pooling_suggestion(&self, issue: &PerformanceIssue) -> OptimizationSuggestion {
        OptimizationSuggestion {
            id: format!("{}_pooling_opt", issue.id),
            issue_id: issue.id.clone(),
            opt_type: OptimizationType::ObjectPooling,
            title: "实现对象池系统".to_string(),
            description: "重用频繁创建和销毁的对象，减少内存分配和GC压力。".to_string(),
            implementation_steps: vec![
                "识别频繁创建的对象类型（如子弹、敌人）".to_string(),
                "创建对象池管理器".to_string(),
                "实现对象获取和释放方法".to_string(),
                "设置池大小限制".to_string(),
                "添加池统计和监控".to_string(),
            ],
            estimated_improvement: 15.0,
            difficulty: 4,
            risk_level: 2,
            priority: OptimizationPriority::High,
            estimated_time_hours: 6.0,
            code_example: self.config.include_code_examples.then(|| {
                r#"
// 对象池实现
struct ObjectPool<T> {
    objects: Vec<Option<T>>,
    create_fn: Box<dyn Fn() -> T>,
}

impl<T> ObjectPool<T> {
    fn acquire(&mut self) -> T {
        for obj in &mut self.objects {
            if obj.is_none() {
                return obj.take().unwrap();
            }
        }
        // 池已满，创建新对象
        (self.create_fn)()
    }

    fn release(&mut self, obj: T) {
        for slot in &mut self.objects {
            if slot.is_none() {
                *slot = Some(obj);
                return;
            }
        }
    }
}
"#.to_string()
            }),
            doc_links: vec![
                "https://docs.example.com/object-pooling".to_string(),
            ],
        }
    }

    /// 创建动态批处理建议
    fn create_dynamic_batching_suggestion(&self, issue: &PerformanceIssue) -> OptimizationSuggestion {
        OptimizationSuggestion {
            id: format!("{}_dynamic_batch_opt", issue.id),
            issue_id: issue.id.clone(),
            opt_type: OptimizationType::Batching,
            title: "实现动态批处理".to_string(),
            description: "在运行时动态合并使用相同材质的小型对象。".to_string(),
            implementation_steps: vec![
                "识别可批处理的对象（相同材质、小型网格）".to_string(),
                "实现动态批处理系统".to_string(),
                "设置批处理阈值（如顶点数<300）".to_string(),
                "测试和验证批处理效果".to_string(),
            ],
            estimated_improvement: 15.0,
            difficulty: 6,
            risk_level: 3,
            priority: OptimizationPriority::Medium,
            estimated_time_hours: 10.0,
            code_example: None,
            doc_links: vec![],
        }
    }

    /// 创建网格简化建议
    fn create_mesh_simplification_suggestion(&self, issue: &PerformanceIssue) -> OptimizationSuggestion {
        OptimizationSuggestion {
            id: format!("{}_mesh_simplify_opt", issue.id),
            issue_id: issue.id.clone(),
            opt_type: OptimizationType::GeometrySimplification,
            title: "简化高多边形网格".to_string(),
            description: "使用网格简化工具降低模型的多边形数量，保持视觉效果。".to_string(),
            implementation_steps: vec![
                "识别高多边形模型（>50K三角形）".to_string(),
                "使用网格简化工具（如Blender的Decimate修改器）".to_string(),
                "创建多个细节级别".to_string(),
                "测试简化后的视觉效果".to_string(),
            ],
            estimated_improvement: 10.0,
            difficulty: 3,
            risk_level: 2,
            priority: OptimizationPriority::Medium,
            estimated_time_hours: 4.0,
            code_example: None,
            doc_links: vec![
                "https://docs.example.com/mesh-simplification".to_string(),
            ],
        }
    }

    /// 创建资源流式加载建议
    fn create_asset_streaming_suggestion(&self, issue: &PerformanceIssue) -> OptimizationSuggestion {
        OptimizationSuggestion {
            id: format!("{}_streaming_opt", issue.id),
            issue_id: issue.id.clone(),
            opt_type: OptimizationType::MemoryOptimization,
            title: "实现资源流式加载".to_string(),
            description: "根据需要动态加载和卸载资源，减少内存占用。".to_string(),
            implementation_steps: vec![
                "识别可以流式加载的大资源".to_string(),
                "实现异步资源加载系统".to_string(),
                "设置卸载策略（距离、时间等）".to_string(),
                "添加加载进度显示".to_string(),
            ],
            estimated_improvement: 20.0,
            difficulty: 7,
            risk_level: 3,
            priority: OptimizationPriority::Medium,
            estimated_time_hours: 16.0,
            code_example: None,
            doc_links: vec![
                "https://docs.example.com/asset-streaming".to_string(),
            ],
        }
    }

    /// 创建内存分配减少建议
    fn create_allocation_reduction_suggestion(&self, issue: &PerformanceIssue) -> OptimizationSuggestion {
        OptimizationSuggestion {
            id: format!("{}_alloc_reduce_opt", issue.id),
            issue_id: issue.id.clone(),
            opt_type: OptimizationType::CodeOptimization,
            title: "减少运行时内存分配".to_string(),
            description: "优化代码以减少每帧的内存分配，降低GC压力。".to_string(),
            implementation_steps: vec![
                "使用性能分析器识别高分配代码路径".to_string(),
                "使用栈分配替代堆分配（如Small Vec）".to_string(),
                "重用临时对象而不是每次创建".to_string(),
                "避免在热路径中进行字符串操作".to_string(),
                "使用对象池（见其他建议）".to_string(),
            ],
            estimated_improvement: 12.0,
            difficulty: 5,
            risk_level: 2,
            priority: OptimizationPriority::High,
            estimated_time_hours: 8.0,
            code_example: self.config.include_code_examples.then(|| {
                r#"
// 使用smallvec避免小数组堆分配
use smallvec::SmallVec;

fn process_vectors() {
    // 使用栈分配的SmallVec，容量为4
    let mut vec: SmallVec<[f32; 4]> = SmallVec::new();
    vec.push(1.0);
    vec.push(2.0);
    // 只有超过4个元素时才会分配到堆上
}

// 重用临时对象
fn process_with_buffer() {
    thread_local! {
        static TEMP_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::new());
    }

    TEMP_BUFFER.with(|buffer| {
        let mut buf = buffer.borrow_mut();
        buf.clear();
        // 使用buf...
    });
}
"#.to_string()
            }),
            doc_links: vec![
                "https://docs.example.com/allocation-optimization".to_string(),
            ],
        }
    }

    /// 创建纹理压缩建议
    fn create_texture_compression_suggestion(&self, issue: &PerformanceIssue) -> OptimizationSuggestion {
        OptimizationSuggestion {
            id: format!("{}_tex_compress_opt", issue.id),
            issue_id: issue.id.clone(),
            opt_type: OptimizationType::TextureCompression,
            title: "使用纹理压缩".to_string(),
            description: "将纹理转换为压缩格式（如ASTC、ETC2、BC7），大幅减少内存占用。".to_string(),
            implementation_steps: vec![
                "识别未压缩的纹理".to_string(),
                "选择目标平台的压缩格式".to_string(),
                "批量转换纹理".to_string(),
                "验证压缩后的视觉质量".to_string(),
            ],
            estimated_improvement: 35.0,
            difficulty: 2,
            risk_level: 1,
            priority: OptimizationPriority::High,
            estimated_time_hours: 3.0,
            code_example: None,
            doc_links: vec![
                "https://docs.example.com/texture-compression".to_string(),
            ],
        }
    }

    /// 创建Mipmap优化建议
    fn create_mipmap_suggestion(&self, issue: &PerformanceIssue) -> OptimizationSuggestion {
        OptimizationSuggestion {
            id: format!("{}_mipmap_opt", issue.id),
            issue_id: issue.id.clone(),
            opt_type: OptimizationType::TextureCompression,
            title: "优化Mipmap设置".to_string(),
            description: "为纹理生成Mipmap并启用三线性过滤，提升远距离渲染质量并可能提升性能。".to_string(),
            implementation_steps: vec![
                "为重要纹理生成Mipmap".to_string(),
                "启用三线性过滤".to_string(),
                "设置Mipmap偏移（Bias）平衡质量和性能".to_string(),
                "考虑使用Mipmap Streaming".to_string(),
            ],
            estimated_improvement: 8.0,
            difficulty: 2,
            risk_level: 1,
            priority: OptimizationPriority::Low,
            estimated_time_hours: 2.0,
            code_example: None,
            doc_links: vec![
                "https://docs.example.com/mipmaps".to_string(),
            ],
        }
    }

    /// 过滤建议
    fn filter_suggestions(&self, mut suggestions: Vec<OptimizationSuggestion>) -> Vec<OptimizationSuggestion> {
        // 按优先级和预估提升排序
        suggestions.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| b.estimated_improvement.partial_cmp(&a.estimated_improvement).unwrap())
        });

        // 限制建议数量
        suggestions.truncate(self.config.max_suggestions);

        // 过滤低影响建议
        suggestions.retain(|s| s.estimated_improvement >= 5.0);

        suggestions
    }

    /// 获取历史建议
    pub fn get_suggestion(&self, id: &str) -> Option<&OptimizationSuggestion> {
        self.suggestion_history.get(id)
    }

    /// 清空历史记录
    pub fn clear_history(&mut self) {
        self.suggestion_history.clear();
    }
}

impl Default for OptimizationAdvisor {
    fn default() -> Self {
        Self::new()
    }
}

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// 帧率
    pub fps: f32,
    /// 帧时间 (ms)
    pub frame_time: f32,
    /// Draw Calls
    pub draw_calls: u32,
    /// 三角形数量
    pub triangles: u32,
    /// 内存使用 (MB)
    pub memory_mb: f32,
    /// 纹理内存 (MB)
    pub texture_memory_mb: f32,
    /// GC时间 (ms)
    pub gc_time_ms: f32,
    /// CPU时间 (ms)
    pub cpu_time: f32,
    /// GPU时间 (ms)
    pub gpu_time: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_advisor_creation() {
        let advisor = OptimizationAdvisor::new();
        assert_eq!(advisor.suggestion_history.len(), 0);
    }

    #[test]
    fn test_issue_detection() {
        let advisor = OptimizationAdvisor::new();
        let metrics = PerformanceMetrics {
            fps: 25.0,  // 低于30 FPS
            frame_time: 40.0,
            draw_calls: 150,
            triangles: 150000,
            memory_mb: 600.0,
            texture_memory_mb: 300.0,
            gc_time_ms: 8.0,
            cpu_time: 25.0,
            gpu_time: 30.0,
        };

        let issues = advisor.detect_issues(&metrics);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.id == "low_fps"));
    }

    #[test]
    fn test_suggestion_generation() {
        let mut advisor = OptimizationAdvisor::new();
        let metrics = PerformanceMetrics {
            fps: 25.0,
            frame_time: 40.0,
            draw_calls: 150,
            triangles: 150000,
            memory_mb: 600.0,
            texture_memory_mb: 300.0,
            gc_time_ms: 8.0,
            cpu_time: 25.0,
            gpu_time: 30.0,
        };

        let plan = advisor.analyze_and_suggest(&metrics);
        assert!(!plan.suggestions.is_empty());
        assert!(plan.total_estimated_improvement > 0.0);
    }

    #[test]
    fn test_suggestion_priority() {
        let mut advisor = OptimizationAdvisor::new();
        let metrics = PerformanceMetrics {
            fps: 20.0,  // 严重问题
            frame_time: 50.0,
            draw_calls: 200,
            triangles: 200000,
            memory_mb: 800.0,
            texture_memory_mb: 400.0,
            gc_time_ms: 10.0,
            cpu_time: 30.0,
            gpu_time: 35.0,
        };

        let plan = advisor.analyze_and_suggest(&metrics);
        assert!(plan.high_priority_count > 0);
    }
}
