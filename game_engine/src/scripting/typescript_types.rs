// TypeScript类型定义生成器
//
// 自动生成TypeScript类型定义文件(.d.ts)，提供完整的类型提示和智能感知

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// 类型定义生成器
#[derive(Debug)]
pub struct TypeScriptDefinitions {
    /// 类型定义
    definitions: HashMap<String, TypeDefinition>,
    /// 接口定义
    interfaces: HashMap<String, InterfaceDefinition>,
    /// 类定义
    classes: HashMap<String, ClassDefinition>,
    /// 函数定义
    functions: HashMap<String, FunctionDefinition>,
}

impl Default for TypeScriptDefinitions {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeScriptDefinitions {
    /// 创建新的类型定义生成器
    pub fn new() -> Self {
        let mut defs = Self {
            definitions: HashMap::new(),
            interfaces: HashMap::new(),
            classes: HashMap::new(),
            functions: HashMap::new(),
        };

        // 添加基础类型定义
        defs.add_primitive_types();
        defs.add_engine_types();

        defs
    }

    /// 添加基础类型
    fn add_primitive_types(&mut self) {
        // 基础类型别名
        self.definitions.insert("Vector3".to_string(), TypeDefinition::Interface {
            name: "Vector3".to_string(),
            description: "3D向量".to_string(),
            properties: vec![
                PropertyDefinition {
                    name: "x".to_string(),
                    type_name: "number".to_string(),
                    description: "X分量".to_string(),
                    optional: false,
                },
                PropertyDefinition {
                    name: "y".to_string(),
                    type_name: "number".to_string(),
                    description: "Y分量".to_string(),
                    optional: false,
                },
                PropertyDefinition {
                    name: "z".to_string(),
                    type_name: "number".to_string(),
                    description: "Z分量".to_string(),
                    optional: false,
                },
            ],
        });

        self.definitions.insert("Vector2".to_string(), TypeDefinition::Interface {
            name: "Vector2".to_string(),
            description: "2D向量".to_string(),
            properties: vec![
                PropertyDefinition {
                    name: "x".to_string(),
                    type_name: "number".to_string(),
                    description: "X分量".to_string(),
                    optional: false,
                },
                PropertyDefinition {
                    name: "y".to_string(),
                    type_name: "number".to_string(),
                    description: "Y分量".to_string(),
                    optional: false,
                },
            ],
        });

        self.definitions.insert("Quaternion".to_string(), TypeDefinition::Interface {
            name: "Quaternion".to_string(),
            description: "四元数（用于旋转）".to_string(),
            properties: vec![
                PropertyDefinition {
                    name: "x".to_string(),
                    type_name: "number".to_string(),
                    description: "X分量".to_string(),
                    optional: false,
                },
                PropertyDefinition {
                    name: "y".to_string(),
                    type_name: "number".to_string(),
                    description: "Y分量".to_string(),
                    optional: false,
                },
                PropertyDefinition {
                    name: "z".to_string(),
                    type_name: "number".to_string(),
                    description: "Z分量".to_string(),
                    optional: false,
                },
                PropertyDefinition {
                    name: "w".to_string(),
                    type_name: "number".to_string(),
                    description: "W分量".to_string(),
                    optional: false,
                },
            ],
        });

        self.definitions.insert("Color".to_string(), TypeDefinition::Interface {
            name: "Color".to_string(),
            description: "颜色".to_string(),
            properties: vec![
                PropertyDefinition {
                    name: "r".to_string(),
                    type_name: "number".to_string(),
                    description: "红色分量 (0-1)".to_string(),
                    optional: false,
                },
                PropertyDefinition {
                    name: "g".to_string(),
                    type_name: "number".to_string(),
                    description: "绿色分量 (0-1)".to_string(),
                    optional: false,
                },
                PropertyDefinition {
                    name: "b".to_string(),
                    type_name: "number".to_string(),
                    description: "蓝色分量 (0-1)".to_string(),
                    optional: false,
                },
                PropertyDefinition {
                    name: "a".to_string(),
                    type_name: "number".to_string(),
                    description: "透明度分量 (0-1)".to_string(),
                    optional: true,
                },
            ],
        });
    }

    /// 添加引擎类型
    fn add_engine_types(&mut self) {
        // Entity类
        self.classes.insert("Entity".to_string(), ClassDefinition {
            name: "Entity".to_string(),
            description: "游戏实体".to_string(),
            properties: vec![
                PropertyDefinition {
                    name: "id".to_string(),
                    type_name: "number".to_string(),
                    description: "实体ID".to_string(),
                    optional: false,
                },
                PropertyDefinition {
                    name: "name".to_string(),
                    type_name: "string".to_string(),
                    description: "实体名称".to_string(),
                    optional: true,
                },
                PropertyDefinition {
                    name: "active".to_string(),
                    type_name: "boolean".to_string(),
                    description: "是否激活".to_string(),
                    optional: false,
                },
            ],
            methods: vec![
                MethodDefinition {
                    name: "setPosition".to_string(),
                    description: "设置位置".to_string(),
                    parameters: vec![
                        ParameterDefinition {
                            name: "position".to_string(),
                            type_name: "Vector3".to_string(),
                            description: "位置向量".to_string(),
                            optional: false,
                        },
                    ],
                    return_type: "void".to_string(),
                },
                MethodDefinition {
                    name: "getPosition".to_string(),
                    description: "获取位置".to_string(),
                    parameters: vec![],
                    return_type: "Vector3".to_string(),
                },
                MethodDefinition {
                    name: "setRotation".to_string(),
                    description: "设置旋转".to_string(),
                    parameters: vec![
                        ParameterDefinition {
                            name: "rotation".to_string(),
                            type_name: "Quaternion".to_string(),
                            description: "旋转四元数".to_string(),
                            optional: false,
                        },
                    ],
                    return_type: "void".to_string(),
                },
                MethodDefinition {
                    name: "getRotation".to_string(),
                    description: "获取旋转".to_string(),
                    parameters: vec![],
                    return_type: "Quaternion".to_string(),
                },
                MethodDefinition {
                    name: "addComponent".to_string(),
                    description: "添加组件".to_string(),
                    parameters: vec![
                        ParameterDefinition {
                            name: "component".to_string(),
                            type_name: "Component".to_string(),
                            description: "组件对象".to_string(),
                            optional: false,
                        },
                    ],
                    return_type: "void".to_string(),
                },
                MethodDefinition {
                    name: "getComponent".to_string(),
                    description: "获取组件".to_string(),
                    parameters: vec![
                        ParameterDefinition {
                            name: "type".to_string(),
                            type_name: "string".to_string(),
                            description: "组件类型".to_string(),
                            optional: false,
                        },
                    ],
                    return_type: "Component | null".to_string(),
                },
                MethodDefinition {
                    name: "destroy".to_string(),
                    description: "销毁实体".to_string(),
                    parameters: vec![],
                    return_type: "void".to_string(),
                },
            ],
        });

        // Component接口
        self.interfaces.insert("Component".to_string(), InterfaceDefinition {
            name: "Component".to_string(),
            description: "组件基类接口".to_string(),
            properties: vec![
                PropertyDefinition {
                    name: "enabled".to_string(),
                    type_name: "boolean".to_string(),
                    description: "是否启用".to_string(),
                    optional: false,
                },
            ],
            methods: vec![
                MethodDefinition {
                    name: "onStart".to_string(),
                    description: "组件启动时调用".to_string(),
                    parameters: vec![],
                    return_type: "void".to_string(),
                },
                MethodDefinition {
                    name: "onUpdate".to_string(),
                    description: "每帧更新时调用".to_string(),
                    parameters: vec![
                        ParameterDefinition {
                            name: "deltaTime".to_string(),
                            type_name: "number".to_string(),
                            description: "时间增量".to_string(),
                            optional: false,
                        },
                    ],
                    return_type: "void".to_string(),
                },
                MethodDefinition {
                    name: "onDestroy".to_string(),
                    description: "组件销毁时调用".to_string(),
                    parameters: vec![],
                    return_type: "void".to_string(),
                },
            ],
        });

        // Engine全局对象
        self.interfaces.insert("EngineAPI".to_string(), InterfaceDefinition {
            name: "EngineAPI".to_string(),
            description: "引擎API".to_string(),
            properties: vec![
                PropertyDefinition {
                    name: "version".to_string(),
                    type_name: "string".to_string(),
                    description: "引擎版本".to_string(),
                    optional: false,
                },
            ],
            methods: vec![
                MethodDefinition {
                    name: "spawnEntity".to_string(),
                    description: "创建新实体".to_string(),
                    parameters: vec![],
                    return_type: "Entity".to_string(),
                },
                MethodDefinition {
                    name: "findEntity".to_string(),
                    description: "查找实体".to_string(),
                    parameters: vec![
                        ParameterDefinition {
                            name: "name".to_string(),
                            type_name: "string".to_string(),
                            description: "实体名称".to_string(),
                            optional: false,
                        },
                    ],
                    return_type: "Entity | null".to_string(),
                },
                MethodDefinition {
                    name: "log".to_string(),
                    description: "输出日志".to_string(),
                    parameters: vec![
                        ParameterDefinition {
                            name: "message".to_string(),
                            type_name: "string".to_string(),
                            description: "日志消息".to_string(),
                            optional: false,
                        },
                    ],
                    return_type: "void".to_string(),
                },
                MethodDefinition {
                    name: "time".to_string(),
                    description: "获取当前时间戳".to_string(),
                    parameters: vec![],
                    return_type: "number".to_string(),
                },
            ],
        });
    }

    /// 生成TypeScript类型定义文件
    pub fn generate(&self) -> String {
        let mut output = String::new();

        // 文件头
        output.push_str("// Type definitions for Game Engine\n");
        output.push_str("// Generated automatically - do not edit\n\n");

        // 声明文件
        output.push_str("declare namespace GameEngine {\n");

        // 生成接口定义
        for (_, interface) in &self.interfaces {
            output.push_str(&self.generate_interface(interface, 1));
        }

        // 生成类定义
        for (_, class) in &self.classes {
            output.push_str(&self.generate_class(class, 1));
        }

        output.push_str("}\n");

        // 全局Engine对象
        output.push_str("\n// Global Engine object\n");
        output.push_str("declare const Engine: GameEngine.EngineAPI;\n");
        output.push_str("declare class Entity implements GameEngine.Component {\n");
        output.push_str("    constructor(id: number);\n");
        output.push_str("    id: number;\n");
        output.push_str("    name?: string;\n");
        output.push_str("    active: boolean;\n");
        output.push_str("    setPosition(position: GameEngine.Vector3): void;\n");
        output.push_str("    getPosition(): GameEngine.Vector3;\n");
        output.push_str("    setRotation(rotation: GameEngine.Quaternion): void;\n");
        output.push_str("    getRotation(): GameEngine.Quaternion;\n");
        output.push_str("    addComponent(component: GameEngine.Component): void;\n");
        output.push_str("    getComponent(type: string): GameEngine.Component | null;\n");
        output.push_str("    destroy(): void;\n");
        output.push_str("}\n");

        output
    }

    /// 生成接口定义
    fn generate_interface(&self, interface: &InterfaceDefinition, indent: usize) -> String {
        let indent_str = "    ".repeat(indent);
        let mut output = String::new();

        output.push_str(&format!("{}// {}\n", indent_str, interface.description));
        output.push_str(&format!("{}export interface {} {{\n", indent_str, interface.name));

        // 属性
        for prop in &interface.properties {
            output.push_str(&format!("{}{}{}: {}{};\n",
                indent_str,
                "    ",
                prop.name,
                prop.type_name,
                if prop.optional { "?" } else { "" }
            ));
        }

        // 方法
        for method in &interface.methods {
            output.push_str(&format!("{}{}{}({}): {};\n",
                indent_str,
                "    ",
                method.name,
                self.generate_parameters(&method.parameters),
                method.return_type
            ));
        }

        output.push_str(&format!("{}}}\n\n", indent_str));

        output
    }

    /// 生成类定义
    fn generate_class(&self, class: &ClassDefinition, indent: usize) -> String {
        let indent_str = "    ".repeat(indent);
        let mut output = String::new();

        output.push_str(&format!("{}// {}\n", indent_str, class.description));
        output.push_str(&format!("{}export class {} {{\n", indent_str, class.name));

        // 属性
        for prop in &class.properties {
            output.push_str(&format!("{}{}{}: {}{};\n",
                indent_str,
                "    ",
                prop.name,
                prop.type_name,
                if prop.optional { "?" } else { "" }
            ));
        }

        // 构造函数
        output.push_str(&format!("{}{}constructor(id: number);\n", indent_str, "    "));

        // 方法
        for method in &class.methods {
            output.push_str(&format!("{}{}{}({}): {};\n",
                indent_str,
                "    ",
                method.name,
                self.generate_parameters(&method.parameters),
                method.return_type
            ));
        }

        output.push_str(&format!("{}}}\n\n", indent_str));

        output
    }

    /// 生成参数列表
    fn generate_parameters(&self, parameters: &[ParameterDefinition]) -> String {
        if parameters.is_empty() {
            return String::new();
        }

        let params: Vec<String> = parameters
            .iter()
            .map(|p| {
                format!("{}: {}{}",
                    p.name,
                    p.type_name,
                    if p.optional { "?" } else { "" }
                )
            })
            .collect();

        params.join(", ")
    }

    /// 写入类型定义文件
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let definitions = self.generate();
        let mut file = File::create(path)?;
        file.write_all(definitions.as_bytes())?;
        Ok(())
    }
}

// ============================================================================
// 类型定义结构
// ============================================================================

/// 类型定义
#[derive(Debug, Clone)]
pub enum TypeDefinition {
    Interface(InterfaceDefinition),
}

/// 接口定义
#[derive(Debug, Clone)]
pub struct InterfaceDefinition {
    pub name: String,
    pub description: String,
    pub properties: Vec<PropertyDefinition>,
    pub methods: Vec<MethodDefinition>,
}

/// 类定义
#[derive(Debug, Clone)]
pub struct ClassDefinition {
    pub name: String,
    pub description: String,
    pub properties: Vec<PropertyDefinition>,
    pub methods: Vec<MethodDefinition>,
}

/// 函数定义
#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterDefinition>,
    pub return_type: String,
}

/// 属性定义
#[derive(Debug, Clone)]
pub struct PropertyDefinition {
    pub name: String,
    pub type_name: String,
    pub description: String,
    pub optional: bool,
}

/// 方法定义
#[derive(Debug, Clone)]
pub struct MethodDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterDefinition>,
    pub return_type: String,
}

/// 参数定义
#[derive(Debug, Clone)]
pub struct ParameterDefinition {
    pub name: String,
    pub type_name: String,
    pub description: String,
    pub optional: bool,
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_definitions() {
        let defs = TypeScriptDefinitions::new();
        let output = defs.generate();

        assert!(output.contains("interface Entity"));
        assert!(output.contains("interface Component"));
        assert!(output.contains("interface EngineAPI"));
        assert!(output.contains("class Entity"));
    }

    #[test]
    fn test_generate_interface() {
        let defs = TypeScriptDefinitions::new();
        let interface = defs.interfaces.get("Component").unwrap();

        let output = defs.generate_interface(interface, 1);
        assert!(output.contains("export interface Component"));
        assert!(output.contains("enabled: boolean"));
        assert!(output.contains("onStart"));
        assert!(output.contains("onUpdate"));
    }

    #[test]
    fn test_generate_class() {
        let defs = TypeScriptDefinitions::new();
        let class = defs.classes.get("Entity").unwrap();

        let output = defs.generate_class(class, 1);
        assert!(output.contains("export class Entity"));
        assert!(output.contains("id: number"));
        assert!(output.contains("setPosition"));
    }

    #[test]
    fn test_write_to_file() {
        let defs = TypeScriptDefinitions::new();
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("game_engine.d.ts");

        let result = defs.write_to_file(&file_path);
        assert!(result.is_ok());

        // 验证文件存在
        assert!(file_path.exists());

        // 清理
        let _ = std::fs::remove_file(&file_path);
    }
}
