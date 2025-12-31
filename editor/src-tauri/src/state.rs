use std::sync::Mutex;

/// 应用状态
///
/// 存储引擎实例和编辑器状态
#[derive(Debug)]
pub struct AppState {
    /// 引擎句柄（使用UUID标识）
    pub engine_handle: Mutex<Option<uuid::Uuid>>,
    /// 是否正在播放场景
    pub is_playing: Mutex<bool>,
    /// 当前选中的实体ID
    pub selected_entity: Mutex<Option<u64>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            engine_handle: Mutex::new(None),
            is_playing: Mutex::new(false),
            selected_entity: Mutex::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
