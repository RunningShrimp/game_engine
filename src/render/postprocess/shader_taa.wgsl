// TAA (Temporal Anti-Aliasing) Shader
// 时间抗锯齿 - 通过累积多帧信息实现高质量抗锯齿
// 需要运动向量来处理动态场景

// Uniforms
struct TaaUniforms {
    screen_size: vec2<f32>,
    pixel_size: vec2<f32>,
    jitter_offset: vec2<f32>,
    blend_factor: f32,
    _pad: f32,
};

@group(0) @binding(0) var current_texture: texture_2d<f32>;
@group(0) @binding(1) var history_texture: texture_2d<f32>;
@group(0) @binding(2) var motion_texture: texture_2d<f32>;
@group(0) @binding(3) var tex_sampler: sampler;
@group(0) @binding(4) var<uniform> uniforms: TaaUniforms;

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

// RGB 到 YCoCg 颜色空间转换（用于更好的颜色比较）
fn rgb_to_ycocg(color: vec3<f32>) -> vec3<f32> {
    let y = dot(color, vec3<f32>(0.25, 0.5, 0.25));
    let co = dot(color, vec3<f32>(0.5, 0.0, -0.5));
    let cg = dot(color, vec3<f32>(-0.25, 0.5, -0.25));
    return vec3<f32>(y, co, cg);
}

// YCoCg 到 RGB 颜色空间转换
fn ycocg_to_rgb(color: vec3<f32>) -> vec3<f32> {
    let y = color.x;
    let co = color.y;
    let cg = color.z;
    let r = y + co - cg;
    let g = y + cg;
    let b = y - co - cg;
    return vec3<f32>(r, g, b);
}

// 采样当前帧
fn sample_current(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(current_texture, tex_sampler, uv);
}

// 采样历史帧
fn sample_history(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(history_texture, tex_sampler, uv);
}

// 采样运动向量
fn sample_motion(uv: vec2<f32>) -> vec2<f32> {
    return textureSample(motion_texture, tex_sampler, uv).rg;
}

// 颜色裁剪 - AABB 包围盒方法
fn clip_aabb(history_color: vec3<f32>, current_color: vec3<f32>, aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> vec3<f32> {
    let p_clip = 0.5 * (aabb_max + aabb_min);
    let e_clip = 0.5 * (aabb_max - aabb_min) + vec3<f32>(0.0001);
    
    let v_clip = history_color - p_clip;
    let v_unit = v_clip / e_clip;
    let a_unit = abs(v_unit);
    let ma_unit = max(a_unit.x, max(a_unit.y, a_unit.z));
    
    if ma_unit > 1.0 {
        return p_clip + v_clip / ma_unit;
    } else {
        return history_color;
    }
}

// 计算邻域颜色的 AABB
fn calculate_neighborhood_aabb(uv: vec2<f32>) -> array<vec3<f32>, 2> {
    let pixel = uniforms.pixel_size;
    
    // 3x3 邻域采样
    let c00 = rgb_to_ycocg(sample_current(uv + vec2<f32>(-pixel.x, -pixel.y)).rgb);
    let c10 = rgb_to_ycocg(sample_current(uv + vec2<f32>(0.0, -pixel.y)).rgb);
    let c20 = rgb_to_ycocg(sample_current(uv + vec2<f32>(pixel.x, -pixel.y)).rgb);
    let c01 = rgb_to_ycocg(sample_current(uv + vec2<f32>(-pixel.x, 0.0)).rgb);
    let c11 = rgb_to_ycocg(sample_current(uv).rgb);
    let c21 = rgb_to_ycocg(sample_current(uv + vec2<f32>(pixel.x, 0.0)).rgb);
    let c02 = rgb_to_ycocg(sample_current(uv + vec2<f32>(-pixel.x, pixel.y)).rgb);
    let c12 = rgb_to_ycocg(sample_current(uv + vec2<f32>(0.0, pixel.y)).rgb);
    let c22 = rgb_to_ycocg(sample_current(uv + vec2<f32>(pixel.x, pixel.y)).rgb);
    
    // 计算最小和最大值
    var aabb_min = min(min(min(c00, c10), min(c20, c01)), min(min(c11, c21), min(c02, min(c12, c22))));
    var aabb_max = max(max(max(c00, c10), max(c20, c01)), max(max(c11, c21), max(c02, max(c12, c22))));
    
    // 扩展一点以减少闪烁
    let expand = (aabb_max - aabb_min) * 0.1;
    aabb_min -= expand;
    aabb_max += expand;
    
    var result: array<vec3<f32>, 2>;
    result[0] = aabb_min;
    result[1] = aabb_max;
    return result;
}

// 计算锐度因子
fn compute_sharpness(current: vec3<f32>, history: vec3<f32>) -> f32 {
    let diff = abs(current - history);
    return 1.0 - smoothstep(0.0, 0.1, length(diff));
}

// 主 TAA 解析
fn taa_resolve(uv: vec2<f32>) -> vec4<f32> {
    // 获取运动向量
    let motion = sample_motion(uv);
    
    // 计算历史帧 UV
    let history_uv = uv - motion;
    
    // 检查历史 UV 是否在有效范围内
    let valid_history = history_uv.x >= 0.0 && history_uv.x <= 1.0 &&
                        history_uv.y >= 0.0 && history_uv.y <= 1.0;
    
    // 采样当前帧和历史帧
    let current_color = sample_current(uv);
    var history_color = sample_history(history_uv);
    
    if !valid_history {
        // 历史不可用，使用当前帧
        return current_color;
    }
    
    // 转换到 YCoCg 空间
    let current_ycocg = rgb_to_ycocg(current_color.rgb);
    var history_ycocg = rgb_to_ycocg(history_color.rgb);
    
    // 计算邻域 AABB 并裁剪历史颜色
    let aabb = calculate_neighborhood_aabb(uv);
    history_ycocg = clip_aabb(history_ycocg, current_ycocg, aabb[0], aabb[1]);
    
    // 转换回 RGB
    let clipped_history = ycocg_to_rgb(history_ycocg);
    
    // 计算运动量以调整混合因子
    let motion_length = length(motion * uniforms.screen_size);
    let motion_factor = clamp(1.0 - motion_length * 0.1, 0.0, 1.0);
    
    // 自适应混合因子
    // 静止场景：更多历史 (0.95)
    // 快速运动：更多当前帧 (0.5)
    let base_blend = uniforms.blend_factor;
    let adaptive_blend = mix(0.5, base_blend, motion_factor);
    
    // 混合当前帧和裁剪后的历史帧
    let final_color = mix(current_color.rgb, clipped_history, adaptive_blend);
    
    return vec4<f32>(final_color, current_color.a);
}

// 片段着色器
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return taa_resolve(in.uv);
}
