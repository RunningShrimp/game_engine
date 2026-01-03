//! wgpu API Compatibility Layer
//!
//! Provides compatibility adapters for different wgpu versions.
//! This module smooths over API changes between wgpu versions.

use wgpu::*;
use wgpu::{TexelCopyBufferLayout as ImageDataLayout, TexelCopyTextureInfo as ImageCopyTexture};

/// Compatibility wrapper for texture copy operations
pub mod texture {
    use super::*;

    /// Texture copy descriptor (compatible with wgpu 27+)
    #[derive(Debug, Clone)]
    pub struct TextureCopyDescriptor<'a> {
        pub texture: &'a Texture,
        pub mip_level: u32,
        pub origin: Origin3d,
        pub aspect: TextureAspect,
    }

    /// Texture data layout (compatible with wgpu 27+)
    #[derive(Debug, Clone)]
    pub struct TextureDataLayoutDescriptor {
        pub offset: u64,
        pub bytes_per_row: Option<u32>,
        pub rows_per_image: Option<u32>,
    }

    /// Convert to wgpu ImageCopyTexture (for wgpu 27+)
    impl<'a> From<TextureCopyDescriptor<'a>> for ImageCopyTexture<'a> {
        fn from(desc: TextureCopyDescriptor<'a>) -> Self {
            ImageCopyTexture {
                texture: desc.texture,
                mip_level: desc.mip_level,
                origin: desc.origin,
                aspect: desc.aspect,
            }
        }
    }
}

/// Compatibility extensions for Queue
pub trait QueueExt {
    /// Write texture with compatible parameters
    fn write_texture_compat(
        &self,
        texture: &Texture,
        mip_level: u32,
        origin: Origin3d,
        aspect: TextureAspect,
        data: &[u8],
        data_layout: &ImageDataLayout,
        size: Extent3d,
    );
}

impl QueueExt for Queue {
    fn write_texture_compat(
        &self,
        texture: &Texture,
        mip_level: u32,
        origin: Origin3d,
        aspect: TextureAspect,
        data: &[u8],
        data_layout: &ImageDataLayout,
        size: Extent3d,
    ) {
        // Use the new API directly
        self.write_texture(
            ImageCopyTexture {
                texture,
                mip_level,
                origin,
                aspect,
            },
            data,
            *data_layout,
            size,
        );
    }
}

/// Compatibility helper for RenderPassColorAttachment
pub struct RenderPassColorAttachmentBuilder<'a> {
    pub view: &'a TextureView,
    pub resolve_target: Option<&'a TextureView>,
    pub ops: Operations<wgpu::Color>,
    pub depth_slice: Option<u32>,
}

impl<'a> RenderPassColorAttachmentBuilder<'a> {
    pub fn new(view: &'a TextureView) -> Self {
        Self {
            view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: StoreOp::Store,
            },
            depth_slice: None, // Required for wgpu 27+
        }
    }

    pub fn with_resolve_target(mut self, target: &'a TextureView) -> Self {
        self.resolve_target = Some(target);
        self
    }

    pub fn with_ops(mut self, load: LoadOp<wgpu::Color>, store: StoreOp) -> Self {
        self.ops = Operations { load, store };
        self
    }

    pub fn with_depth_slice(mut self, slice: Option<u32>) -> Self {
        self.depth_slice = slice;
        self
    }

    pub fn build(self) -> RenderPassColorAttachment<'a> {
        RenderPassColorAttachment {
            view: self.view,
            resolve_target: self.resolve_target,
            ops: self.ops,
            depth_slice: self.depth_slice,
        }
    }
}

/// Compatibility helper for integer type conversions
pub mod integer {
    /// Safely convert i32 to u8 with clamping
    pub fn i32_to_u8_clamped(value: i32) -> u8 {
        value.clamp(0, 255) as u8
    }

    /// Safely convert usize to NonZeroUsize
    pub fn usize_to_nonzero(value: usize) -> Option<std::num::NonZeroUsize> {
        std::num::NonZeroUsize::new(value)
    }

    /// Safely convert usize to NonZeroUsize with default fallback
    pub fn usize_to_nonzero_or_default(value: usize) -> std::num::NonZeroUsize {
        std::num::NonZeroUsize::new(value).unwrap_or(std::num::NonZeroUsize::new(1).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i32_to_u8_clamped() {
        assert_eq!(integer::i32_to_u8_clamped(0), 0);
        assert_eq!(integer::i32_to_u8_clamped(128), 128);
        assert_eq!(integer::i32_to_u8_clamped(255), 255);
        assert_eq!(integer::i32_to_u8_clamped(300), 255);
        assert_eq!(integer::i32_to_u8_clamped(-10), 0);
    }

    #[test]
    fn test_usize_to_nonzero() {
        assert!(integer::usize_to_nonzero(0).is_none());
        assert!(integer::usize_to_nonzero(1).is_some());
        assert_eq!(integer::usize_to_nonzero_or_default(0).get(), 1);
        assert_eq!(integer::usize_to_nonzero_or_default(10).get(), 10);
    }
}
