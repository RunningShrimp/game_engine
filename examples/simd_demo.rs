/// SIMD功能演示程序

use game_engine::performance::simd::{
    detect_cpu_features, print_cpu_info, SimdBackend,
    math::{Vec4Simd, Vec3Simd, Mat4Simd, QuatSimd, VectorOps},
    batch::{
        BatchTransform, BatchInterpolation, BatchSkinning, BatchParticle,
        BatchConfig, BoneInfluence, Particle,
    },
};

fn main() {
    println!("=== SIMD功能演示 ===\n");
    
    // 1. 检测CPU特性
    println!("1. CPU特性检测");
    println!("-".repeat(50));
    print_cpu_info();
    println!();
    
    let backend = SimdBackend::best_available();
    println!("最优SIMD后端: {:?}", backend);
    println!("向量宽度: {:?}", backend.width());
    println!("f32通道数: {}", backend.f32_lanes());
    println!();
    
    // 2. 向量运算
    println!("2. 向量运算测试");
    println!("-".repeat(50));
    
    let v1 = Vec4Simd::new(1.0, 2.0, 3.0, 4.0);
    let v2 = Vec4Simd::new(5.0, 6.0, 7.0, 8.0);
    
    println!("v1 = {:?}", v1.data);
    println!("v2 = {:?}", v2.data);
    println!("v1 · v2 = {}", v1.dot(&v2));
    println!("v1 + v2 = {:?}", v1.add(&v2).data);
    println!("v1 - v2 = {:?}", v1.sub(&v2).data);
    println!("v1 * 2.0 = {:?}", v1.mul(2.0).data);
    println!("|v1| = {}", v1.length());
    println!("normalize(v1) = {:?}", v1.normalize().data);
    println!();
    
    // 3. 3D向量叉积
    println!("3. 3D向量叉积");
    println!("-".repeat(50));
    
    let v3a = Vec3Simd::new(1.0, 0.0, 0.0);
    let v3b = Vec3Simd::new(0.0, 1.0, 0.0);
    let cross = v3a.cross(&v3b);
    
    println!("v3a = {:?}", v3a.data);
    println!("v3b = {:?}", v3b.data);
    println!("v3a × v3b = {:?}", cross.data);
    println!();
    
    // 4. 矩阵运算
    println!("4. 矩阵运算测试");
    println!("-".repeat(50));
    
    let m1 = Mat4Simd::identity();
    let m2 = Mat4Simd::identity();
    let m_result = m1.mul(&m2);
    
    println!("单位矩阵 * 单位矩阵 = 单位矩阵");
    println!("结果矩阵对角线: [{}, {}, {}, {}]",
             m_result.data[0][0], m_result.data[1][1],
             m_result.data[2][2], m_result.data[3][3]);
    
    let v = Vec4Simd::new(1.0, 2.0, 3.0, 1.0);
    let transformed = m1.transform(&v);
    println!("单位矩阵变换向量 {:?} = {:?}", v.data, transformed.data);
    println!();
    
    // 5. 四元数运算
    println!("5. 四元数运算");
    println!("-".repeat(50));
    
    let q1 = QuatSimd::identity();
    let q2 = QuatSimd::identity();
    let q_result = q1.mul(&q2);
    
    println!("单位四元数 * 单位四元数 = {:?}", q_result.data);
    println!();
    
    // 6. 批量变换测试
    println!("6. 批量顶点变换");
    println!("-".repeat(50));
    
    let config = BatchConfig::default();
    let transformer = BatchTransform::new(config);
    
    let identity = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    
    let vertices = vec![[1.0, 2.0, 3.0, 1.0]; 1000];
    let mut output = vec![[0.0; 4]; 1000];
    
    let stats = transformer.transform_vertices(&identity, &vertices, &mut output);
    println!("处理顶点数: {}", stats.elements_processed);
    println!("处理时间: {} μs", stats.processing_time_us);
    println!("吞吐量: {:.2} 顶点/秒", stats.throughput());
    println!("使用后端: {:?}", stats.backend_used);
    println!();
    
    // 7. 批量插值测试
    println!("7. 批量线性插值");
    println!("-".repeat(50));
    
    let interpolator = BatchInterpolation::new(config.clone());
    
    let a = vec![[0.0, 0.0, 0.0, 1.0]; 1000];
    let b = vec![[1.0, 1.0, 1.0, 1.0]; 1000];
    let mut lerp_output = vec![[0.0; 4]; 1000];
    
    let stats = interpolator.lerp(&a, &b, 0.5, &mut lerp_output);
    println!("插值元素数: {}", stats.elements_processed);
    println!("处理时间: {} μs", stats.processing_time_us);
    println!("吞吐量: {:.2} 元素/秒", stats.throughput());
    println!("结果示例: {:?}", lerp_output[0]);
    println!();
    
    // 8. 骨骼蒙皮测试
    println!("8. 骨骼蒙皮（LBS）");
    println!("-".repeat(50));
    
    let skinning = BatchSkinning::new(config.clone());
    
    let skin_vertices = vec![[1.0, 0.0, 0.0]; 100];
    let skin_normals = vec![[0.0, 1.0, 0.0]; 100];
    let influences = vec![BoneInfluence {
        bone_indices: [0, 1, 0, 0],
        bone_weights: [0.7, 0.3, 0.0, 0.0],
    }; 100];
    
    let bone_matrices = vec![identity; 2];
    let mut output_vertices = vec![[0.0; 3]; 100];
    let mut output_normals = vec![[0.0; 3]; 100];
    
    let stats = skinning.linear_blend_skinning(
        &skin_vertices,
        &skin_normals,
        &influences,
        &bone_matrices,
        &mut output_vertices,
        &mut output_normals,
    );
    
    println!("蒙皮顶点数: {}", stats.elements_processed);
    println!("处理时间: {} μs", stats.processing_time_us);
    println!("吞吐量: {:.2} 顶点/秒", stats.throughput());
    println!();
    
    // 9. 粒子系统测试
    println!("9. 粒子系统更新");
    println!("-".repeat(50));
    
    let particle_processor = BatchParticle::new(config);
    
    let mut particles = vec![Particle {
        position: [0.0, 10.0, 0.0],
        velocity: [1.0, 0.0, 0.0],
        acceleration: [0.0, -9.8, 0.0],
        life: 1.0,
        size: 1.0,
        rotation: 0.0,
        color: [1.0, 1.0, 1.0, 1.0],
    }; 10000];
    
    let stats = particle_processor.update_particles(&mut particles, 0.016);
    println!("更新粒子数: {}", stats.elements_processed);
    println!("处理时间: {} μs", stats.processing_time_us);
    println!("吞吐量: {:.2} 粒子/秒", stats.throughput());
    println!("粒子示例位置: {:?}", particles[0].position);
    println!();
    
    // 10. 力场测试
    println!("10. 粒子力场");
    println!("-".repeat(50));
    
    let stats = particle_processor.apply_force_field(
        &mut particles,
        [0.0, 0.0, 0.0],
        1000.0,
        50.0,
    );
    
    println!("处理粒子数: {}", stats.elements_processed);
    println!("处理时间: {} μs", stats.processing_time_us);
    println!("粒子加速度示例: {:?}", particles[0].acceleration);
    println!();
    
    println!("=== 演示完成 ===");
}
