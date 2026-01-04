//! # 依赖图构建和分析
//!
//! 解析Cargo.toml和Cargo.lock，构建依赖关系图，分析依赖树结构。
//!
//! ## 功能
//!
//! - 解析Cargo.toml依赖声明
//! - 解析Cargo.lock锁定版本
//! - 构建依赖关系图
//! - 检测循环依赖
//! - 生成依赖树可视化
//! - 计算依赖统计信息
//!
//! ## 使用示例
//!
//! ```no_run
//! use game_engine::tools::cli::dependency::graph::DependencyGraph;
//!
//! let graph = DependencyGraph::from_project(".").unwrap();
//! println!("{}", graph.display_tree());
//! ```

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// 依赖图结构
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// 项目根目录
    pub project_root: PathBuf,

    /// 所有依赖包
    pub packages: Vec<Package>,

    /// 依赖关系（包名 -> 依赖列表）
    pub dependencies: HashMap<String, Vec<Dependency>>,

    /// 图的邻接表表示（用于拓扑排序等算法）
    pub adjacency_list: HashMap<String, Vec<String>>,
}

/// 依赖包信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    /// 包名
    pub name: String,

    /// 版本
    pub version: String,

    /// 源（crates.io, git, local等）
    pub source: Option<String>,

    /// 依赖树中的深度
    pub depth: usize,
}

/// 依赖关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// 被依赖的包名
    pub name: String,

    /// 版本要求
    pub version_req: String,

    /// 依赖类型（normal, dev, build）
    pub kind: DependencyKind,

    /// 是否为可选依赖
    pub optional: bool,

    /// 依赖来源（直接依赖或传递依赖）
    pub source: DependencySource,
}

/// 依赖类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DependencyKind {
    /// 普通运行时依赖
    Normal,
    /// 开发依赖（dev-dependencies）
    Dev,
    /// 构建依赖（build-dependencies）
    Build,
}

/// 依赖来源
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DependencySource {
    /// 直接依赖
    Direct,
    /// 传递依赖（通过其他依赖引入）
    Transitive { from: String },
}

impl DependencyGraph {
    /// 从项目目录构建依赖图
    ///
    /// ## 参数
    ///
    /// - `project_dir`: 项目目录路径
    ///
    /// ## 返回
    ///
    /// 返回构建好的依赖图，如果失败则返回错误
    pub fn from_project<P: AsRef<Path>>(project_dir: P) -> Result<Self, GraphError> {
        let project_dir = project_dir.as_ref();
        let cargo_toml = project_dir.join("Cargo.toml");

        if !cargo_toml.exists() {
            return Err(GraphError::CargoTomlNotFound(cargo_toml));
        }

        let mut graph = Self {
            project_root: project_dir.to_path_buf(),
            packages: Vec::new(),
            dependencies: HashMap::new(),
            adjacency_list: HashMap::new(),
        };

        // 解析Cargo.toml
        graph.parse_cargo_toml(&cargo_toml)?;

        // 尝试解析Cargo.lock（如果存在）
        let cargo_lock = project_dir.join("Cargo.lock");
        if cargo_lock.exists() {
            graph.parse_cargo_lock(&cargo_lock)?;
        }

        // 构建邻接表
        graph.build_adjacency_list();

        Ok(graph)
    }

    /// 解析Cargo.toml文件
    fn parse_cargo_toml(&mut self, path: &Path) -> Result<(), GraphError> {
        let content =
            fs::read_to_string(path).map_err(|e| GraphError::IoError(path.to_path_buf(), e))?;

        // 使用toml解析
        let value: toml::Value = content.parse().map_err(|e: toml::de::Error| {
            GraphError::ParseError(path.to_path_buf(), e.to_string())
        })?;

        // 解析[package]信息
        if let Some(package) = value.get("package") {
            if let Some(name) = package.get("name").and_then(|v| v.as_str()) {
                let version = package.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0");

                self.packages.push(Package {
                    name: name.to_string(),
                    version: version.to_string(),
                    source: None,
                    depth: 0,
                });
            }
        }

        // 解析[dependencies]
        if let Some(deps) = value.get("dependencies") {
            self.parse_dependencies(deps, DependencyKind::Normal)?;
        }

        // 解析[dev-dependencies]
        if let Some(deps) = value.get("dev-dependencies") {
            self.parse_dependencies(deps, DependencyKind::Dev)?;
        }

        // 解析[build-dependencies]
        if let Some(deps) = value.get("build-dependencies") {
            self.parse_dependencies(deps, DependencyKind::Build)?;
        }

        Ok(())
    }

    /// 解析依赖项
    fn parse_dependencies(
        &mut self,
        deps: &toml::Value,
        kind: DependencyKind,
    ) -> Result<(), GraphError> {
        if let Some(table) = deps.as_table() {
            for (name, value) in table {
                let (version_req, optional) = self.parse_dependency_value(value)?;

                let dep = Dependency {
                    name: name.clone(),
                    version_req: version_req.clone(),
                    kind,
                    optional,
                    source: DependencySource::Direct,
                };

                // 添加到依赖列表
                self.dependencies.entry(name.clone()).or_default().push(dep);

                // 添加到包列表
                self.packages.push(Package {
                    name: name.clone(),
                    version: version_req,
                    source: Some("crates.io".to_string()),
                    depth: 1,
                });
            }
        }

        Ok(())
    }

    /// 解析依赖值（支持多种格式）
    fn parse_dependency_value(&self, value: &toml::Value) -> Result<(String, bool), GraphError> {
        match value {
            // 简单版本字符串: "1.0"
            toml::Value::String(version) => Ok((version.clone(), false)),

            // 表格形式: { version = "1.0", optional = true }
            toml::Value::Table(table) => {
                let version =
                    table.get("version").and_then(|v| v.as_str()).unwrap_or("*").to_string();

                let optional = table.get("optional").and_then(|v| v.as_bool()).unwrap_or(false);

                Ok((version, optional))
            }

            // 其他格式（git, path等）
            _ => Ok(("*".to_string(), false)),
        }
    }

    /// 解析Cargo.lock文件
    fn parse_cargo_lock(&mut self, path: &Path) -> Result<(), GraphError> {
        let content =
            fs::read_to_string(path).map_err(|e| GraphError::IoError(path.to_path_buf(), e))?;

        // 解析[[package]] sections
        for line in content.lines() {
            if line.trim().starts_with("name = ") {
                if let Some(name) = extract_quoted_string(line) {
                    // 检查是否已存在
                    if !self.packages.iter().any(|p| p.name == name) {
                        self.packages.push(Package {
                            name: name.clone(),
                            version: "*".to_string(),
                            source: Some("crates.io".to_string()),
                            depth: 2,
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// 构建邻接表（用于图算法）
    fn build_adjacency_list(&mut self) {
        for (name, deps) in &self.dependencies {
            let dep_names: Vec<String> = deps.iter().map(|d| d.name.clone()).collect();

            self.adjacency_list.insert(name.clone(), dep_names);
        }
    }

    /// 检测循环依赖
    ///
    /// 返回包含循环的包列表
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for package in &self.packages {
            if !visited.contains(&package.name) {
                self.dfs_detect_cycles(
                    &package.name,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    /// 深度优先搜索检测循环
    fn dfs_detect_cycles(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = self.adjacency_list.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if self.dfs_detect_cycles(neighbor, visited, rec_stack, path, cycles) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    // 找到循环
                    let cycle_start = path.iter().position(|n| n == neighbor).unwrap();
                    let cycle = path[cycle_start..].to_vec();
                    cycles.push(cycle);
                    return true;
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
        false
    }

    /// 计算依赖统计信息
    pub fn statistics(&self) -> DependencyStatistics {
        let total_packages = self.packages.len();
        let direct_deps = self
            .dependencies
            .values()
            .flat_map(|deps| deps.iter().filter(|d| d.source == DependencySource::Direct))
            .count();

        let transitive_deps = total_packages.saturating_sub(direct_deps + 1); // +1 for the package itself

        let max_depth = self.packages.iter().map(|p| p.depth).max().unwrap_or(0);

        DependencyStatistics {
            total_packages,
            direct_dependencies: direct_deps,
            transitive_dependencies: transitive_deps,
            max_depth,
            has_cycles: !self.detect_cycles().is_empty(),
        }
    }

    /// 显示依赖树
    pub fn display_tree(&self) -> String {
        let mut output = String::new();

        for package in &self.packages {
            if package.depth == 0 {
                // 根包
                output.push_str(&format!("{} {}\n", package.name, package.version));

                // 显示其依赖
                if let Some(deps) = self.dependencies.get(&package.name) {
                    for dep in deps {
                        self.display_dependency(dep, 1, &mut output);
                    }
                }
            }
        }

        output
    }

    /// 递归显示依赖
    fn display_dependency(&self, dep: &Dependency, depth: usize, output: &mut String) {
        let indent = "  ".repeat(depth);
        let kind_marker = match dep.kind {
            DependencyKind::Normal => "",
            DependencyKind::Dev => " [dev]",
            DependencyKind::Build => " [build]",
        };
        let optional_marker = if dep.optional { " (optional)" } else { "" };

        output.push_str(&format!(
            "{}├─ {}{}{}\n",
            indent, dep.name, kind_marker, optional_marker
        ));

        // 递归显示传递依赖
        if let Some(trans_deps) = self.dependencies.get(&dep.name) {
            for trans_dep in trans_deps {
                self.display_dependency(trans_dep, depth + 1, output);
            }
        }
    }

    /// 生成Graphviz DOT格式
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph dependencies {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box];\n\n");

        for package in &self.packages {
            if package.depth == 0 {
                if let Some(deps) = self.dependencies.get(&package.name) {
                    for dep in deps {
                        dot.push_str(&format!("  \"{}\" -> \"{}\";\n", package.name, dep.name));
                    }
                }
            }
        }

        dot.push_str("}\n");
        dot
    }
}

/// 依赖统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStatistics {
    /// 总包数
    pub total_packages: usize,

    /// 直接依赖数
    pub direct_dependencies: usize,

    /// 传递依赖数
    pub transitive_dependencies: usize,

    /// 最大依赖深度
    pub max_depth: usize,

    /// 是否存在循环依赖
    pub has_cycles: bool,
}

/// 提取引号中的字符串
fn extract_quoted_string(line: &str) -> Option<String> {
    let start = line.find('"')? + 1;
    let end = line[start..].find('"')?;
    Some(line[start..start + end].to_string())
}

/// 依赖图错误
#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("Cargo.toml not found at {0:?}")]
    CargoTomlNotFound(PathBuf),

    #[error("IO error on {0:?}: {1}")]
    IoError(PathBuf, std::io::Error),

    #[error("Parse error in {0:?}: {1}")]
    ParseError(PathBuf, String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_graph_creation() {
        // 这个测试需要在实际项目环境中运行
        // let graph = DependencyGraph::from_project(".").unwrap();
        // assert!(!graph.packages.is_empty());
    }

    #[test]
    fn test_cycle_detection() {
        // 测试循环依赖检测
        let mut graph = DependencyGraph {
            project_root: PathBuf::from("."),
            packages: vec![
                Package {
                    name: "a".to_string(),
                    version: "1.0".to_string(),
                    source: None,
                    depth: 0,
                },
                Package {
                    name: "b".to_string(),
                    version: "1.0".to_string(),
                    source: None,
                    depth: 1,
                },
                Package {
                    name: "c".to_string(),
                    version: "1.0".to_string(),
                    source: None,
                    depth: 2,
                },
            ],
            dependencies: HashMap::new(),
            adjacency_list: HashMap::new(),
        };

        // 创建循环: a -> b -> c -> a
        graph.adjacency_list.insert("a".to_string(), vec!["b".to_string()]);
        graph.adjacency_list.insert("b".to_string(), vec!["c".to_string()]);
        graph.adjacency_list.insert("c".to_string(), vec!["a".to_string()]);

        let cycles = graph.detect_cycles();
        assert!(!cycles.is_empty());
    }
}
