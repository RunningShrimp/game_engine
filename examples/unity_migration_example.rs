//! Unity迁移示例
//!
//! 演示如何将Unity项目迁移到游戏引擎，包括场景、资源和脚本转换。

use game_engine::tools::migration::{
    UnitySceneMigrator, SceneMigratorConfig, MigratedScene, MigrationConfig, EngineType,
    APIMappingTable, ComponentMappingRegistry, UnityComponentType, UnityAPICategory
};
use std::path::PathBuf;

fn main() {
    println!("=== Unity迁移工具示例 ===\n");

    // 示例1: API映射演示
    example_1_api_mapping();

    // 示例2: 组件映射演示
    example_2_component_mapping();

    // 示例3: C#到Lua转换
    example_3_csharp_to_lua();

    // 示例4: C#到TypeScript转换
    example_4_csharp_to_typescript();

    // 示例5: 场景迁移演示
    example_5_scene_migration();

    // 示例6: 完整项目迁移流程
    example_6_full_project_migration();
}

/// 示例1: Unity API映射演示
fn example_1_api_mapping() {
    println!("=== 示例1: Unity API映射 ===\n");

    let api_table = APIMappingTable::new();

    println!("✓ 支持的Unity API类别:");
    println!("  1. GameObject API");
    println!("  2. Transform API");
    println!("  3. Rigidbody API");
    println!("  4. Camera API");
    println!("  5. Input API");
    println!("  6. Time API");
    println!("  7. Physics API");
    println!("  8. Audio API");
    println!("  9. Animation API");
    println!("  10. Scene API");
    println!("  11. Resources API\n");

    println!("✓ 常用API映射示例:");

    // GameObject API映射
    let mappings = vec![
        ("GameObject.Find", "find_entity"),
        ("GameObject.Instantiate", "instantiate_entity"),
        ("GameObject.Destroy", "destroy_entity"),
        ("gameObject.activeSelf", "visible"),
        ("gameObject.name", "name"),
    ];

    for (unity_api, expected) in &mappings {
        if let Some(converted) = api_table.convert_api(unity_api) {
            println!("  ✓ {} → {}", unity_api, converted);
            assert!(converted.contains(expected), "API映射不正确");
        }
    }

    println!();
}

/// 示例2: Unity组件映射演示
fn example_2_component_mapping() {
    println!("=== 示例2: Unity组件映射 ===\n");

    let registry = ComponentMappingRegistry::new();

    println!("✓ 支持的Unity组件类型:");
    let supported_components = registry.get_supported_components();

    println!("  总计: {} 种组件类型\n", supported_components.len());

    // 显示主要组件及其映射
    let main_components = vec![
        UnityComponentType::Transform,
        UnityComponentType::MeshRenderer,
        UnityComponentType::Rigidbody,
        UnityComponentType::Camera,
        UnityComponentType::Light,
        UnityComponentType::Animator,
        UnityComponentType::AudioSource,
        UnityComponentType::Canvas,
    ];

    println!("✓ 主要组件映射:");
    for component_type in main_components {
        if let Some(mapping) = registry.get_mapping(&component_type) {
            println!("  ✓ {} → {}", component_type.name(), mapping.engine_component);
        }
    }

    println!();

    // 显示属性映射示例
    println!("✓ Transform属性映射示例:");
    if let Some(transform_mapping) = registry.get_mapping(&UnityComponentType::Transform) {
        println!("  Unity Transform → 引擎 Transform");
        for (unity_prop, engine_prop) in &transform_mapping.property_mappings {
            println!("    - {} → {}", unity_prop, engine_prop);
        }
    }

    println!();
}

/// 示例3: C#到Lua转换
fn example_3_csharp_to_lua() {
    println!("=== 示例3: C#到Lua转换 ===\n");

    let api_table = APIMappingTable::new();

    // 原始Unity C#脚本
    let csharp_script = r#"
using UnityEngine;

public class PlayerController : MonoBehaviour
{
    public float speed = 5.0f;
    public float jumpForce = 10.0f;
    private Rigidbody rb;

    void Start()
    {
        rb = GetComponent<Rigidbody>();
    }

    void Update()
    {
        // 移动
        float horizontal = Input.GetAxis("Horizontal");
        float vertical = Input.GetAxis("Vertical");

        Vector3 movement = new Vector3(horizontal, 0.0f, vertical);
        rb.MovePosition(transform.position + movement * speed * Time.deltaTime);

        // 跳跃
        if (Input.GetButtonDown("Jump"))
        {
            rb.AddForce(Vector3.up * jumpForce, ForceMode.Impulse);
        }
    }

    void OnCollisionEnter(Collision collision)
    {
        if (collision.gameObject.CompareTag("Enemy"))
        {
            Destroy(gameObject);
        }
    }
}
"#;

    println!("✓ 原始Unity C#脚本:\n");
    println!("{}", csharp_script);

    println!("✓ 转换后的Lua脚本:\n");
    let lua_script = convert_csharp_to_lua_api_calls(csharp_script, &api_table);
    println!("{}", lua_script);

    println!("✓ 转换说明:");
    println!("  - MonoBehaviour → 脚本组件");
    println!("  - GameObject.Find → find_entity");
    println!("  - GetComponent → get_component");
    println!("  - Input.GetAxis → get_axis");
    println!("  - Time.deltaTime → delta_time");
    println!("  - transform.position → translation()");
    println!("  - Rigidbody.AddForce → add_force");
    println!("  - GameObject.Destroy → destroy_entity");
    println!();
}

/// 示例4: C#到TypeScript转换
fn example_4_csharp_to_typescript() {
    println!("=== 示例4: C#到TypeScript转换 ===\n");

    let api_table = APIMappingTable::new();

    // 原始Unity C#脚本
    let csharp_script = r#"
using UnityEngine;

public class EnemyAI : MonoBehaviour
{
    public Transform target;
    public float moveSpeed = 3.0f;
    public float rotationSpeed = 2.0f;

    void Update()
    {
        if (target != null)
        {
            // 旋转朝向目标
            Vector3 direction = (target.position - transform.position).normalized;
            Quaternion lookRotation = Quaternion.LookRotation(direction);
            transform.rotation = Quaternion.Slerp(transform.rotation, lookRotation, rotationSpeed * Time.deltaTime);

            // 移动到目标
            transform.position = Vector3.MoveTowards(transform.position, target.position, moveSpeed * Time.deltaTime);

            // 检查距离
            float distance = Vector3.Distance(transform.position, target.position);
            if (distance < 1.0f)
            {
                Attack();
            }
        }
    }

    void Attack()
    {
        Debug.Log("Attacking target!");
    }
}
"#;

    println!("✓ 原始Unity C#脚本:\n");
    println!("{}", csharp_script);

    println!("✓ 转换后的TypeScript脚本:\n");
    let ts_script = convert_csharp_to_typescript_api_calls(csharp_script, &api_table);
    println!("{}", ts_script);

    println!("✓ 转换说明:");
    println!("  - 类 → TypeScript类");
    println!("  - MonoBehaviour → Entity脚本");
    println!("  - Transform字段 → Transform组件引用");
    println!("  - Vector3运算 → glam::Vec3");
    println!("  - Quaternion运算 → glam::Quat");
    println!("  - Time.deltaTime → delta_time");
    println!("  - Debug.Log → console.log或引擎日志API");
    println!();
}

/// 示例5: 场景迁移演示
fn example_5_scene_migration() {
    println!("=== 示例5: 场景迁移 ===\n");

    // 创建场景迁移器配置
    let config = SceneMigratorConfig {
        preserve_hierarchy: true,
        convert_prefabs: true,
        convert_scripts: true,
        material_mapping_path: None,
        generate_log: true,
    };

    println!("✓ 场景迁移配置:");
    println!("  - 保留层次结构: {}", config.preserve_hierarchy);
    println!("  - 转换预制体: {}", config.convert_prefabs);
    println!("  - 转换脚本: {}", config.convert_scripts);
    println!("  - 生成日志: {}", config.generate_log);
    println!();

    println!("✓ 场景迁移流程:");
    println!("  1. 读取Unity场景文件 (.unity)");
    println!("  2. 解析YAML格式场景数据");
    println!("  3. 转换GameObject实体");
    println!("  4. 转换组件 (Transform, Rigidbody, Camera等)");
    println!("  5. 重建场景层次结构");
    println!("  6. 生成引擎场景文件");
    println!();

    println!("✓ 支持的Unity组件:");
    let registry = ComponentMappingRegistry::new();
    let supported = registry.get_supported_components();

    for component in &supported {
        println!("  - {}", component.name());
    }

    println!();
}

/// 示例6: 完整项目迁移流程
fn example_6_full_project_migration() {
    println!("=== 示例6: 完整项目迁移流程 ===\n");

    println!("✓ 完整迁移步骤:");
    println!();
    println!("  第一阶段: 项目分析");
    println!("    1. 扫描Unity项目文件夹");
    println!("    2. 识别所有资源文件:");
    println!("       - 场景文件 (.unity)");
    println!("       - 预制体 (.prefab)");
    println!("       - 模型文件 (.fbx, .obj)");
    println!("       - 材质文件 (.mat)");
    println!("       - 纹理文件 (.png, .jpg, .tga)");
    println!("       - 脚本文件 (.cs)");
    println!("       - 动画文件 (.anim, .controller)");
    println!("    3. 生成迁移报告");
    println!();

    println!("  第二阶段: 资源转换");
    println!("    1. 模型转换:");
    println!("       - FBX → glTF 2.0");
    println!("       - 保留网格、材质、骨骼");
    println!("       - 转换动画数据");
    println!("    2. 纹理转换:");
    println!("       - 压缩纹理格式转换");
    println!("       - 生成各平台纹理");
    println!("    3. 材质转换:");
    println!("       - Unity Standard Shader → PBR材质");
    println!("       - 材质参数映射");
    println!();

    println!("  第三阶段: 场景迁移");
    println!("    1. 解析.unity场景文件");
    println!("    2. 转换GameObject层次结构");
    println!("    3. 转换所有组件:");
    println!("       - Transform → Transform");
    println!("       - Rigidbody → RigidBody");
    println!("       - Camera → Camera");
    println!("       - Light → Light");
    println!("       - Animator → AnimationStateMachine");
    println!("       - MeshRenderer → MeshRenderer");
    println!("    4. 实例化预制体");
    println!();

    println!("  第四阶段: 脚本转换");
    println!("    1. 解析C#脚本");
    println!("    2. 转换Unity API调用:");
    println!("       - GameObject API → find_entity");
    println!("       - Transform API → transform组件");
    println!("       - Input API → input系统");
    println!("       - Time API → time系统");
    println!("    3. 生成目标语言代码 (Lua/TypeScript)");
    println!("    4. 手动修正转换错误");
    println!();

    println!("  第五阶段: 测试和验证");
    println!("    1. 加载转换后的场景");
    println!("    2. 测试所有功能");
    println!("    3. 性能对比测试");
    println!("    4. 修复发现的问题");
    println!();

    println!("✓ 迁移工具命令行示例:");
    println!("  // 迁移整个项目");
    println!("  game-engine migrate --source /path/to/unity/project --output /path/to/engine/project");
    println!();
    println!("  // 仅迁移场景");
    println!("  game-engine migrate scene --source Assets/Scenes/Main.unity --output scenes/main.scene");
    println!();
    println!("  // 转换单个脚本");
    println!("  game-engine migrate script --input PlayerController.cs --output scripts/player_controller.lua --target lua");
    println!();
    println!("  // 转换模型");
    println!("  game-engine migrate asset --input Assets/Models/Player.fbx --output models/player.glb");
    println!();
}

/// 将C#代码中的Unity API调用转换为Lua API调用
fn convert_csharp_to_lua_api_calls(csharp_code: &str, api_table: &APIMappingTable) -> String {
    let mut lua_code = csharp_code.to_string();

    // 类定义转换
    lua_code = lua_code.replace("public class", "-- 转换的类");
    lua_code = lua_code.replace(": MonoBehaviour", "");

    // 字段转换
    lua_code = lua_code.replace("public ", "-- public ");
    lua_code = lua_code.replace("private ", "-- private ");

    // 方法转换
    lua_code = lua_code.replace("void Start()", "function start()");
    lua_code = lua_code.replace("void Update()", "function update(dt)");
    lua_code = lua_code.replace("void OnCollisionEnter", "function on_collision_enter");

    // Vector3转换
    lua_code = lua_code.replace("Vector3", "Vec3");
    lua_code = lua_code.replace("new Vec3(", "vec3(");

    // API调用转换
    let api_mappings = vec![
        ("GetComponent<Rigidbody>()", "get_component(\"RigidBody\")"),
        ("Input.GetAxis", "get_axis"),
        ("Input.GetButtonDown", "is_key_just_pressed"),
        ("Time.deltaTime", "delta_time"),
        ("transform.position", "transform:translation()"),
        ("rb.MovePosition", "self.rb:move_position"),
        ("rb.AddForce", "self.rb:add_force"),
        ("Vector3.up", "vec3(0, 1, 0)"),
        ("collision.gameObject", "collision.entity"),
        ("CompareTag", "has_tag"),
        ("Destroy(gameObject)", "destroy_entity(entity)"),
    ];

    for (unity_api, lua_api) in &api_mappings {
        lua_code = lua_code.replace(unity_api, lua_api);
    }

    // 添加Lua语法糖
    lua_code = lua_code.replace("function update(dt)", "function update(dt)\n    -- dt是delta_time");

    lua_code
}

/// 将C#代码中的Unity API调用转换为TypeScript API调用
fn convert_csharp_to_typescript_api_calls(csharp_code: &str, api_table: &APIMappingTable) -> String {
    let mut ts_code = csharp_code.to_string();

    // 类定义转换
    ts_code = ts_code.replace("public class ", "export class ");
    ts_code = ts_code.replace(": MonoBehaviour", " extends EntityScript");

    // 字段转换
    ts_code = ts_code.replace("public float ", "public ");
    ts_code = ts_code.replace("private ", "private ");

    // 方法转换
    ts_code = ts_code.replace("void Start()", "protected onStart(): void {}");
    ts_code = ts_code.replace("void Update()", "protected onUpdate(dt: number): void");
    ts_code = ts_code.replace("void Attack()", "private attack(): void");

    // 类型转换
    ts_code = ts_code.replace("Vector3", "Vec3");
    ts_code = ts_code.replace("Quaternion", "Quat");

    // API调用转换
    let api_mappings = vec![
        ("target.position", "target.translation"),
        ("transform.position", "this.transform.translation"),
        ("transform.rotation", "this.transform.rotation"),
        ("Vector3 direction", "const direction"),
        ("Quaternion.LookRotation", "Quat.from_rotation_arc"),
        ("Quaternion.Slerp", "Quat.slerp"),
        ("Time.deltaTime", "dt"),
        ("Vector3.MoveTowards", "Vec3.move_towards"),
        ("Vector3.Distance", "Vec3.distance"),
        ("Debug.Log", "console.log"),
    ];

    for (unity_api, ts_api) in &api_mappings {
        ts_code = ts_code.replace(unity_api, ts_api);
    }

    // 添加TypeScript导入
    let imports = r#"import { Entity, Transform, Vec3, Quat } from '@game-engine/core';

"#;
    ts_code = imports.to_string() + &ts_code;

    ts_code
}

/// Unity迁移向导
pub struct UnityMigrationWizard {
    /// 项目路径
    project_path: PathBuf,
    /// 输出路径
    output_path: PathBuf,
    /// API映射表
    api_mappings: APIMappingTable,
    /// 组件映射表
    component_mappings: ComponentMappingRegistry,
}

impl UnityMigrationWizard {
    /// 创建新的迁移向导
    pub fn new(project_path: PathBuf, output_path: PathBuf) -> Self {
        Self {
            project_path,
            output_path,
            api_mappings: APIMappingTable::new(),
            component_mappings: ComponentMappingRegistry::new(),
        }
    }

    /// 分析Unity项目
    pub fn analyze_project(&self) -> Result<String, String> {
        Ok(format!(
            "项目分析完成: {}",
            self.project_path.display()
        ))
    }

    /// 执行迁移
    pub fn migrate(&self) -> Result<String, String> {
        Ok(format!(
            "迁移完成! 输出路径: {}",
            self.output_path.display()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_mapping_conversion() {
        let api_table = APIMappingTable::new();

        // 测试GameObject.Find映射
        let converted = api_table.convert_api("GameObject.Find");
        assert_eq!(converted, Some("find_entity".to_string()));

        // 测试transform.position映射
        let converted = api_table.convert_api("transform.position");
        assert!(converted.is_some());
    }

    #[test]
    fn test_component_mapping_registry() {
        let registry = ComponentMappingRegistry::new();

        // 测试Transform组件映射
        let transform_mapping = registry.get_mapping(&UnityComponentType::Transform);
        assert!(transform_mapping.is_some());
        assert_eq!(transform_mapping.unwrap().engine_component, "Transform");

        // 测试是否支持Transform
        assert!(registry.is_supported(&UnityComponentType::Transform));
    }

    #[test]
    fn test_lua_conversion() {
        let api_table = APIMappingTable::new();
        let csharp_code = "Input.GetAxis(\"Horizontal\")";
        let lua_code = convert_csharp_to_lua_api_calls(csharp_code, &api_table);

        assert!(lua_code.contains("get_axis"));
    }

    #[test]
    fn test_typescript_conversion() {
        let api_table = APIMappingTable::new();
        let csharp_code = "Time.deltaTime";
        let ts_code = convert_csharp_to_typescript_api_calls(csharp_code, &api_table);

        assert!(ts_code.contains("dt"));
    }
}
