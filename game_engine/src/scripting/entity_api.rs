// 实体API - 提供脚本系统与ECS的完整实体操作接口
//
// 本模块为脚本系统提供全面的实体和组件管理功能，包括：
// - 实体创建与销毁
// - 组件添加、移除、查询和修改
// - 高级实体查询
// - 实体模板系统
// - 批量操作支持

use crate::ecs::*;
use crate::error::lock_safety::LockError;
use crate::error::safe_lock;
use crate::scripting::system::{ScriptContext, ScriptLanguage, ScriptResult, ScriptValue};
use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 实体API - 提供脚本友好的实体操作接口
///
/// # 功能
///
/// - **实体创建**: 创建新实体，支持模板系统
/// - **实体查询**: 按组件、名称、ID查询实体
/// - **组件管理**: 添加、移除、获取、修改组件
/// - **批量操作**: 支持批量组件操作
/// - **链式查询**: 提供流畅的查询API
///
/// # 线程安全
///
/// 所有实体操作都通过Arc<Mutex<World>>进行，确保线程安全。
pub struct EntityApi {
    /// ECS World引用
    world: Arc<Mutex<World>>,
    /// 实体模板存储
    templates: HashMap<String, EntityTemplate>,
    /// 实体名称映射
    entity_names: Arc<Mutex<HashMap<String, Entity>>>,
}

/// 实体模板定义
///
/// 模板允许预定义实体结构，包括组件和初始值。
#[derive(Clone, Debug)]
pub struct EntityTemplate {
    /// 模板名称
    pub name: String,
    /// 包含的组件列表
    pub components: Vec<TemplateComponent>,
}

/// 模板组件定义
#[derive(Clone, Debug)]
pub enum TemplateComponent {
    /// Transform组件
    Transform {
        position: [f32; 3],
        rotation: [f32; 4],
        scale: [f32; 3],
    },
    /// Sprite组件
    Sprite {
        color: [f32; 4],
        tex_index: u32,
        layer: f32,
    },
    /// Velocity组件
    Velocity { linear: [f32; 3], angular: [f32; 3] },
    /// 自定义组件 (使用ScriptValue存储)
    Custom { name: String, data: ScriptValue },
}

impl EntityApi {
    /// 辅助函数：转换LockError为String
    fn lock_result<T>(result: Result<T, LockError>) -> Result<T, String> {
        result.map_err(|e| e.to_string())
    }

    /// 创建新的实体API
    ///
    /// # 参数
    ///
    /// - `world`: ECS World的线程安全引用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use bevy_ecs::prelude::*;
    /// use std::sync::{Arc, Mutex};
    /// use game_engine::scripting::entity_api::EntityApi;
    ///
    /// let world = World::new();
    /// let entity_api = EntityApi::new(Arc::new(Mutex::new(world)));
    /// ```
    pub fn new(world: Arc<Mutex<World>>) -> Self {
        let mut api = Self {
            world,
            templates: HashMap::new(),
            entity_names: Arc::new(Mutex::new(HashMap::new())),
        };

        // 注册内置模板
        api.register_builtin_templates();

        api
    }

    /// 注册内置实体模板
    fn register_builtin_templates(&mut self) {
        // 敌人模板
        self.templates.insert(
            "Enemy".to_string(),
            EntityTemplate {
                name: "Enemy".to_string(),
                components: vec![
                    TemplateComponent::Transform {
                        position: [0.0, 0.0, 0.0],
                        rotation: [0.0, 0.0, 0.0, 1.0],
                        scale: [1.0, 1.0, 1.0],
                    },
                    TemplateComponent::Sprite {
                        color: [1.0, 0.0, 0.0, 1.0],
                        tex_index: 0,
                        layer: 0.0,
                    },
                    TemplateComponent::Velocity {
                        linear: [0.0, 0.0, 0.0],
                        angular: [0.0, 0.0, 0.0],
                    },
                ],
            },
        );

        // 玩家模板
        self.templates.insert(
            "Player".to_string(),
            EntityTemplate {
                name: "Player".to_string(),
                components: vec![
                    TemplateComponent::Transform {
                        position: [0.0, 0.0, 0.0],
                        rotation: [0.0, 0.0, 0.0, 1.0],
                        scale: [1.0, 1.0, 1.0],
                    },
                    TemplateComponent::Sprite {
                        color: [0.0, 1.0, 0.0, 1.0],
                        tex_index: 1,
                        layer: 1.0,
                    },
                    TemplateComponent::Velocity {
                        linear: [0.0, 0.0, 0.0],
                        angular: [0.0, 0.0, 0.0],
                    },
                ],
            },
        );

        // 道具模板
        self.templates.insert(
            "Prop".to_string(),
            EntityTemplate {
                name: "Prop".to_string(),
                components: vec![
                    TemplateComponent::Transform {
                        position: [0.0, 0.0, 0.0],
                        rotation: [0.0, 0.0, 0.0, 1.0],
                        scale: [1.0, 1.0, 1.0],
                    },
                    TemplateComponent::Sprite {
                        color: [1.0, 1.0, 0.0, 1.0],
                        tex_index: 2,
                        layer: 0.5,
                    },
                ],
            },
        );
    }

    /// 创建实体
    ///
    /// # 参数
    ///
    /// - `template_name`: 可选的模板名称，如果为None则创建空实体
    ///
    /// # 返回
    ///
    /// 返回新创建的实体ID
    ///
    /// # Lua示例
    ///
    /// ```lua
    /// local entity = create_entity("Enemy")
    /// ```
    pub fn create_entity(&self, template_name: Option<&str>) -> Result<Entity, String> {
        let mut world = Self::lock_result(safe_lock(&self.world, "EntityApi.world"))?;

        let entity = if let Some(template_name) = template_name {
            // 使用模板创建实体
            let template = self
                .templates
                .get(template_name)
                .ok_or_else(|| format!("Template '{}' not found", template_name))?;

            let mut entity = world.spawn_empty();

            // 应用模板组件
            for component in &template.components {
                match component {
                    TemplateComponent::Transform {
                        position,
                        rotation,
                        scale,
                    } => {
                        entity.insert(Transform {
                            pos: Vec3::new(position[0], position[1], position[2]),
                            rot: Quat::from_xyzw(
                                rotation[0],
                                rotation[1],
                                rotation[2],
                                rotation[3],
                            ),
                            scale: Vec3::new(scale[0], scale[1], scale[2]),
                        });
                    }
                    TemplateComponent::Sprite {
                        color,
                        tex_index,
                        layer,
                    } => {
                        entity.insert(Sprite {
                            color: *color,
                            tex_index: *tex_index,
                            layer: *layer,
                            ..Default::default()
                        });
                    }
                    TemplateComponent::Velocity { linear, angular } => {
                        entity.insert(Velocity {
                            lin: Vec3::new(linear[0], linear[1], linear[2]),
                            ang: Vec3::new(angular[0], angular[1], angular[2]),
                        });
                    }
                    TemplateComponent::Custom { .. } => {
                        // 自定义组件需要特殊处理，暂时跳过
                    }
                }
            }

            entity.id()
        } else {
            // 创建空实体
            world.spawn_empty().id()
        };

        Ok(entity)
    }

    /// 销毁实体
    ///
    /// # 参数
    ///
    /// - `entity`: 要销毁的实体
    ///
    /// # 返回
    ///
    /// 成功返回Ok(())，失败返回错误信息
    ///
    /// # Lua示例
    ///
    /// ```lua
    /// destroy_entity(entity_id)
    /// ```
    pub fn destroy_entity(&self, entity: Entity) -> Result<(), String> {
        let mut world = Self::lock_result(safe_lock(&self.world, "EntityApi.world"))?;

        // 从名称映射中移除
        if let Ok(mut names) =
            Self::lock_result(safe_lock(&self.entity_names, "EntityApi.entity_names"))
        {
            names.retain(|_, e| *e != entity);
        }

        if world.despawn(entity) {
            Ok(())
        } else {
            Err(format!(
                "Entity {:?} not found or already despawned",
                entity
            ))
        }
    }

    /// 查找包含特定组件的所有实体
    ///
    /// # 参数
    ///
    /// - `component_name`: 组件名称 (当前支持: "Transform", "Sprite", "Velocity")
    ///
    /// # 返回
    ///
    /// 返回包含该组件的所有实体ID列表
    ///
    /// # Lua示例
    ///
    /// ```lua
    /// local enemies = find_entities_with_component("Transform")
    /// ```
    pub fn find_entities_with_component(
        &self,
        component_name: &str,
    ) -> Result<Vec<Entity>, String> {
        let mut world = Self::lock_result(safe_lock(&self.world, "EntityApi.world"))?;

        match component_name {
            "Transform" => {
                let entities: Vec<Entity> =
                    world.query::<(Entity, &Transform)>().iter(&world).map(|(e, _)| e).collect();
                Ok(entities)
            }
            "Sprite" => {
                let entities: Vec<Entity> =
                    world.query::<(Entity, &Sprite)>().iter(&world).map(|(e, _)| e).collect();
                Ok(entities)
            }
            "Velocity" => {
                let entities: Vec<Entity> =
                    world.query::<(Entity, &Velocity)>().iter(&world).map(|(e, _)| e).collect();
                Ok(entities)
            }
            _ => Err(format!("Unknown component type: {}", component_name)),
        }
    }

    /// 按名称查找实体
    ///
    /// # 参数
    ///
    /// - `name`: 实体名称
    ///
    /// # 返回
    ///
    /// 返回实体ID（如果存在）
    ///
    /// # Lua示例
    ///
    /// ```lua
    /// local player = find_entity_by_name("Player")
    /// ```
    pub fn find_entity_by_name(&self, name: &str) -> Result<Option<Entity>, String> {
        let names = Self::lock_result(safe_lock(&self.entity_names, "EntityApi.entity_names"))?;
        Ok(names.get(name).copied())
    }

    /// 为实体命名
    ///
    /// # 参数
    ///
    /// - `entity`: 实体ID
    /// - `name`: 名称
    ///
    /// # Lua示例
    ///
    /// ```lua
    /// name_entity(entity_id, "MyEnemy")
    /// ```
    pub fn name_entity(&self, entity: Entity, name: String) -> Result<(), String> {
        let mut names = Self::lock_result(safe_lock(&self.entity_names, "EntityApi.entity_names"))?;
        names.insert(name, entity);
        Ok(())
    }

    /// 添加组件到实体
    ///
    /// # 参数
    ///
    /// - `entity`: 目标实体
    /// - `component_name`: 组件类型名称
    /// - `data`: 组件初始数据
    ///
    /// # 返回
    ///
    /// 成功返回Ok(())，失败返回错误信息
    ///
    /// # Lua示例
    ///
    /// ```lua
    /// add_component(entity_id, "Transform", {x=0, y=0, z=0})
    /// ```
    pub fn add_component(
        &self,
        entity: Entity,
        component_name: &str,
        data: &ScriptValue,
    ) -> Result<(), String> {
        let mut world = Self::lock_result(safe_lock(&self.world, "EntityApi.world"))?;

        match component_name {
            "Transform" => {
                let mut entity_mut = world.get_entity_mut(entity).map_err(|e| e.to_string())?;

                // 解析Transform数据
                let (pos, rot, scale) = if let ScriptValue::Object(map) = data {
                    let pos = Self::extract_vec3(map, "position")?;
                    let rot = Self::extract_quat(map, "rotation")?;
                    let scale = Self::extract_vec3(map, "scale")?;
                    (pos, rot, scale)
                } else {
                    (Vec3::ZERO, Quat::IDENTITY, Vec3::ONE)
                };

                entity_mut.insert(Transform { pos, rot, scale });
                Ok(())
            }
            "Sprite" => {
                let mut entity_mut = world.get_entity_mut(entity).map_err(|e| e.to_string())?;

                let sprite = if let ScriptValue::Object(map) = data {
                    Sprite {
                        color: Self::extract_color(map, "color")?,
                        tex_index: Self::extract_number(map, "tex_index")? as u32,
                        layer: Self::extract_number(map, "layer")? as f32,
                        ..Default::default()
                    }
                } else {
                    Sprite::default()
                };

                entity_mut.insert(sprite);
                Ok(())
            }
            "Velocity" => {
                let mut entity_mut = world.get_entity_mut(entity).map_err(|e| e.to_string())?;

                let velocity = if let ScriptValue::Object(map) = data {
                    Velocity {
                        lin: Self::extract_vec3(map, "linear")?,
                        ang: Self::extract_vec3(map, "angular")?,
                    }
                } else {
                    Velocity::default()
                };

                entity_mut.insert(velocity);
                Ok(())
            }
            _ => Err(format!("Unknown component type: {}", component_name)),
        }
    }

    /// 批量添加组件
    ///
    /// # 参数
    ///
    /// - `entity`: 目标实体
    /// - `components`: 组件数据映射 (组件名 -> 数据)
    ///
    /// # Lua示例
    ///
    /// ```lua
    /// add_components(entity_id, {
    ///     Transform = {x=0, y=0, z=0},
    ///     Sprite = {color={1,1,1,1}, tex_index=0}
    /// })
    /// ```
    pub fn add_components(
        &self,
        entity: Entity,
        components: &HashMap<String, ScriptValue>,
    ) -> Result<(), String> {
        for (component_name, data) in components {
            self.add_component(entity, component_name, data)?;
        }
        Ok(())
    }

    /// 移除实体的组件
    ///
    /// # 参数
    ///
    /// - `entity`: 目标实体
    /// - `component_name`: 组件类型名称
    ///
    /// # 返回
    ///
    /// 成功返回Ok(())，失败返回错误信息
    ///
    /// # Lua示例
    ///
    /// ```lua
    /// remove_component(entity_id, "Velocity")
    /// ```
    pub fn remove_component(&self, entity: Entity, component_name: &str) -> Result<(), String> {
        let mut world = Self::lock_result(safe_lock(&self.world, "EntityApi.world"))?;
        let mut entity_mut = world.get_entity_mut(entity).map_err(|e| e.to_string())?;

        match component_name {
            "Transform" => {
                entity_mut.remove::<Transform>();
                Ok(())
            }
            "Sprite" => {
                entity_mut.remove::<Sprite>();
                Ok(())
            }
            "Velocity" => {
                entity_mut.remove::<Velocity>();
                Ok(())
            }
            _ => Err(format!("Unknown component type: {}", component_name)),
        }
    }

    /// 检查实体是否有特定组件
    ///
    /// # 参数
    ///
    /// - `entity`: 目标实体
    /// - `component_name`: 组件类型名称
    ///
    /// # 返回
    ///
    /// 返回true/false表示组件是否存在
    ///
    /// # Lua示例
    ///
    /// ```lua
    /// if has_component(entity_id, "Transform") then
    ///     -- 处理
    /// end
    /// ```
    pub fn has_component(&self, entity: Entity, component_name: &str) -> Result<bool, String> {
        let world = Self::lock_result(safe_lock(&self.world, "EntityApi.world"))?;

        match component_name {
            "Transform" => {
                let result = world.get::<Transform>(entity).is_some();
                Ok(result)
            }
            "Sprite" => {
                let result = world.get::<Sprite>(entity).is_some();
                Ok(result)
            }
            "Velocity" => {
                let result = world.get::<Velocity>(entity).is_some();
                Ok(result)
            }
            _ => Err(format!("Unknown component type: {}", component_name)),
        }
    }

    /// 获取实体组件数据
    ///
    /// # 参数
    ///
    /// - `entity`: 目标实体
    /// - `component_name`: 组件类型名称
    ///
    /// # 返回
    ///
    /// 返回组件数据（转换为ScriptValue）
    ///
    /// # Lua示例
    ///
    /// ```lua
    /// local transform = get_component(entity_id, "Transform")
    /// print(transform.position.x)
    /// ```
    pub fn get_component(
        &self,
        entity: Entity,
        component_name: &str,
    ) -> Result<ScriptValue, String> {
        let mut world = Self::lock_result(safe_lock(&self.world, "EntityApi.world"))?;

        match component_name {
            "Transform" => {
                let transform = world
                    .get::<Transform>(entity)
                    .ok_or_else(|| "Transform component not found".to_string())?;
                let mut map = HashMap::new();
                map.insert(
                    "position".to_string(),
                    Self::vec3_to_script_value(transform.pos),
                );
                map.insert(
                    "rotation".to_string(),
                    Self::quat_to_script_value(transform.rot),
                );
                map.insert(
                    "scale".to_string(),
                    Self::vec3_to_script_value(transform.scale),
                );
                Ok(ScriptValue::Object(map))
            }
            "Sprite" => {
                let sprite = world
                    .get::<Sprite>(entity)
                    .ok_or_else(|| "Sprite component not found".to_string())?;
                let mut map = HashMap::new();
                map.insert(
                    "color".to_string(),
                    Self::color_to_script_value(sprite.color),
                );
                map.insert(
                    "tex_index".to_string(),
                    ScriptValue::Integer(sprite.tex_index as i64),
                );
                map.insert(
                    "layer".to_string(),
                    ScriptValue::Number(sprite.layer as f64),
                );
                Ok(ScriptValue::Object(map))
            }
            "Velocity" => {
                let velocity = world
                    .get::<Velocity>(entity)
                    .ok_or_else(|| "Velocity component not found".to_string())?;
                let mut map = HashMap::new();
                map.insert(
                    "linear".to_string(),
                    Self::vec3_to_script_value(velocity.lin),
                );
                map.insert(
                    "angular".to_string(),
                    Self::vec3_to_script_value(velocity.ang),
                );
                Ok(ScriptValue::Object(map))
            }
            _ => Err(format!("Unknown component type: {}", component_name)),
        }
    }

    /// 设置组件数据
    ///
    /// # 参数
    ///
    /// - `entity`: 目标实体
    /// - `component_name`: 组件类型名称
    /// - `data`: 新的组件数据
    ///
    /// # 返回
    ///
    /// 成功返回Ok(())，失败返回错误信息
    ///
    /// # Lua示例
    ///
    /// ```lua
    /// set_component_data(entity_id, "Transform", {x=10, y=20, z=0})
    /// ```
    pub fn set_component_data(
        &self,
        entity: Entity,
        component_name: &str,
        data: &ScriptValue,
    ) -> Result<(), String> {
        let mut world = Self::lock_result(safe_lock(&self.world, "EntityApi.world"))?;

        match component_name {
            "Transform" => {
                let mut transform = world
                    .get_mut::<Transform>(entity)
                    .ok_or_else(|| "Transform component not found".to_string())?;

                if let ScriptValue::Object(map) = data {
                    if let Ok(pos) = Self::extract_vec3(map, "position") {
                        transform.pos = pos;
                    }
                    if let Ok(rot) = Self::extract_quat(map, "rotation") {
                        transform.rot = rot;
                    }
                    if let Ok(scale) = Self::extract_vec3(map, "scale") {
                        transform.scale = scale;
                    }
                }
                Ok(())
            }
            "Sprite" => {
                let mut sprite = world
                    .get_mut::<Sprite>(entity)
                    .ok_or_else(|| "Sprite component not found".to_string())?;

                if let ScriptValue::Object(map) = data {
                    if let Ok(color) = Self::extract_color(map, "color") {
                        sprite.color = color;
                    }
                    if let Ok(tex_index) = Self::extract_number(map, "tex_index") {
                        sprite.tex_index = tex_index as u32;
                    }
                    if let Ok(layer) = Self::extract_number(map, "layer") {
                        sprite.layer = layer as f32;
                    }
                }
                Ok(())
            }
            "Velocity" => {
                let mut velocity = world
                    .get_mut::<Velocity>(entity)
                    .ok_or_else(|| "Velocity component not found".to_string())?;

                if let ScriptValue::Object(map) = data {
                    if let Ok(lin) = Self::extract_vec3(map, "linear") {
                        velocity.lin = lin;
                    }
                    if let Ok(ang) = Self::extract_vec3(map, "angular") {
                        velocity.ang = ang;
                    }
                }
                Ok(())
            }
            _ => Err(format!("Unknown component type: {}", component_name)),
        }
    }

    // ========================================
    // 辅助方法：数据转换
    // ========================================

    fn vec3_to_script_value(vec: Vec3) -> ScriptValue {
        let mut map = HashMap::new();
        map.insert("x".to_string(), ScriptValue::Number(vec.x as f64));
        map.insert("y".to_string(), ScriptValue::Number(vec.y as f64));
        map.insert("z".to_string(), ScriptValue::Number(vec.z as f64));
        ScriptValue::Object(map)
    }

    fn quat_to_script_value(quat: Quat) -> ScriptValue {
        let mut map = HashMap::new();
        map.insert("x".to_string(), ScriptValue::Number(quat.x as f64));
        map.insert("y".to_string(), ScriptValue::Number(quat.y as f64));
        map.insert("z".to_string(), ScriptValue::Number(quat.z as f64));
        map.insert("w".to_string(), ScriptValue::Number(quat.w as f64));
        ScriptValue::Object(map)
    }

    fn color_to_script_value(color: [f32; 4]) -> ScriptValue {
        let map = vec![
            ("r".to_string(), ScriptValue::Number(color[0] as f64)),
            ("g".to_string(), ScriptValue::Number(color[1] as f64)),
            ("b".to_string(), ScriptValue::Number(color[2] as f64)),
            ("a".to_string(), ScriptValue::Number(color[3] as f64)),
        ]
        .into_iter()
        .collect();
        ScriptValue::Object(map)
    }

    fn extract_vec3(map: &HashMap<String, ScriptValue>, key: &str) -> Result<Vec3, String> {
        if let Some(ScriptValue::Object(obj)) = map.get(key) {
            let x = Self::extract_number(obj, "x")? as f32;
            let y = Self::extract_number(obj, "y")? as f32;
            let z = Self::extract_number(obj, "z")? as f32;
            Ok(Vec3::new(x, y, z))
        } else {
            Err(format!("Expected Vec3 at key '{}'", key))
        }
    }

    fn extract_quat(map: &HashMap<String, ScriptValue>, key: &str) -> Result<Quat, String> {
        if let Some(ScriptValue::Object(obj)) = map.get(key) {
            let x = Self::extract_number(obj, "x")? as f32;
            let y = Self::extract_number(obj, "y")? as f32;
            let z = Self::extract_number(obj, "z")? as f32;
            let w = Self::extract_number(obj, "w")? as f32;
            Ok(Quat::from_xyzw(x, y, z, w))
        } else {
            Err(format!("Expected Quat at key '{}'", key))
        }
    }

    fn extract_color(map: &HashMap<String, ScriptValue>, key: &str) -> Result<[f32; 4], String> {
        if let Some(ScriptValue::Object(obj)) = map.get(key) {
            let r = Self::extract_number(obj, "r")? as f32;
            let g = Self::extract_number(obj, "g")? as f32;
            let b = Self::extract_number(obj, "b")? as f32;
            let a = Self::extract_number(obj, "a")? as f32;
            Ok([r, g, b, a])
        } else {
            Err(format!("Expected color at key '{}'", key))
        }
    }

    fn extract_number(map: &HashMap<String, ScriptValue>, key: &str) -> Result<f64, String> {
        if let Some(value) = map.get(key) {
            match value {
                ScriptValue::Integer(i) => Ok(*i as f64),
                ScriptValue::Number(f) => Ok(*f),
                _ => Err(format!("Expected number at key '{}'", key)),
            }
        } else {
            Err(format!("Missing key '{}'", key))
        }
    }
}

/// 实体查询构建器 - 提供流畅的链式查询API
///
/// # 示例
///
/// ```lua
/// local visible_enemies = query_entities()
///     :with_component("Transform")
///     :with_component("Sprite")
///     :where(function(e) return e.Transform.x > 100 end)
///     :result()
/// ```
pub struct EntityQueryBuilder {
    world: Arc<Mutex<World>>,
    required_components: Vec<String>,
}

impl EntityQueryBuilder {
    /// 创建新的查询构建器
    pub fn new(world: Arc<Mutex<World>>) -> Self {
        Self {
            world,
            required_components: Vec::new(),
        }
    }

    /// 添加必需的组件
    pub fn with_component(mut self, component: &str) -> Self {
        self.required_components.push(component.to_string());
        self
    }

    /// 执行查询并返回结果
    ///
    /// 注意: 当前实现只支持简单的组件查询，复杂的where条件需要
    /// 在查询结果上进行过滤
    pub fn result(self) -> Result<Vec<Entity>, String> {
        let mut world = safe_lock(&self.world, "EntityQueryBuilder.world")
            .map_err(|e| format!("Failed to lock world: {}", e))?;

        // 根据要求的组件组合进行查询
        if self.required_components.is_empty() {
            // 返回所有实体 - 使用查询来获取
            let entities: Vec<Entity> = world.query::<Entity>().iter(&world).collect();
            return Ok(entities);
        }

        // 简化实现：只检查第一个要求的组件
        if let Some(first_component) = self.required_components.first() {
            match first_component.as_str() {
                "Transform" => {
                    let entities: Vec<Entity> = world
                        .query::<(Entity, &Transform)>()
                        .iter(&world)
                        .map(|(e, _)| e)
                        .collect();
                    Ok(entities)
                }
                "Sprite" => {
                    let entities: Vec<Entity> =
                        world.query::<(Entity, &Sprite)>().iter(&world).map(|(e, _)| e).collect();
                    Ok(entities)
                }
                "Velocity" => {
                    let entities: Vec<Entity> =
                        world.query::<(Entity, &Velocity)>().iter(&world).map(|(e, _)| e).collect();
                    Ok(entities)
                }
                _ => Err(format!("Unknown component type: {}", first_component)),
            }
        } else {
            Ok(Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_world() -> Arc<Mutex<World>> {
        Arc::new(Mutex::new(World::new()))
    }

    #[test]
    fn test_create_empty_entity() {
        let world = create_test_world();
        let api = EntityApi::new(world);

        let entity = api.create_entity(None).unwrap();
        assert!(entity.to_bits() > 0);
    }

    #[test]
    fn test_create_entity_from_template() {
        let world = create_test_world();
        let api = EntityApi::new(world);

        let entity = api.create_entity(Some("Enemy")).unwrap();

        // 检查组件是否存在
        assert!(api.has_component(entity, "Transform").unwrap());
        assert!(api.has_component(entity, "Sprite").unwrap());
        assert!(api.has_component(entity, "Velocity").unwrap());
    }

    #[test]
    fn test_add_and_remove_component() {
        let world = create_test_world();
        let api = EntityApi::new(world.clone());

        let entity = api.create_entity(None).unwrap();

        // 添加Transform组件
        let mut transform_data = HashMap::new();
        let pos_data = vec![
            ("x".to_string(), ScriptValue::Number(10.0)),
            ("y".to_string(), ScriptValue::Number(20.0)),
            ("z".to_string(), ScriptValue::Number(30.0)),
        ]
        .into_iter()
        .collect();
        transform_data.insert("position".to_string(), ScriptValue::Object(pos_data));

        api.add_component(entity, "Transform", &ScriptValue::Object(transform_data))
            .unwrap();

        assert!(api.has_component(entity, "Transform").unwrap());

        // 移除组件
        api.remove_component(entity, "Transform").unwrap();
        assert!(!api.has_component(entity, "Transform").unwrap());
    }

    #[test]
    fn test_find_entities_with_component() {
        let world = create_test_world();
        let api = EntityApi::new(world.clone());

        // 创建几个实体
        let entity1 = api.create_entity(Some("Player")).unwrap();
        let entity2 = api.create_entity(Some("Enemy")).unwrap();
        let entity3 = api.create_entity(Some("Prop")).unwrap();

        // 查询所有包含Transform的实体
        let entities = api.find_entities_with_component("Transform").unwrap();

        assert!(entities.contains(&entity1));
        assert!(entities.contains(&entity2));
        assert!(entities.contains(&entity3));
    }

    #[test]
    fn test_entity_naming() {
        let world = create_test_world();
        let api = EntityApi::new(world);

        let entity = api.create_entity(None).unwrap();
        api.name_entity(entity, "TestEntity".to_string()).unwrap();

        let found = api.find_entity_by_name("TestEntity").unwrap();
        assert_eq!(found, Some(entity));
    }

    #[test]
    fn test_get_and_set_component() {
        let world = create_test_world();
        let api = EntityApi::new(world);

        let entity = api.create_entity(Some("Player")).unwrap();

        // 获取Transform
        let transform = api.get_component(entity, "Transform").unwrap();
        assert!(matches!(transform, ScriptValue::Object(_)));

        // 设置新位置
        let mut new_pos = HashMap::new();
        new_pos.insert("x".to_string(), ScriptValue::Number(100.0));
        new_pos.insert("y".to_string(), ScriptValue::Number(200.0));
        new_pos.insert("z".to_string(), ScriptValue::Number(300.0));

        let mut transform_data = HashMap::new();
        transform_data.insert("position".to_string(), ScriptValue::Object(new_pos));

        api.set_component_data(entity, "Transform", &ScriptValue::Object(transform_data))
            .unwrap();

        // 验证新位置
        let updated = api.get_component(entity, "Transform").unwrap();
        if let ScriptValue::Object(map) = updated {
            if let Some(ScriptValue::Object(pos)) = map.get("position") {
                assert_eq!(pos.get("x"), Some(&ScriptValue::Number(100.0)));
                assert_eq!(pos.get("y"), Some(&ScriptValue::Number(200.0)));
                assert_eq!(pos.get("z"), Some(&ScriptValue::Number(300.0)));
            }
        }
    }

    #[test]
    fn test_batch_add_components() {
        let world = create_test_world();
        let api = EntityApi::new(world);

        let entity = api.create_entity(None).unwrap();

        // 准备多个组件
        let mut components = HashMap::new();

        let mut transform_data = HashMap::new();
        let pos_data = vec![
            ("x".to_string(), ScriptValue::Number(1.0)),
            ("y".to_string(), ScriptValue::Number(2.0)),
            ("z".to_string(), ScriptValue::Number(3.0)),
        ]
        .into_iter()
        .collect();
        transform_data.insert("position".to_string(), ScriptValue::Object(pos_data));
        components.insert("Transform".to_string(), ScriptValue::Object(transform_data));

        api.add_components(entity, &components).unwrap();

        assert!(api.has_component(entity, "Transform").unwrap());
    }

    #[test]
    fn test_query_builder() {
        let world = create_test_world();
        let api = EntityApi::new(world);

        let _entity1 = api.create_entity(Some("Player")).unwrap();
        let _entity2 = api.create_entity(Some("Enemy")).unwrap();
        let _entity3 = api.create_entity(Some("Prop")).unwrap();

        let builder = EntityQueryBuilder::new(api.world.clone());
        let entities = builder.with_component("Transform").result().unwrap();

        assert_eq!(entities.len(), 3);
    }

    #[test]
    fn test_destroy_entity() {
        let world = create_test_world();
        let api = EntityApi::new(world);

        let entity = api.create_entity(Some("Enemy")).unwrap();
        api.name_entity(entity, "TestEnemy".to_string()).unwrap();

        // 销毁实体
        api.destroy_entity(entity).unwrap();

        // 验证实体已销毁（无法再找到）
        let found = api.find_entity_by_name("TestEnemy").unwrap();
        assert_eq!(found, None);
    }
}
