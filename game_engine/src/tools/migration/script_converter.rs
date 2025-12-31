//! Unity脚本转换工具
//!
//! C#到Lua和TypeScript的自动转换系统。

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[cfg(feature = "regex")]
use super::{MigrationError, MigrationPhase, MigrationProgress};
#[cfg(feature = "regex")]
use regex::Regex;

/// Unity脚本转换器
#[cfg(feature = "regex")]
pub struct UnityScriptConverter {
    /// API映射表
    api_mappings: HashMap<String, ApiMapping>,

    /// 进度回调
    progress_callback: Option<Box<dyn Fn(MigrationProgress) + Send + Sync>>,
}

/// API映射
#[derive(Debug, Clone)]
pub struct ApiMapping {
    /// Unity API
    pub unity_api: String,

    /// 目标引擎API
    pub engine_api: String,

    /// 转换规则
    pub conversion_rule: ConversionRule,
}

/// 转换规则
#[derive(Debug, Clone)]
pub enum ConversionRule {
    /// 直接映射
    Direct,
    /// 属性访问
    PropertyAccess,
    /// 方法调用
    MethodCall,
    /// 自定义转换
    Custom(String),
}

#[cfg(feature = "regex")]
impl UnityScriptConverter {
    /// 创建新的脚本转换器
    pub fn new() -> Self {
        Self {
            api_mappings: Self::build_default_mappings(),
            progress_callback: None,
        }
    }

    /// 设置进度回调
    pub fn with_progress_callback(
        mut self,
        callback: Box<dyn Fn(MigrationProgress) + Send + Sync>,
    ) -> Self {
        self.progress_callback = Some(callback);
        self
    }

    /// 添加API映射
    pub fn add_api_mapping(&mut self, unity_api: String, engine_api: String, rule: ConversionRule) {
        self.api_mappings.insert(
            unity_api,
            ApiMapping {
                unity_api: unity_api.clone(),
                engine_api,
                conversion_rule: rule,
            },
        );
    }

    /// 转换C#到Lua
    pub fn convert_csharp_to_lua(
        &self,
        csharp_code: &str,
        script_name: &str,
    ) -> Result<ConvertedScript, MigrationError> {
        self.report_progress(0, 4, "Parsing C# code".to_string());

        // 1. 解析C#代码
        let parsed = self.parse_csharp(csharp_code)?;

        self.report_progress(1, 4, "Converting API calls".to_string());

        // 2. 转换API调用
        let converted = self.convert_apis(&parsed, ScriptTarget::Lua)?;

        self.report_progress(2, 4, "Generating Lua code".to_string());

        // 3. 生成Lua代码
        let lua_code = self.generate_lua(&converted, script_name)?;

        self.report_progress(3, 4, "Lua conversion complete".to_string());

        Ok(ConvertedScript {
            code: lua_code,
            language: ScriptLanguage::Lua,
            dependencies: converted.dependencies,
        })
    }

    /// 转换C#到TypeScript
    pub fn convert_csharp_to_typescript(
        &self,
        csharp_code: &str,
        script_name: &str,
    ) -> Result<ConvertedScript, MigrationError> {
        self.report_progress(0, 4, "Parsing C# code".to_string());

        // 1. 解析C#代码
        let parsed = self.parse_csharp(csharp_code)?;

        self.report_progress(1, 4, "Converting API calls".to_string());

        // 2. 转换API调用
        let converted = self.convert_apis(&parsed, ScriptTarget::TypeScript)?;

        self.report_progress(2, 4, "Generating TypeScript code".to_string());

        // 3. 生成TypeScript代码
        let ts_code = self.generate_typescript(&converted, script_name)?;

        self.report_progress(3, 4, "TypeScript conversion complete".to_string());

        Ok(ConvertedScript {
            code: ts_code,
            language: ScriptLanguage::TypeScript,
            dependencies: converted.dependencies,
        })
    }

    /// 批量转换目录
    pub fn convert_directory(
        &self,
        input_dir: PathBuf,
        output_dir: PathBuf,
        target: ScriptTarget,
    ) -> Result<Vec<ConversionResult>, MigrationError> {
        let mut results = Vec::new();

        // 读取目录中的所有.cs文件
        let entries =
            fs::read_dir(&input_dir).map_err(|e| MigrationError::FileReadError(e.to_string()))?;

        let total = entries.count() as u32;

        for (index, entry) in fs::read_dir(&input_dir)
            .map_err(|e| MigrationError::FileReadError(e.to_string()))?
            .enumerate()
        {
            let entry = entry.map_err(|e| MigrationError::FileReadError(e.to_string()))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("cs") {
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("script");

                let csharp_code = fs::read_to_string(&path)
                    .map_err(|e| MigrationError::FileReadError(e.to_string()))?;

                let converted = match target {
                    ScriptTarget::Lua => self.convert_csharp_to_lua(&csharp_code, file_name)?,
                    ScriptTarget::TypeScript => {
                        self.convert_csharp_to_typescript(&csharp_code, file_name)?
                    }
                };

                let output_path = output_dir.join(converted.language.extension()).join(format!(
                    "{}.{}",
                    file_name,
                    converted.language.extension()
                ));

                // 确保输出目录存在
                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| MigrationError::ConversionError(e.to_string()))?;
                }

                fs::write(&output_path, &converted.code)
                    .map_err(|e| MigrationError::ConversionError(e.to_string()))?;

                results.push(ConversionResult {
                    input_path: path,
                    output_path,
                    success: true,
                    warnings: vec![],
                });

                self.report_progress(index as u32 + 1, total, format!("Converted {}", file_name));
            }
        }

        Ok(results)
    }

    /// 解析C#代码
    fn parse_csharp(&self, code: &str) -> Result<ParsedScript, MigrationError> {
        // 使用正则表达式解析C#代码
        // 实际生产环境应该使用语法分析器

        let using_re = Regex::new(r"using\s+([^;]+);").unwrap();
        let class_re = Regex::new(r"public\s+class\s+(\w+)").unwrap();
        let field_re = Regex::new(r"public\s+(\w+)\s+(\w+)").unwrap();
        let method_re = Regex::new(r"void\s+(\w+)\s*\(").unwrap();

        let mut usings = Vec::new();
        for cap in using_re.captures_iter(code) {
            if let Some(using) = cap.get(1) {
                usings.push(using.as_str().to_string());
            }
        }

        let class_name = class_re
            .captures(code)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or("UnknownClass".to_string());

        let mut fields = Vec::new();
        for cap in field_re.captures_iter(code) {
            if let (Some(type_name), Some(field_name)) = (cap.get(1), cap.get(2)) {
                fields.push(Field {
                    field_type: type_name.as_str().to_string(),
                    name: field_name.as_str().to_string(),
                });
            }
        }

        let mut methods = Vec::new();
        for cap in method_re.captures_iter(code) {
            if let Some(method_name) = cap.get(1) {
                methods.push(method_name.as_str().to_string());
            }
        }

        Ok(ParsedScript {
            class_name,
            usings,
            fields,
            methods,
            raw_code: code.to_string(),
        })
    }

    /// 转换API调用
    fn convert_apis(
        &self,
        parsed: &ParsedScript,
        target: ScriptTarget,
    ) -> Result<ConvertedIntermediate, MigrationError> {
        let mut converted_code = parsed.raw_code.clone();
        let mut dependencies = Vec::new();

        // 应用API映射
        for (unity_api, mapping) in &self.api_mappings {
            let replacement = match target {
                ScriptTarget::Lua => format!("Engine.{}", mapping.engine_api),
                ScriptTarget::TypeScript => format!(
                    "import {{ {} }} from '@game-engine/{}';\n{}",
                    mapping.engine_api, mapping.engine_api, mapping.engine_api
                ),
            };

            converted_code = converted_code.replace(unity_api, &replacement);
        }

        // C#特定转换
        converted_code = converted_code.replace("GameObject", "Entity");
        converted_code = converted_code.replace("Transform", "Transform");
        converted_code = converted_code.Replace("Rigidbody", "RigidBody");
        converted_code = converted_code.replace("Vector3", "Vec3");
        converted_code = converted_code.replace("Quaternion", "Quat");

        // 类型转换
        converted_code = converted_code.replace("public class", "class");
        converted_code = converted_code.replace("public void", "function");
        converted_code = converted_code.replace("void ", "");
        converted_code = converted_code.Replace("private ", "");
        converted_code = converted_code.replace("protected ", "");

        Ok(ConvertedIntermediate {
            code: converted_code,
            dependencies,
        })
    }

    /// 生成Lua代码
    fn generate_lua(
        &self,
        converted: &ConvertedIntermediate,
        script_name: &str,
    ) -> Result<String, MigrationError> {
        let mut lua_code = String::new();

        // 添加引擎模块
        lua_code.push_str("local Engine = require('engine')\n\n");

        // Lua使用表模拟类
        lua_code.push_str(&format!("local {} = {{}}\n\n", script_name));

        // 添加构造函数
        lua_code.push_str(&format!("function {}.new()\n", script_name));
        lua_code.push_str(&format!(
            "    local self = setmetatable({{}}, {})\n",
            script_name
        ));
        lua_code.push_str(&format!("    return self\n"));
        lua_code.push_str("end\n\n");

        // 转换后的代码
        lua_code.push_str(&converted.code);

        lua_code.push_str(&format!("\nreturn {}", script_name));

        Ok(lua_code)
    }

    /// 生成TypeScript代码
    fn generate_typescript(
        &self,
        converted: &ConvertedIntermediate,
        script_name: &str,
    ) -> Result<String, MigrationError> {
        let mut ts_code = String::new();

        // 添加导入
        ts_code.push_str(
            "import { Engine, Entity, Transform, RigidBody } from '@game-engine/core';\n\n",
        );

        // 类声明
        ts_code.push_str(&format!("export class {} {{\n", script_name));

        // 字段和方法的转换代码
        ts_code.push_str(&converted.code);

        ts_code.push_str("}\n");

        Ok(ts_code)
    }

    /// 构建默认API映射
    fn build_default_mappings() -> HashMap<String, ApiMapping> {
        let mut mappings = HashMap::new();

        // GameObject API
        mappings.insert(
            "GameObject.Find".to_string(),
            ApiMapping {
                unity_api: "GameObject.Find".to_string(),
                engine_api: "find_entity".to_string(),
                conversion_rule: ConversionRule::MethodCall,
            },
        );

        mappings.insert(
            "Instantiate".to_string(),
            ApiMapping {
                unity_api: "Instantiate".to_string(),
                engine_api: "spawn_entity".to_string(),
                conversion_rule: ConversionRule::MethodCall,
            },
        );

        // Transform API
        mappings.insert(
            "transform.position".to_string(),
            ApiMapping {
                unity_api: "transform.position".to_string(),
                engine_api: "transform.position".to_string(),
                conversion_rule: ConversionRule::PropertyAccess,
            },
        );

        mappings.insert(
            "transform.rotation".to_string(),
            ApiMapping {
                unity_api: "transform.rotation".to_string(),
                engine_api: "transform.rotation".to_string(),
                conversion_rule: ConversionRule::PropertyAccess,
            },
        );

        // Rigidbody API
        mappings.insert(
            "GetComponent<Rigidbody>".to_string(),
            ApiMapping {
                unity_api: "GetComponent<Rigidbody>".to_string(),
                engine_api: "get_component<RigidBody>".to_string(),
                conversion_rule: ConversionRule::MethodCall,
            },
        );

        mappings.insert(
            "AddForce".to_string(),
            ApiMapping {
                unity_api: "AddForce".to_string(),
                engine_api: "apply_force".to_string(),
                conversion_rule: ConversionRule::MethodCall,
            },
        );

        // Time API
        mappings.insert(
            "Time.deltaTime".to_string(),
            ApiMapping {
                unity_api: "Time.deltaTime".to_string(),
                engine_api: "delta_time".to_string(),
                conversion_rule: ConversionRule::PropertyAccess,
            },
        );

        mappings
    }

    /// 报告进度
    fn report_progress(&self, completed: u32, total: u32, message: String) {
        if let Some(callback) = &self.progress_callback {
            let progress = MigrationProgress {
                total_steps: total,
                completed_steps: completed,
                current_phase: MigrationPhase::ConvertingScripts,
            };
            callback(progress);
        }
    }
}

#[cfg(feature = "regex")]
impl Default for UnityScriptConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// 脚本目标语言
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptTarget {
    Lua,
    TypeScript,
}

/// 脚本语言
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptLanguage {
    Lua,
    TypeScript,
    Python,
}

impl ScriptLanguage {
    pub fn extension(&self) -> &str {
        match self {
            ScriptLanguage::Lua => "lua",
            ScriptLanguage::TypeScript => "ts",
            ScriptLanguage::Python => "py",
        }
    }
}

/// 解析的脚本
#[derive(Debug, Clone)]
struct ParsedScript {
    class_name: String,
    usings: Vec<String>,
    fields: Vec<Field>,
    methods: Vec<String>,
    raw_code: String,
}

/// 字段
#[derive(Debug, Clone)]
struct Field {
    field_type: String,
    name: String,
}

/// 转换中间表示
#[derive(Debug, Clone)]
struct ConvertedIntermediate {
    code: String,
    dependencies: Vec<String>,
}

/// 转换后的脚本
#[derive(Debug, Clone)]
pub struct ConvertedScript {
    pub code: String,
    pub language: ScriptLanguage,
    pub dependencies: Vec<String>,
}

/// 转换结果
#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub success: bool,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_converter_creation() {
        let converter = UnityScriptConverter::new();
        assert!(!converter.api_mappings.is_empty());
    }

    #[test]
    fn test_api_mappings() {
        let converter = UnityScriptConverter::new();
        assert!(converter.api_mappings.contains_key("GameObject.Find"));
        assert!(converter.api_mappings.contains_key("Time.deltaTime"));
    }

    #[test]
    fn test_script_language_extension() {
        assert_eq!(ScriptLanguage::Lua.extension(), "lua");
        assert_eq!(ScriptLanguage::TypeScript.extension(), "ts");
        assert_eq!(ScriptLanguage::Python.extension(), "py");
    }

    #[test]
    fn test_parse_csharp() {
        let converter = UnityScriptConverter::new();
        let csharp_code = r#"
using UnityEngine;

public class Player : MonoBehaviour {
    public float speed = 5.0f;
    private Rigidbody rb;

    void Start() {
        rb = GetComponent<Rigidbody>();
    }

    void Update() {
        float moveHorizontal = Input.GetAxis("Horizontal");
        float moveVertical = Input.GetAxis("Vertical");
        Vector3 movement = new Vector3(moveHorizontal, 0.0f, moveVertical);
        rb.AddForce(movement * speed);
    }
}
"#;

        let parsed = converter.parse_csharp(csharp_code);
        assert!(parsed.is_ok());

        let parsed = parsed.unwrap();
        assert_eq!(parsed.class_name, "Player");
        assert!(parsed.usings.contains(&"UnityEngine".to_string()));
        assert!(parsed.methods.contains(&"Start".to_string()));
        assert!(parsed.methods.contains(&"Update".to_string()));
    }
}
