use crate::render::wgpu::Instance;
use glam::Mat4;

// ============================================================================
// 声明式场景图 (Flutter-like Layer Tree)
// ============================================================================

/// 场景图层节点 - 支持嵌套变换、裁剪、透明度
#[derive(Clone, Debug, PartialEq)]
pub enum Layer {
    /// 容器节点，包含变换矩阵和子节点
    Container {
        transform: Mat4,
        children: Vec<Layer>,
    },
    /// 图片/网格渲染节点
    Picture {
        item: LayerItem,
    },
    /// 透明度节点
    Opacity {
        alpha: f32,
        child: Box<Layer>,
    },
    /// 裁剪矩形节点
    ClipRect {
        rect: [f32; 4], // x, y, width, height
        child: Box<Layer>,
    },
    /// 特效节点 (模糊、阴影等)
    Effect {
        effect_type: EffectType,
        child: Box<Layer>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum EffectType {
    Blur { radius: f32 },
    DropShadow { offset: [f32; 2], blur: f32, color: [f32; 4] },
    ColorMatrix([f32; 20]), // 4x5 color matrix
}

/// 渲染命令差异类型
#[derive(Clone, Debug)]
pub enum DiffCommand {
    Insert { index: usize, layer: Layer },
    Remove { index: usize },
    Update { index: usize, old: Layer, new: Layer },
    Move { from: usize, to: usize },
}

impl Layer {
    /// 计算两棵树的差异
    pub fn diff(old: &[Layer], new: &[Layer]) -> Vec<DiffCommand> {
        let mut commands = Vec::new();
        Self::diff_recursive(old, new, &mut commands, 0);
        commands
    }
    
    fn diff_recursive(
        old: &[Layer],
        new: &[Layer],
        commands: &mut Vec<DiffCommand>,
        base_index: usize,
    ) {
        let mut old_idx = 0;
        let mut new_idx = 0;
        
        while old_idx < old.len() || new_idx < new.len() {
            match (old.get(old_idx), new.get(new_idx)) {
                (Some(o), Some(n)) => {
                    if std::mem::discriminant(o) == std::mem::discriminant(n) {
                        // 同类型节点，检查是否需要更新
                        if o != n {
                            commands.push(DiffCommand::Update {
                                index: base_index + new_idx,
                                old: o.clone(),
                                new: n.clone(),
                            });
                        }
                        // 递归处理子节点
                        if let (Layer::Container { children: oc, .. }, 
                                Layer::Container { children: nc, .. }) = (o, n) {
                            Self::diff_recursive(oc, nc, commands, 0);
                        }
                    } else {
                        // 类型不同，删除旧的，插入新的
                        commands.push(DiffCommand::Remove { index: base_index + old_idx });
                        commands.push(DiffCommand::Insert {
                            index: base_index + new_idx,
                            layer: n.clone(),
                        });
                    }
                    old_idx += 1;
                    new_idx += 1;
                }
                (Some(_), None) => {
                    // 旧节点多余，删除
                    commands.push(DiffCommand::Remove { index: base_index + old_idx });
                    old_idx += 1;
                }
                (None, Some(n)) => {
                    // 新节点，插入
                    commands.push(DiffCommand::Insert {
                        index: base_index + new_idx,
                        layer: n.clone(),
                    });
                    new_idx += 1;
                }
                (None, None) => break,
            }
        }
    }
    
    /// 展开为扁平的渲染项列表
    pub fn flatten(&self, parent_transform: Mat4, parent_alpha: f32, items: &mut Vec<FlattenedItem>) {
        match self {
            Layer::Container { transform, children } => {
                let global_transform = parent_transform * *transform;
                for child in children {
                    child.flatten(global_transform, parent_alpha, items);
                }
            }
            Layer::Picture { item } => {
                let mut flat_item = FlattenedItem {
                    item: item.clone(),
                    global_transform: parent_transform,
                    alpha: parent_alpha,
                    clip: None,
                };
                // 应用变换到位置
                let pos = parent_transform.transform_point3(glam::vec3(item.pos[0], item.pos[1], 0.0));
                flat_item.item.pos = [pos.x, pos.y];
                flat_item.item.color[3] *= parent_alpha;
                items.push(flat_item);
            }
            Layer::Opacity { alpha, child } => {
                child.flatten(parent_transform, parent_alpha * alpha, items);
            }
            Layer::ClipRect { rect, child } => {
                // TODO: 传递裁剪区域到子节点
                child.flatten(parent_transform, parent_alpha, items);
            }
            Layer::Effect { effect_type: _, child } => {
                // TODO: 特效处理需要离屏渲染
                child.flatten(parent_transform, parent_alpha, items);
            }
        }
    }
}

/// 展平后的渲染项
#[derive(Clone, Debug)]
pub struct FlattenedItem {
    pub item: LayerItem,
    pub global_transform: Mat4,
    pub alpha: f32,
    pub clip: Option<[f32; 4]>,
}

// ============================================================================
// 原有的 LayerItem (保持向后兼容)
// ============================================================================

#[derive(Clone, Debug)]
pub struct LayerItem { 
    pub pos:[f32;2], 
    pub scale:[f32;2], 
    pub rot:f32, 
    pub color:[f32;4], 
    pub uv_off:[f32;2], 
    pub uv_scale:[f32;2], 
    pub tex:u32, 
    pub normal_tex: u32,
    pub layer:f32, 
    pub target:u32 
}

impl PartialEq for LayerItem {
    fn eq(&self, other: &Self) -> bool {
        let eps = 0.0001;
        (self.pos[0] - other.pos[0]).abs() < eps &&
        (self.pos[1] - other.pos[1]).abs() < eps &&
        (self.scale[0] - other.scale[0]).abs() < eps &&
        (self.scale[1] - other.scale[1]).abs() < eps &&
        (self.rot - other.rot).abs() < eps &&
        self.color == other.color && 
        self.uv_off == other.uv_off &&
        self.uv_scale == other.uv_scale &&
        self.tex == other.tex &&
        self.normal_tex == other.normal_tex &&
        (self.layer - other.layer).abs() < eps &&
        self.target == other.target
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct LayerTree { pub items: Vec<LayerItem> }

impl LayerTree {
    pub fn add(&mut self, it: LayerItem) { self.items.push(it); }
    pub fn to_instances(&self) -> Vec<Instance> {
        let mut out: Vec<Instance> = self.items.iter().map(|it| Instance { 
            pos: it.pos, 
            scale: it.scale, 
            rot: it.rot, 
            target: it.target, 
            color: it.color, 
            uv_offset: it.uv_off, 
            uv_scale: it.uv_scale, 
            layer: it.layer, 
            tex_index: it.tex, 
            normal_tex_index: it.normal_tex,
            msdf:0.0, 
            px_range:0.0,
            _pad: [0.0, 0.0, 0.0]
        }).collect();
        out.sort_by(|a,b| {
            match a.target.cmp(&b.target) {
                std::cmp::Ordering::Equal => a.layer.partial_cmp(&b.layer).unwrap_or(std::cmp::Ordering::Equal),
                other => other,
            }
        });
        out
    }
}

#[derive(Clone)]
pub enum Target { Main, Ui, Offscreen(u32) }

#[derive(Clone)]
pub enum RenderCommand { SetTarget(Target), Draw { start:u32, end:u32, tex_idx:usize, scissor:Option<[u32;4]> }, DrawUi { count:u32 } }

#[derive(Default, Clone)]
pub struct RenderGraph { pub commands: Vec<RenderCommand> }

pub fn build_commands(instances: &[Instance]) -> RenderGraph {
    let mut g = RenderGraph::default();
    if instances.is_empty() { return g; }
    
    let mut cur_target = instances[0].target;
    g.commands.push(if cur_target == 0 { RenderCommand::SetTarget(Target::Main) } else { RenderCommand::SetTarget(Target::Offscreen(cur_target)) });
    
    let mut start = 0u32;
    let mut cur_tex = instances[0].tex_index as usize;
    let mut cur_layer = instances[0].layer;
    
    for (i, inst) in instances.iter().enumerate() {
        let ti = inst.tex_index as usize;
        if inst.target != cur_target {
            g.commands.push(RenderCommand::Draw { start, end: i as u32, tex_idx: cur_tex, scissor: None });
            cur_target = inst.target;
            g.commands.push(if cur_target == 0 { RenderCommand::SetTarget(Target::Main) } else { RenderCommand::SetTarget(Target::Offscreen(cur_target)) });
            start = i as u32;
            cur_tex = ti; cur_layer = inst.layer;
        } else if ti != cur_tex || (inst.layer - cur_layer).abs() > f32::EPSILON {
            g.commands.push(RenderCommand::Draw { start, end: i as u32, tex_idx: cur_tex, scissor: None });
            start = i as u32;
            cur_tex = ti; cur_layer = inst.layer;
        }
    }
    g.commands.push(RenderCommand::Draw { start, end: instances.len() as u32, tex_idx: cur_tex, scissor: None });
    g
}

pub fn build_from_world(world: &mut bevy_ecs::world::World) -> LayerTree {

    use crate::ecs::{Transform, PreviousTransform, Sprite, Time, TileMap};
    let mut lt = LayerTree::default();
    
    let time = world.get_resource::<Time>().unwrap();
    let alpha = time.alpha as f32;

    // Sprites
    let mut query = world.query::<(&Transform, Option<&PreviousTransform>, &Sprite)>();
    for (t, pt, s) in query.iter(world) { 
        let (pos, scale, rot) = if let Some(prev) = pt {
            (
                prev.pos.lerp(t.pos, alpha),
                prev.scale.lerp(t.scale, alpha),
                prev.rot.slerp(t.rot, alpha)
            )
        } else {
            (t.pos, t.scale, t.rot)
        };

        lt.add(LayerItem { 
            pos: [pos.x, pos.y], 
            scale: [scale.x, scale.y], 
            rot: rot.to_euler(glam::EulerRot::XYZ).2, 
            color: s.color, 
            uv_off: s.uv_off, 
            uv_scale: s.uv_scale, 
            tex: s.tex_index, 
            normal_tex: s.normal_tex_index,
            layer: s.layer,
            target: 0 
        }); 
    }

    // TileMaps
    let mut query_tm = world.query::<(&Transform, &TileMap)>();
    for (t, tm) in query_tm.iter(world) {
        let start_x = t.pos.x - (tm.width as f32 * tm.tile_size[0]) / 2.0;
        let start_y = t.pos.y - (tm.height as f32 * tm.tile_size[1]) / 2.0;
        
        for y in 0..tm.height {
            for x in 0..tm.width {
                let idx = (y * tm.width + x) as usize;
                if idx < tm.tiles.len() {
                    // TileMap expansion to LayerTree currently handled in tilemap_build_system by spawning sprites
                }
            }
        }
    }

    lt
}

pub struct RenderCache {
    pub last_tree: Option<LayerTree>,
    pub last_instances: Vec<Instance>,
}

impl Default for RenderCache {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderCache {
    pub fn new() -> Self { Self { last_tree: None, last_instances: Vec::new() } }
    
    pub fn update(&mut self, new_tree: LayerTree) -> &Vec<Instance> {
        if let Some(last) = &self.last_tree {
            if last == &new_tree {
                return &self.last_instances;
            }
        }
        self.last_instances = new_tree.to_instances();
        self.last_tree = Some(new_tree);
        &self.last_instances
    }
}
