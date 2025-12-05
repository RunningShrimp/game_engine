//! Unified Command/Event Protocol
//!
//! This protocol defines a language-agnostic interface between
//! scripting languages and the engine core.

use serde::{Deserialize, Serialize};

/// Commands sent from scripts to the engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BindingCommand {
    // Entity Management
    SpawnEntity {
        components: Vec<ComponentData>,
    },
    DespawnEntity {
        entity_id: u64,
    },

    // Component Operations
    SetComponent {
        entity_id: u64,
        component: ComponentData,
    },
    GetComponent {
        entity_id: u64,
        component_type: String,
    },
    RemoveComponent {
        entity_id: u64,
        component_type: String,
    },

    // Transform
    SetPosition {
        entity_id: u64,
        x: f32,
        y: f32,
        z: f32,
    },
    SetRotation {
        entity_id: u64,
        x: f32,
        y: f32,
        z: f32,
        w: f32,
    },
    SetScale {
        entity_id: u64,
        x: f32,
        y: f32,
        z: f32,
    },

    // Audio
    PlaySound {
        name: String,
        path: String,
        volume: f32,
        looped: bool,
    },
    StopSound {
        name: String,
    },
    SetVolume {
        name: String,
        volume: f32,
    },

    // Input Query
    IsKeyPressed {
        key: String,
    },
    GetMousePosition,

    // Scene
    LoadScene {
        path: String,
    },

    // Resource
    LoadTexture {
        path: String,
    },
    LoadMesh {
        path: String,
    },

    // Custom
    Custom {
        name: String,
        data: String,
    },
}

/// Events sent from engine to scripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BindingEvent {
    // Lifecycle
    OnStart {
        entity_id: u64,
    },
    OnUpdate {
        entity_id: u64,
        delta_time: f32,
    },
    OnFixedUpdate {
        entity_id: u64,
        fixed_delta: f32,
    },
    OnDestroy {
        entity_id: u64,
    },

    // Physics
    OnCollisionEnter {
        entity_a: u64,
        entity_b: u64,
    },
    OnCollisionExit {
        entity_a: u64,
        entity_b: u64,
    },
    OnTriggerEnter {
        entity_a: u64,
        entity_b: u64,
    },
    OnTriggerExit {
        entity_a: u64,
        entity_b: u64,
    },

    // Input
    OnKeyDown {
        key: String,
    },
    OnKeyUp {
        key: String,
    },
    OnMouseDown {
        button: u8,
        x: f32,
        y: f32,
    },
    OnMouseUp {
        button: u8,
        x: f32,
        y: f32,
    },

    // Resource
    OnResourceLoaded {
        handle: u64,
        resource_type: String,
    },
    OnResourceFailed {
        path: String,
        error: String,
    },

    // Response to queries
    ComponentData {
        entity_id: u64,
        component: ComponentData,
    },
    MousePosition {
        x: f32,
        y: f32,
    },
    KeyState {
        key: String,
        pressed: bool,
    },

    // Custom
    Custom {
        name: String,
        data: String,
    },
}

/// Serializable component data for cross-language transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentData {
    Transform {
        position: [f32; 3],
        rotation: [f32; 4],
        scale: [f32; 3],
    },
    Sprite {
        color: [f32; 4],
        texture_id: u32,
        uv_offset: [f32; 2],
        uv_scale: [f32; 2],
    },
    Camera {
        is_active: bool,
        projection_type: String, // "perspective" or "orthographic"
        fov: f32,
        near: f32,
        far: f32,
    },
    RigidBody {
        body_type: String, // "dynamic", "static", "kinematic"
        mass: f32,
    },
    AudioSource {
        path: String,
        volume: f32,
        looped: bool,
        playing: bool,
    },
    Script {
        source: String,
    },
    Custom {
        type_name: String,
        json_data: String,
    },
}

/// Result type for binding operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BindingResult {
    Success,
    EntityId(u64),
    Component(ComponentData),
    Error(String),
    Value(String), // JSON-serialized value
}

/// Trait for language-specific binding adapters
pub trait BindingAdapter {
    /// Initialize the binding adapter
    fn init(&mut self);

    /// Execute a command from script
    fn execute_command(&mut self, cmd: BindingCommand) -> BindingResult;

    /// Dispatch an event to scripts
    fn dispatch_event(&mut self, event: BindingEvent);

    /// Poll for pending commands from scripts
    fn poll_commands(&mut self) -> Vec<BindingCommand>;

    /// Cleanup
    fn shutdown(&mut self);
}
