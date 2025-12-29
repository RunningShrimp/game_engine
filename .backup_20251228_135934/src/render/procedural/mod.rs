//  程序化内容生成模块
//
//  提供程序化内容生成功能：
//  - 噪声生成（Perlin、Simplex、Worley）
//  - 网格生成（地形、洞穴、基础几何）
//  - 纹理生成（云层、大理石、木纹）
//
//  ## 架构
//
//  ```text
//  procedural/
//  ├── noise.rs              噪声算法
//  ├── mesh_generator.rs     网格生成器
//  ├── texture_generator.rs  纹理生成器
//  └── mod.rs               模块声明
//  ```
//
//  ## 使用示例
//
//  ```rust,no_run
//  // 生成地形
//  let terrain_gen = TerrainGenerator::default_config();
//  let terrain = terrain_gen.generate();
//
//  // 生成云层纹理
//  let cloud_gen = CloudTextureGenerator::default_config();
//  let cloud_texture = cloud_gen.generate(512, 512);
//
//  // 生成噪声
//  let perlin = PerlinNoise::new(42);
//  let value = perlin.noise2d(0.5, 0.5);
//  ```

pub mod mesh_generator;
pub mod mesh_simplification;
pub mod noise;
pub mod texture_generator;

// 重新导出噪声相关类型
pub use noise::{
    NoiseGenerator, PerlinNoise, SimplexNoise, WorleyNoise,
    NoiseConfig,
};

// 重新导出网格生成器相关类型
pub use mesh_generator::{
    MeshGenerator, PrimitiveGenerator, TerrainGenerator, CaveGenerator,
};

// 重新导出纹理生成器相关类型
pub use texture_generator::{
    TextureGenerator,
    CloudTextureGenerator, MarbleTextureGenerator, WoodTextureGenerator,
    NoiseTextureGenerator,
    NoiseType, ColorMode,
};

// 重新导出网格简化相关类型
pub use mesh_simplification::{
    MeshSimplifier, SimplificationConfig, SimplificationStats,
    simplify_mesh, LODGenerator,
};
