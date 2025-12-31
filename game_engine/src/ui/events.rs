//! UI事件系统
//!
//! 处理UI组件的输入事件和事件传播。

use crate::ui::ComponentId;
use bevy_ecs::prelude::*;
use glam::Vec2;
use std::collections::HashMap;

/// UI事件
#[derive(Debug, Clone)]
pub enum UIEvent {
    /// 鼠标点击
    MouseClick {
        position: Vec2,
        button: MouseButton,
    },
    /// 鼠标释放
    MouseRelease {
        position: Vec2,
        button: MouseButton,
    },
    /// 鼠标移动
    MouseMove {
        position: Vec2,
        delta: Vec2,
    },
    /// 鼠标滚轮
    MouseScroll {
        delta: f32,
    },
    /// 键盘按下
    KeyDown {
        key: String,
        code: String,
    },
    /// 键盘释放
    KeyUp {
        key: String,
        code: String,
    },
    /// 字符输入
    Char {
        char: char,
    },
    /// 焦点获得
    FocusGained,
    /// 焦点丢失
    FocusLost,
    /// 值改变
    ValueChanged {
        new_value: String,
    },
    /// 自定义事件
    Custom {
        event_type: String,
        data: HashMap<String, String>,
    },
}

/// 鼠标按钮
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// 事件相位
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPhase {
    /// 捕获阶段（从根到目标）
    Capture,
    /// 目标阶段
    Target,
    /// 冒泡阶段（从目标到根）
    Bubbling,
}

/// UI事件管理器
///
/// 管理UI事件的分发和传播。
pub struct UIEventManager {
    /// 事件队列
    event_queue: Vec<UIEvent>,

    /// 事件监听器
    listeners: HashMap<ComponentId, Vec<EventListener>>,

    /// 当前捕获的组件
    captured: Option<ComponentId>,

    /// 当前悬停的组件
    hovered: Option<ComponentId>,
}

/// 事件监听器
#[derive(Clone)]
pub struct EventListener {
    /// 事件类型
    pub event_type: String,

    /// 回调函数
    pub callback: Box<dyn Fn(&UIEvent) -> bool + Send + Sync>,
}

impl UIEventManager {
    /// 创建新的事件管理器
    pub fn new() -> Self {
        Self {
            event_queue: Vec::new(),
            listeners: HashMap::new(),
            captured: None,
            hovered: None,
        }
    }

    /// 添加事件监听器
    pub fn add_listener(&mut self, component_id: ComponentId, listener: EventListener) {
        self.listeners
            .entry(component_id)
            .or_insert_with(Vec::new)
            .push(listener);
    }

    /// 移除组件的所有监听器
    pub fn remove_listeners(&mut self, component_id: ComponentId) {
        self.listeners.remove(&component_id);
    }

    /// 发送事件
    pub fn send_event(&mut self, event: UIEvent) {
        self.event_queue.push(event);
    }

    /// 处理事件队列
    pub fn process_events(&mut self) {
        let events = std::mem::take(&mut self.event_queue);

        for event in events {
            self.dispatch_event(&event);
        }
    }

    /// 分发事件
    fn dispatch_event(&mut self, event: &UIEvent) {
        // 如果有捕获的组件，只发送给它
        if let Some(captured) = self.captured {
            self.notify_component(captured, event);
            return;
        }

        // 否则，广播给所有监听器
        for (component_id, listeners) in &self.listeners {
            for listener in listeners {
                let event_type = Self::get_event_type(event);

                if listener.event_type == event_type || listener.event_type == "*" {
                    if (listener.callback)(event) {
                        // 事件被处理，停止传播
                        break;
                    }
                }
            }
        }
    }

    /// 通知组件
    fn notify_component(&self, component_id: ComponentId, event: &UIEvent) {
        if let Some(listeners) = self.listeners.get(&component_id) {
            let event_type = Self::get_event_type(event);

            for listener in listeners {
                if listener.event_type == event_type || listener.event_type == "*" {
                    if (listener.callback)(event) {
                        break;
                    }
                }
            }
        }
    }

    /// 获取事件类型字符串
    fn get_event_type(event: &UIEvent) -> String {
        match event {
            UIEvent::MouseClick { .. } => "click".to_string(),
            UIEvent::MouseRelease { .. } => "release".to_string(),
            UIEvent::MouseMove { .. } => "mousemove".to_string(),
            UIEvent::MouseScroll { .. } => "scroll".to_string(),
            UIEvent::KeyDown { .. } => "keydown".to_string(),
            UIEvent::KeyUp { .. } => "keyup".to_string(),
            UIEvent::Char { .. } => "char".to_string(),
            UIEvent::FocusGained => "focus".to_string(),
            UIEvent::FocusLost => "blur".to_string(),
            UIEvent::ValueChanged { .. } => "change".to_string(),
            UIEvent::Custom { event_type, .. } => event_type.clone(),
        }
    }

    /// 设置捕获的组件
    pub fn set_capture(&mut self, component_id: ComponentId) {
        self.captured = Some(component_id);
    }

    /// 释放捕获
    pub fn release_capture(&mut self) {
        self.captured = None;
    }

    /// 设置悬停的组件
    pub fn set_hovered(&mut self, component_id: ComponentId) {
        self.hovered = Some(component_id);
    }

    /// 获取悬停的组件
    pub fn hovered(&self) -> Option<ComponentId> {
        self.hovered
    }

    /// 清除悬停
    pub fn clear_hovered(&mut self) {
        self.hovered = None;
    }
}

impl Default for UIEventManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 点击检测器
///
/// 执行UI组件的点击检测。
pub struct HitTester {
    /// 组件位置映射
    components: HashMap<ComponentId, HitBox>,
}

/// 点击框
#[derive(Debug, Clone)]
pub struct HitBox {
    /// 最小位置
    pub min: Vec2,
    /// 最大位置
    pub max: Vec2,
    /// 组件ID
    pub component_id: ComponentId,
}

impl HitTester {
    /// 创建新的点击检测器
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// 注册组件
    pub fn register(&mut self, component_id: ComponentId, position: Vec2, size: Vec2) {
        let hit_box = HitBox {
            min: position,
            max: position + size,
            component_id,
        };

        self.components.insert(component_id, hit_box);
    }

    /// 移除组件
    pub fn unregister(&mut self, component_id: ComponentId) {
        self.components.remove(&component_id);
    }

    /// 测试点击
    pub fn test_click(&self, position: Vec2) -> Option<ComponentId> {
        // 从后往前查找（z-index顺序）
        for hit_box in self.components.values().rev() {
            if position.x >= hit_box.min.x
                && position.x <= hit_box.max.x
                && position.y >= hit_box.min.y
                && position.y <= hit_box.max.y
            {
                return Some(hit_box.component_id);
            }
        }

        None
    }

    /// 测试悬停
    pub fn test_hover(&self, position: Vec2) -> Option<ComponentId> {
        self.test_click(position)
    }
}

impl Default for HitTester {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_manager_creation() {
        let manager = UIEventManager::new();
        assert!(manager.captured.is_none());
        assert!(manager.hovered().is_none());
    }

    #[test]
    fn test_hit_tester() {
        let mut tester = HitTester::new();
        let id = ComponentId::new();

        tester.register(id, Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        assert!(tester.test_click(Vec2::new(125.0, 125.0)).is_some());
        assert!(tester.test_click(Vec2::new(50.0, 50.0)).is_none());
    }

    #[test]
    fn test_mouse_button_enum() {
        assert_eq!(MouseButton::Left, MouseButton::Left);
        assert_ne!(MouseButton::Left, MouseButton::Right);
    }
}
