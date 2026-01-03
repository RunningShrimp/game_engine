// Asset Preview Generator
// 资源预览生成器

use std::path::PathBuf;
use anyhow::{Context, Result};
use clap::Parser;
use image::{ImageBuffer, Rgb, RgbImage};
use obj::Obj;
use std::fs::File;
use std::io::BufWriter;

#[derive(Parser)]
#[command(name = "preview-generator")]
#[command(about = "Generate previews for asset store assets", long_about = None)]
struct Cli {
    /// 输入文件路径
    input: PathBuf,
    /// 输出预览图路径
    #[arg(short, long)]
    output: PathBuf,
    /// 预览图宽度
    #[arg(short, long, default_value_t = 512)]
    width: u32,
    /// 预览图高度
    #[arg(short = 'h', long, default_value_t = 512)]
    height: u32,
    /// 资源类型
    #[arg(short, long)]
    asset_type: String,
}

struct PreviewGenerator {
    width: u32,
    height: u32,
}

impl PreviewGenerator {
    fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// 为3D模型生成预览
    fn generate_model_preview(&self, model_path: &PathBuf) -> Result<RgbImage> {
        // 这里简化实现，实际应该使用渲染引擎
        // 可以集成 three.rs 或其他 3D 渲染库

        // 创建一个简单的渐变背景
        let mut img = RgbImage::new(self.width, self.height);

        for y in 0..self.height {
            for x in 0..self.width {
                let r = (x as f32 / self.width as f32 * 255.0) as u8;
                let g = (y as f32 / self.height as f32 * 255.0) as u8;
                let b = 128;
                img.put_pixel(x, y, Rgb([r, g, b]));
            }
        }

        // 尝试加载模型信息
        if let Ok(model) = Obj::load(model_path) {
            println!("Loaded 3D model: {} vertices", model.vertices.len());
            // 实际渲染应该在这里进行
        }

        Ok(img)
    }

    /// 为纹理生成预览
    fn generate_texture_preview(&self, texture_path: &PathBuf) -> Result<RgbImage> {
        // 直接加载纹理
        let img = image::open(texture_path)
            .context("Failed to open texture")?
            .to_rgb8();

        // 调整大小
        let resized = image::imageops::resize(&img, self.width, self.height, image::imageops::FilterType::Lanczos3);

        Ok(resized)
    }

    /// 为音频生成预览（波形图）
    fn generate_audio_preview(&self, audio_path: &PathBuf) -> Result<RgbImage> {
        // 这里简化实现，实际应该解析音频文件生成波形
        let mut img = RgbImage::new(self.width, self.height);

        // 创建黑色背景
        for y in 0..self.height {
            for x in 0..self.width {
                img.put_pixel(x, y, Rgb([20, 20, 20]));
            }
        }

        // 模拟波形
        let center_y = self.height / 2;
        for x in 0..self.width {
            let amplitude = (x as f32 / self.width as f32 * std::f32::consts::PI * 4.0).sin().abs()
                * (self.height / 4) as f32;
            let y1 = center_y as f32 - amplitude;
            let y2 = center_y as f32 + amplitude;

            for y in y1 as u32..=y2 as u32 {
                if y < self.height {
                    img.put_pixel(x, y, Rgb([100, 200, 255]));
                }
            }
        }

        Ok(img)
    }

    /// 为材质生成预览
    fn generate_material_preview(&self, material_path: &PathBuf) -> Result<RgbImage> {
        // 创建一个球体或平面来展示材质
        let mut img = RgbImage::new(self.width, self.height);

        // 简单渐变
        for y in 0..self.height {
            for x in 0..self.width {
                let factor = y as f32 / self.height as f32;
                let r = (200.0 * (1.0 - factor)) as u8;
                let g = (150.0 * factor) as u8;
                let b = 100;
                img.put_pixel(x, y, Rgb([r, g, b]));
            }
        }

        Ok(img)
    }

    /// 为脚本生成预览（代码截图）
    fn generate_script_preview(&self, script_path: &PathBuf) -> Result<RgbImage> {
        let mut img = RgbImage::new(self.width, self.height);

        // 创建深色背景（类似代码编辑器）
        for y in 0..self.height {
            for x in 0..self.width {
                img.put_pixel(x, y, Rgb([30, 30, 40]));
            }
        }

        // 读取脚本内容并显示
        if let Ok(content) = std::fs::read_to_string(script_path) {
            let lines: Vec<&str> = content.lines().take(20).collect();
            let font_size = 14;
            let line_height = font_size + 4;

            for (i, line) in lines.iter().enumerate() {
                let y = (i + 1) as u32 * line_height as u32;
                if y < self.height {
                    // 简化：只画行号
                    for x in 0..30 {
                        img.put_pixel(x, y, Rgb([100, 100, 100]));
                    }
                }
            }
        }

        Ok(img)
    }

    /// 生成预览
    fn generate_preview(&self, input_path: &PathBuf, asset_type: &str) -> Result<RgbImage> {
        match asset_type.to_lowercase().as_str() {
            "model" | "model_3d" => self.generate_model_preview(input_path),
            "texture" => self.generate_texture_preview(input_path),
            "audio" | "sound" => self.generate_audio_preview(input_path),
            "material" => self.generate_material_preview(input_path),
            "script" => self.generate_script_preview(input_path),
            _ => {
                // 默认：文件图标
                let mut img = RgbImage::new(self.width, self.height);
                for y in 0..self.height {
                    for x in 0..self.width {
                        img.put_pixel(x, y, Rgb([50, 50, 60]));
                    }
                }
                Ok(img)
            }
        }
    }

    /// 保存预览
    fn save_preview(&self, preview: &RgbImage, output_path: &PathBuf) -> Result<()> {
        // 确保输出目录存在
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = File::create(output_path)?;
        let ref_file = BufWriter::new(file);
        let mut encoder = image::codecs::png::PngEncoder::new(ref_file);
        encoder.encode(
            preview.as_raw(),
            self.width,
            self.height,
            image::ExtendedColorType::Rgb8,
        )?;

        Ok(())
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("🎨 Generating preview...");
    println!("Input: {:?}", cli.input);
    println!("Output: {:?}", cli.output);
    println!("Size: {}x{}", cli.width, cli.height);
    println!("Type: {}", cli.asset_type);

    if !cli.input.exists() {
        anyhow::bail!("Input file does not exist: {:?}", cli.input);
    }

    let generator = PreviewGenerator::new(cli.width, cli.height);

    println!("Processing...");
    let preview = generator.generate_preview(&cli.input, &cli.asset_type)?;

    println!("Saving preview...");
    generator.save_preview(&preview, &cli.output)?;

    println!("✅ Preview generated successfully!");
    Ok(())
}
