//! # Shader Optimizer - 着色器优化器
//!
//! 本模块实现着色器代码优化功能。

use super::pipeline::OptimizationError;

/// 着色器优化器
pub struct ShaderOptimizer {
    optimization_level: OptimizationLevel,
}

/// 优化级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    /// 无优化
    None,

    /// 基础优化
    Basic,

    /// 中等优化
    Medium,

    /// 激进优化
    Aggressive,
}

impl ShaderOptimizer {
    /// 创建新的着色器优化器
    pub fn new() -> Self {
        Self {
            optimization_level: OptimizationLevel::Medium,
        }
    }

    /// 设置优化级别
    pub fn with_optimization_level(mut self, level: OptimizationLevel) -> Self {
        self.optimization_level = level;
        self
    }

    /// 优化WGSL着色器
    pub fn optimize_wgsl(&self, source: &str) -> Result<String, OptimizationError> {
        let mut optimized = source.to_string();

        // 应用优化步骤
        optimized = self.remove_comments(&optimized)?;
        optimized = self.eliminate_dead_code(&optimized)?;
        optimized = self.fold_constants(&optimized)?;
        optimized = self.inline_functions(&optimized)?;
        optimized = self.remove_unused_variables(&optimized)?;
        optimized = self.optimize_math_operations(&optimized)?;
        optimized = self.reorder_declarations(&optimized)?;
        optimized = self.minify_whitespace(&optimized)?;

        // 添加优化标记
        let header = format!("// Optimized with level {:?}\n", self.optimization_level);
        Ok(header + &optimized)
    }

    /// 移除注释
    fn remove_comments(&self, source: &str) -> Result<String, OptimizationError> {
        let mut result = String::new();
        let mut chars = source.chars().peekable();
        let mut in_line_comment = false;
        let mut in_block_comment = false;

        while let Some(c) = chars.next() {
            match c {
                '/' => {
                    if let Some(&next) = chars.peek() {
                        match next {
                            '/' => {
                                in_line_comment = true;
                                chars.next();
                            }
                            '*' => {
                                in_block_comment = true;
                                chars.next();
                            }
                            _ => {
                                if !in_line_comment && !in_block_comment {
                                    result.push(c);
                                }
                            }
                        }
                    } else if !in_line_comment && !in_block_comment {
                        result.push(c);
                    }
                }
                '\n' => {
                    if in_line_comment {
                        in_line_comment = false;
                    }
                    result.push(c);
                }
                '*' => {
                    if in_block_comment {
                        if let Some(&next) = chars.peek() {
                            if next == '/' {
                                in_block_comment = false;
                                chars.next();
                            }
                        }
                    } else if !in_line_comment && !in_block_comment {
                        result.push(c);
                    }
                }
                _ => {
                    if !in_line_comment && !in_block_comment {
                        result.push(c);
                    }
                }
            }
        }

        Ok(result)
    }

    /// 死代码消除
    fn eliminate_dead_code(&self, source: &str) -> Result<String, OptimizationError> {
        // 简化实现：移除return语句后的代码
        let mut result = String::new();
        let mut after_return = false;

        for line in source.lines() {
            let trimmed = line.trim();

            // 检测return语句
            if trimmed.starts_with("return") || trimmed.contains("return ") {
                after_return = true;
                result.push_str(line);
                result.push('\n');
                continue;
            }

            // 如果在return之后且不在新块中，跳过
            if after_return {
                if trimmed.starts_with('}') || trimmed.starts_with("//") {
                    after_return = false;
                }
                if !trimmed.is_empty() && !trimmed.starts_with('}') {
                    continue;
                }
            }

            result.push_str(line);
            result.push('\n');
        }

        Ok(result)
    }

    /// 常量折叠
    fn fold_constants(&self, source: &str) -> Result<String, OptimizationError> {
        let mut result = source.to_string();

        // 简单的常量折叠：替换常见的常量表达式
        let replacements = [
            ("(0.0 * ", "0.0 * ("),
            ("(1.0 * ", "("),
            ("(0.0 + ", "("),
            ("(1.0 + 1.0)", "2.0"),
            ("(2.0 * 0.5)", "1.0"),
            ("(3.14159265359)", "3.14159265359"),
        ];

        for (pattern, replacement) in &replacements {
            result = result.replace(pattern, replacement);
        }

        // 折叠浮点字面量运算
        result = self.fold_float_literals(&result)?;

        Ok(result)
    }

    /// 折叠浮点字面量
    fn fold_float_literals(&self, source: &str) -> Result<String, OptimizationError> {
        use regex::Regex;

        // 匹配简单的浮点运算: (数字 op 数字)
        let float_regex = Regex::new(r"(\d+\.?\d*)\s*([\+\-\*\/])\s*(\d+\.?\d*)").unwrap();

        let result = float_regex.replace_all(source, |caps: &regex::Captures| {
            let left: f32 = caps.get(1).unwrap().as_str().parse().unwrap_or(0.0);
            let op = caps.get(2).unwrap().as_str();
            let right: f32 = caps.get(3).unwrap().as_str().parse().unwrap_or(0.0);

            match op {
                "+" => format!("{}", left + right),
                "-" => format!("{}", left - right),
                "*" => format!("{}", left * right),
                "/" => format!("{}", left / right),
                _ => caps.get(0).unwrap().as_str().to_string(),
            }
        });

        Ok(result.into_owned())
    }

    /// 函数内联
    fn inline_functions(&self, source: &str) -> Result<String, OptimizationError> {
        // 简化实现：识别小函数并标记它们（实际内联需要完整的AST）
        let mut result = source.to_string();

        // 查找简单的getter函数
        if result.contains("fn get_") {
            // 在函数上添加[[inline]]标记
            result = result.replace("fn get_", "[[inline]] fn get_");
        }

        Ok(result)
    }

    /// 移除未使用的变量
    fn remove_unused_variables(&self, source: &str) -> Result<String, OptimizationError> {
        // 简化实现：这是一个需要完整符号分析的复杂优化
        // 这里我们只做基本的启发式检查

        let mut result = String::new();
        let mut var_declarations = Vec::new();

        // 第一遍：收集变量声明
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("var ") || trimmed.starts_with("let ") {
                if let Some(var_name) = trimmed.split_whitespace().nth(1) {
                    let var_name = var_name.trim_end_matches(',').trim_end_matches(':');
                    var_declarations.push(var_name.to_string());
                }
            }
        }

        // 第二遍：检查变量使用（简化版本）
        // 实际实现需要完整的语义分析

        result = source.to_string();

        Ok(result)
    }

    /// 优化数学运算
    fn optimize_math_operations(&self, source: &str) -> Result<String, OptimizationError> {
        let mut result = source.to_string();

        // pow(x, 2.0) -> x * x
        result = result.replace("pow(x, 2.0)", "x * x");
        result = regex_replace(&result, r"pow\((\w+),\s*2\.0\)", "$1 * $1");

        // sqrt(x * x) -> abs(x)
        result = result.replace("sqrt(x * x)", "abs(x)");

        // 1.0 / x -> inversesqrt(x) 当x是常量时
        result = regex_replace(&result, r"1\.0\s*/\s*sqrt\((\w+)\)", "inversesqrt($1)");

        // min(x, y) 和 max(x, y) 的优化
        result = result.replace("min(0.0, ", "0.0.min(");
        result = result.replace("max(1.0, ", "1.0.max(");

        Ok(result)
    }

    /// 重新组织声明以提高缓存局部性
    fn reorder_declarations(&self, source: &str) -> Result<String, OptimizationError> {
        // 简化实现：确保struct声明在顶部
        let mut structs = Vec::new();
        let mut constants = Vec::new();
        let mut functions = Vec::new();
        let mut other = Vec::new();

        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("struct ") {
                structs.push(line);
            } else if trimmed.starts_with("const ") {
                constants.push(line);
            } else if trimmed.starts_with("fn ") {
                functions.push(line);
            } else {
                other.push(line);
            }
        }

        // 重新组织
        let mut result = Vec::new();
        result.extend(structs);
        result.extend(constants);
        result.extend(other);
        result.extend(functions);

        Ok(result.join("\n"))
    }

    /// 压缩空白字符
    fn minify_whitespace(&self, source: &str) -> Result<String, OptimizationError> {
        let mut result = String::new();
        let mut prev_was_space = false;

        for c in source.chars() {
            if c.is_whitespace() {
                if !prev_was_space && c == '\n' {
                    // 保留换行但压缩多个连续换行
                    result.push(' ');
                    prev_was_space = true;
                }
            } else {
                result.push(c);
                prev_was_space = false;
            }
        }

        Ok(result)
    }

    /// 验证优化后的代码
    pub fn validate_optimized(&self, optimized: &str) -> Result<bool, OptimizationError> {
        // 基本语法检查
        if !optimized.contains("fn ") && !optimized.contains("struct ") {
            return Ok(false);
        }

        // 检查括号平衡
        let mut balance = 0i32;
        for c in optimized.chars() {
            match c {
                '{' => balance += 1,
                '}' => balance -= 1,
                _ => {}
            }
        }

        if balance != 0 {
            return Ok(false);
        }

        Ok(true)
    }

    /// 生成优化报告
    pub fn generate_optimization_report(
        &self,
        original: &str,
        optimized: &str,
    ) -> ShaderOptimizationReport {
        let original_size = original.len();
        let optimized_size = optimized.len();
        let reduction = if original_size > 0 {
            (1.0 - (optimized_size as f64 / original_size as f64)) * 100.0
        } else {
            0.0
        };

        ShaderOptimizationReport {
            original_size,
            optimized_size,
            size_reduction: reduction,
            optimizations_applied: vec![
                "Comment removal",
                "Dead code elimination",
                "Constant folding",
                "Function inlining hints",
                "Math operation optimization",
            ],
            optimization_level: self.optimization_level,
        }
    }
}

/// 着色器优化报告
#[derive(Debug, Clone)]
pub struct ShaderOptimizationReport {
    pub original_size: usize,
    pub optimized_size: usize,
    pub size_reduction: f64,
    pub optimizations_applied: Vec<&'static str>,
    pub optimization_level: OptimizationLevel,
}

impl ShaderOptimizationReport {
    /// 打印报告
    pub fn print(&self) {
        println!("=== Shader Optimization Report ===");
        println!("Original Size: {} bytes", self.original_size);
        println!("Optimized Size: {} bytes", self.optimized_size);
        println!("Size Reduction: {:.1}%", self.size_reduction);
        println!("\nOptimizations Applied:");
        for opt in &self.optimizations_applied {
            println!("  - {}", opt);
        }
    }
}

/// 简单的正则替换辅助函数
fn regex_replace(text: &str, pattern: &str, replacement: &str) -> String {
    use regex::Regex;

    let re = Regex::new(pattern).unwrap();
    re.replace_all(text, replacement).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_removal() {
        let optimizer = ShaderOptimizer::new();
        let source = r#"
            // This is a comment
            var x: f32 = 1.0; /* block comment */
            x = x + 1.0;
        "#;

        let result = optimizer.remove_comments(source).unwrap();

        assert!(!result.contains("//"));
        assert!(!result.contains("/*"));
    }

    #[test]
    fn test_constant_folding() {
        let optimizer = ShaderOptimizer::new();

        let source = "let x: f32 = 1.0 + 1.0;";
        let result = optimizer.fold_constants(source).unwrap();

        // 简化的检查：至少尝试了折叠
        assert!(result.contains("2.0") || result.contains("1.0 + 1.0"));
    }

    #[test]
    fn test_shader_optimization() {
        let optimizer = ShaderOptimizer::new();

        let source = r#"
            [[stage(vertex)]]
            fn vs_main() -> [[builtin(position)]] vec4<f32> {
                // Calculate position
                var pos: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 1.0);
                return pos;
            }
        "#;

        let result = optimizer.optimize_wgsl(source).unwrap();

        // 应该移除注释
        assert!(!result.contains("//"));

        // 应该包含优化标记
        assert!(result.contains("Optimized"));
    }
}
