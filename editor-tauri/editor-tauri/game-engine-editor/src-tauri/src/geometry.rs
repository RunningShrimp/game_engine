use glam::Vec3;

/// Vertex data structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

// Note: bytemuck traits are implemented in webgpu_renderer.rs to avoid orphan rule

impl Vertex {
    pub fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Self {
        Self {
            position,
            normal,
            uv,
        }
    }
}

/// Generate a cube mesh
pub fn create_cube(size: f32) -> (Vec<Vertex>, Vec<u16>) {
    let half = size / 2.0;
    let vertices = vec![
        // Front face
        Vertex::new([-half, -half, half], [0.0, 0.0, 1.0], [0.0, 0.0]),
        Vertex::new([half, -half, half], [0.0, 0.0, 1.0], [1.0, 0.0]),
        Vertex::new([half, half, half], [0.0, 0.0, 1.0], [1.0, 1.0]),
        Vertex::new([-half, half, half], [0.0, 0.0, 1.0], [0.0, 1.0]),
        // Back face
        Vertex::new([half, -half, -half], [0.0, 0.0, -1.0], [0.0, 0.0]),
        Vertex::new([-half, -half, -half], [0.0, 0.0, -1.0], [1.0, 0.0]),
        Vertex::new([-half, half, -half], [0.0, 0.0, -1.0], [1.0, 1.0]),
        Vertex::new([half, half, -half], [0.0, 0.0, -1.0], [0.0, 1.0]),
        // Top face
        Vertex::new([-half, half, half], [0.0, 1.0, 0.0], [0.0, 0.0]),
        Vertex::new([half, half, half], [0.0, 1.0, 0.0], [1.0, 0.0]),
        Vertex::new([half, half, -half], [0.0, 1.0, 0.0], [1.0, 1.0]),
        Vertex::new([-half, half, -half], [0.0, 1.0, 0.0], [0.0, 1.0]),
        // Bottom face
        Vertex::new([-half, -half, -half], [0.0, -1.0, 0.0], [0.0, 0.0]),
        Vertex::new([half, -half, -half], [0.0, -1.0, 0.0], [1.0, 0.0]),
        Vertex::new([half, -half, half], [0.0, -1.0, 0.0], [1.0, 1.0]),
        Vertex::new([-half, -half, half], [0.0, -1.0, 0.0], [0.0, 1.0]),
        // Right face
        Vertex::new([half, -half, half], [1.0, 0.0, 0.0], [0.0, 0.0]),
        Vertex::new([half, -half, -half], [1.0, 0.0, 0.0], [1.0, 0.0]),
        Vertex::new([half, half, -half], [1.0, 0.0, 0.0], [1.0, 1.0]),
        Vertex::new([half, half, half], [1.0, 0.0, 0.0], [0.0, 1.0]),
        // Left face
        Vertex::new([-half, -half, -half], [-1.0, 0.0, 0.0], [0.0, 0.0]),
        Vertex::new([-half, -half, half], [-1.0, 0.0, 0.0], [1.0, 0.0]),
        Vertex::new([-half, half, half], [-1.0, 0.0, 0.0], [1.0, 1.0]),
        Vertex::new([-half, half, -half], [-1.0, 0.0, 0.0], [0.0, 1.0]),
    ];

    let indices = vec![
        0, 1, 2, 2, 3, 0, // Front
        4, 5, 6, 6, 7, 4, // Back
        8, 9, 10, 10, 11, 8, // Top
        12, 13, 14, 14, 15, 12, // Bottom
        16, 17, 18, 18, 19, 16, // Right
        20, 21, 22, 22, 23, 20, // Left
    ];

    (vertices, indices)
}

/// Generate a sphere mesh
pub fn create_sphere(radius: f32, segments: u32, rings: u32) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices
    for ring in 0..=rings {
        let theta = (ring as f32 / rings as f32) * std::f32::consts::PI;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for segment in 0..=segments {
            let phi = (segment as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = cos_phi * sin_theta;
            let y = cos_theta;
            let z = sin_phi * sin_theta;
            let normal = [x, y, z];
            let position = [x * radius, y * radius, z * radius];

            let u = segment as f32 / segments as f32;
            let v = ring as f32 / rings as f32;

            vertices.push(Vertex::new(position, normal, [u, v]));
        }
    }

    // Generate indices
    for ring in 0..rings {
        for segment in 0..segments {
            let current = ring * (segments + 1) + segment;
            let next = current + (segments + 1);

            indices.push(current as u16);
            indices.push(next as u16);
            indices.push((current + 1) as u16);

            indices.push((current + 1) as u16);
            indices.push(next as u16);
            indices.push((next + 1) as u16);
        }
    }

    (vertices, indices)
}

/// Generate a grid plane
pub fn create_grid(size: f32, divisions: u32) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let half = size / 2.0;
    let step = size / divisions as f32;

    // Generate grid lines
    for i in 0..=divisions {
        let pos = -half + i as f32 * step;

        // Line along X axis
        vertices.push(Vertex::new([-half, 0.0, pos], [0.0, 1.0, 0.0], [0.0, 0.0]));
        vertices.push(Vertex::new([half, 0.0, pos], [0.0, 1.0, 0.0], [1.0, 0.0]));

        // Line along Z axis
        vertices.push(Vertex::new([pos, 0.0, -half], [0.0, 1.0, 0.0], [0.0, 0.0]));
        vertices.push(Vertex::new([pos, 0.0, half], [0.0, 1.0, 0.0], [1.0, 0.0]));
    }

    // Generate indices for lines
    for i in 0..vertices.len() as u16 {
        indices.push(i);
    }

    (vertices, indices)
}

/// Mesh structure for rendering
#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Mesh {
    pub fn cube(size: f32) -> Self {
        let (vertices, indices) = create_cube(size);
        Self { vertices, indices }
    }

    pub fn sphere(radius: f32, segments: u32, rings: u32) -> Self {
        let (vertices, indices) = create_sphere(radius, segments, rings);
        Self { vertices, indices }
    }

    pub fn grid(size: f32, divisions: u32) -> Self {
        let (vertices, indices) = create_grid(size, divisions);
        Self { vertices, indices }
    }
}
