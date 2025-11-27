# Game Engine Implementation Plan

Based on the technical review in `opt.plan1.md`, this document outlines the roadmap for addressing performance bottlenecks, implementing missing features, and improving engineering quality.

## Phase 1: Core Performance & Critical Features (Weeks 1-4)
**Focus:** 3D Rendering Performance, Asset Loading, and Skeletal Animation.

### 1.1 3D Mesh Instanced Rendering (P0)
**Goal:** Reduce Draw Calls by 70-90% for repeated meshes.
- [x] **Refactor `RenderService`**:
    - Identify compatible meshes (same geometry & material) during scene build.
    - Group render objects into `InstanceBatch` structures. (Implemented in `render_pbr` via sorting and batching)
- [x] **Update `wgpu` Pipeline**:
    - Modify vertex shaders to accept instance data (transform matrix, color tint).
    - Create instance buffers in `DoubleBufferedInstances`. (Added `instance_buffer_3d` to `WgpuRenderer`)
- [x] **Implement Batching Logic**:
    - Update `src/performance/batch_renderer.rs` to handle 3D meshes (currently likely 2D only). (Implemented directly in `wgpu.rs` for PBR)
    - Implement dynamic batching for moving objects.

### 1.2 Asynchronous Asset Processing (P0)
**Goal:** Eliminate main thread blocking during texture/model loading.
- [ ] **Offload Texture Decoding**:
    - Modify `src/render/backend.rs` or `src/resources/mod.rs`.
    - Use `tokio::task::spawn_blocking` for `image::load_from_memory`.
- [ ] **Parallel Model Loading**:
    - Ensure GLTF/OBJ parsing happens on the thread pool.
    - Upload to GPU using a staging buffer on the render thread only when data is ready.

### 1.3 Skeletal Animation System (P0)
**Goal:** Support complex character animations.
- [ ] **Data Structures**:
    - Define `Skeleton`, `Bone`, and `Skin` components in `src/animation/`.
    - Implement `SkinnedMesh` component.
- [ ] **Import Pipeline**:
    - Update asset loader to parse skinning data (weights/indices) from GLTF.
- [ ] **Runtime System**:
    - Create `skinning_system` to compute bone matrices.
    - Implement vertex shader skinning in `src/assets/shaders/pbr.wgsl`.

---

## Phase 2: Physics & Particle Systems (Weeks 5-8)
**Focus:** Physics optimization and visual effects.

### 2.1 Physics Optimization (P0/P1)
**Goal:** Reduce overhead of physics-to-transform synchronization.
- [ ] **Dirty Flag Implementation**:
    - Add `ChangeTracker` or utilize Bevy ECS `Changed<T>` filters efficiently.
    - Modify `sync_physics_to_transform_system` in `src/physics/mod.rs` to skip sleeping bodies.
- [ ] **SIMD Integration**:
    - Apply `src/performance/simd` optimizations to AABB checks if not already handled by Rapier.

### 2.2 Particle System Runtime (P1)
**Goal:** Enable the particle editor's data to function in-game.
- [ ] **Particle Emitter Component**:
    - Port editor data structures to runtime components.
- [ ] **Compute Shader Simulation**:
    - Implement GPU-based particle updates (position, velocity, lifetime) using Compute Shaders.
    - Avoid CPU-side simulation for large particle counts.
- [ ] **Renderer Integration**:
    - Render particles using instancing (quads) or point sprites.

---

## Phase 3: Engineering Quality & Extensions (Weeks 9-12)
**Focus:** Stability, Testing, and Developer Experience.

### 3.1 Testing Framework (P1)
**Goal:** Increase confidence in core systems.
- [ ] **Render Tests**:
    - Create a headless `wgpu` test harness in `src/render/tests.rs`.
    - Implement image comparison tests for regression checking.
- [ ] **Physics Unit Tests**:
    - Add deterministic tests for collision events in `src/physics/tests.rs`.

### 3.2 Plugin System (P1)
**Goal:** Allow modular extension of the engine.
- [ ] **Trait Definition**:
    - Define `EnginePlugin` trait (build, setup, update hooks).
- [ ] **Plugin Manager**:
    - Create a registry to load/unload plugins at startup.
    - Refactor existing systems (Audio, Physics) to be "internal plugins".

### 3.3 Documentation (P1)
- [ ] **API Docs**: Ensure public structs in `lib.rs` and `core/` have `///` comments.
- [ ] **Architecture Diagrams**: Create Mermaid diagrams for the Render Pipeline and ECS flow.

---

## Phase 4: Future/On-Demand (Low Priority)
- **Network System**: Evaluate `quinn` integration for multiplayer.
- **AI Navigation**: Implement A* or NavMesh integration.
- **UI Framework**: Investigate replacing/wrapping `egui` for runtime game UI.
