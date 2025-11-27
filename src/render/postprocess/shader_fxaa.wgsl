// FXAA (Fast Approximate Anti-Aliasing) Shader
// 基于 NVIDIA FXAA 3.11 算法
// 高性能边缘平滑抗锯齿

// Uniforms
struct FxaaUniforms {
    screen_size: vec2<f32>,
    pixel_size: vec2<f32>,
    edge_threshold: f32,
    edge_threshold_min: f32,
    subpix_quality: f32,
    _pad: f32,
};

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: FxaaUniforms;

// 顶点着色器输出
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// 全屏三角形顶点着色器
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // 生成全屏三角形
    let x = f32(i32(vertex_index & 1u) * 4 - 1);
    let y = f32(i32(vertex_index >> 1u) * 4 - 1);
    
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    
    return out;
}

// 计算亮度
fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.299, 0.587, 0.114));
}

// FXAA 纹理采样
fn fxaa_sample(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(input_texture, input_sampler, uv);
}

// FXAA 亮度采样
fn fxaa_luma(uv: vec2<f32>) -> f32 {
    let color = fxaa_sample(uv);
    return luminance(color.rgb);
}

// 主 FXAA 算法
fn fxaa(uv: vec2<f32>) -> vec4<f32> {
    let pixel = uniforms.pixel_size;
    
    // 采样当前像素及其4个邻居的亮度
    let luma_m = fxaa_luma(uv);
    let luma_n = fxaa_luma(uv + vec2<f32>(0.0, -pixel.y));
    let luma_s = fxaa_luma(uv + vec2<f32>(0.0, pixel.y));
    let luma_e = fxaa_luma(uv + vec2<f32>(pixel.x, 0.0));
    let luma_w = fxaa_luma(uv + vec2<f32>(-pixel.x, 0.0));
    
    // 计算局部对比度
    let luma_max = max(max(max(luma_n, luma_s), max(luma_e, luma_w)), luma_m);
    let luma_min = min(min(min(luma_n, luma_s), min(luma_e, luma_w)), luma_m);
    let luma_range = luma_max - luma_min;
    
    // 如果对比度太低，跳过抗锯齿
    if luma_range < max(uniforms.edge_threshold_min, luma_max * uniforms.edge_threshold) {
        return fxaa_sample(uv);
    }
    
    // 采样对角邻居
    let luma_nw = fxaa_luma(uv + vec2<f32>(-pixel.x, -pixel.y));
    let luma_ne = fxaa_luma(uv + vec2<f32>(pixel.x, -pixel.y));
    let luma_sw = fxaa_luma(uv + vec2<f32>(-pixel.x, pixel.y));
    let luma_se = fxaa_luma(uv + vec2<f32>(pixel.x, pixel.y));
    
    // 计算边缘方向
    let luma_ns = luma_n + luma_s;
    let luma_ew = luma_e + luma_w;
    let luma_nwne = luma_nw + luma_ne;
    let luma_swse = luma_sw + luma_se;
    
    let edge_horz = abs(-2.0 * luma_w + luma_nwne) + abs(-2.0 * luma_m + luma_ns) * 2.0 + abs(-2.0 * luma_e + luma_swse);
    let edge_vert = abs(-2.0 * luma_n + (luma_nw + luma_ne)) + abs(-2.0 * luma_m + luma_ew) * 2.0 + abs(-2.0 * luma_s + (luma_sw + luma_se));
    
    let is_horizontal = edge_horz >= edge_vert;
    
    // 选择边缘方向上的梯度
    var luma_neg: f32;
    var luma_pos: f32;
    if is_horizontal {
        luma_neg = luma_n;
        luma_pos = luma_s;
    } else {
        luma_neg = luma_w;
        luma_pos = luma_e;
    }
    
    let gradient_neg = abs(luma_neg - luma_m);
    let gradient_pos = abs(luma_pos - luma_m);
    
    var step_length: f32;
    var luma_local_avg: f32;
    if gradient_neg >= gradient_pos {
        step_length = select(-pixel.x, -pixel.y, is_horizontal);
        luma_local_avg = 0.5 * (luma_neg + luma_m);
    } else {
        step_length = select(pixel.x, pixel.y, is_horizontal);
        luma_local_avg = 0.5 * (luma_pos + luma_m);
    }
    
    // 计算采样位置
    var current_uv = uv;
    if is_horizontal {
        current_uv.y += step_length * 0.5;
    } else {
        current_uv.x += step_length * 0.5;
    }
    
    // 沿边缘搜索
    var offset: vec2<f32>;
    if is_horizontal {
        offset = vec2<f32>(pixel.x, 0.0);
    } else {
        offset = vec2<f32>(0.0, pixel.y);
    }
    
    var uv_neg = current_uv - offset;
    var uv_pos = current_uv + offset;
    
    let gradient_scaled = max(gradient_neg, gradient_pos) * 0.25;
    
    var luma_end_neg = fxaa_luma(uv_neg) - luma_local_avg;
    var luma_end_pos = fxaa_luma(uv_pos) - luma_local_avg;
    
    var reached_neg = abs(luma_end_neg) >= gradient_scaled;
    var reached_pos = abs(luma_end_pos) >= gradient_scaled;
    var reached_both = reached_neg && reached_pos;
    
    // 迭代搜索边缘端点
    let iterations: i32 = 12;
    for (var i: i32 = 0; i < iterations && !reached_both; i = i + 1) {
        if !reached_neg {
            uv_neg -= offset;
            luma_end_neg = fxaa_luma(uv_neg) - luma_local_avg;
            reached_neg = abs(luma_end_neg) >= gradient_scaled;
        }
        if !reached_pos {
            uv_pos += offset;
            luma_end_pos = fxaa_luma(uv_pos) - luma_local_avg;
            reached_pos = abs(luma_end_pos) >= gradient_scaled;
        }
        reached_both = reached_neg && reached_pos;
    }
    
    // 计算到边缘端点的距离
    var dist_neg: f32;
    var dist_pos: f32;
    if is_horizontal {
        dist_neg = uv.x - uv_neg.x;
        dist_pos = uv_pos.x - uv.x;
    } else {
        dist_neg = uv.y - uv_neg.y;
        dist_pos = uv_pos.y - uv.y;
    }
    
    let dist_final = min(dist_neg, dist_pos);
    let edge_length = dist_neg + dist_pos;
    
    // 计算子像素偏移
    let subpix_offset = (-2.0 * (luma_n + luma_s + luma_e + luma_w) + (luma_nw + luma_ne + luma_sw + luma_se)) / 12.0;
    let subpix_offset_final = clamp(abs(subpix_offset) / luma_range, 0.0, 1.0);
    let subpix_factor = (-2.0 * subpix_offset_final + 3.0) * subpix_offset_final * subpix_offset_final;
    let subpix_offset_value = subpix_factor * subpix_factor * uniforms.subpix_quality;
    
    // 确定是否需要在负方向采样
    let is_luma_center_smaller = luma_m < luma_local_avg;
    let correct_variation = ((select(luma_end_neg, luma_end_pos, dist_neg < dist_pos)) < 0.0) != is_luma_center_smaller;
    
    var pixel_offset = dist_final / edge_length - 0.5;
    pixel_offset = select(0.0, pixel_offset, correct_variation);
    
    let final_offset = max(pixel_offset, subpix_offset_value);
    
    // 最终采样
    var final_uv = uv;
    if is_horizontal {
        final_uv.y += final_offset * step_length;
    } else {
        final_uv.x += final_offset * step_length;
    }
    
    return fxaa_sample(final_uv);
}

// 片段着色器
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return fxaa(in.uv);
}
