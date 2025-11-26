use super::api::ScriptApi;
use super::system::{ScriptValue, ScriptResult};
use bevy_ecs::prelude::*;
use crate::ecs::{Transform, Sprite};
use glam::{Vec3, Vec4};
use std::sync::{Arc, Mutex};

/// 图形和UI脚本绑定
pub struct GraphicsUiBindings {
    world: Arc<Mutex<World>>,
}

impl GraphicsUiBindings {
    pub fn new(world: Arc<Mutex<World>>) -> Self {
        Self { world }
    }
    
    /// 注册图形和UI相关的脚本API
    pub fn register_api(&self, api: &mut ScriptApi) {
        // 图形相关API
        self.register_graphics_api(api);
        
        // UI相关API
        self.register_ui_api(api);
    }
    
    /// 注册图形相关API
    fn register_graphics_api(&self, api: &mut ScriptApi) {
        let world = self.world.clone();
        
        // 设置精灵颜色
        api.register_function("set_sprite_color", move |args| {
            if let (Some(ScriptValue::Int(entity_id)), 
                    Some(ScriptValue::Float(r)), 
                    Some(ScriptValue::Float(g)), 
                    Some(ScriptValue::Float(b)),
                    Some(ScriptValue::Float(a))) = 
                (args.get(0), args.get(1), args.get(2), args.get(3), args.get(4)) {
                let mut world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);
                
                if let Some(mut sprite) = world.get_mut::<Sprite>(entity) {
                    sprite.color = [*r as f32, *g as f32, *b as f32, *a as f32];
                    ScriptResult::Success("Sprite color updated".to_string())
                } else {
                    ScriptResult::Error("Sprite component not found".to_string())
                }
            } else {
                ScriptResult::Error("set_sprite_color() requires entity_id, r, g, b, a".to_string())
            }
        });
        
        let world = self.world.clone();
        
        // 设置精灵大小
        api.register_function("set_sprite_size", move |args| {
            if let (Some(ScriptValue::Int(entity_id)), 
                    Some(ScriptValue::Float(width)), 
                    Some(ScriptValue::Float(height))) = 
                (args.get(0), args.get(1), args.get(2)) {
                let mut world = world.lock().unwrap();
                let entity = Entity::from_bits(*entity_id as u64);
                
                if let Some(mut sprite) = world.get_mut::<Sprite>(entity) {
                    sprite.uv_scale = [*width as f32, *height as f32];
                    ScriptResult::Success("Sprite size updated".to_string())
                } else {
                    ScriptResult::Error("Sprite component not found".to_string())
                }
            } else {
                ScriptResult::Error("set_sprite_size() requires entity_id, width, height".to_string())
            }
        });
        
        let world = self.world.clone();
        
        // 绘制线条 (简化版,实际需要添加到渲染队列)
        api.register_function("draw_line", move |args| {
            if let (Some(ScriptValue::Float(x1)), Some(ScriptValue::Float(y1)),
                    Some(ScriptValue::Float(x2)), Some(ScriptValue::Float(y2)),
                    Some(ScriptValue::Float(r)), Some(ScriptValue::Float(g)),
                    Some(ScriptValue::Float(b))) = 
                (args.get(0), args.get(1), args.get(2), args.get(3),
                 args.get(4), args.get(5), args.get(6)) {
                // 实际实现需要将线条添加到调试渲染队列
                ScriptResult::Success(format!(
                    "Drawing line from ({}, {}) to ({}, {}) with color ({}, {}, {})",
                    x1, y1, x2, y2, r, g, b
                ))
            } else {
                ScriptResult::Error("draw_line() requires x1, y1, x2, y2, r, g, b".to_string())
            }
        });
        
        // 绘制圆形 (简化版)
        api.register_function("draw_circle", move |args| {
            if let (Some(ScriptValue::Float(x)), Some(ScriptValue::Float(y)),
                    Some(ScriptValue::Float(radius)),
                    Some(ScriptValue::Float(r)), Some(ScriptValue::Float(g)),
                    Some(ScriptValue::Float(b))) = 
                (args.get(0), args.get(1), args.get(2),
                 args.get(3), args.get(4), args.get(5)) {
                ScriptResult::Success(format!(
                    "Drawing circle at ({}, {}) with radius {} and color ({}, {}, {})",
                    x, y, radius, r, g, b
                ))
            } else {
                ScriptResult::Error("draw_circle() requires x, y, radius, r, g, b".to_string())
            }
        });
        
        // 绘制矩形 (简化版)
        api.register_function("draw_rect", move |args| {
            if let (Some(ScriptValue::Float(x)), Some(ScriptValue::Float(y)),
                    Some(ScriptValue::Float(width)), Some(ScriptValue::Float(height)),
                    Some(ScriptValue::Float(r)), Some(ScriptValue::Float(g)),
                    Some(ScriptValue::Float(b))) = 
                (args.get(0), args.get(1), args.get(2), args.get(3),
                 args.get(4), args.get(5), args.get(6)) {
                ScriptResult::Success(format!(
                    "Drawing rect at ({}, {}) with size {}x{} and color ({}, {}, {})",
                    x, y, width, height, r, g, b
                ))
            } else {
                ScriptResult::Error("draw_rect() requires x, y, width, height, r, g, b".to_string())
            }
        });
        
        let world = self.world.clone();
        
        // 设置相机位置
        api.register_function("set_camera_position", move |args| {
            if let (Some(ScriptValue::Float(x)), Some(ScriptValue::Float(y)), Some(ScriptValue::Float(z))) = 
                (args.get(0), args.get(1), args.get(2)) {
                // 实际实现需要找到Camera组件并更新其Transform
                ScriptResult::Success(format!("Camera position set to ({}, {}, {})", x, y, z))
            } else {
                ScriptResult::Error("set_camera_position() requires x, y, z".to_string())
            }
        });
    }
    
    /// 注册UI相关API
    fn register_ui_api(&self, api: &mut ScriptApi) {
        // 显示文本 (占位实现)
        api.register_function("ui_text", move |args| {
            if let (Some(ScriptValue::String(text)), 
                    Some(ScriptValue::Float(x)), 
                    Some(ScriptValue::Float(y))) = 
                (args.get(0), args.get(1), args.get(2)) {
                ScriptResult::Success(format!("Displaying text '{}' at ({}, {})", text, x, y))
            } else {
                ScriptResult::Error("ui_text() requires text, x, y".to_string())
            }
        });
        
        // 显示按钮 (占位实现)
        api.register_function("ui_button", move |args| {
            if let (Some(ScriptValue::String(label)), 
                    Some(ScriptValue::Float(x)), 
                    Some(ScriptValue::Float(y))) = 
                (args.get(0), args.get(1), args.get(2)) {
                // 实际实现需要集成UI系统
                ScriptResult::Success(format!("Button '{}' at ({}, {})", label, x, y))
            } else {
                ScriptResult::Error("ui_button() requires label, x, y".to_string())
            }
        });
        
        // 显示滑块 (占位实现)
        api.register_function("ui_slider", move |args| {
            if let (Some(ScriptValue::String(label)), 
                    Some(ScriptValue::Float(min)), 
                    Some(ScriptValue::Float(max)),
                    Some(ScriptValue::Float(value))) = 
                (args.get(0), args.get(1), args.get(2), args.get(3)) {
                ScriptResult::Success(format!(
                    "Slider '{}' with range [{}, {}] and value {}",
                    label, min, max, value
                ))
            } else {
                ScriptResult::Error("ui_slider() requires label, min, max, value".to_string())
            }
        });
        
        // 显示图片 (占位实现)
        api.register_function("ui_image", move |args| {
            if let (Some(ScriptValue::String(image_path)), 
                    Some(ScriptValue::Float(x)), 
                    Some(ScriptValue::Float(y)),
                    Some(ScriptValue::Float(width)),
                    Some(ScriptValue::Float(height))) = 
                (args.get(0), args.get(1), args.get(2), args.get(3), args.get(4)) {
                ScriptResult::Success(format!(
                    "Image '{}' at ({}, {}) with size {}x{}",
                    image_path, x, y, width, height
                ))
            } else {
                ScriptResult::Error("ui_image() requires image_path, x, y, width, height".to_string())
            }
        });
        
        // 显示面板 (占位实现)
        api.register_function("ui_panel", move |args| {
            if let (Some(ScriptValue::String(title)), 
                    Some(ScriptValue::Float(x)), 
                    Some(ScriptValue::Float(y)),
                    Some(ScriptValue::Float(width)),
                    Some(ScriptValue::Float(height))) = 
                (args.get(0), args.get(1), args.get(2), args.get(3), args.get(4)) {
                ScriptResult::Success(format!(
                    "Panel '{}' at ({}, {}) with size {}x{}",
                    title, x, y, width, height
                ))
            } else {
                ScriptResult::Error("ui_panel() requires title, x, y, width, height".to_string())
            }
        });
        
        // 获取鼠标位置 (占位实现)
        api.register_function("get_mouse_position", move |_args| {
            // 实际实现需要从输入系统获取鼠标位置
            ScriptResult::Success("Mouse position: (0, 0)".to_string())
        });
        
        // 检查按键状态 (占位实现)
        api.register_function("is_key_pressed", move |args| {
            if let Some(ScriptValue::String(key)) = args.get(0) {
                // 实际实现需要从输入系统查询按键状态
                ScriptResult::Success(format!("Key '{}' is not pressed", key))
            } else {
                ScriptResult::Error("is_key_pressed() requires a key name".to_string())
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_graphics_bindings() {
        let mut world = World::new();
        let world_arc = Arc::new(Mutex::new(world));
        
        let bindings = GraphicsUiBindings::new(world_arc.clone());
        let mut api = ScriptApi::new();
        bindings.register_api(&mut api);
        
        // 创建实体并添加Sprite组件
        let entity = {
            let mut world = world_arc.lock().unwrap();
            world.spawn(Sprite::default()).id()
        };
        let entity_id = entity.to_bits() as i64;
        
        // 设置精灵颜色
        let result = api.call("set_sprite_color", &[
            ScriptValue::Int(entity_id),
            ScriptValue::Float(1.0),
            ScriptValue::Float(0.0),
            ScriptValue::Float(0.0),
            ScriptValue::Float(1.0),
        ]);
        assert!(matches!(result, ScriptResult::Success(_)));
    }
    
    #[test]
    fn test_ui_bindings() {
        let mut world = World::new();
        let world_arc = Arc::new(Mutex::new(world));
        
        let bindings = GraphicsUiBindings::new(world_arc.clone());
        let mut api = ScriptApi::new();
        bindings.register_api(&mut api);
        
        // 显示文本
        let result = api.call("ui_text", &[
            ScriptValue::String("Hello, World!".to_string()),
            ScriptValue::Float(100.0),
            ScriptValue::Float(200.0),
        ]);
        assert!(matches!(result, ScriptResult::Success(_)));
        
        // 显示按钮
        let result = api.call("ui_button", &[
            ScriptValue::String("Click Me".to_string()),
            ScriptValue::Float(150.0),
            ScriptValue::Float(250.0),
        ]);
        assert!(matches!(result, ScriptResult::Success(_)));
    }
}
