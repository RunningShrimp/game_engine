//! UI框架核心实现
//!
//! 定义UI组件的抽象接口和UI管理器。

use crate::ui::{events::UIEvent, layout::RectTransform};
use bevy_ecs::prelude::*;
use glam::Vec2;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// UI组件ID
pub type ComponentId = Uuid;

/// UI组件trait
///
/// 所有UI组件都必须实现此trait。
pub trait UIComponent: Send + Sync {
    /// 获取组件ID
    fn id(&self) -> ComponentId;

    /// 获取组件名称
    fn name(&self) -> &str;

    /// 更新组件
    fn update(&mut self, delta_time: f32, context: &UIContext);

    /// 渲染组件
    fn render(&self, context: &UIContext);

    /// 处理事件
    fn handle_event(&mut self, event: &UIEvent, context: &UIContext) -> bool;

    /// 获取子组件
    fn children(&self) -> &[ComponentId];

    /// 添加子组件
    fn add_child(&mut self, child: ComponentId);

    /// 移除子组件
    fn remove_child(&mut self, child: ComponentId);

    /// 获取RectTransform
    fn rect_transform(&self) -> &RectTransform;

    /// 获取可变RectTransform
    fn rect_transform_mut(&mut self) -> &mut RectTransform;

    /// 设置可见性
    fn set_visible(&mut self, visible: bool);

    /// 是否可见
    fn is_visible(&self) -> bool;
}

/// UI上下文
///
/// 提供UI渲染和交互所需的上下文信息。
pub struct UIContext {
    /// 画布尺寸
    pub canvas_size: Vec2,
    /// 鼠标位置
    pub mouse_position: Vec2,
    /// 是否按下鼠标
    pub mouse_down: bool,
    /// 按下的键
    pub pressed_keys: HashMap<String, bool>,
    /// Delta时间
    pub delta_time: f32,
    /// 当前时间
    pub current_time: f64,
}

impl Default for UIContext {
    fn default() -> Self {
        Self {
            canvas_size: Vec2::new(1920.0, 1080.0),
            mouse_position: Vec2::ZERO,
            mouse_down: false,
            pressed_keys: HashMap::new(),
            delta_time: 0.016,
            current_time: 0.0,
        }
    }
}

/// UI管理器
///
/// 管理所有UI组件的生命周期和渲染。
pub struct UIManager {
    /// 所有UI组件
    components: HashMap<ComponentId, Box<dyn UIComponent>>,

    /// 根组件ID列表
    roots: Vec<ComponentId>,

    /// 焦点组件
    focused_component: Option<ComponentId>,

    /// 悬停组件
    hovered_component: Option<ComponentId>,

    /// UI状态
    state: UIState,
}

/// UI状态
#[derive(Debug, Clone, Default)]
pub struct UIState {
    /// 捕获的组件（用于拖拽等）
    pub captured: Option<ComponentId>,
    /// 是否正在拖拽
    pub is_dragging: bool,
    /// 拖拽偏移
    pub drag_offset: Vec2,
}

impl UIManager {
    /// 创建新的UI管理器
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            roots: Vec::new(),
            focused_component: None,
            hovered_component: None,
            state: UIState::default(),
        }
    }

    /// 添加根组件
    pub fn add_root(&mut self, component: Box<dyn UIComponent>) -> ComponentId {
        let id = component.id();
        self.roots.push(id);
        self.components.insert(id, component);
        id
    }

    /// 获取组件
    pub fn get_component(&self, id: ComponentId) -> Option<&dyn UIComponent> {
        self.components.get(&id).map(|c| c.as_ref())
    }

    /// 获取可变组件
    pub fn get_component_mut(&mut self, id: ComponentId) -> Option<&mut dyn UIComponent> {
        self.components.get_mut(&id).map(|c| c.as_mut())
    }

    /// 移除组件
    pub fn remove_component(&mut self, id: ComponentId) -> Option<Box<dyn UIComponent>> {
        // 从根列表中移除
        self.roots.retain(|&root_id| root_id != id);

        // 递归移除所有子组件
        self.remove_component_recursive(id)
    }

    fn remove_component_recursive(&mut self, id: ComponentId) -> Option<Box<dyn UIComponent>> {
        if let Some(component) = self.components.remove(&id) {
            // 移除所有子组件
            for child_id in component.children() {
                self.remove_component_recursive(*child_id);
            }
            Some(component)
        } else {
            None
        }
    }

    /// 更新UI
    pub fn update(&mut self) {
        // 更新所有根组件
        let roots: Vec<ComponentId> = self.roots.clone();

        for root_id in roots {
            self.update_component_recursive(root_id);
        }
    }

    fn update_component_recursive(&mut self, id: ComponentId) {
        let children: Vec<ComponentId> = if let Some(component) = self.components.get(&id) {
            component.children().to_vec()
        } else {
            return;
        };

        // 更新子组件
        for child_id in children {
            self.update_component_recursive(child_id);
        }

        // 更新当前组件
        let context = UIContext::default();
        if let Some(component) = self.components.get_mut(&id) {
            component.update(0.016, &context);
        }
    }

    /// 渲染UI
    pub fn render(&self) {
        let context = UIContext::default();

        for root_id in &self.roots {
            self.render_component_recursive(*root_id, &context);
        }
    }

    fn render_component_recursive(&self, id: ComponentId, context: &UIContext) {
        // 渲染子组件
        if let Some(component) = self.components.get(&id) {
            // 先渲染子组件
            for child_id in component.children() {
                self.render_component_recursive(*child_id, context);
            }

            // 渲染当前组件
            component.render(context);
        }
    }

    /// 处理事件
    pub fn handle_event(&mut self, event: UIEvent) {
        let context = UIContext::default();

        // 从根组件开始传播事件
        for root_id in &self.roots {
            if self.propagate_event(*root_id, &event, &context) {
                break; // 事件已处理
            }
        }
    }

    fn propagate_event(&mut self, id: ComponentId, event: &UIEvent, context: &UIContext) -> bool {
        if let Some(component) = self.components.get_mut(&id) {
            // 先传递给子组件
            for child_id in component.children().to_vec() {
                if self.propagate_event(*child_id, event, context) {
                    return true;
                }
            }

            // 处理当前组件
            component.handle_event(event, context)
        } else {
            false
        }
    }

    /// 设置焦点
    pub fn set_focus(&mut self, id: ComponentId) {
        self.focused_component = Some(id);
    }

    /// 获取焦点组件
    pub fn focused_component(&self) -> Option<ComponentId> {
        self.focused_component
    }

    /// 清除焦点
    pub fn clear_focus(&mut self) {
        self.focused_component = None;
    }

    /// 获取根组件数量
    pub fn root_count(&self) -> usize {
        self.roots.len()
    }

    /// 获取UI状态
    pub fn state(&self) -> &UIState {
        &self.state
    }

    /// 获取可变UI状态
    pub fn state_mut(&mut self) -> &mut UIState {
        &mut self.state
    }
}

impl Default for UIManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_manager_creation() {
        let manager = UIManager::new();
        assert_eq!(manager.root_count(), 0);
        assert!(manager.focused_component().is_none());
    }

    #[test]
    fn test_ui_context_default() {
        let context = UIContext::default();
        assert_eq!(context.canvas_size, Vec2::new(1920.0, 1080.0));
    }

    #[test]
    fn test_ui_state_default() {
        let state = UIState::default();
        assert!(state.captured.is_none());
        assert!(!state.is_dragging);
    }
}
