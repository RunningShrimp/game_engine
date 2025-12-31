/// 编辑器事件
///
/// 定义编辑器中可能发生的事件类型
#[derive(Debug, Clone)]
pub enum EditorEvent {
    /// 实体创建事件
    EntityCreated { entity_id: u64, name: String },
    /// 实体删除事件
    EntityDeleted { entity_id: u64 },
    /// 实体选择事件
    EntitySelected { entity_id: Option<u64> },
    /// 组件更新事件
    ComponentUpdated {
        entity_id: u64,
        component_type: String,
    },
    /// 场景加载事件
    SceneLoaded { scene_name: String },
    /// 场景保存事件
    SceneSaved { scene_name: String },
    /// 播放状态改变事件
    PlaybackStateChanged { is_playing: bool },
    /// 资源导入事件
    AssetImported { asset_path: String },
    /// 错误事件
    Error { message: String },
}

/// 事件监听器trait
pub trait EventListener {
    fn on_event(&self, event: &EditorEvent);
}
