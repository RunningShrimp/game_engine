//  编辑器接口抽象
//
//  定义编辑器相关的trait，使core模块不直接依赖editor模块，
//  消除core <-> editor循环依赖。

use winit::event::WindowEvent;

/// 编辑器事件处理接口
///
/// 抽象编辑器的事件处理功能，使core模块可以不依赖具体的EditorContext实现。
pub trait EditorEventHandler {
    /// 处理窗口事件
    ///
    /// # 参数
    ///
    /// * `event` - 窗口事件
    ///
    /// # 返回
    ///
    /// 如果事件被编辑器消费则返回true
    fn handle_window_event(&mut self, event: &WindowEvent) -> bool;
}

// 为EditorContext实现这个trait的impl应该在editor模块中
// 这样core模块只需要知道trait，不需要知道具体的EditorContext类型
