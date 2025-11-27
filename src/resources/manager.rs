use std::{path::{Path, PathBuf}, sync::{Arc, RwLock}, time::Duration};
use bevy_ecs::prelude::*;
// use crossbeam_channel:: {unbounded, Receiver, Sender};
use tokio::sync::mpsc;
use tokio::sync::oneshot;
// use futures::future::FutureExt;
use crate::render::wgpu::WgpuRenderer;
use super::atlas::Atlas;
// use super::runtime::global_runtime;
use std::collections::HashMap;

// --- GLTF Support ---

#[derive(Clone, Debug)]
pub struct GltfScene {
    pub data: Arc<(gltf::Document, Vec<gltf::buffer::Data>, Vec<gltf::image::Data>)>,
    pub json: Option<serde_json::Value>,
}

// --- Handle System ---

#[derive(Clone, Debug)]
pub enum LoadState<T> {
    Loading,
    Loaded(T),
    Failed(String),
}

#[derive(Debug)]
pub struct AssetContainer<T> {
    pub state: RwLock<LoadState<T>>,
}

#[derive(Clone, Component, Debug)]
pub struct Handle<T: 'static + Send + Sync> {
    pub container: Arc<AssetContainer<T>>, 
}

impl<T: 'static + Send + Sync> Handle<T> {
    pub fn new_loading() -> Self {
        Self {
            container: Arc::new(AssetContainer {
                state: RwLock::new(LoadState::Loading),
            }),
        }
    }

    pub fn get(&self) -> Option<T> where T: Clone {
        self.container.state.read()
            .ok()  // ✅ 处理锁中毒情况
            .and_then(|state| match &*state {
                LoadState::Loaded(v) => Some(v.clone()),
                _ => None,
            })
    }
    
    pub fn is_loaded(&self) -> bool {
        self.container.state.read()
            .ok()  // ✅ 处理锁中毒情况
            .map(|state| matches!(*state, LoadState::Loaded(_)))
            .unwrap_or(false)  // ✅ 锁中毒时返回false
    }

    // Removed get_ref method as it couldn't be implemented safely due to lifetime issues

    /// 非阻塞方式获取资源，立即返回结果
    pub fn get_non_blocking(&self) -> Option<T> where T: Clone {
        self.container.state.try_read()
            .ok()  // ✅ 处理锁中毒情况
            .and_then(|state| match &*state {
                LoadState::Loaded(v) => Some(v.clone()),
                _ => None,
            })
    }

    /// 带超时的资源获取，在指定时间内尝试获取资源
    pub fn get_with_timeout(&self, timeout: Duration) -> Option<T> where T: Clone {
        let start = std::time::Instant::now();

        // 先尝试直接获取，也许正好资源已准备好
        if let Some(result) = self.get_non_blocking() {
            return Some(result);
        }

        // 自适应等待策略：初始短等待，逐渐增加等待时间
        let mut wait_time = Duration::from_micros(100); // 初始100微秒
        let max_wait_time = Duration::from_millis(10);   // 最大10毫秒

        while start.elapsed() < timeout {
            std::thread::sleep(wait_time);

            if let Some(result) = self.get_non_blocking() {
                return Some(result);
            }

            // 指数退避，但不超过最大等待时间
            wait_time = (wait_time * 2).min(max_wait_time);
        }

        // 超时后最后一次尝试
        self.get_non_blocking()
    }

    /// 阻塞等待资源加载完成（注意：这会阻塞当前线程）
    pub fn get_blocking(&self) -> Option<T> where T: Clone {
        loop {
            match self.container.state.read() {
                Ok(state) => match &*state {
                    LoadState::Loaded(v) => return Some(v.clone()),
                    LoadState::Failed(_) => return None,
                    LoadState::Loading => {} // 继续等待
                },
                Err(_) => {
                    // 锁中毒，返回None但不panic
                    return None;
                }
            }
            // 短暂休眠避免CPU占用过高
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// 获取资源状态信息（安全的元数据获取）
    pub fn get_status(&self) -> Result<String, &'static str> {
        self.container.state.read()
            .map_err(|_| "Lock poisoned")
            .map(|state| match &*state {
                LoadState::Loading => "loading".to_string(),
                LoadState::Loaded(_) => "loaded".to_string(),
                LoadState::Failed(err) => format!("failed: {}", err),
            })
    }
}

// --- Asset Server ---

enum AssetTask {
    Texture { path: PathBuf, handle: Handle<u32>, is_linear: bool, start: std::time::Instant },
    Atlas { path: PathBuf, handle: Handle<Atlas>, start: std::time::Instant },
    Gltf { path: PathBuf, handle: Handle<GltfScene>, start: std::time::Instant },
}

pub enum AssetResult {
    Bytes(Vec<u8>),
    Image(image::RgbaImage),
    Gltf(GltfScene),
}

#[derive(Resource)]
pub struct AssetServer {
    tx: mpsc::UnboundedSender<AssetTask>,
    rx: mpsc::UnboundedReceiver<(AssetTask, Result<AssetResult, String>)>,
    worker_handle: Option<std::thread::JoinHandle<()>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

#[derive(Clone, Debug)]
pub enum AssetEvent {
    TextureLoaded(Handle<u32>, f32),
    AtlasLoaded(Handle<Atlas>, f32),
    GltfLoaded(Handle<GltfScene>, f32),
    TextureFailed(Handle<u32>, String),
    AtlasFailed(Handle<Atlas>, String),
    GltfFailed(Handle<GltfScene>, String),
}

impl Default for AssetServer {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetServer {
    pub fn new() -> Self {
        let (task_tx, task_rx) = mpsc::unbounded_channel::<AssetTask>();
        let (done_tx, done_rx) = mpsc::unbounded_channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let worker_handle = std::thread::Builder::new()
            .name("asset-loader".to_string())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create asset loader runtime");

                rt.block_on(async move {
                    let mut shutdown_rx = shutdown_rx;
                    let mut task_rx = task_rx;

                    loop {
                        tokio::select! {
                            _ = &mut shutdown_rx => {
                                log::info!("Asset loader received shutdown signal");
                                break;
                            }
                            task = task_rx.recv() => {
                                match task {
                                    Some(task) => {
                                        let tx = done_tx.clone();
                                        tokio::spawn(async move {
                                            let result = match &task {
                                                AssetTask::Texture { path, .. } => {
                                                    match tokio::fs::read(path).await {
                                                        Ok(bytes) => {
                                                            // Decode in blocking task
                                                            let decode_res = tokio::task::spawn_blocking(move || {
                                                                image::load_from_memory(&bytes)
                                                                    .map(|img| AssetResult::Image(img.to_rgba8()))
                                                                    .map_err(|e| e.to_string())
                                                            }).await;

                                                            match decode_res {
                                                                Ok(res) => res,
                                                                Err(e) => Err(e.to_string()),
                                                            }
                                                        },
                                                        Err(e) => Err(e.to_string()),
                                                    }
                                                },
                                                AssetTask::Atlas { path, .. } => {
                                                    tokio::fs::read(path).await
                                                        .map(AssetResult::Bytes)
                                                        .map_err(|e| e.to_string())
                                                },
                                                AssetTask::Gltf { path, .. } => {
                                                    match tokio::fs::read(path).await {
                                                        Ok(bytes) => {
                                                            let bytes_for_import = bytes.clone();
                                                            let load_res = tokio::task::spawn_blocking(move || {
                                                                gltf::import_slice(&bytes_for_import)
                                                            }).await;

                                                            match load_res {
                                                                Ok(Ok(data)) => {
                                                                    // 尝试解析 JSON（.gltf），GLB 会失败后回退 None
                                                                    let json = String::from_utf8(bytes.clone())
                                                                        .ok()
                                                                        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());
                                                                    Ok(AssetResult::Gltf(GltfScene { data: Arc::new(data), json }))
                                                                    },
                                                                    Ok(Err(e)) => Err(e.to_string()),
                                                                    Err(e) => Err(e.to_string()),
                                                                }
                                                                                                                 },
                                                        Err(e) => Err(e.to_string()),
                                                    }
                                                },
                                            };

                                            let _ = tx.send((task, result));
                                        });
                                    }
                                    None => {
                                        log::info!("Asset task channel closed");
                                        break;
                                    }
                                }
                            }
                        }
                    }
                });
            })
            .expect("Failed to spawn asset loader thread");

        Self {
            tx: task_tx,
            rx: done_rx,
            worker_handle: Some(worker_handle),
            shutdown_tx: Some(shutdown_tx),
        }
    }

    pub fn load_texture(&self, path: &Path) -> Handle<u32> {
        let handle = Handle::new_loading();
        let task = AssetTask::Texture { path: path.to_path_buf(), handle: handle.clone(), is_linear: false, start: std::time::Instant::now() };
        let _ = self.tx.send(task);
        handle
    }

    pub fn load_texture_linear(&self, path: &Path) -> Handle<u32> {
        let handle = Handle::new_loading();
        let task = AssetTask::Texture { path: path.to_path_buf(), handle: handle.clone(), is_linear: true, start: std::time::Instant::now() };
        let _ = self.tx.send(task);
        handle
    }

    pub fn load_atlas(&self, path: &Path) -> Handle<Atlas> {
        let handle = Handle::new_loading();
        let task = AssetTask::Atlas { path: path.to_path_buf(), handle: handle.clone(), start: std::time::Instant::now() };
        let _ = self.tx.send(task);
        handle
    }

    pub fn load_gltf(&self, path: &Path) -> Handle<GltfScene> {
        let handle = Handle::new_loading();
        let task = AssetTask::Gltf { path: path.to_path_buf(), handle: handle.clone(), start: std::time::Instant::now() };
        let _ = self.tx.send(task);
        handle
    }

    // This must be called in the main thread loop
    pub fn update(&mut self, renderer: &mut WgpuRenderer) -> Vec<AssetEvent> {
        let mut events = Vec::new();
        while let Ok((task, result)) = self.rx.try_recv() {
            match (task, result) {
                (AssetTask::Texture { handle, is_linear, start, .. }, Ok(AssetResult::Image(img))) => {
                    let ms = std::time::Instant::now().duration_since(start).as_secs_f32() * 1000.0;
                    if let Some(tex_id) = renderer.load_texture_from_image(img, is_linear) {
                         if let Ok(mut state) = handle.container.state.write() {
                             *state = LoadState::Loaded(tex_id);
                         }  // ✅ 处理锁中毒情况，忽略更新失败
                         events.push(AssetEvent::TextureLoaded(handle.clone(), ms));
                    } else {
                         if let Ok(mut state) = handle.container.state.write() {
                             *state = LoadState::Failed("Failed to create texture".to_string());
                         }  // ✅ 处理锁中毒情况，忽略更新失败
                         events.push(AssetEvent::TextureFailed(handle.clone(), "Failed to create texture".to_string()));
                    }
                },
                (AssetTask::Atlas { handle, start, .. }, Ok(AssetResult::Bytes(bytes))) => {
                    let ms = std::time::Instant::now().duration_since(start).as_secs_f32() * 1000.0;
                    if let Ok(json_str) = String::from_utf8(bytes) {
                        if let Some(atlas) = Atlas::from_json(&json_str) {
                             if let Ok(mut state) = handle.container.state.write() {
                                 *state = LoadState::Loaded(atlas);
                             }  // ✅ 处理锁中毒情况，忽略更新失败
                             events.push(AssetEvent::AtlasLoaded(handle.clone(), ms));
                        } else {
                             if let Ok(mut state) = handle.container.state.write() {
                                 *state = LoadState::Failed("Invalid Atlas JSON".to_string());
                             }  // ✅ 处理锁中毒情况，忽略更新失败
                             events.push(AssetEvent::AtlasFailed(handle.clone(), "Invalid Atlas JSON".to_string()));
                        }
                    } else {
                         if let Ok(mut state) = handle.container.state.write() {
                             *state = LoadState::Failed("Invalid UTF-8".to_string());
                         }  // ✅ 处理锁中毒情况，忽略更新失败
                         events.push(AssetEvent::AtlasFailed(handle.clone(), "Invalid UTF-8".to_string()));
                    }
                },
                (AssetTask::Gltf { handle, start, .. }, Ok(AssetResult::Gltf(scene))) => {
                    let ms = std::time::Instant::now().duration_since(start).as_secs_f32() * 1000.0;
                    if let Ok(mut state) = handle.container.state.write() {
                        *state = LoadState::Loaded(scene);
                    }  // ✅ 处理锁中毒情况，忽略更新失败
                    events.push(AssetEvent::GltfLoaded(handle.clone(), ms));
                },
                (AssetTask::Texture { handle, .. }, Err(e)) => {
                    if let Ok(mut state) = handle.container.state.write() {
                        *state = LoadState::Failed(e.clone());
                    }  // ✅ 处理锁中毒情况，忽略更新失败
                    events.push(AssetEvent::TextureFailed(handle.clone(), e));
                },
                (AssetTask::Atlas { handle, .. }, Err(e)) => {
                    if let Ok(mut state) = handle.container.state.write() {
                        *state = LoadState::Failed(e.clone());
                    }  // ✅ 处理锁中毒情况，忽略更新失败
                    events.push(AssetEvent::AtlasFailed(handle.clone(), e));
                },
                (AssetTask::Gltf { handle, .. }, Err(e)) => {
                    if let Ok(mut state) = handle.container.state.write() {
                        *state = LoadState::Failed(e.clone());
                    }  // ✅ 处理锁中毒情况，忽略更新失败
                    events.push(AssetEvent::GltfFailed(handle.clone(), e));
                },
                _ => {}
            }
        }
        events
    }
    
    // Helper for legacy code compatibility
    pub fn atlas_region(&self, atlas_handle: &Handle<Atlas>, sprite_name: &str) -> Option<([f32; 2], [f32; 2])> {
        if let Some(atlas) = atlas_handle.get() {
            return atlas.get(sprite_name);
        }
        None
    }
}

impl Drop for AssetServer {
    fn drop(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }

        if let Some(handle) = self.worker_handle.take() {
            if let Err(e) = handle.join() {
                log::error!("Asset loader thread panicked: {:?}", e);
            }
        }
    }
}

// 将GLTF场景导入为引擎批次，可选生成切线与纹理绑定组
pub fn import_gltf_to_world(world: &mut bevy_ecs::world::World, renderer: &mut WgpuRenderer, handle: &Handle<GltfScene>) {
    // use gltf::Primitive;
    if let Some(scene) = handle.get() {
        let (doc, buffers, images) = &*scene.data;
        let pbr = match &renderer.pbr_renderer { Some(p) => p, None => return };
        let sampler = renderer.create_sampler();

        for mesh in doc.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buf| Some(&buffers[buf.index()]));
                let positions: Vec<[f32;3]> = reader.read_positions().map(|it| it.collect()).unwrap_or_default();
                let normals: Vec<[f32;3]> = reader.read_normals().map(|it| it.collect()).unwrap_or_else(|| vec![[0.0,1.0,0.0]; positions.len()]);
                // 选择UV集索引：优先 baseColor，其次 MR/normal/AO/emissive
                let mut texcoord_index = 0u32;
                let mt = primitive.material();
                if let Some(info) = mt.pbr_metallic_roughness().base_color_texture() { texcoord_index = info.tex_coord(); }
                else if let Some(info) = mt.pbr_metallic_roughness().metallic_roughness_texture() { texcoord_index = info.tex_coord(); }
                else if let Some(info) = mt.normal_texture() { texcoord_index = info.tex_coord(); }
                else if let Some(info) = mt.occlusion_texture() { texcoord_index = info.tex_coord(); }
                else if let Some(info) = mt.emissive_texture() { texcoord_index = info.tex_coord(); }
                let uvs: Vec<[f32;2]> = reader
                    .read_tex_coords(texcoord_index)
                    .and_then(|tc| Some(tc.into_f32()))
                    .map(|it| it.collect())
                    .unwrap_or_else(|| vec![[0.0,0.0]; positions.len()]);
                let mut tangents: Vec<[f32;4]> = reader.read_tangents().map(|it| it.collect()).unwrap_or_default();
                let indices: Vec<u32> = reader.read_indices().map(|r| r.into_u32().collect()).unwrap_or_else(|| (0..positions.len() as u32).collect());

                if tangents.is_empty() {
                    tangents = generate_tangents(&positions, &normals, &uvs, &indices);
                }

                let mut vertices = Vec::with_capacity(positions.len());
                for i in 0..positions.len() {
                    vertices.push(crate::render::mesh::Vertex3D { pos: positions[i], normal: normals[i], uv: uvs[i], tangent: tangents[i] });
                }
                let gpu_mesh = renderer.create_gpu_mesh(&vertices, &indices);

                // 构建纹理绑定组（五贴图）并持久化纹理
                let default_img = image::RgbaImage::from_raw(1,1,vec![255,255,255,255]).unwrap();
                let mr = primitive.material().pbr_metallic_roughness();
                let bc_img = mr.base_color_texture().map(|info| &images[info.texture().source().index()]).map(to_rgba).unwrap_or(default_img.clone());
                let mr_img = mr.metallic_roughness_texture().map(|info| &images[info.texture().source().index()]).map(to_rgba).unwrap_or(default_img.clone());
                let n_img = primitive.material().normal_texture().map(|info| &images[info.texture().source().index()]).map(to_rgba).unwrap_or(default_img.clone());
                let ao_img = primitive.material().occlusion_texture().map(|info| &images[info.texture().source().index()]).map(to_rgba).unwrap_or(default_img.clone());
                let em_img = primitive.material().emissive_texture().map(|info| &images[info.texture().source().index()]).map(to_rgba).unwrap_or(default_img.clone());
                let tex_set = pbr.create_texture_set_from_images(renderer.device(), renderer.queue(), [bc_img, mr_img, n_img, ao_img, em_img], [true, false, false, false, true]);
                let tex_bg = std::sync::Arc::new(tex_set.bind_group);

                // GLTF 材质参数映射
                let mut mat = crate::render::pbr::PbrMaterial::default();
                let base = mr.base_color_factor();
                mat.base_color = glam::Vec4::from_array(base);
                mat.metallic = mr.metallic_factor();
                mat.roughness = mr.roughness_factor();
                mat.emissive = glam::Vec3::from_array(primitive.material().emissive_factor());
                mat.normal_scale = primitive.material().normal_texture().map(|n| n.scale()).unwrap_or(1.0);
                mat.ambient_occlusion = primitive.material().occlusion_texture().map(|o| o.strength()).unwrap_or(1.0);
                // KHR_texture_transform（UV变换）解析（仅 .gltf JSON 可用）
                if let Some(ref json) = scene.json {
                    if let Some(materials) = json.get("materials").and_then(|v| v.as_array()) {
                        if let Some(mi) = primitive.material().index() {
                            if let Some(mj) = materials.get(mi) {
                                if let Some(pbr) = mj.get("pbrMetallicRoughness") {
                                    if let Some(bct) = pbr.get("baseColorTexture") {
                                        if let Some(ext) = bct.get("extensions") {
                                            if let Some(tt) = ext.get("KHR_texture_transform") {
                                                if let Some(off) = tt.get("offset").and_then(|x| x.as_array()) {
                                                    if off.len() >= 2 {
                                                        mat.uv_offset = [off[0].as_f64().unwrap_or(0.0) as f32, off[1].as_f64().unwrap_or(0.0) as f32];
                                                    }
                                                }
                                                if let Some(scl) = tt.get("scale").and_then(|x| x.as_array()) {
                                                    if scl.len() >= 2 {
                                                        mat.uv_scale = [scl[0].as_f64().unwrap_or(1.0) as f32, scl[1].as_f64().unwrap_or(1.0) as f32];
                                                    }
                                                }
                                                if let Some(rot) = tt.get("rotation").and_then(|x| x.as_f64()) {
                                                    mat.uv_rotation = rot as f32;
                                                }
                                                if let Some(tc) = tt.get("texCoord").and_then(|x| x.as_u64()) {
                                                    let tc_i = tc as u32;
                                                    let uvs2: Vec<[f32;2]> = reader
                                                        .read_tex_coords(tc_i)
                                                        .and_then(|tc| Some(tc.into_f32()))
                                                        .map(|it| it.collect())
                                                        .unwrap_or_else(|| uvs.clone());
                                                    // 用新版 UV 替换
                                                    for i in 0..positions.len() {
                                                        vertices[i].uv = uvs2[i];
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // 材质注册与复用
                let mat_id = primitive.material().index().unwrap_or(0) as u64;
                let mut registry = world.get_resource_or_insert_with::<MaterialRegistry>(Default::default);
                let (material_bg, material_buf) = if let Some((bg, buf, tex)) = registry.materials.get(&mat_id) {
                    (bg.clone(), buf.clone())
                } else {
                    let (bg, buf) = pbr.create_material_bind_group(renderer.device(), renderer.queue(), &mat);
                    // 登记
                    registry.materials.insert(mat_id, (bg.clone(), buf.clone(), tex_bg.clone()));
                    (bg, buf)
                };

                let mesh_id = mesh.index() as u64;
                let comp = crate::render::instance_batch::Mesh3DRenderer {
                    mesh: gpu_mesh,
                    material_bind_group: material_bg,
                    textures_bind_group: Some(tex_bg),
                    material_uniform_buffer: Some(material_buf),
                    mesh_id,
                    material_id: mat_id,
                    visible: true,
                };
                let transform = crate::ecs::Transform::default();
                world.spawn((comp, transform));
            }
        }
    }
}

// 纹理UV变换扩展（KHR_texture_transform）解析可在后续版本加入；当前使用默认值

// 删除视图构建占位函数，改为直接创建纹理集合并持久化在绑定组中

fn to_rgba(data: &gltf::image::Data) -> image::RgbaImage {
    match data.format {
        gltf::image::Format::R8G8B8A8 => image::RgbaImage::from_raw(data.width, data.height, data.pixels.clone()).unwrap_or_else(|| image::RgbaImage::new(data.width, data.height)),
        gltf::image::Format::R8G8B8 => {
            let mut rgba = Vec::with_capacity((data.width * data.height * 4) as usize);
            for i in (0..data.pixels.len()).step_by(3) {
                rgba.extend_from_slice(&[data.pixels[i], data.pixels[i+1], data.pixels[i+2], 255]);
            }
            image::RgbaImage::from_raw(data.width, data.height, rgba).unwrap()
        }
        _ => image::RgbaImage::new(data.width, data.height),
    }
}

fn generate_tangents(positions: &[[f32;3]], normals: &[[f32;3]], uvs: &[[f32;2]], indices: &[u32]) -> Vec<[f32;4]> {
    let mut tangents = vec![[0.0f32;4]; positions.len()];
    for tri in indices.chunks(3) {
        if tri.len() < 3 { continue; }
        let i0 = tri[0] as usize; let i1 = tri[1] as usize; let i2 = tri[2] as usize;
        let p0 = glam::Vec3::from_array(positions[i0]);
        let p1 = glam::Vec3::from_array(positions[i1]);
        let p2 = glam::Vec3::from_array(positions[i2]);
        let uv0 = glam::Vec2::from_array(uvs[i0]);
        let uv1 = glam::Vec2::from_array(uvs[i1]);
        let uv2 = glam::Vec2::from_array(uvs[i2]);
        let dp1 = p1 - p0; let dp2 = p2 - p0;
        let duv1 = uv1 - uv0; let duv2 = uv2 - uv0;
        let r = 1.0 / (duv1.x * duv2.y - duv1.y * duv2.x);
        let t = (dp1 * duv2.y - dp2 * duv1.y) * r;
        let n0 = glam::Vec3::from_array(normals[i0]);
        let t0 = (t - n0 * n0.dot(t)).normalize_or_zero();
        tangents[i0] = [t0.x, t0.y, t0.z, 1.0];
        tangents[i1] = tangents[i0];
        tangents[i2] = tangents[i0];
    }
    tangents
}
#[derive(Resource, Default)]
pub struct MaterialRegistry {
    pub materials: HashMap<u64, (
        std::sync::Arc<wgpu::BindGroup>, // material uniform BG
        std::sync::Arc<wgpu::Buffer>,    // material uniform buffer
        std::sync::Arc<wgpu::BindGroup>, // textures BG
    )>,
}

#[derive(Resource, Default)]
pub struct MaterialPendingUpdates {
    pub params: Vec<(u64, crate::render::pbr::PbrMaterial)>,
}

impl MaterialPendingUpdates {
    pub fn push(&mut self, id: u64, mat: crate::render::pbr::PbrMaterial) {
        self.params.push((id, mat));
    }
    pub fn take_all(&mut self) -> Vec<(u64, crate::render::pbr::PbrMaterial)> {
        std::mem::take(&mut self.params)
    }
}

impl MaterialRegistry {
    pub fn update_material_params(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pbr: &crate::render::pbr_renderer::PbrRenderer,
        mat_id: u64,
        mat: &crate::render::pbr::PbrMaterial,
    ) -> bool {
        if let Some((bg, buf, _tex)) = self.materials.get_mut(&mat_id) {
            let uniform = crate::render::pbr_renderer::PbrRenderer::encode_material_uniform(mat);
            queue.write_buffer(buf, 0, bytemuck::bytes_of(&uniform));
            // bind group布局不变，无需重建
            true
        } else {
            // 创建并登记
            let (new_bg, new_buf) = pbr.create_material_bind_group(device, queue, mat);
            self.materials.insert(mat_id, (new_bg.clone(), new_buf.clone(), wgpu_dummy_bg(device, &pbr.textures_bgl)));
            true
        }
    }

    pub fn update_textures(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pbr: &crate::render::pbr_renderer::PbrRenderer,
        mat_id: u64,
        images: [image::RgbaImage; 5],
        srgb: [bool; 5],
    ) -> bool {
        let tex_set = pbr.create_texture_set_from_images(device, queue, images, srgb);
        let tex_bg = std::sync::Arc::new(tex_set.bind_group);
        if let Some(entry) = self.materials.get_mut(&mat_id) {
            let (_, _, old_tex_bg) = entry;
            *old_tex_bg = tex_bg;
            true
        } else {
            false
        }
    }
}

fn wgpu_dummy_bg(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> std::sync::Arc<wgpu::BindGroup> {
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("DummyTex"),
        size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
    std::sync::Arc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("DummyTexBG"),
        layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&view) },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&view) },
            wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&view) },
            wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&view) },
            wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::Sampler(&sampler) },
        ],
    }))
}
