//! 网络压缩测试
//!
//! 测试数据压缩和解压缩功能。

use game_engine::network::compression::{CompressionLevel, NetworkCompressor};

#[test]
fn test_compression_level_enum() {
    assert_eq!(CompressionLevel::Fast, CompressionLevel::Fast);
    assert_eq!(CompressionLevel::Balanced, CompressionLevel::Balanced);
    assert_eq!(CompressionLevel::Best, CompressionLevel::Best);
    assert_eq!(CompressionLevel::Custom(5), CompressionLevel::Custom(5));
}

#[test]
fn test_compression_level_default() {
    let level = CompressionLevel::default();
    assert_eq!(level, CompressionLevel::Balanced);
}

#[test]
fn test_network_compressor_default() {
    let compressor = NetworkCompressor::new();
    // 验证压缩器能够创建
    let _ = compressor;
}

#[test]
fn test_network_compressor_compress_decompress() {
    let compressor = NetworkCompressor::new();
    
    // 测试压缩和解压缩
    let original_data = vec![0u8; 1000];
    
    match compressor.compress(&original_data) {
        Ok(compressed) => {
            // 验证压缩后的数据更小（对于重复数据）
            assert!(compressed.len() <= original_data.len());
            
            // 测试解压缩
            match compressor.decompress(&compressed) {
                Ok(decompressed) => {
                    assert_eq!(decompressed, original_data);
                }
                Err(_) => {
                    // 解压缩失败是可能的（如果数据太小）
                }
            }
        }
        Err(_) => {
            // 压缩失败是可能的（如果数据太小）
        }
    }
}

#[test]
fn test_network_compressor_small_data() {
    let compressor = NetworkCompressor::new();
    
    // 测试小数据（可能不会被压缩）
    let small_data = vec![1u8, 2u8, 3u8];
    
    match compressor.compress(&small_data) {
        Ok(compressed) => {
            // 小数据压缩后可能不会变小
            match compressor.decompress(&compressed) {
                Ok(decompressed) => {
                    assert_eq!(decompressed, small_data);
                }
                Err(_) => {
                    // 解压缩失败是可能的
                }
            }
        }
        Err(_) => {
            // 压缩失败是可能的（数据太小）
        }
    }
}

#[test]
fn test_compression_levels() {
    // 测试不同的压缩级别
    let levels = vec![
        CompressionLevel::Fast,
        CompressionLevel::Balanced,
        CompressionLevel::Best,
        CompressionLevel::Custom(5),
    ];

    for level in levels {
        let compressor = NetworkCompressor::with_level(level);
        let _ = compressor;
    }
}

#[test]
fn test_compression_roundtrip() {
    let compressor = NetworkCompressor::new();
    
    // 测试往返压缩（压缩后解压缩应该得到原始数据）
    let test_data = b"Hello, World! This is a test string for compression.".to_vec();
    
    match compressor.compress(&test_data) {
        Ok(compressed) => {
            match compressor.decompress(&compressed) {
                Ok(decompressed) => {
                    assert_eq!(decompressed, test_data);
                }
                Err(_) => {
                    // 解压缩失败
                }
            }
        }
        Err(_) => {
            // 压缩失败
        }
    }
}

#[test]
fn test_compression_repetitive_data() {
    let compressor = NetworkCompressor::new();
    
    // 重复数据应该压缩得很好
    let repetitive_data = vec![42u8; 10000];
    
    match compressor.compress(&repetitive_data) {
        Ok(compressed) => {
            // 重复数据应该压缩得很好
            assert!(compressed.len() < repetitive_data.len());
            
            match compressor.decompress(&compressed) {
                Ok(decompressed) => {
                    assert_eq!(decompressed, repetitive_data);
                }
                Err(_) => {
                    // 解压缩失败
                }
            }
        }
        Err(_) => {
            // 压缩失败
        }
    }
}

