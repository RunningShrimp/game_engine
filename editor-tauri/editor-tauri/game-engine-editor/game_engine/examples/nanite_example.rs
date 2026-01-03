//! # Nanite Virtual Geometry System Example
//!
//! This example demonstrates how to use the Nanite virtual geometry system
//! to render high-poly meshes with real-time performance.

use game_engine::render::nanite::*;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Nanite Virtual Geometry System Example ===\n");

    // Initialize wgpu
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .ok_or("No adapter found")?;

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Nanite Device"),
                required_features: wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES,
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .await?;

    println!("✓ GPU initialized");
    println!("  Adapter: {:?}", adapter.get_info());
    println!();

    // Create Nanite system
    let nanite_config = NaniteConfig {
        max_triangles_per_cluster: 128,
        max_lod_depth: 8,
        target_screen_space_error: 1.0,
        enable_occlusion_culling: true,
        enable_compute_acceleration: true,
        min_culling_cluster_size: 4,
        instance_buffer_size_mb: 256,
    };

    let mut nanite_system = NaniteSystem::new(&device, nanite_config)?;
    println!("✓ Nanite system initialized");
    println!();

    // Example 1: Create a simple mesh
    println!("=== Example 1: Simple Cube Mesh ===");
    let cube_vertices = create_cube_vertices();
    let cube_indices = create_cube_indices();

    let cube_mesh_id = nanite_system.register_mesh(&device, &cube_vertices, &cube_indices)?;
    println!("✓ Cube mesh registered (ID: {})", cube_mesh_id);

    let cube_hierarchy = nanite_system.hierarchy(cube_mesh_id).unwrap();
    println!("  Total triangles: {}", cube_hierarchy.total_triangles);
    println!("  Total clusters: {}", cube_hierarchy.cluster_count());
    println!("  Max LOD depth: {}", cube_hierarchy.max_depth);
    println!();

    // Example 2: Create a high-poly sphere
    println!("=== Example 2: High-Poly Sphere ===");
    let sphere_vertices = create_sphere_vertices(64, 64);
    let sphere_indices = create_sphere_indices(64, 64);

    let sphere_mesh_id = nanite_system.register_mesh(&device, &sphere_vertices, &sphere_indices)?;
    println!("✓ Sphere mesh registered (ID: {})", sphere_mesh_id);

    let sphere_hierarchy = nanite_system.hierarchy(sphere_mesh_id).unwrap();
    println!("  Total triangles: {}", sphere_hierarchy.total_triangles);
    println!("  Total clusters: {}", sphere_hierarchy.cluster_count());
    println!();

    // Example 3: Simulate rendering loop
    println!("=== Example 3: Rendering Simulation ===");

    let camera = Camera {
        position: [0.0, 0.0, 10.0],
        view_matrix: create_view_matrix([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
        projection_matrix: create_perspective_matrix(std::f32::consts::PI / 4.0, 16.0 / 9.0, 0.1, 1000.0),
        fov_y: std::f32::consts::PI / 4.0,
        aspect_ratio: 16.0 / 9.0,
        near_plane: 0.1,
        far_plane: 1000.0,
    };

    let mut frame_count = 0;
    let start_time = Instant::now();
    let mut total_visible_triangles = 0usize;

    for frame in 0..10 {
        let delta_time = 0.016; // Simulate 60 FPS

        // Update camera position (simulate orbit)
        let angle = (frame as f32 / 10.0) * std::f32::consts::PI * 2.0;
        let radius = 10.0;
        let cam_pos = [
            angle.sin() * radius,
            2.0,
            angle.cos() * radius,
        ];

        let mut moving_camera = camera.clone();
        moving_camera.position = cam_pos;
        moving_camera.view_matrix = create_view_matrix(cam_pos, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);

        // Update Nanite system
        let stats = nanite_system.update(&device, &queue, &moving_camera, delta_time)?;

        total_visible_triangles += stats.visible_triangles;
        frame_count += 1;

        if frame % 3 == 0 {
            println!("Frame {}: {} visible clusters, {} visible triangles, {:.2} ms",
                frame,
                stats.visible_clusters,
                stats.visible_triangles,
                stats.frame_time_ms
            );
        }
    }

    let elapsed = start_time.elapsed();
    let avg_triangles = total_visible_triangles / frame_count;

    println!();
    println!("Performance Summary:");
    println!("  Total frames: {}", frame_count);
    println!("  Average visible triangles: {}", avg_triangles);
    println!("  Total time: {:.2}s", elapsed.as_secs_f64());
    println!("  GPU memory: {:.2} MB", nanite_system.buffer_manager().memory_usage_mb());
    println!();

    // Example 4: Quality control
    println!("=== Example 4: Adaptive Quality ===");
    let quality_controller = nanite_system.quality_controller();

    println!("Current quality multiplier: {:.2}", quality_controller.current_quality());
    println!("Is stabilized: {}", quality_controller.is_stabilized());

    // Simulate quality adjustment
    quality_controller.set_target_quality(1.5);
    println!("Target quality set to 1.5 (high quality)");

    for _ in 0..5 {
        let _metrics = quality_controller.update(0.016)?;
        println!("  Quality adjusted to: {:.2}", quality_controller.current_quality());
    }
    println!();

    // Example 5: LOD presets
    println!("=== Example 5: Quality Presets ===");

    let presets = [
        QualityPreset::Ultra,
        QualityPreset::High,
        QualityPreset::Medium,
        QualityPreset::Low,
    ];

    for preset in presets {
        println!("{:?}: Quality={:.1}, Target FPS={:.0}, SSE={:.1}",
            preset,
            preset.quality_multiplier(),
            preset.target_fps(),
            preset.sse_threshold()
        );
    }
    println!();

    // Example 6: Screen Space Error calculation
    println!("=== Example 6: Screen Space Error ===");

    let distances = [1.0, 5.0, 10.0, 50.0, 100.0];
    let geometric_error = 0.1;

    println!("Geometric error: {:.3}", geometric_error);
    println!("Screen height: 1080, FOV: 45°");
    println!();

    for distance in distances {
        let sse = quality_controller.calculate_sse(
            geometric_error,
            distance,
            1080.0,
            std::f32::consts::PI / 4.0
        );

        println!("Distance {:.1}m: SSE={:.2} pixels ({:.2}%) - {}",
            distance,
            sse.error_pixels,
            sse.error_percentage,
            if sse.is_acceptable() { "✓ Acceptable" } else { "✗ Too high" }
        );
    }
    println!();

    // Example 7: Memory statistics
    println!("=== Example 7: Memory Statistics ===");
    let buffer_manager = nanite_system.buffer_manager();

    println!("Total GPU memory: {:.2} MB", buffer_manager.memory_usage_mb());
    println!("Instance buffer usage: {:.1}%", buffer_manager.instance_buffer_usage());
    println!("Total buffers: {}", buffer_manager.buffer_count());
    println!();

    println!("=== Example Complete ===");
    println!("The Nanite system successfully:");
    println!("  ✓ Clustered meshes into hierarchical structures");
    println!("  ✓ Performed view frustum culling");
    println!("  ✓ Selected appropriate LOD levels");
    println!("  ✓ Managed GPU buffers efficiently");
    println!("  ✓ Adapted quality based on performance");

    Ok(())
}

// === Helper Functions ===

fn create_cube_vertices() -> Vec<Vec3> {
    vec![
        // Front face
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0],
        // Back face
        [-1.0, -1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [1.0, 1.0, -1.0],
        [1.0, -1.0, -1.0],
    ]
}

fn create_cube_indices() -> Vec<u32> {
    vec![
        // Front
        0, 1, 2, 2, 3, 0,
        // Right
        1, 7, 6, 6, 2, 1,
        // Back
        7, 4, 5, 5, 6, 7,
        // Left
        4, 0, 3, 3, 5, 4,
        // Top
        3, 2, 6, 6, 5, 3,
        // Bottom
        4, 7, 1, 1, 0, 4,
    ]
}

fn create_sphere_vertices(segments_u: usize, segments_v: usize) -> Vec<Vec3> {
    let mut vertices = Vec::new();

    for v in 0..=segments_v {
        let theta = (v as f32 / segments_v as f32) * std::f32::consts::PI;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for u in 0..=segments_u {
            let phi = (u as f32 / segments_u as f32) * 2.0 * std::f32::consts::PI;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = cos_phi * sin_theta;
            let y = cos_theta;
            let z = sin_phi * sin_theta;

            vertices.push([x, y, z]);
        }
    }

    vertices
}

fn create_sphere_indices(segments_u: usize, segments_v: usize) -> Vec<u32> {
    let mut indices = Vec::new();

    for v in 0..segments_v {
        for u in 0..segments_u {
            let p0 = (v * (segments_u + 1) + u) as u32;
            let p1 = (p0 + 1) as u32;
            let p2 = ((v + 1) * (segments_u + 1) + u) as u32;
            let p3 = (p2 + 1) as u32;

            if v != 0 {
                indices.extend_from_slice(&[p0, p2, p1]);
            }
            if v != segments_v - 1 {
                indices.extend_from_slice(&[p1, p2, p3]);
            }
        }
    }

    indices
}

fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    let f = normalize_vec3(sub_vec3(center, eye));
    let s = normalize_vec3(cross_vec3(f, up));
    let u = cross_vec3(s, f);

    [
        [s[0], u[0], -f[0], 0.0],
        [s[1], u[1], -f[1], 0.0],
        [s[2], u[2], -f[2], 0.0],
        [-dot_vec3(s, eye), -dot_vec3(u, eye), dot_vec3(f, eye), 1.0],
    ]
}

fn create_perspective_matrix(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let f = 1.0 / (fov_y * 0.5).tan();

    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, (far + near) / (near - far), -1.0],
        [0.0, 0.0, (2.0 * far * near) / (near - far), 0.0],
    ]
}

fn sub_vec3(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot_vec3(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross_vec3(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize_vec3(v: Vec3) -> Vec3 {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, 0.0, 0.0]
    }
}
