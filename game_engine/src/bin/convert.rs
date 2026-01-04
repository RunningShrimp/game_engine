//! # 模型格式转换工具 - 主程序
//!
//! 支持以下格式之间的转换：
//! - glTF (`.gltf`) - JSON格式的3D模型
//! - GLB (`.glb`) - 二进制格式的3D模型
//! - OBJ (`.obj`) - Wavefront OBJ格式

#![allow(clippy::type_complexity)]
#![allow(clippy::len_zero)]
#![allow(clippy::map_entry)]
#![allow(clippy::uninlined_format_args)]

use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

/// 模型格式枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelFormat {
    GlTF,
    GLB,
    OBJ,
}

impl ModelFormat {
    /// 从文件扩展名解析格式
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "gltf" => Some(ModelFormat::GlTF),
            "glb" => Some(ModelFormat::GLB),
            "obj" => Some(ModelFormat::OBJ),
            _ => None,
        }
    }

    /// 获取文件扩展名
    pub fn extension(&self) -> &str {
        match self {
            ModelFormat::GlTF => "gltf",
            ModelFormat::GLB => "glb",
            ModelFormat::OBJ => "obj",
        }
    }

    /// 获取格式名称
    pub fn name(&self) -> &str {
        match self {
            ModelFormat::GlTF => "glTF",
            ModelFormat::GLB => "GLB",
            ModelFormat::OBJ => "OBJ",
        }
    }
}

/// 转换错误类型
#[derive(Debug)]
pub enum ConversionError {
    Io(std::io::Error),
    UnsupportedFormat(String),
    ParseError(String),
    ExportError(String),
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::Io(e) => write!(f, "IO错误:{e}"),
            ConversionError::UnsupportedFormat(msg) => write!(f, "不支持的格式:{msg}"),
            ConversionError::ParseError(msg) => write!(f, "解析错误:{msg}"),
            ConversionError::ExportError(msg) => write!(f, "导出错误:{msg}"),
        }
    }
}

impl From<std::io::Error> for ConversionError {
    fn from(e: std::io::Error) -> Self {
        ConversionError::Io(e)
    }
}

/// 中间表示格式，用于在格式之间转换
#[derive(Debug, Clone)]
pub struct IntermediateModel {
    /// 模型名称
    pub name: String,
    /// 顶点位置
    pub positions: Vec<[f32; 3]>,
    /// 法向量
    pub normals: Vec<[f32; 3]>,
    /// 纹理坐标
    pub tex_coords: Vec<[f32; 2]>,
    /// 索引
    pub indices: Vec<u32>,
    /// 材质名称
    pub material_name: Option<String>,
}

impl IntermediateModel {
    /// 创建一个新的空模型
    pub fn new(name: String) -> Self {
        Self {
            name,
            positions: Vec::new(),
            normals: Vec::new(),
            tex_coords: Vec::new(),
            indices: Vec::new(),
            material_name: None,
        }
    }

    /// 计算模型统计信息
    pub fn stats(&self) -> ModelStats {
        ModelStats {
            vertex_count: self.positions.len(),
            triangle_count: self.indices.len() / 3,
            has_normals: !self.normals.is_empty(),
            has_tex_coords: !self.tex_coords.is_empty(),
        }
    }
}

/// 模型统计信息
#[derive(Debug)]
pub struct ModelStats {
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub has_normals: bool,
    pub has_tex_coords: bool,
}

/// 从glTF/GLB加载模型
fn load_gltf(path: &Path) -> Result<IntermediateModel, ConversionError> {
    println!(
        "正在加载glTF文件: {path_display}",
        path_display = path.display()
    );

    #[cfg(feature = "gltf")]
    {
        use gltf::Gltf;

        let file = File::open(path)?;
        let reader = std::io::BufReader::new(file);

        // 判断是glTF还是GLB
        let is_glb = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("glb"))
            .unwrap_or(false);

        let (document, buffers, _images) = if is_glb {
            gltf::import_slice(&fs::read(path)?)
                .map_err(|e| ConversionError::ParseError(format!("GLB解析失败: {e}")))?
        } else {
            let gltf = Gltf::from_reader(reader)
                .map_err(|e| ConversionError::ParseError(format!("glTF解析失败: {e}")))?;

            // 读取外部buffer
            let buffer_path = path.parent().unwrap_or(Path::new("."));
            let mut buffers = Vec::new();
            for buffer in gltf.buffers() {
                match buffer.source() {
                    gltf::buffer::Source::Bin => {
                        return Err(ConversionError::ParseError(
                            "不支持的glTF格式: 内嵌二进制数据".to_string(),
                        ));
                    }
                    gltf::buffer::Source::Uri(uri) => {
                        let uri_path = buffer_path.join(uri);
                        let data = fs::read(&uri_path)?;
                        buffers.push(gltf::buffer::Data(data));
                    }
                }
            }

            (gltf.document, buffers, Vec::new())
        };

        // 提取第一个网格
        let mesh = document
            .meshes()
            .next()
            .ok_or_else(|| ConversionError::ParseError("glTF文件中没有找到网格".to_string()))?;

        let primitive = mesh
            .primitives()
            .next()
            .ok_or_else(|| ConversionError::ParseError("网格中没有找到几何体".to_string()))?;

        let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(|data| &*data.0));

        let mut model = IntermediateModel::new(mesh.name().unwrap_or("Mesh").to_string());

        // 读取顶点位置
        if let Some(positions) = reader.read_positions() {
            model.positions = positions.collect();
        } else {
            return Err(ConversionError::ParseError("缺少顶点位置数据".to_string()));
        }

        // 读取法向量
        if let Some(normals) = reader.read_normals() {
            model.normals = normals.collect();
        }

        // 读取纹理坐标
        if let Some(tex_coords) = reader.read_tex_coords(0) {
            model.tex_coords = tex_coords.into_f32().collect();
        }

        // 读取索引
        if let Some(indices) = reader.read_indices() {
            model.indices = indices.into_u32().collect();
        }

        println!("✓ 成功加载glTF模型");
        Ok(model)
    }

    #[cfg(not(feature = "gltf"))]
    {
        Err(ConversionError::UnsupportedFormat(
            "glTF支持未启用。请使用 --features gltf 编译".to_string(),
        ))
    }
}

/// 从OBJ加载模型
fn load_obj(path: &Path) -> Result<IntermediateModel, ConversionError> {
    println!(
        "正在加载OBJ文件: {path_display}",
        path_display = path.display()
    );

    let content = fs::read_to_string(path)?;

    let mut model = IntermediateModel::new(
        path.file_stem().and_then(|s| s.to_str()).unwrap_or("Mesh").to_string(),
    );

    let mut vertices: Vec<([f32; 3], Option<[f32; 3]>, Option<[f32; 2]>)> = Vec::new();
    let mut indices_map: std::collections::HashMap<(usize, Option<usize>, Option<usize>), u32> =
        std::collections::HashMap::new();

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut tex_coords: Vec<[f32; 2]> = Vec::new();
    let mut face_indices: Vec<(usize, Option<usize>, Option<usize>)> = Vec::new();

    // 解析OBJ文件
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "v" => {
                // 顶点位置
                if parts.len() >= 4 {
                    let x = parts[1].parse::<f32>().unwrap_or(0.0);
                    let y = parts[2].parse::<f32>().unwrap_or(0.0);
                    let z = parts[3].parse::<f32>().unwrap_or(0.0);
                    positions.push([x, y, z]);
                }
            }
            "vn" => {
                // 法向量
                if parts.len() >= 4 {
                    let x = parts[1].parse::<f32>().unwrap_or(0.0);
                    let y = parts[2].parse::<f32>().unwrap_or(0.0);
                    let z = parts[3].parse::<f32>().unwrap_or(0.0);
                    normals.push([x, y, z]);
                }
            }
            "vt" => {
                // 纹理坐标
                if parts.len() >= 3 {
                    let u = parts[1].parse::<f32>().unwrap_or(0.0);
                    let v = parts[2].parse::<f32>().unwrap_or(0.0);
                    tex_coords.push([u, v]);
                }
            }
            "f" => {
                // 面
                for part in &parts[1..] {
                    let parts: Vec<&str> = part.split('/').collect();

                    let v_idx = if parts.len() > 0 && !parts[0].is_empty() {
                        parts[0].parse::<usize>().ok().map(|i| if i == 0 { 0 } else { i - 1 })
                    } else {
                        None
                    };

                    let vt_idx = if parts.len() > 1 && !parts[1].is_empty() {
                        parts[1].parse::<usize>().ok().map(|i| if i == 0 { 0 } else { i - 1 })
                    } else {
                        None
                    };

                    let vn_idx = if parts.len() > 2 && !parts[2].is_empty() {
                        parts[2].parse::<usize>().ok().map(|i| if i == 0 { 0 } else { i - 1 })
                    } else if parts.len() > 1 && !parts[1].is_empty() {
                        // 有些OBJ格式使用 v//vn 格式
                        parts[1].parse::<usize>().ok().map(|i| if i == 0 { 0 } else { i - 1 })
                    } else {
                        None
                    };

                    if let Some(vi) = v_idx {
                        face_indices.push((vi, vt_idx, vn_idx));
                    }
                }
            }
            _ => {}
        }
    }

    // 构建索引化的网格
    let mut next_index: u32 = 0;
    for (v_idx, vt_idx, vn_idx) in face_indices {
        let key = (v_idx, vt_idx, vn_idx);

        if !indices_map.contains_key(&key) {
            let pos = positions.get(v_idx).copied().unwrap_or([0.0, 0.0, 0.0]);
            let normal = vn_idx.and_then(|i| normals.get(i).copied());
            let tex_coord = vt_idx.and_then(|i| tex_coords.get(i).copied());

            vertices.push((pos, normal, tex_coord));
            indices_map.insert(key, next_index);
            next_index += 1;
        }

        model.indices.push(indices_map[&key]);
    }

    // 展开顶点数据
    for (pos, normal, tex_coord) in vertices {
        model.positions.push(pos);
        if let Some(n) = normal {
            model.normals.push(n);
        }
        if let Some(t) = tex_coord {
            model.tex_coords.push(t);
        }
    }

    println!("✓ 成功加载OBJ模型");
    Ok(model)
}

/// 保存为glTF格式
fn save_gltf(model: &IntermediateModel, path: &Path) -> Result<(), ConversionError> {
    println!(
        "正在保存glTF文件: {path_display}",
        path_display = path.display()
    );

    #[cfg(feature = "gltf")]
    {
        use serde_json::json;

        // 创建buffer
        let buffer_uri = format!(
            "{}.bin",
            path.file_stem().and_then(|s| s.to_str()).unwrap_or("buffer")
        );

        // 准备二进制数据
        let mut buffer_data = Vec::new();
        let vertex_count = model.positions.len();

        // 写入位置数据
        let positions_offset = buffer_data.len();
        for pos in &model.positions {
            buffer_data.extend_from_slice(bytemuck::cast_slice(&pos[..]));
        }
        let positions_length = buffer_data.len() - positions_offset;

        // 写入法向量数据
        let (normals_offset, normals_length) = if !model.normals.is_empty() {
            let offset = buffer_data.len();
            for normal in &model.normals {
                buffer_data.extend_from_slice(bytemuck::cast_slice(&normal[..]));
            }
            (Some(offset), buffer_data.len() - offset)
        } else {
            (None, 0)
        };

        // 写入索引数据
        let indices_offset = buffer_data.len();
        for index in &model.indices {
            buffer_data.extend_from_slice(&index.to_le_bytes());
        }
        let indices_length = buffer_data.len() - indices_offset;

        // 创建buffer views
        let mut buffer_views = Vec::new();

        // Position buffer view
        let positions_view_index = buffer_views.len();
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": positions_offset,
            "byteLength": positions_length,
            "target": 34962 // ARRAY_BUFFER
        }));

        // Normals buffer view
        let normals_view_index = if !model.normals.is_empty() {
            let idx = buffer_views.len();
            buffer_views.push(json!({
                "buffer": 0,
                "byteOffset": normals_offset,
                "byteLength": normals_length,
                "target": 34962 // ARRAY_BUFFER
            }));
            Some(idx)
        } else {
            None
        };

        // Indices buffer view
        let indices_view_index = buffer_views.len();
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": indices_offset,
            "byteLength": indices_length,
            "target": 34963 // ELEMENT_ARRAY_BUFFER
        }));

        // 创建accessors
        let mut accessors = Vec::new();

        // Positions accessor
        let positions_accessor_index = accessors.len();
        accessors.push(json!({
            "bufferView": positions_view_index,
            "componentType": 5126, // FLOAT
            "count": vertex_count,
            "type": "VEC3",
            "min": [0.0, 0.0, 0.0], // 简化
            "max": [1.0, 1.0, 1.0]  // 简化
        }));

        // Normals accessor
        let normals_accessor_index = if !model.normals.is_empty() {
            let idx = accessors.len();
            accessors.push(json!({
                "bufferView": normals_view_index.unwrap(),
                "componentType": 5126, // FLOAT
                "count": vertex_count,
                "type": "VEC3"
            }));
            Some(idx)
        } else {
            None
        };

        // Indices accessor
        let indices_accessor_index = accessors.len();
        accessors.push(json!({
            "bufferView": indices_view_index,
            "componentType": 5125, // UNSIGNED_INT
            "count": model.indices.len(),
            "type": "SCALAR"
        }));

        // 创建primitive attributes
        let mut attributes = serde_json::Map::new();
        attributes.insert("POSITION".to_string(), json!(positions_accessor_index));

        if let Some(normals_accessor) = normals_accessor_index {
            attributes.insert("NORMAL".to_string(), json!(normals_accessor));
        }

        // 创建primitive
        let primitive = json!({
            "attributes": attributes,
            "indices": indices_accessor_index,
            "mode": 4 // TRIANGLES
        });

        // 创建完整的glTF JSON结构
        let gltf = json!({
            "asset": {
                "version": "2.0",
                "generator": "game_engine model converter"
            },
            "buffers": [{
                "uri": buffer_uri,
                "byteLength": buffer_data.len()
            }],
            "bufferViews": buffer_views,
            "accessors": accessors,
            "meshes": [{
                "name": model.name,
                "primitives": [primitive]
            }],
            "nodes": [{
                "mesh": 0
            }],
            "scenes": [{
                "nodes": [0]
            }],
            "scene": 0
        });

        // 写入JSON文件
        let gltf_json = serde_json::to_string_pretty(&gltf)
            .map_err(|e| ConversionError::ExportError(format!("JSON序列化失败: {e}")))?;

        let mut file = BufWriter::new(File::create(path)?);
        file.write_all(gltf_json.as_bytes())?;

        // 写入buffer数据
        let buffer_path = path.with_extension("bin");
        let mut buffer_file = BufWriter::new(File::create(&buffer_path)?);
        buffer_file.write_all(&buffer_data)?;

        println!("✓ 成功保存glTF文件");
        Ok(())
    }

    #[cfg(not(feature = "gltf"))]
    {
        Err(ConversionError::UnsupportedFormat(
            "glTF支持未启用。请使用 --features gltf 编译".to_string(),
        ))
    }
}

/// 保存为GLB格式
fn save_glb(model: &IntermediateModel, path: &Path) -> Result<(), ConversionError> {
    println!(
        "正在保存GLB文件: {path_display}",
        path_display = path.display()
    );

    #[cfg(feature = "gltf")]
    {
        // GLB需要将所有数据打包到一个文件中
        // 这里简化处理，先保存为glTF然后提示用户
        println!("注意: 完整的GLB导出需要更复杂的实现");
        println!("提示: 可以使用 gltf-pipeline 或其他工具将生成的glTF转换为GLB");

        // 临时保存为glTF
        let gltf_path = path.with_extension("gltf");
        save_gltf(model, &gltf_path)?;

        println!(
            "已保存为glTF格式: {gltf_path_display}",
            gltf_path_display = gltf_path.display()
        );
        Ok(())
    }

    #[cfg(not(feature = "gltf"))]
    {
        Err(ConversionError::UnsupportedFormat(
            "glTF支持未启用。请使用 --features gltf 编译".to_string(),
        ))
    }
}

/// 保存为OBJ格式
fn save_obj(model: &IntermediateModel, path: &Path) -> Result<(), ConversionError> {
    println!(
        "正在保存OBJ文件: {path_display}",
        path_display = path.display()
    );

    let mut file = BufWriter::new(File::create(path)?);

    // 写入头部
    writeln!(file, "# OBJ file generated by game_engine model converter")?;
    writeln!(file, "o {}", model.name)?;
    writeln!(file)?;

    // 写入顶点位置
    for pos in &model.positions {
        writeln!(file, "v {} {} {}", pos[0], pos[1], pos[2])?;
    }

    // 写入法向量
    if !model.normals.is_empty() {
        writeln!(file)?;
        for normal in &model.normals {
            writeln!(file, "vn {} {} {}", normal[0], normal[1], normal[2])?;
        }
    }

    // 写入纹理坐标
    if !model.tex_coords.is_empty() {
        writeln!(file)?;
        for tex_coord in &model.tex_coords {
            writeln!(file, "vt {} {}", tex_coord[0], tex_coord[1])?;
        }
    }

    // 写入面
    writeln!(file)?;
    let has_normals = !model.normals.is_empty();
    let has_tex_coords = !model.tex_coords.is_empty();

    for chunk in model.indices.chunks(3) {
        if chunk.len() != 3 {
            continue;
        }

        let v0 = chunk[0] + 1; // OBJ索引从1开始
        let v1 = chunk[1] + 1;
        let v2 = chunk[2] + 1;

        if has_normals && has_tex_coords {
            write!(
                file,
                "f {}/{}/{} {}/{}/{} {}/{}/{}",
                v0, v0, v0, v1, v1, v1, v2, v2, v2
            )?;
        } else if has_normals {
            write!(file, "f {}//{} {}//{} {}//{}", v0, v0, v1, v1, v2, v2)?;
        } else if has_tex_coords {
            write!(file, "f {}/{} {}/{} {}/{}", v0, v0, v1, v1, v2, v2)?;
        } else {
            write!(file, "f {} {} {}", v0, v1, v2)?;
        }
        writeln!(file)?;
    }

    println!("✓ 成功保存OBJ文件");
    Ok(())
}

/// 转换模型格式
pub fn convert(input_path: &Path, output_path: &Path) -> Result<(), ConversionError> {
    println!("========================================");
    println!("3D模型格式转换器");
    println!("========================================");
    println!();

    // 检测输入格式
    let input_ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| ConversionError::UnsupportedFormat("无法确定输入文件扩展名".to_string()))?;

    let input_format = ModelFormat::from_extension(input_ext).ok_or_else(|| {
        ConversionError::UnsupportedFormat(format!("不支持的输入格式: {input_ext}"))
    })?;

    // 检测输出格式
    let output_ext = output_path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| ConversionError::UnsupportedFormat("无法确定输出文件扩展名".to_string()))?;

    let output_format = ModelFormat::from_extension(output_ext).ok_or_else(|| {
        ConversionError::UnsupportedFormat(format!("不支持的输出格式: {output_ext}"))
    })?;

    println!(
        "输入文件: {input_path_display}",
        input_path_display = input_path.display()
    );
    println!(
        "输入格式: {input_format_name}",
        input_format_name = input_format.name()
    );
    println!(
        "输出文件: {output_path_display}",
        output_path_display = output_path.display()
    );
    println!(
        "输出格式: {output_format_name}",
        output_format_name = output_format.name()
    );
    println!();

    // 加载模型
    let model = match input_format {
        ModelFormat::GlTF | ModelFormat::GLB => load_gltf(input_path)?,
        ModelFormat::OBJ => load_obj(input_path)?,
    };

    // 显示模型统计
    let stats = model.stats();
    println!("模型统计:");
    println!("  顶点数: {}", stats.vertex_count);
    println!("  三角面数: {}", stats.triangle_count);
    println!("  法向量: {}", if stats.has_normals { "是" } else { "否" });
    println!(
        "  纹理坐标: {}",
        if stats.has_tex_coords { "是" } else { "否" }
    );
    println!();

    // 保存模型
    match output_format {
        ModelFormat::GlTF => save_gltf(&model, output_path)?,
        ModelFormat::GLB => save_glb(&model, output_path)?,
        ModelFormat::OBJ => save_obj(&model, output_path)?,
    }

    println!();
    println!("========================================");
    println!("✓ 转换成功完成!");
    println!("========================================");

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("3D模型格式转换器");
        println!();
        println!("用法:");
        println!("  {arg0} <输入文件> <输出文件>", arg0 = args[0]);
        println!();
        println!("支持的格式:");
        println!("  - glTF (.gltf) - JSON格式的3D模型");
        println!("  - GLB (.glb)   - 二进制格式的3D模型");
        println!("  - OBJ (.obj)   - Wavefront OBJ格式");
        println!();
        println!("示例:");
        println!("  {arg0} model.gltf model.glb", arg0 = args[0]);
        println!("  {arg0} model.glb model.obj", arg0 = args[0]);
        println!("  {arg0} mesh.obj output.gltf", arg0 = args[0]);
        println!();
        println!("注意: FBX格式需要额外处理，暂不支持。");
        std::process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);
    let output_path = PathBuf::from(&args[2]);

    if let Err(e) = convert(&input_path, &output_path) {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }
}
