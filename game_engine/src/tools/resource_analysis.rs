//! # 资源依赖分析工具
//!
//! 分析项目中的资源依赖关系，检测未使用的资源。
//!
//! ## 功能特性
//!
//! - **依赖图生成**: 构建完整的资源依赖关系图
//! - **未使用资源检测**: 找出未被引用的资源文件
//! - **循环依赖检测**: 识别资源之间的循环引用
//! - **冗余资产清理**: 自动清理重复或过时的资源
//!
//! ## 使用场景
//!
//! - **项目清理**: 定期清理未使用的资源
//! - **优化构建**: 减少打包的资源体积
//! - **依赖分析**: 了解资源之间的关系
//! - **重构支持**: 安全地重构资源结构

use std::collections::{HashMap, HashSet};
use std::hash::Hasher;
use std::path::{Path, PathBuf};

/// 资源节点
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceNode {
    pub path: PathBuf,
    pub resource_type: ResourceType,
}

/// 资源类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Mesh,
    Texture,
    Material,
    Shader,
    Audio,
    Scene,
    Animation,
    Prefab,
    Script,
    Unknown,
}

impl ResourceType {
    /// 从文件扩展名推断资源类型
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "obj" | "fbx" | "gltf" | "glb" => ResourceType::Mesh,
            "png" | "jpg" | "jpeg" | "ktx" | "astc" => ResourceType::Texture,
            "mat" | "mtl" => ResourceType::Material,
            "wgsl" | "vert" | "frag" => ResourceType::Shader,
            "mp3" | "wav" | "ogg" => ResourceType::Audio,
            "scn" | "scene" => ResourceType::Scene,
            "anim" => ResourceType::Animation,
            "prefab" => ResourceType::Prefab,
            "js" | "lua" | "py" => ResourceType::Script,
            _ => ResourceType::Unknown,
        }
    }
}

/// 资源依赖图
#[derive(Clone, Debug)]
pub struct ResourceDependencyGraph {
    nodes: HashSet<ResourceNode>,
    edges: HashMap<ResourceNode, Vec<ResourceNode>>,  // 资源 -> 它依赖的资源
    reverse_edges: HashMap<ResourceNode, Vec<ResourceNode>>,  // 资源 -> 依赖它的资源
}

impl ResourceDependencyGraph {
    /// 创建新的依赖图
    pub fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: HashMap::new(),
            reverse_edges: HashMap::new(),
        }
    }

    /// 添加资源节点
    pub fn add_node(&mut self, node: ResourceNode) {
        self.nodes.insert(node.clone());
        self.edges.entry(node.clone()).or_insert_with(Vec::new);
        self.reverse_edges.entry(node.clone()).or_insert_with(Vec::new);
    }

    /// 添加依赖关系
    pub fn add_dependency(&mut self, from: ResourceNode, to: ResourceNode) {
        // from 依赖 to
        self.edges.entry(from.clone()).or_insert_with(Vec::new).push(to.clone());
        self.reverse_edges.entry(to.clone()).or_insert_with(Vec::new).push(from);
    }

    /// 获取资源的直接依赖
    pub fn get_dependencies(&self, resource: &ResourceNode) -> Vec<ResourceNode> {
        self.edges.get(resource).cloned().unwrap_or_default()
    }

    /// 获取依赖于此资源的资源
    pub fn get_dependents(&self, resource: &ResourceNode) -> Vec<ResourceNode> {
        self.reverse_edges.get(resource).cloned().unwrap_or_default()
    }

    /// 检测循环依赖
    pub fn detect_cycles(&self) -> Vec<Vec<ResourceNode>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        for node in &self.nodes {
            if !visited.contains(node) {
                if let Some(cycle) = self.detect_cycle_from_node(node, &mut visited, &mut visiting) {
                    cycles.push(cycle);
                }
            }
        }

        cycles
    }

    fn detect_cycle_from_node(
        &self,
        node: &ResourceNode,
        visited: &mut HashSet<ResourceNode>,
        visiting: &mut HashSet<ResourceNode>,
    ) -> Option<Vec<ResourceNode>> {
        visiting.insert(node.clone());

        if let Some(deps) = self.edges.get(node) {
            for dep in deps {
                if visiting.contains(dep) {
                    // 发现循环
                    let cycle = vec![node.clone(), dep.clone()];
                    return Some(cycle);
                }

                if !visited.contains(dep) {
                    if let Some(mut cycle) = self.detect_cycle_from_node(dep, visited, visiting) {
                        cycle.push(node.clone());
                        return Some(cycle);
                    }
                }
            }
        }

        visiting.remove(node);
        visited.insert(node.clone());
        None
    }

    /// 获取所有资源
    pub fn all_resources(&self) -> Vec<ResourceNode> {
        self.nodes.iter().cloned().collect()
    }

    /// 计算资源大小（依赖数量）
    pub fn resource_size(&self, resource: &ResourceNode) -> usize {
        self.edges.get(resource).map(|v| v.len()).unwrap_or(0)
    }

    /// 获取根节点（不被任何资源依赖的资源）
    pub fn get_root_nodes(&self) -> Vec<ResourceNode> {
        self.nodes
            .iter()
            .filter(|node| {
                self.reverse_edges.get(*node).map(|v| v.is_empty()).unwrap_or(true)
            })
            .cloned()
            .collect()
    }

    /// 获取叶子节点（不依赖任何资源的资源）
    pub fn get_leaf_nodes(&self) -> Vec<ResourceNode> {
        self.nodes
            .iter()
            .filter(|node| {
                self.edges.get(*node).map(|v| v.is_empty()).unwrap_or(true)
            })
            .cloned()
            .collect()
    }
}

impl Default for ResourceDependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// 资源扫描器
pub struct ResourceScanner {
    base_path: PathBuf,
}

impl ResourceScanner {
    /// 创建新的资源扫描器
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// 扫描目录中的所有资源
    pub fn scan(&self) -> Result<Vec<ResourceNode>, String> {
        let mut resources = Vec::new();

        fn scan_dir(dir: &Path, resources: &mut Vec<ResourceNode>) -> Result<(), String> {
            let entries = std::fs::read_dir(dir)
                .map_err(|e| format!("Failed to read directory: {}", e))?;

            for entry in entries {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let path = entry.path();

                if path.is_dir() {
                    scan_dir(&path, resources)?;
                } else {
                    if let Some(ext) = path.extension() {
                        if let Some(ext_str) = ext.to_str() {
                            let resource_type = ResourceType::from_extension(ext_str);
                            if resource_type != ResourceType::Unknown {
                                resources.push(ResourceNode {
                                    path: path.clone(),
                                    resource_type,
                                });
                            }
                        }
                    }
                }
            }

            Ok(())
        }

        scan_dir(&self.base_path, &mut resources)?;
        Ok(resources)
    }
}

/// 资源引用分析器
pub struct ResourceReferenceAnalyzer {
    graph: ResourceDependencyGraph,
}

impl ResourceReferenceAnalyzer {
    /// 创建新的分析器
    pub fn new() -> Self {
        Self {
            graph: ResourceDependencyGraph::new(),
        }
    }

    /// 分析场景文件中的资源引用
    pub fn analyze_scene_file(&mut self, scene_path: &Path) -> Result<(), String> {
        // 添加场景节点
        let scene_node = ResourceNode {
            path: scene_path.to_path_buf(),
            resource_type: ResourceType::Scene,
        };
        self.graph.add_node(scene_node.clone());

        // 读取场景文件内容
        let content = std::fs::read_to_string(scene_path)
            .map_err(|e| format!("Failed to read scene file: {}", e))?;

        // 分析引用（简化实现，实际需要解析具体格式）
        self.analyze_references(&content, scene_path, &scene_node);

        Ok(())
    }

    fn analyze_references(&mut self, content: &str, base_path: &Path, from_node: &ResourceNode) {
        // 查找文件引用（简化模式匹配）
        for line in content.lines() {
            // 查找可能的资源路径
            if line.contains("\"") || line.contains("'") {
                let potential_paths = self.extract_paths_from_line(line);

                for path_str in potential_paths {
                    let path = PathBuf::from(path_str);

                    // 跳过无效路径
                    if !path.exists() {
                        continue;
                    }

                    let resource_type = if let Some(ext) = path.extension() {
                        ResourceType::from_extension(ext.to_str().unwrap_or(""))
                    } else {
                        ResourceType::Unknown
                    };

                    if resource_type != ResourceType::Unknown {
                        let resource_node = ResourceNode {
                            path: path.clone(),
                            resource_type,
                        };

                        self.graph.add_node(resource_node.clone());
                        self.graph.add_dependency(from_node.clone(), resource_node);
                    }
                }
            }
        }
    }

    fn extract_paths_from_line(&self, line: &str) -> Vec<String> {
        let mut paths = Vec::new();

        // 简单提取引号中的内容
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '"' || chars[i] == '\'' {
                let quote = chars[i];
                i += 1;

                let start = i;
                while i < chars.len() && chars[i] != quote {
                    i += 1;
                }

                if i < chars.len() {
                    let path_str = &line[start..i];
                    if path_str.contains('.') && !path_str.contains("http") {
                        // 可能是文件路径
                        paths.push(path_str.to_string());
                    }
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        paths
    }

    /// 获取依赖图
    pub fn get_graph(&self) -> &ResourceDependencyGraph {
        &self.graph
    }

    /// 转移所有权获取依赖图
    pub fn into_graph(self) -> ResourceDependencyGraph {
        self.graph
    }
}

impl Default for ResourceReferenceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// 未使用资源检测器
pub struct UnusedResourceDetector {
    graph: ResourceDependencyGraph,
    entry_points: Vec<ResourceNode>,
}

impl UnusedResourceDetector {
    /// 创建新的检测器
    pub fn new(graph: ResourceDependencyGraph) -> Self {
        Self {
            graph,
            entry_points: Vec::new(),
        }
    }

    /// 设置入口点（从这些资源开始追踪）
    pub fn set_entry_points(&mut self, entry_points: Vec<ResourceNode>) {
        self.entry_points = entry_points;
    }

    /// 检测未使用的资源
    pub fn detect_unused(&self) -> Vec<ResourceNode> {
        let mut reachable = HashSet::new();

        // 从入口点开始DFS
        for entry in &self.entry_points {
            self.dfs_reachable(entry, &mut reachable);
        }

        // 未达节点即为未使用
        self.graph
            .all_resources()
            .into_iter()
            .filter(|resource| !reachable.contains(resource))
            .collect()
    }

    fn dfs_reachable(&self, node: &ResourceNode, reachable: &mut HashSet<ResourceNode>) {
        if reachable.contains(node) {
            return;
        }

        reachable.insert(node.clone());

        for dep in self.graph.get_dependencies(node) {
            self.dfs_reachable(&dep, reachable);
        }
    }

    /// 获取使用统计
    pub fn get_usage_stats(&self) -> UsageStats {
        let total_resources = self.graph.all_resources().len();
        let reachable_count = {
            let mut reachable = HashSet::new();
            for entry in &self.entry_points {
                self.dfs_reachable(entry, &mut reachable);
            }
            reachable.len()
        };

        let unused_count = total_resources - reachable_count;

        UsageStats {
            total_resources,
            used_resources: reachable_count,
            unused_resources: unused_count,
            usage_ratio: if total_resources > 0 {
                reachable_count as f32 / total_resources as f32
            } else {
                1.0
            },
        }
    }
}

/// 使用统计
#[derive(Clone, Copy, Debug)]
pub struct UsageStats {
    pub total_resources: usize,
    pub used_resources: usize,
    pub unused_resources: usize,
    pub usage_ratio: f32,
}

/// 冗余资产清理器
pub struct RedundantAssetCleaner {
    graph: ResourceDependencyGraph,
}

impl RedundantAssetCleaner {
    /// 创建新的清理器
    pub fn new(graph: ResourceDependencyGraph) -> Self {
        Self { graph }
    }

    /// 查找重复的资源（通过哈希值）
    pub fn find_duplicates(&self) -> Result<Vec<Vec<ResourceNode>>, String> {
        let mut hash_map: HashMap<u64, Vec<ResourceNode>> = HashMap::new();

        for resource in self.graph.all_resources() {
            let hash = self.compute_file_hash(&resource.path)?;
            hash_map.entry(hash).or_insert_with(Vec::new).push(resource);
        }

        // 返回有重复的组
        Ok(hash_map
            .into_values()
            .filter(|v| v.len() > 1)
            .collect())
    }

    /// 计算文件哈希
    fn compute_file_hash(&self, path: &Path) -> Result<u64, String> {
        use std::io::Read;

        let file = std::fs::File::open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let mut reader = std::io::BufReader::new(file);

        let mut buffer = [0u8; 8192];
        loop {
            let n = reader.read(&mut buffer)
                .map_err(|e| format!("Failed to read file: {}", e))?;
            if n == 0 {
                break;
            }
            use std::hash::Hash;
            buffer[..n].hash(&mut hasher);
        }

        Ok(hasher.finish())
    }

    /// 获取可安全删除的资源
    pub fn get_safe_to_delete(&self, entry_points: &[ResourceNode]) -> Vec<ResourceNode> {
        let mut detector = UnusedResourceDetector::new(self.graph.clone());
        detector.set_entry_points(entry_points.to_vec());
        detector.detect_unused()
    }
}

/// 依赖分析报告生成器
pub struct DependencyReportGenerator {
    graph: ResourceDependencyGraph,
}

impl DependencyReportGenerator {
    /// 创建新的报告生成器
    pub fn new(graph: ResourceDependencyGraph) -> Self {
        Self { graph }
    }

    /// 生成文本报告
    pub fn generate_text_report(&self) -> String {
        let mut report = String::new();

        report.push_str("# 资源依赖分析报告\n\n");

        // 基本统计
        let resources = self.graph.all_resources();
        report.push_str(&format!("总资源数: {}\n\n", resources.len()));

        // 按类型分组
        let mut by_type: HashMap<ResourceType, Vec<&ResourceNode>> = HashMap::new();
        for resource in &resources {
            by_type
                .entry(resource.resource_type)
                .or_insert_with(Vec::new)
                .push(resource);
        }

        report.push_str("## 资源类型分布\n\n");
        for (resource_type, nodes) in by_type.iter() {
            report.push_str(&format!("- {:?}: {}\n", resource_type, nodes.len()));
        }

        report.push_str("\n## 根节点（入口点）\n\n");
        let roots = self.graph.get_root_nodes();
        for root in &roots {
            report.push_str(&format!("- {}\n", root.path.display()));
        }

        // 循环依赖
        let cycles = self.graph.detect_cycles();
        if !cycles.is_empty() {
            report.push_str("\n## 警告：检测到循环依赖\n\n");
            for (i, cycle) in cycles.iter().enumerate() {
                report.push_str(&format!("循环 {}:\n", i + 1));
                for node in cycle {
                    report.push_str(&format!("  -> {}\n", node.path.display()));
                }
                report.push_str("\n");
            }
        }

        report
    }

    /// 生成JSON报告
    pub fn generate_json_report(&self) -> Result<String, String> {
        let resources = self.graph.all_resources();

        let mut resource_list = Vec::new();
        for resource in &resources {
            let dependencies = self.graph.get_dependencies(resource);
            let dependents = self.graph.get_dependents(resource);

            let resource_data = serde_json::json!({
                "path": resource.path,
                "type": format!("{:?}", resource.resource_type),
                "dependencies": dependencies.len(),
                "dependents": dependents.len(),
                "dependency_paths": dependencies.iter().map(|n| n.path.display().to_string()).collect::<Vec<_>>(),
                "dependent_paths": dependents.iter().map(|n| n.path.display().to_string()).collect::<Vec<_>>(),
            });

            resource_list.push(resource_data);
        }

        serde_json::to_string_pretty(&resource_list)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))
    }
}

// =============================================================================
// 辅助函数
// =============================================================================

/// 简单的依赖分析入口
pub fn analyze_project_resources(project_path: &Path) -> Result<AnalysisResult, String> {
    // 1. 扫描所有资源
    let scanner = ResourceScanner::new(project_path.to_path_buf());
    let resources = scanner.scan()?;

    // 2. 分析引用关系
    let mut analyzer = ResourceReferenceAnalyzer::new();
    for resource in &resources {
        if resource.resource_type == ResourceType::Scene {
            if let Err(e) = analyzer.analyze_scene_file(&resource.path) {
                eprintln!("Warning: {}", e);
            }
        }
    }

    // 3. 获取依赖图
    let graph = analyzer.into_graph();

    // 4. 检测未使用资源
    let roots = graph.get_root_nodes();
    let mut detector = UnusedResourceDetector::new(graph.clone());
    detector.set_entry_points(roots);

    let unused = detector.detect_unused();
    let stats = detector.get_usage_stats();

    // 5. 检测循环依赖
    let cycles = graph.detect_cycles();

    // 6. 生成报告
    let report_gen = DependencyReportGenerator::new(graph.clone());
    let text_report = report_gen.generate_text_report();
    let json_report = report_gen.generate_json_report()?;

    Ok(AnalysisResult {
        graph,
        unused,
        cycles,
        stats,
        text_report,
        json_report,
    })
}

/// 分析结果
#[derive(Clone, Debug)]
pub struct AnalysisResult {
    pub graph: ResourceDependencyGraph,
    pub unused: Vec<ResourceNode>,
    pub cycles: Vec<Vec<ResourceNode>>,
    pub stats: UsageStats,
    pub text_report: String,
    pub json_report: String,
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_cycle_detection() {
        let mut graph = ResourceDependencyGraph::new();

        let node_a = ResourceNode {
            path: PathBuf::from("a.scn"),
            resource_type: ResourceType::Scene,
        };
        let node_b = ResourceNode {
            path: PathBuf::from("b.mesh"),
            resource_type: ResourceType::Mesh,
        };
        let node_c = ResourceNode {
            path: PathBuf::from("c.mat"),
            resource_type: ResourceType::Material,
        };

        graph.add_node(node_a.clone());
        graph.add_node(node_b.clone());
        graph.add_node(node_c.clone());

        // 创建循环: A -> B -> C -> A
        graph.add_dependency(node_a.clone(), node_b.clone());
        graph.add_dependency(node_b.clone(), node_c.clone());
        graph.add_dependency(node_c.clone(), node_a.clone());

        let cycles = graph.detect_cycles();
        assert_eq!(cycles.len(), 1);
    }

    #[test]
    fn test_unused_detection() {
        let mut graph = ResourceDependencyGraph::new();

        let root = ResourceNode {
            path: PathBuf::from("root.scn"),
            resource_type: ResourceType::Scene,
        };
        let used = ResourceNode {
            path: PathBuf::from("used.mesh"),
            resource_type: ResourceType::Mesh,
        };
        let unused = ResourceNode {
            path: PathBuf::from("unused.mesh"),
            resource_type: ResourceType::Mesh,
        };

        graph.add_node(root.clone());
        graph.add_node(used.clone());
        graph.add_node(unused.clone());

        graph.add_dependency(root.clone(), used.clone());
        // unused 不被任何资源依赖

        let mut detector = UnusedResourceDetector::new(graph);
        detector.set_entry_points(vec![root]);

        let unused_resources = detector.detect_unused();
        assert_eq!(unused_resources.len(), 1);
        assert_eq!(unused_resources[0].path, PathBuf::from("unused.mesh"));
    }

    #[test]
    fn test_resource_type_from_extension() {
        assert_eq!(ResourceType::from_extension("obj"), ResourceType::Mesh);
        assert_eq!(ResourceType::from_extension("png"), ResourceType::Texture);
        assert_eq!(ResourceType::from_extension("wav"), ResourceType::Audio);
    }
}
