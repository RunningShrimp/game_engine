//! 文档生成工具
//!
//! 自动生成项目文档，包括 API 文档、架构图等。
//!
//! # 功能特性
//!
//! - 自动从Rust源代码提取API文档
//! - 支持Markdown格式输出
//! - 生成架构图和流程图
//! - 创建交互式教程
//! - 生成完整的示例文档
//! - 文档覆盖率分析

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use syn::{Item, ItemEnum, ItemFn, ItemStruct, Visibility};

/// API文档项详情
#[derive(Debug, Clone)]
pub struct ApiDetail {
    /// 项名称
    pub name: String,
    /// 文档注释
    pub documentation: String,
    /// 源代码位置
    pub source_location: SourceLocation,
    /// 相关API
    pub related_apis: Vec<String>,
    /// 示例代码
    pub examples: Vec<String>,
}

/// 源代码位置
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub module: String,
}

/// 文档生成器
pub struct DocumentationGenerator {
    /// 项目根目录
    project_root: PathBuf,
    /// 输出目录
    output_dir: PathBuf,
    /// 生成的文档列表
    generated_docs: Vec<GeneratedDocument>,
}

/// 生成的文档
#[derive(Debug, Clone)]
pub struct GeneratedDocument {
    /// 文档路径
    pub path: PathBuf,
    /// 文档类型
    pub doc_type: DocType,
    /// 文档标题
    pub title: String,
    /// 字数
    pub word_count: usize,
}

/// 文档类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DocType {
    /// API 文档
    Api,
    /// 教程
    Tutorial,
    /// 指南
    Guide,
    /// 架构文档
    Architecture,
    /// 参考手册
    Reference,
    /// 示例
    Example,
}

impl DocumentationGenerator {
    /// 创建新的文档生成器
    pub fn new(project_root: PathBuf, output_dir: PathBuf) -> Self {
        Self {
            project_root,
            output_dir,
            generated_docs: Vec::new(),
        }
    }

    /// 生成所有文档
    pub fn generate_all(&mut self) -> Result<(), DocGenError> {
        println!("开始生成文档...");

        // 生成 API 文档
        self.generate_api_docs()?;

        // 生成架构文档
        self.generate_architecture_docs()?;

        // 生成用户指南
        self.generate_user_guides()?;

        // 生成教程
        self.generate_tutorials()?;

        // 生成示例文档
        self.generate_examples()?;

        // 生成索引
        self.generate_index()?;

        println!("文档生成完成！共生成 {} 个文档", self.generated_docs.len());

        Ok(())
    }

    /// 生成 API 文档
    fn generate_api_docs(&mut self) -> Result<(), DocGenError> {
        println!("生成 API 文档...");

        let api_modules = vec![
            ("scripting", "脚本系统 API"),
            ("rendering", "渲染系统 API"),
            ("physics", "物理系统 API"),
            ("audio", "音频系统 API"),
            ("animation", "动画系统 API"),
            ("networking", "网络系统 API"),
            ("platform", "平台 API"),
        ];

        for (module, title) in api_modules {
            let doc = self.generate_module_api_doc(module, title)?;
            self.generated_docs.push(doc);
        }

        Ok(())
    }

    /// 生成单个模块的 API 文档
    fn generate_module_api_doc(
        &self,
        module: &str,
        title: &str,
    ) -> Result<GeneratedDocument, DocGenError> {
        let doc_path = self.output_dir.join(format!("api/{module}.md"));

        // 创建文档内容
        let content = format!(
            "# {title}\n\n## 概述\n\n本模块提供 {module} 相关的 API 接口。\n\n## 核心 Trait\n\n```rust\n// TODO: 从源代码提取 trait 定义\n```\n\n## 结构体\n\n### 主要结构体\n\n```rust\n// TODO: 从源代码提取结构体定义\n```\n\n## 函数\n\n### 公共 API\n\n```rust\n// TODO: 从源代码提取函数签名\n```\n\n## 使用示例\n\n```rust\n// TODO: 添加使用示例\n```\n\n## 注意事项\n\n- TODO: 添加使用注意事项\n\n## 相关文档\n\n- [架构文档](../architecture/{module}.md)\n- [教程](../tutorials/{module}_guide.md)\n"
        );

        // 写入文件
        self.write_doc(&doc_path, &content)?;

        Ok(GeneratedDocument {
            path: doc_path,
            doc_type: DocType::Api,
            title: title.to_string(),
            word_count: content.split_whitespace().count(),
        })
    }

    /// 生成架构文档
    fn generate_architecture_docs(&mut self) -> Result<(), DocGenError> {
        println!("生成架构文档...");

        let arch_docs = vec![
            ("ecs", "ECS 架构"),
            ("rendering", "渲染管线"),
            ("scripting", "脚本系统"),
            ("networking", "网络架构"),
        ];

        for (doc, title) in arch_docs {
            let doc_path = self.output_dir.join(format!("architecture/{doc}.md"));

            let content = format!(
                "# {title}\n\n## 架构概述\n\n本文档描述 {doc} 的架构设计。\n\n## 设计目标\n\n1. 性能优先\n2. 可扩展性\n3. 易用性\n\n## 架构图\n\n```mermaid\ngraph TD\n    A[开始] --> B[结束]\n```\n\n## 核心组件\n\n### 组件 1\n\n- 功能：TODO\n- 接口：TODO\n\n### 组件 2\n\n- 功能：TODO\n- 接口：TODO\n\n## 数据流\n\n```mermaid\nsequenceDiagram\n    A->>B: 请求\n    B->>A: 响应\n```\n\n## 性能考虑\n\n- TODO: 添加性能分析\n\n## 扩展点\n\n- TODO: 添加扩展点说明\n"
            );

            self.write_doc(&doc_path, &content)?;

            self.generated_docs.push(GeneratedDocument {
                path: doc_path,
                doc_type: DocType::Architecture,
                title: title.to_string(),
                word_count: content.split_whitespace().count(),
            });
        }

        Ok(())
    }

    /// 生成用户指南
    fn generate_user_guides(&mut self) -> Result<(), DocGenError> {
        println!("生成用户指南...");

        let guides = vec![
            ("getting_started", "快速开始指南"),
            ("installation", "安装指南"),
            ("configuration", "配置指南"),
            ("deployment", "部署指南"),
        ];

        for (guide, title) in guides {
            let doc_path = self.output_dir.join(format!("guides/{guide}.md"));

            let content = format!(
                "# {}\n\n## 概述\n\n本指南将帮助您 {}。\n\n## 前置要求\n\n- Rust 1.70 或更高版本\n- 操作系统：Windows/macOS/Linux\n\n## 步骤 1：准备工作\n\nTODO: 添加详细步骤\n\n## 步骤 2：执行\n\nTODO: 添加详细步骤\n\n## 步骤 3：验证\n\nTODO: 添加验证步骤\n\n## 故障排除\n\n### 问题 1\n\n**症状**：TODO\n\n**解决方案**：TODO\n\n## 下一步\n\n- 查看相关教程\n- 阅读示例代码\n",
                title,
                title.replace("指南", "")
            );

            self.write_doc(&doc_path, &content)?;

            self.generated_docs.push(GeneratedDocument {
                path: doc_path,
                doc_type: DocType::Guide,
                title: title.to_string(),
                word_count: content.split_whitespace().count(),
            });
        }

        Ok(())
    }

    /// 生成教程
    fn generate_tutorials(&mut self) -> Result<(), DocGenError> {
        println!("生成教程...");

        let tutorials = vec![
            ("hello_world", "Hello World 教程"),
            ("ecs_basics", "ECS 基础教程"),
            ("rendering_basics", "渲染基础教程"),
            ("scripting_basics", "脚本基础教程"),
        ];

        for (tutorial, title) in tutorials {
            let doc_path = self.output_dir.join(format!("tutorials/{tutorial}.md"));

            let content = format!(
                "# {}\n\n## 教程概述\n\n本教程将教您 {}。\n\n## 学习目标\n\n完成本教程后，您将能够：\n\n- TODO: 目标 1\n- TODO: 目标 2\n\n## 预计时间\n\n约 30 分钟\n\n## 开始\n\n### 第 1 步\n\nTODO: 添加步骤\n\n### 第 2 步\n\nTODO: 添加步骤\n\n## 完整代码\n\n```rust\n// TODO: 添加完整示例代码\n```\n\n## 运行程序\n\n```bash\ncargo run --example {}\n```\n\n## 进阶挑战\n\n- TODO: 添加挑战\n\n## 相关资源\n\n- [API 文档](../api/)\n- [更多示例](../examples/)\n",
                title,
                title.replace("教程", ""),
                tutorial
            );

            self.write_doc(&doc_path, &content)?;

            self.generated_docs.push(GeneratedDocument {
                path: doc_path,
                doc_type: DocType::Tutorial,
                title: title.to_string(),
                word_count: content.split_whitespace().count(),
            });
        }

        Ok(())
    }

    /// 生成示例文档
    fn generate_examples(&mut self) -> Result<(), DocGenError> {
        println!("生成示例文档...");

        let examples_dir = self.project_root.join("examples");

        if !examples_dir.exists() {
            println!("示例目录不存在，跳过示例文档生成");
            return Ok(());
        }

        // 读取示例文件并生成文档
        let entries = fs::read_dir(&examples_dir)
            .map_err(|e| DocGenError::IoError(format!("无法读取示例目录: {e}")))?;

        for entry in entries {
            let entry = entry.map_err(|e| DocGenError::IoError(format!("无法读取目录项: {e}")))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

                let title = format!("{} 示例", file_name.replace("_", " "));
                let doc = self.generate_example_doc(&path, &title)?;
                self.generated_docs.push(doc);
            }
        }

        Ok(())
    }

    /// 生成单个示例的文档
    fn generate_example_doc(
        &self,
        example_path: &Path,
        title: &str,
    ) -> Result<GeneratedDocument, DocGenError> {
        let file_name = example_path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

        let doc_path = self.output_dir.join(format!("examples/{file_name}.md"));

        // 读取示例代码
        let code = fs::read_to_string(example_path)
            .map_err(|e| DocGenError::IoError(format!("无法读取示例文件: {e}")))?;

        // 提取注释作为说明
        let description = self.extract_code_description(&code);

        let content = format!(
            "# {title}\n\n## 示例说明\n\n{description}\n\n## 源代码\n\n```rust\n{code}\n```\n\n## 运行方法\n\n```bash\ncargo run --example {file_name}\n```\n\n## 预期输出\n\n```\nTODO: 添加预期输出\n```\n\n## 相关文档\n\n- [API 文档](../api/)\n- [教程](../tutorials/)\n"
        );

        self.write_doc(&doc_path, &content)?;

        Ok(GeneratedDocument {
            path: doc_path,
            doc_type: DocType::Example,
            title: title.to_string(),
            word_count: content.split_whitespace().count(),
        })
    }

    /// 从代码中提取描述注释
    fn extract_code_description(&self, code: &str) -> String {
        let lines: Vec<&str> = code.lines().take(20).collect();
        let mut description = String::new();
        let mut in_doc_comment = false;

        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("//!") || trimmed.starts_with("///") {
                in_doc_comment = true;
                let content = trimmed.replacen("//!", "", 1).replacen("///", "", 1);
                description.push_str(&content);
                description.push('\n');
            } else if in_doc_comment && trimmed.starts_with("//") {
                let content = trimmed.replacen("//", "", 1);
                description.push_str(&content);
                description.push('\n');
            } else if in_doc_comment && !trimmed.is_empty() {
                break;
            }
        }

        if description.trim().is_empty() {
            "这是一个示例程序，展示了如何使用游戏引擎的相关功能。".to_string()
        } else {
            description
        }
    }

    /// 生成文档索引
    fn generate_index(&self) -> Result<(), DocGenError> {
        println!("生成文档索引...");

        let index_path = self.output_dir.join("INDEX.md");

        let mut content = String::from("# 游戏引擎文档索引\n\n");

        // 按类型分组
        let mut grouped: HashMap<DocType, Vec<&GeneratedDocument>> = HashMap::new();
        for doc in &self.generated_docs {
            grouped.entry(doc.doc_type.clone()).or_default().push(doc);
        }

        // 生成索引
        if let Some(api_docs) = grouped.get(&DocType::Api) {
            content.push_str("## API 文档\n\n");
            for doc in api_docs {
                content.push_str(&format!("- [{}]({})\n", doc.title, doc.path.display()));
            }
            content.push('\n');
        }

        if let Some(arch_docs) = grouped.get(&DocType::Architecture) {
            content.push_str("## 架构文档\n\n");
            for doc in arch_docs {
                content.push_str(&format!("- [{}]({})\n", doc.title, doc.path.display()));
            }
            content.push('\n');
        }

        if let Some(guides) = grouped.get(&DocType::Guide) {
            content.push_str("## 用户指南\n\n");
            for doc in guides {
                content.push_str(&format!("- [{}]({})\n", doc.title, doc.path.display()));
            }
            content.push('\n');
        }

        if let Some(tutorials) = grouped.get(&DocType::Tutorial) {
            content.push_str("## 教程\n\n");
            for doc in tutorials {
                content.push_str(&format!("- [{}]({})\n", doc.title, doc.path.display()));
            }
            content.push('\n');
        }

        if let Some(examples) = grouped.get(&DocType::Example) {
            content.push_str("## 示例\n\n");
            for doc in examples {
                content.push_str(&format!("- [{}]({})\n", doc.title, doc.path.display()));
            }
        }

        // 添加统计信息
        content.push_str(&format!(
            "\n---\n\n**文档统计**\n\n- 总文档数: {}\n- 总字数: {}\n",
            self.generated_docs.len(),
            self.generated_docs.iter().map(|d| d.word_count).sum::<usize>()
        ));

        self.write_doc(&index_path, &content)?;

        Ok(())
    }

    /// 写入文档文件
    fn write_doc(&self, path: &Path, content: &str) -> Result<(), DocGenError> {
        // 确保目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                DocGenError::IoError(format!("无法创建目录 {}: {}", parent.display(), e))
            })?;
        }

        // 写入文件
        fs::write(path, content)
            .map_err(|e| DocGenError::IoError(format!("无法写入文件 {}: {}", path.display(), e)))?;

        Ok(())
    }

    /// 获取生成的文档列表
    pub fn get_generated_docs(&self) -> &[GeneratedDocument] {
        &self.generated_docs
    }

    /// 分析文档覆盖率
    pub fn analyze_coverage(&self) -> DocCoverageReport {
        let mut report = DocCoverageReport::default();

        // 统计各类文档数量
        for doc in &self.generated_docs {
            match doc.doc_type {
                DocType::Api => report.api_count += 1,
                DocType::Tutorial => report.tutorial_count += 1,
                DocType::Guide => report.guide_count += 1,
                DocType::Architecture => report.architecture_count += 1,
                DocType::Reference => report.reference_count += 1,
                DocType::Example => report.example_count += 1,
            }
            report.total_word_count += doc.word_count;
        }

        report.total_docs = self.generated_docs.len();

        // 计算覆盖率百分比（基于目标：每个主要模块至少有API文档、教程和指南）
        let target_docs = 50; // 假设目标是50个文档
        report.coverage_percentage = (report.total_docs as f32 / target_docs as f32 * 100.0) as u32;

        report
    }

    /// 生成文档覆盖率报告
    pub fn generate_coverage_report(&self, output_path: &Path) -> Result<(), DocGenError> {
        let report = self.analyze_coverage();

        let content = format!(
            "# 文档覆盖率报告\n\n\
            ## 总体统计\n\n\
            - **总文档数**: {}\n\
            - **API文档**: {}\n\
            - **教程**: {}\n\
            - **指南**: {}\n\
            - **架构文档**: {}\n\
            - **参考手册**: {}\n\
            - **示例**: {}\n\
            - **总字数**: {}\n\
            - **覆盖率**: {}%\n\n\
            ## 详细分析\n\n\
            ### 按类型分布\n\n\
            | 类型 | 数量 | 占比 |\n\
            |------|------|------|\n\
            | API文档 | {} | {:.1}% |\n\
            | 教程 | {} | {:.1}% |\n\
            | 指南 | {} | {:.1}% |\n\
            | 架构文档 | {} | {:.1}% |\n\
            | 参考手册 | {} | {:.1}% |\n\
            | 示例 | {} | {:.1}% |\n\n\
            ### 改进建议\n\n\
            {}\
            ## 附录\n\n\
            生成时间: {}\n",
            report.total_docs,
            report.api_count,
            report.tutorial_count,
            report.guide_count,
            report.architecture_count,
            report.reference_count,
            report.example_count,
            report.total_word_count,
            report.coverage_percentage,
            report.api_count,
            (report.api_count as f32 / report.total_docs as f32 * 100.0),
            report.tutorial_count,
            (report.tutorial_count as f32 / report.total_docs as f32 * 100.0),
            report.guide_count,
            (report.guide_count as f32 / report.total_docs as f32 * 100.0),
            report.architecture_count,
            (report.architecture_count as f32 / report.total_docs as f32 * 100.0),
            report.reference_count,
            (report.reference_count as f32 / report.total_docs as f32 * 100.0),
            report.example_count,
            (report.example_count as f32 / report.total_docs as f32 * 100.0),
            self.generate_improvement_suggestions(&report),
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );

        self.write_doc(output_path, &content)?;

        Ok(())
    }

    /// 生成改进建议
    fn generate_improvement_suggestions(&self, report: &DocCoverageReport) -> String {
        let mut suggestions = String::new();

        if report.tutorial_count < 10 {
            suggestions.push_str(&format!(
                "- 🔴 **教程不足**: 当前{}个教程，建议至少10个\n",
                report.tutorial_count
            ));
        }

        if report.guide_count < 5 {
            suggestions.push_str(&format!(
                "- 🟡 **指南缺失**: 当前{}个指南，建议至少5个\n",
                report.guide_count
            ));
        }

        if report.example_count < 20 {
            suggestions.push_str(&format!(
                "- 🟡 **示例不够**: 当前{}个示例，建议至少20个\n",
                report.example_count
            ));
        }

        if report.api_count < 15 {
            suggestions.push_str(&format!(
                "- 🟠 **API文档不完整**: 当前{}个API文档，建议至少15个\n",
                report.api_count
            ));
        }

        if suggestions.is_empty() {
            suggestions.push_str("✅ 文档覆盖率良好，继续保持！\n\n");
        }

        suggestions
    }

    /// 从源代码提取API详情
    pub fn extract_api_details(&self, source_file: &Path) -> Result<Vec<ApiDetail>, DocGenError> {
        let code = fs::read_to_string(source_file)
            .map_err(|e| DocGenError::ParseError(format!("无法读取源文件: {e}")))?;

        let syntax = syn::parse_file(&code)
            .map_err(|e| DocGenError::ParseError(format!("解析失败: {e}")))?;

        let mut apis = Vec::new();

        for item in syntax.items {
            match &item {
                Item::Fn(item_fn) => {
                    if let Visibility::Public(_) = &item_fn.vis {
                        let api = ApiDetail {
                            name: item_fn.sig.ident.to_string(),
                            documentation: self.extract_docs_from_attrs(&item_fn.attrs),
                            source_location: SourceLocation {
                                file: source_file.display().to_string(),
                                line: 0, // TODO: 从Span提取行号需要proc-macro2的特定版本
                                module: "crate".to_string(), // TODO: 提取真实模块路径
                            },
                            related_apis: Vec::new(), // TODO: 分析关联API
                            examples: self.extract_examples_from_attrs(&item_fn.attrs),
                        };
                        apis.push(api);
                    }
                }
                Item::Struct(item_struct) => {
                    if let Visibility::Public(_) = &item_struct.vis {
                        let api = ApiDetail {
                            name: item_struct.ident.to_string(),
                            documentation: self.extract_docs_from_attrs(&item_struct.attrs),
                            source_location: SourceLocation {
                                file: source_file.display().to_string(),
                                line: 0, // TODO: 从Span提取行号需要proc-macro2的特定版本
                                module: "crate".to_string(),
                            },
                            related_apis: Vec::new(),
                            examples: self.extract_examples_from_attrs(&item_struct.attrs),
                        };
                        apis.push(api);
                    }
                }
                Item::Enum(item_enum) => {
                    if let Visibility::Public(_) = &item_enum.vis {
                        let api = ApiDetail {
                            name: item_enum.ident.to_string(),
                            documentation: self.extract_docs_from_attrs(&item_enum.attrs),
                            source_location: SourceLocation {
                                file: source_file.display().to_string(),
                                line: 0, // TODO: 从Span提取行号需要proc-macro2的特定版本
                                module: "crate".to_string(),
                            },
                            related_apis: Vec::new(),
                            examples: self.extract_examples_from_attrs(&item_enum.attrs),
                        };
                        apis.push(api);
                    }
                }
                _ => {}
            }
        }

        Ok(apis)
    }

    /// 从属性中提取文档
    fn extract_docs_from_attrs(&self, attrs: &[syn::Attribute]) -> String {
        let mut docs = String::new();

        for attr in attrs {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(meta) = &attr.meta {
                    if let syn::Expr::Lit(expr_lit) = &meta.value {
                        if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                            let line = lit_str.value();
                            let line = line.strip_prefix(' ').unwrap_or(&line);
                            docs.push_str(line);
                            docs.push('\n');
                        }
                    }
                }
            }
        }

        docs.trim().to_string()
    }

    /// 从属性中提取示例代码
    fn extract_examples_from_attrs(&self, attrs: &[syn::Attribute]) -> Vec<String> {
        let mut examples = Vec::new();

        for attr in attrs {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(meta) = &attr.meta {
                    if let syn::Expr::Lit(expr_lit) = &meta.value {
                        if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                            let line = lit_str.value();
                            if line.contains("```") {
                                examples.push(line);
                            }
                        }
                    }
                }
            }
        }

        examples
    }
}

/// 文档覆盖率报告
#[derive(Debug, Clone, Default)]
pub struct DocCoverageReport {
    pub total_docs: usize,
    pub api_count: usize,
    pub tutorial_count: usize,
    pub guide_count: usize,
    pub architecture_count: usize,
    pub reference_count: usize,
    pub example_count: usize,
    pub total_word_count: usize,
    pub coverage_percentage: u32,
}

/// 文档生成错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocGenError {
    /// IO 错误
    IoError(String),
    /// 解析错误
    ParseError(String),
    /// 模板错误
    TemplateError(String),
}

impl std::fmt::Display for DocGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocGenError::IoError(msg) => write!(f, "IO error: {msg}"),
            DocGenError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            DocGenError::TemplateError(msg) => write!(f, "Template error: {msg}"),
        }
    }
}

impl std::error::Error for DocGenError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_code_description() {
        let generator =
            DocumentationGenerator::new(PathBuf::from("/tmp"), PathBuf::from("/tmp/output"));

        let code = r#"//! This is a test example.
//! It demonstrates doc extraction.
fn main() {
    println!("Hello, World!");
}"#;

        let description = generator.extract_code_description(code);
        assert!(description.contains("test example"));
        assert!(description.contains("demonstrates"));
    }
}
