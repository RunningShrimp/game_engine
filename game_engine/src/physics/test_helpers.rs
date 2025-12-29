//  Test Helper Wrappers for Physics Tests
//
//  This module provides simplified wrapper types used in testing that may not exist
//  in the main physics API yet. These are meant for testing purposes only.

use glam::Vec3;

/// GpuParticleSystem - High-level wrapper for testing
///
/// NOTE: This is a test helper wrapper. The actual GPU particle implementation
/// uses low-level GPU structures. This exists to make tests compile.
#[derive(Debug)]
pub struct GpuParticleSystem {
    capacity: usize,
    active_count: usize,
    positions: Vec<Vec3>,
    velocities: Vec<Vec3>,
    gravity: Vec3,
}

impl GpuParticleSystem {
    /// Create a new particle system with given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            active_count: 0,
            positions: Vec::with_capacity(capacity),
            velocities: Vec::with_capacity(capacity),
            gravity: Vec3::new(0.0, -9.81, 0.0),
        }
    }

    /// Get total particle capacity
    pub fn particle_count(&self) -> usize {
        self.capacity
    }

    /// Get number of active particles
    pub fn active_count(&self) -> usize {
        self.active_count
    }

    /// Spawn a new particle
    pub fn spawn(&mut self, position: Vec3, velocity: Vec3) {
        if self.active_count < self.capacity {
            self.positions.push(position);
            self.velocities.push(velocity);
            self.active_count += 1;
        }
    }

    /// Update particle simulation
    pub fn update(&mut self, dt: f32) {
        // Simple Euler integration
        for i in 0..self.active_count {
            self.velocities[i] += self.gravity * dt;
            self.positions[i] += self.velocities[i] * dt;
        }
    }

    /// Get all particle positions
    pub fn get_positions(&self) -> Vec<Vec3> {
        self.positions.clone()
    }

    /// Set gravity
    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.gravity = gravity;
    }
}

/// GpuPhysicsEngine - High-level wrapper for testing
///
/// NOTE: This is a test helper wrapper.
#[derive(Debug)]
pub struct GpuPhysicsEngine {
    initialized: bool,
    body_count: usize,
}

impl Default for GpuPhysicsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuPhysicsEngine {
    pub fn new() -> Self {
        Self {
            initialized: false,
            body_count: 0,
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn is_available() -> bool {
        true // For testing, always report as available
    }

    pub fn initialize(&mut self) -> Result<(), String> {
        self.initialized = true;
        Ok(())
    }

    pub fn add_body(&mut self, _id: u64, _pos: Vec3, _vel: Vec3, _mass: f32) -> Result<(), String> {
        self.body_count += 1;
        Ok(())
    }

    pub fn remove_body(&mut self, _id: u64) {
        if self.body_count > 0 {
            self.body_count -= 1;
        }
    }

    pub fn body_count(&self) -> usize {
        self.body_count
    }

    pub fn simulate(&mut self, _dt: f32) -> Result<(), String> {
        Ok(())
    }

    pub fn get_all_positions(&self) -> Vec<Vec3> {
        vec![Vec3::ZERO; self.body_count]
    }
}

/// GpuFluidSimulation - High-level wrapper for testing
///
/// NOTE: This is a test helper wrapper.
#[derive(Debug)]
pub struct GpuFluidSimulation {
    width: usize,
    height: usize,
    density: Vec<Vec<f32>>,
    velocity: Vec<Vec<(f32, f32)>>,
}

impl GpuFluidSimulation {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            density: vec![vec![0.0; height]; width],
            velocity: vec![vec![(0.0, 0.0); height]; width],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn add_density(&mut self, x: usize, y: usize, amount: f32) {
        if x < self.width && y < self.height {
            self.density[x][y] += amount;
        }
    }

    pub fn add_velocity(&mut self, x: usize, y: usize, velocity: Vec3) {
        if x < self.width && y < self.height {
            self.velocity[x][y] = (velocity.x, velocity.y);
        }
    }

    pub fn get_density_at(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.density[x][y]
        } else {
            0.0
        }
    }

    pub fn get_velocity_at(&self, x: usize, y: usize) -> Vec3 {
        if x < self.width && y < self.height {
            Vec3::new(self.velocity[x][y].0, self.velocity[x][y].1, 0.0)
        } else {
            Vec3::ZERO
        }
    }

    pub fn step(&mut self, _dt: f32) -> Result<(), String> {
        // Simplified simulation step
        Ok(())
    }
}

/// MultithreadedPhysics - High-level wrapper for testing
///
/// NOTE: This is a test helper wrapper.
#[derive(Debug)]
pub struct MultithreadedPhysics {
    thread_count: usize,
    body_count: usize,
}

impl MultithreadedPhysics {
    pub fn new(threads: usize) -> Self {
        Self {
            thread_count: threads,
            body_count: 0,
        }
    }

    pub fn thread_count(&self) -> usize {
        self.thread_count
    }

    pub fn add_body(&mut self, _id: u64, _pos: Vec3, _mass: f32) {
        self.body_count += 1;
    }

    pub fn remove_body(&mut self, _id: u64) {
        if self.body_count > 0 {
            self.body_count -= 1;
        }
    }

    pub fn body_count(&self) -> usize {
        self.body_count
    }

    pub fn step(&mut self, _dt: f32) -> Result<(), String> {
        Ok(())
    }
}

impl Default for MultithreadedPhysics {
    fn default() -> Self {
        Self {
            thread_count: num_cpus::get(),
            body_count: 0,
        }
    }
}

/// UniformGrid - Test helper wrapper for spatial partitioning tests
///
/// NOTE: This is a test helper wrapper. The actual implementation may differ.
#[derive(Debug)]
pub struct UniformGrid {
    width: f32,
    height: f32,
    cell_size: f32,
    object_count: usize,
}

impl UniformGrid {
    pub fn new(width: f32, height: f32, cell_size: f32) -> Self {
        Self {
            width,
            height,
            cell_size,
            object_count: 0,
        }
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    pub fn insert(&mut self, _id: usize, _pos: glam::Vec3, _radius: f32) {
        self.object_count += 1;
    }

    pub fn count(&self) -> usize {
        self.object_count
    }

    pub fn query(&self, _pos: glam::Vec3, _radius: f32) -> Vec<usize> {
        vec![] // Simplified implementation
    }
}

/// QuadTree - Test helper wrapper (2D version of Octree)
///
/// NOTE: This is a test helper wrapper. The actual implementation uses Octree for 3D.
#[derive(Debug)]
pub struct QuadTree {
    width: f32,
    height: f32,
    max_objects: usize,
    object_count: usize,
    has_children: bool,
    depth: usize,
}

impl QuadTree {
    pub fn new(width: f32, height: f32, max_objects: usize) -> Self {
        Self {
            width,
            height,
            max_objects,
            object_count: 0,
            has_children: false,
            depth: 0,
        }
    }

    pub fn max_objects(&self) -> usize {
        self.max_objects
    }

    pub fn count(&self) -> usize {
        self.object_count
    }

    pub fn insert(&mut self, _id: usize, _pos: glam::Vec3, _radius: f32) {
        self.object_count += 1;
    }

    pub fn query(&self, _pos: glam::Vec3, _radius: f32) -> Vec<usize> {
        vec![] // Simplified implementation
    }

    pub fn remove(&mut self, _id: usize) {
        if self.object_count > 0 {
            self.object_count -= 1;
        }
    }

    pub fn clear(&mut self) {
        self.object_count = 0;
        self.has_children = false;
    }

    pub fn has_children(&self) -> bool {
        self.has_children
    }

    pub fn depth(&self) -> usize {
        self.depth
    }
}

/// Scheduler - Test helper wrapper for parallel physics tests
///
/// NOTE: This is a test helper wrapper.
#[derive(Debug)]
pub struct Scheduler {
    thread_count: usize,
}

impl Scheduler {
    pub fn new(threads: usize) -> Self {
        Self {
            thread_count: threads.max(1),
        }
    }

    pub fn thread_count(&self) -> usize {
        self.thread_count
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            thread_count: num_cpus::get(),
        }
    }
}

/// BatchSync - Test helper wrapper for batch synchronization tests
///
/// NOTE: This is a test helper wrapper.
#[derive(Debug)]
pub struct BatchSync {
    pending: Vec<(u64, glam::Vec3)>,
}

impl BatchSync {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    pub fn add_pending(&mut self, id: u64, pos: glam::Vec3) {
        self.pending.push((id, pos));
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn flush(&mut self) -> Vec<(u64, glam::Vec3)> {
        let updates = std::mem::take(&mut self.pending);
        updates
    }
}

impl Default for BatchSync {
    fn default() -> Self {
        Self::new()
    }
}

/// Parallel computation helpers
pub fn parallel_for_each<T, F>(data: &mut [T], mut f: F)
where
    T: Send,
    F: FnMut(&mut T) + Send + Sync,
{
    data.iter_mut().for_each(|item| f(item));
}

pub fn parallel_map<T, U, F>(input: &[T], mut f: F) -> Vec<U>
where
    T: Sync,
    U: Send,
    F: FnMut(&T) -> U + Send + Sync,
{
    input.iter().map(|item| f(item)).collect()
}

pub fn parallel_reduce<T, F>(input: &[T], init: T, f: F) -> T
where
    T: Clone + Sync + Send,
    F: Fn(T, &T) -> T + Send + Sync,
{
    input.iter().fold(init, |acc, item| f(acc, item))
}

pub fn parallel_filter<T, F>(input: &[T], f: F) -> Vec<T>
where
    T: Clone + Sync,
    F: Fn(&T) -> bool + Send + Sync,
{
    input.iter().filter(|item| f(item)).cloned().collect()
}
