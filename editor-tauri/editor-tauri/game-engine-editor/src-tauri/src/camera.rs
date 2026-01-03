use glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};

/// Camera controller for 3D viewport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    /// Camera position in world space
    pub position: Vec3,
    /// Target point to look at
    pub target: Vec3,
    /// Up vector
    pub up: Vec3,
    /// Field of view in degrees
    pub fov_degrees: f32,
    /// Near clipping plane distance
    pub near: f32,
    /// Far clipping plane distance
    pub aspect_ratio: f32,
}

impl Camera {
    /// Create a new camera
    pub fn new(position: Vec3, target: Vec3, fov_degrees: f32, aspect_ratio: f32) -> Self {
        Self {
            position,
            target,
            up: Vec3::Y,
            fov_degrees,
            near: 0.1,
            aspect_ratio,
        }
    }

    /// Create a default camera positioned to look at origin
    pub fn default_camera(aspect_ratio: f32) -> Self {
        Self::new(
            Vec3::new(5.0, 5.0, 5.0),
            Vec3::new(0.0, 0.0, 0.0),
            45.0,
            aspect_ratio,
        )
    }

    /// Calculate view matrix
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// Calculate projection matrix
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh_gl(
            self.fov_degrees.to_radians(),
            self.aspect_ratio,
            self.near,
            1000.0,
        )
    }

    /// Get the combined view-projection matrix
    pub fn view_proj_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Rotate camera around target (orbit control)
    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        let offset = self.position - self.target;
        let radius = offset.length();

        // Convert to spherical coordinates
        let mut yaw = offset.y.atan2(offset.x);
        let mut pitch = (offset.z / radius).acos();

        // Update angles
        yaw += delta_yaw;
        pitch = (pitch + delta_pitch).clamp(0.1, std::f32::consts::PI - 0.1);

        // Convert back to Cartesian
        let x = radius * pitch.sin() * yaw.cos();
        let y = radius * pitch.sin() * yaw.sin();
        let z = radius * pitch.cos();

        self.position = self.target + Vec3::new(x, y, z);
    }

    /// Pan camera
    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(self.up).normalize();
        let up = right.cross(forward).normalize();

        self.position += right * delta_x + up * delta_y;
        self.target += right * delta_x + up * delta_y;
    }

    /// Zoom camera
    pub fn zoom(&mut self, delta: f32) {
        let direction = (self.target - self.position).normalize();
        let distance = self.position.distance(self.target);

        let new_distance = (distance - delta).clamp(1.0, 100.0);
        self.position = self.target - direction * new_distance;
    }

    /// Update aspect ratio
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::default_camera(16.0 / 9.0)
    }
}

/// Camera controller state
#[derive(Debug, Clone, Copy)]
pub struct CameraController {
    pub orbiting: bool,
    pub panning: bool,
    pub last_mouse_x: f32,
    pub last_mouse_y: f32,
    pub orbit_speed: f32,
    pub pan_speed: f32,
    pub zoom_speed: f32,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            orbiting: false,
            panning: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
            orbit_speed: 0.005,
            pan_speed: 0.01,
            zoom_speed: 0.1,
        }
    }

    pub fn handle_mouse_down(&mut self, x: f32, y: f32, button: u32) {
        match button {
            2 => self.orbiting = true,  // Right click
            1 => self.panning = true,   // Middle click
            _ => {}
        }
        self.last_mouse_x = x;
        self.last_mouse_y = y;
    }

    pub fn handle_mouse_up(&mut self, _button: u32) {
        self.orbiting = false;
        self.panning = false;
    }

    pub fn handle_mouse_move(&mut self, x: f32, y: f32, camera: &mut Camera) {
        let dx = x - self.last_mouse_x;
        let dy = y - self.last_mouse_y;

        if self.orbiting {
            camera.orbit(dx * self.orbit_speed, dy * self.orbit_speed);
        } else if self.panning {
            let distance = camera.position.distance(camera.target);
            let pan_scale = distance * self.pan_speed;
            camera.pan(-dx * pan_scale, dy * pan_scale);
        }

        self.last_mouse_x = x;
        self.last_mouse_y = y;
    }

    pub fn handle_scroll(&mut self, delta: f32, camera: &mut Camera) {
        camera.zoom(delta * self.zoom_speed);
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self::new()
    }
}
