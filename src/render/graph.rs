use crate::ecs::{TileChunkConfig, TileSet, Viewport};
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
    Picture { item: LayerItem },
    /// 透明度节点
    Opacity { alpha: f32, child: Box<Layer> },
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
    Blur {
        radius: f32,
    },
    DropShadow {
        offset: [f32; 2],
        blur: f32,
        color: [f32; 4],
    },
    ColorMatrix([f32; 20]), // 4x5 color matrix
}

/// 渲染命令差异类型
#[derive(Clone, Debug)]
pub enum DiffCommand {
    Insert {
        index: usize,
        layer: Layer,
    },
    Remove {
        index: usize,
    },
    Update {
        index: usize,
        old: Layer,
        new: Layer,
    },
    Move {
        from: usize,
        to: usize,
    },
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
                        if let (
                            Layer::Container { children: oc, .. },
                            Layer::Container { children: nc, .. },
                        ) = (o, n)
                        {
                            Self::diff_recursive(oc, nc, commands, 0);
                        }
                    } else {
                        // 类型不同，删除旧的，插入新的
                        commands.push(DiffCommand::Remove {
                            index: base_index + old_idx,
                        });
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
                    commands.push(DiffCommand::Remove {
                        index: base_index + old_idx,
                    });
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
    pub fn flatten(
        &self,
        parent_transform: Mat4,
        parent_alpha: f32,
        items: &mut Vec<FlattenedItem>,
    ) {
        match self {
            Layer::Container {
                transform,
                children,
            } => {
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
                let pos =
                    parent_transform.transform_point3(glam::vec3(item.pos[0], item.pos[1], 0.0));
                flat_item.item.pos = [pos.x, pos.y];
                flat_item.item.color[3] *= parent_alpha;
                items.push(flat_item);
            }
            Layer::Opacity { alpha, child } => {
                child.flatten(parent_transform, parent_alpha * alpha, items);
            }
            Layer::ClipRect { rect, child } => {
                // 应用裁剪区域：将rect转换为裁剪变换矩阵
                // rect: [x, y, width, height]
                let scale = glam::Vec3::new(rect[2], rect[3], 1.0);
                let translation = glam::Vec3::new(rect[0], rect[1], 0.0);
                let clip_transform = glam::Mat4::from_scale_rotation_translation(
                    scale,
                    glam::Quat::IDENTITY,
                    translation,
                );
                let clipped_transform = parent_transform * clip_transform;
                child.flatten(clipped_transform, parent_alpha, items);
            }
            Layer::Effect {
                effect_type: _,
                child,
            } => {
                // NOTE: 特效处理需要离屏渲染，当前使用简化实现
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
    pub pos: [f32; 2],
    pub scale: [f32; 2],
    pub rot: f32,
    pub color: [f32; 4],
    pub uv_off: [f32; 2],
    pub uv_scale: [f32; 2],
    pub tex: u32,
    pub normal_tex: u32,
    pub layer: f32,
    pub target: u32,
    pub chunk: u32,
}

impl PartialEq for LayerItem {
    fn eq(&self, other: &Self) -> bool {
        let eps = 0.0001;
        (self.pos[0] - other.pos[0]).abs() < eps
            && (self.pos[1] - other.pos[1]).abs() < eps
            && (self.scale[0] - other.scale[0]).abs() < eps
            && (self.scale[1] - other.scale[1]).abs() < eps
            && (self.rot - other.rot).abs() < eps
            && self.color == other.color
            && self.uv_off == other.uv_off
            && self.uv_scale == other.uv_scale
            && self.tex == other.tex
            && self.normal_tex == other.normal_tex
            && (self.layer - other.layer).abs() < eps
            && self.target == other.target
            && self.chunk == other.chunk
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct LayerTree {
    pub items: Vec<LayerItem>,
}

impl LayerTree {
    pub fn add(&mut self, it: LayerItem) {
        self.items.push(it);
    }
    pub fn to_instances(&self) -> Vec<Instance> {
        let mut out: Vec<Instance> = self
            .items
            .iter()
            .map(|it| Instance {
                pos: it.pos,
                scale: it.scale,
                rot: it.rot,
                target: it.target,
                chunk: it.chunk,
                color: it.color,
                uv_offset: it.uv_off,
                uv_scale: it.uv_scale,
                layer: it.layer,
                tex_index: it.tex,
                normal_tex_index: it.normal_tex,
                msdf: 0.0,
                px_range: 0.0,
            })
            .collect();
        out.sort_by(|a, b| match a.target.cmp(&b.target) {
            std::cmp::Ordering::Equal => match a.tex_index.cmp(&b.tex_index) {
                std::cmp::Ordering::Equal => match a
                    .layer
                    .partial_cmp(&b.layer)
                    .unwrap_or(std::cmp::Ordering::Equal)
                {
                    std::cmp::Ordering::Equal => a.chunk.cmp(&b.chunk),
                    other => other,
                },
                other => other,
            },
            other => other,
        });
        out
    }
}

#[derive(Clone)]
pub enum Target {
    Main,
    Ui,
    Offscreen(u32),
}

#[derive(Clone)]
pub enum RenderCommand {
    SetTarget(Target),
    Draw {
        start: u32,
        end: u32,
        tex_idx: usize,
        scissor: Option<[u32; 4]>,
    },
    DrawUi {
        count: u32,
    },
}

#[derive(Default, Clone)]
pub struct RenderGraph {
    pub commands: Vec<RenderCommand>,
}

pub fn build_commands(instances: &[Instance]) -> RenderGraph {
    let mut g = RenderGraph::default();
    if instances.is_empty() {
        return g;
    }

    let mut cur_target = instances[0].target;
    g.commands.push(if cur_target == 0 {
        RenderCommand::SetTarget(Target::Main)
    } else {
        RenderCommand::SetTarget(Target::Offscreen(cur_target))
    });

    let mut start = 0u32;
    let mut cur_tex = instances[0].tex_index as usize;
    let mut cur_layer = instances[0].layer;
    let mut cur_chunk = instances[0].chunk;

    for (i, inst) in instances.iter().enumerate() {
        let ti = inst.tex_index as usize;
        if inst.target != cur_target {
            g.commands.push(RenderCommand::Draw {
                start,
                end: i as u32,
                tex_idx: cur_tex,
                scissor: None,
            });
            cur_target = inst.target;
            g.commands.push(if cur_target == 0 {
                RenderCommand::SetTarget(Target::Main)
            } else {
                RenderCommand::SetTarget(Target::Offscreen(cur_target))
            });
            start = i as u32;
            cur_tex = ti;
            cur_layer = inst.layer;
            cur_chunk = inst.chunk;
        } else if ti != cur_tex
            || (inst.layer - cur_layer).abs() > f32::EPSILON
            || inst.chunk != cur_chunk
        {
            g.commands.push(RenderCommand::Draw {
                start,
                end: i as u32,
                tex_idx: cur_tex,
                scissor: None,
            });
            start = i as u32;
            cur_tex = ti;
            cur_layer = inst.layer;
            cur_chunk = inst.chunk;
        }
    }
    g.commands.push(RenderCommand::Draw {
        start,
        end: instances.len() as u32,
        tex_idx: cur_tex,
        scissor: None,
    });
    g
}

pub fn build_from_world(world: &mut bevy_ecs::world::World) -> LayerTree {
    use crate::ecs::{PreviousTransform, Sprite, TileMap, Time, Transform};
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
                prev.rot.slerp(t.rot, alpha),
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
            target: 0,
            chunk: 0,
        });
    }

    // TileMaps
    let mut query_tm = world.query::<(&Transform, &TileMap)>();
    let tileset = world.get_resource::<TileSet>().cloned();
    let vp = world.get_resource::<Viewport>().copied();
    let chunk_cfg = world.get_resource::<TileChunkConfig>().copied();
    let mut cam_q = world.query::<(&crate::ecs::Transform, &crate::ecs::Camera)>();
    let mut cam_pos = glam::Vec3::new(
        vp.map(|v| v.width as f32 * 0.5).unwrap_or(400.0),
        vp.map(|v| v.height as f32 * 0.5).unwrap_or(300.0),
        0.0,
    );
    for (t, c) in cam_q.iter(world) {
        if c.is_active {
            cam_pos = t.pos;
            break;
        }
    }
    for (t, tm) in query_tm.iter(world) {
        let (vpw, vph) = vp
            .map(|v| (v.width as f32, v.height as f32))
            .unwrap_or((800.0, 600.0));
        let half_w = vpw * 0.5;
        let half_h = vph * 0.5;
        let base_x = t.pos.x - (tm.width as f32 * tm.tile_size[0]) * 0.5;
        let base_y = t.pos.y - (tm.height as f32 * tm.tile_size[1]) * 0.5;
        let view_min_x = cam_pos.x - half_w;
        let view_max_x = cam_pos.x + half_w;
        let view_min_y = cam_pos.y - half_h;
        let view_max_y = cam_pos.y + half_h;
        let cfg_w = chunk_cfg.map(|c| c.size[0]).unwrap_or(0);
        let cfg_h = chunk_cfg.map(|c| c.size[1]).unwrap_or(0);
        let chunk_w = if cfg_w != 0 {
            cfg_w
        } else if tm.chunk_size[0] == 0 {
            16
        } else {
            tm.chunk_size[0]
        };
        let chunk_h = if cfg_h != 0 {
            cfg_h
        } else if tm.chunk_size[1] == 0 {
            16
        } else {
            tm.chunk_size[1]
        };
        let chunk_cols = (tm.width + chunk_w - 1) / chunk_w;
        for y in 0..tm.height {
            for x in 0..tm.width {
                let idx = (y * tm.width + x) as usize;
                if idx >= tm.tiles.len() {
                    continue;
                }
                let id = &tm.tiles[idx];
                if id.is_empty() {
                    continue;
                }
                if let Some(ts) = tileset.as_ref() {
                    if let Some((uv_off, uv_scale)) = ts.tiles.get(id).cloned() {
                        let px = base_x + (x as f32 + 0.5) * tm.tile_size[0];
                        let py = base_y + (y as f32 + 0.5) * tm.tile_size[1];
                        if px < view_min_x - tm.tile_size[0]
                            || py < view_min_y - tm.tile_size[1]
                            || px > view_max_x + tm.tile_size[0]
                            || py > view_max_y + tm.tile_size[1]
                        {
                            continue;
                        }
                        let cx = x / chunk_w;
                        let cy = y / chunk_h;
                        let chunk_id = cy * chunk_cols + cx;
                        lt.add(LayerItem {
                            pos: [px, py],
                            scale: [tm.tile_size[0], tm.tile_size[1]],
                            rot: 0.0,
                            color: [1.0, 1.0, 1.0, 1.0],
                            uv_off,
                            uv_scale,
                            tex: tm.atlas_tex_index,
                            normal_tex: 0,
                            layer: tm.layer,
                            target: 0,
                            chunk: chunk_id,
                        });
                    }
                }
            }
        }
    }

    lt
}

#[derive(Default)]
pub struct RenderCache {
    pub last_tree: Option<LayerTree>,
    pub last_instances: Vec<Instance>,
    pub culled_count: u32,
    pub total_count: u32,
}

impl RenderCache {
    pub fn new() -> Self {
        Self::default()
    }

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

    /// 获取剔除统计信息
    pub fn culling_stats(&self) -> (u32, u32) {
        (self.culled_count, self.total_count)
    }
}

/// 2D视口剔除器
#[derive(Clone, Debug)]
pub struct ViewportCuller {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub margin: f32,
}

impl ViewportCuller {
    /// 从视口和相机位置创建剔除器
    pub fn new(
        viewport_width: f32,
        viewport_height: f32,
        camera_pos: glam::Vec3,
        margin: f32,
    ) -> Self {
        let half_w = viewport_width * 0.5;
        let half_h = viewport_height * 0.5;
        Self {
            min_x: camera_pos.x - half_w - margin,
            max_x: camera_pos.x + half_w + margin,
            min_y: camera_pos.y - half_h - margin,
            max_y: camera_pos.y + half_h + margin,
            margin,
        }
    }

    /// 检查2D对象是否在视口内
    #[inline]
    pub fn is_visible(&self, x: f32, y: f32, half_width: f32, half_height: f32) -> bool {
        x + half_width >= self.min_x
            && x - half_width <= self.max_x
            && y + half_height >= self.min_y
            && y - half_height <= self.max_y
    }

    /// 检查点是否在视口内
    #[inline]
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }
}

/// 带视锥剔除的世界构建函数
pub fn build_from_world_culled(world: &mut bevy_ecs::world::World) -> (LayerTree, u32, u32) {
    use crate::ecs::{PreviousTransform, Sprite, TileMap, Time, Transform};
    let mut lt = LayerTree::default();
    let mut culled_count = 0u32;
    let mut total_count = 0u32;

    let time = world.get_resource::<Time>().unwrap();
    let alpha = time.alpha as f32;

    // 获取视口信息
    let vp = world.get_resource::<Viewport>().copied();
    let (vpw, vph) = vp
        .map(|v| (v.width as f32, v.height as f32))
        .unwrap_or((800.0, 600.0));

    // 获取相机位置
    let mut cam_q = world.query::<(&crate::ecs::Transform, &crate::ecs::Camera)>();
    let mut cam_pos = glam::Vec3::new(vpw * 0.5, vph * 0.5, 0.0);
    for (t, c) in cam_q.iter(world) {
        if c.is_active {
            cam_pos = t.pos;
            break;
        }
    }

    // 创建视口剔除器，添加100像素的margin防止边缘闪烁
    let culler = ViewportCuller::new(vpw, vph, cam_pos, 100.0);

    // Sprites with culling
    let mut query = world.query::<(&Transform, Option<&PreviousTransform>, &Sprite)>();
    for (t, pt, s) in query.iter(world) {
        total_count += 1;

        let (pos, scale, rot) = if let Some(prev) = pt {
            (
                prev.pos.lerp(t.pos, alpha),
                prev.scale.lerp(t.scale, alpha),
                prev.rot.slerp(t.rot, alpha),
            )
        } else {
            (t.pos, t.scale, t.rot)
        };

        // 视锥剔除检查
        let half_w = scale.x * 0.5;
        let half_h = scale.y * 0.5;
        if !culler.is_visible(pos.x, pos.y, half_w, half_h) {
            culled_count += 1;
            continue;
        }

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
            target: 0,
            chunk: 0,
        });
    }

    // TileMaps (已有视口剔除)
    let mut query_tm = world.query::<(&Transform, &TileMap)>();
    let tileset = world.get_resource::<TileSet>().cloned();
    let chunk_cfg = world.get_resource::<TileChunkConfig>().copied();

    for (t, tm) in query_tm.iter(world) {
        let base_x = t.pos.x - (tm.width as f32 * tm.tile_size[0]) * 0.5;
        let base_y = t.pos.y - (tm.height as f32 * tm.tile_size[1]) * 0.5;
        let cfg_w = chunk_cfg.map(|c| c.size[0]).unwrap_or(0);
        let cfg_h = chunk_cfg.map(|c| c.size[1]).unwrap_or(0);
        let chunk_w = if cfg_w != 0 {
            cfg_w
        } else if tm.chunk_size[0] == 0 {
            16
        } else {
            tm.chunk_size[0]
        };
        let chunk_h = if cfg_h != 0 {
            cfg_h
        } else if tm.chunk_size[1] == 0 {
            16
        } else {
            tm.chunk_size[1]
        };
        let chunk_cols = (tm.width + chunk_w - 1) / chunk_w;

        for y in 0..tm.height {
            for x in 0..tm.width {
                total_count += 1;
                let idx = (y * tm.width + x) as usize;
                if idx >= tm.tiles.len() {
                    continue;
                }
                let id = &tm.tiles[idx];
                if id.is_empty() {
                    continue;
                }
                if let Some(ts) = tileset.as_ref() {
                    if let Some((uv_off, uv_scale)) = ts.tiles.get(id).cloned() {
                        let px = base_x + (x as f32 + 0.5) * tm.tile_size[0];
                        let py = base_y + (y as f32 + 0.5) * tm.tile_size[1];

                        // 使用视口剔除器
                        let half_tile_w = tm.tile_size[0] * 0.5;
                        let half_tile_h = tm.tile_size[1] * 0.5;
                        if !culler.is_visible(px, py, half_tile_w, half_tile_h) {
                            culled_count += 1;
                            continue;
                        }

                        let cx = x / chunk_w;
                        let cy = y / chunk_h;
                        let chunk_id = cy * chunk_cols + cx;
                        lt.add(LayerItem {
                            pos: [px, py],
                            scale: [tm.tile_size[0], tm.tile_size[1]],
                            rot: 0.0,
                            color: [1.0, 1.0, 1.0, 1.0],
                            uv_off,
                            uv_scale,
                            tex: tm.atlas_tex_index,
                            normal_tex: 0,
                            layer: tm.layer,
                            target: 0,
                            chunk: chunk_id,
                        });
                    }
                }
            }
        }
    }

    (lt, culled_count, total_count)
}
