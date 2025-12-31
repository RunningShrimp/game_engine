//! # 模型格式转换工具 - 主程序
//!
//! 这是一个独立的CLI工具，用于在不同3D模型格式之间转换。
//!
//! ## 编译和运行
//!
//! ```bash
//! # 编译
//! cargo build --release --bin convert
//!
//! # 运行
//! ./target/release/convert input.obj output.gltf
//! ```

use std::path::PathBuf;
use model_converter::{ModelConverter, ConversionOptions, ModelFormat, TextureOptions};

fn main() {
    let args = parse_args();

    if let Err(e) = run(&args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// 命令行参数
struct Args {
    input: PathBuf,
    output: Option<PathBuf>,
    input_format: Option<ModelFormat>,
    output_format: ModelFormat,
    batch_mode: bool,
    output_dir: Option<PathBuf>,
    keep_original: bool,
    recursive: bool,
    overwrite: bool,
    verbose: bool,
    embed_textures: bool,
}

/// 解析命令行参数
fn parse_args() -> Args {
    let mut args = Args {
        input: PathBuf::new(),
        output: None,
        input_format: None,
        output_format: ModelFormat::GlTF,
        batch_mode: false,
        output_dir: None,
        keep_original: true,
        recursive: true,
        overwrite: false,
        verbose: false,
        embed_textures: false,
    };

    let mut cli_args = std::env::args().skip(1);

    while let Some(arg) = cli_args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "-v" | "--verbose" => {
                args.verbose = true;
            }
            "--batch" => {
                args.batch_mode = true;
            }
            "--recursive" => {
                args.recursive = true;
            }
            "--no-recursive" => {
                args.recursive = false;
            }
            "--overwrite" => {
                args.overwrite = true;
            }
            "--delete-original" => {
                args.keep_original = false;
            }
            "--embed-textures" => {
                args.embed_textures = true;
            }
            "--from" => {
                if let Some(format_str) = cli_args.next() {
                    args.input_format = parse_format(&format_str);
                }
            }
            "--to" => {
                if let Some(format_str) = cli_args.next() {
                    args.output_format = parse_format(&format_str);
                }
            }
            "--output-dir" => {
                if let Some(dir) = cli_args.next() {
                    args.output_dir = Some(PathBuf::from(dir));
                }
            }
            input => {
                // 第一个位置参数是输入
                if args.input.as_os_str().is_empty() {
                    args.input = PathBuf::from(input);
                } else if args.output.is_none() {
                    // 第二个位置参数是输出
                    args.output = Some(PathBuf::from(input));
                } else {
                    eprintln!("Unexpected argument: {}", input);
                    std::process::exit(1);
                }
            }
        }
    }

    // 验证必填参数
    if args.input.as_os_str().is_empty() {
        eprintln!("Error: Missing input file");
        print_usage();
        std::process::exit(1);
    }

    // Batch模式需要output-dir
    if args.batch_mode && args.output_dir.is_none() {
        eprintln!("Error: Batch mode requires --output-dir");
        std::process::exit(1);
    }

    args
}

/// 解析格式字符串
fn parse_format(s: &str) -> Option<ModelFormat> {
    ModelFormat::from_extension(s.trim_start_matches('.'))
}

/// 运行转换
fn run(args: &Args) -> Result<(), String> {
    let options = ConversionOptions {
        input_format: args.input_format,
        output_format: args.output_format,
        keep_original: args.keep_original,
        recursive: args.recursive,
        overwrite: args.overwrite,
        verbose: args.verbose,
        texture_options: TextureOptions {
            embed_textures: args.embed_textures,
            ..Default::default()
        },
    };

    let mut converter = ModelConverter::new(options);

    if args.batch_mode {
        // 批量转换模式
        let output_dir = args.output_dir.as_ref().unwrap();
        println!("Converting files from {} to {} (format: {:?})",
            args.input.display(), output_dir.display(), args.output_format);

        let results = converter.convert_directory(&args.input, output_dir);
        converter.print_summary();

        let successful = results.iter().filter(|r| r.success).count();
        println!("Converted {} files", successful);

        if successful == 0 {
            return Err("No files were converted".to_string());
        }
    } else {
        // 单文件转换模式
        let output = args.output.as_ref().unwrap_or_else(|| {
            // 自动生成输出文件名
            let mut output = args.input.clone();
            output.set_extension(args.output_format.extension());
            output
        });

        println!("Converting {} to {} (to {:?})",
            args.input.display(), output.display(), args.output_format);

        let result = converter.convert_file(&args.input, output);

        if result.success {
            println!("✓ Conversion completed in {} ms", result.duration_ms);
            if result.size_diff != 0 {
                let change = if result.size_diff > 0 { "+" } else { "" };
                println!("  Size change: {}{} bytes", change, result.size_diff);
            }
        } else {
            let error = result.error.as_ref().map(|s| s.as_str()).unwrap_or("Unknown error");
            return Err(format!("Conversion failed: {}", error));
        }
    }

    Ok(())
}

/// 打印帮助信息
fn print_help() {
    println!("Game Engine 3D Model Converter");
    println!();
    println!("USAGE:");
    println!("  convert [OPTIONS] <INPUT> [OUTPUT]");
    println!("  convert --batch [OPTIONS] <INPUT_DIR> --output-dir <OUTPUT_DIR>");
    println!();
    println!("OPTIONS:");
    println!("  -h, --help              Print this help message");
    println!("  -v, --verbose           Show detailed progress");
    println!("  --batch                 Batch mode (convert all files in directory)");
    println!("  --recursive             Scan directories recursively (default)");
    println!("  --no-recursive          Do not scan directories recursively");
    println!("  --overwrite             Overwrite existing output files");
    println!("  --delete-original       Delete input files after successful conversion");
    println!("  --embed-textures        Embed textures in GLB output");
    println!("  --from <FORMAT>         Specify input format (obj/fbx/gltf/glb)");
    println!("  --to <FORMAT>           Specify output format (obj/fbx/gltf/glb)");
    println!("  --output-dir <DIR>      Output directory for batch mode");
    println!();
    println!("FORMATS:");
    println!("  obj   Wavefront OBJ");
    println!("  fbx   Autodesk FBX");
    println!("  gltf  GLTF JSON format");
    println!("  glb   GLTF binary format");
    println!();
    println!("EXAMPLES:");
    println!("  # Convert OBJ to GLTF");
    println!("  convert model.obj model.gltf");
    println!();
    println!("  # Convert all OBJ files in directory to FBX");
    println!("  convert --batch ./models --output-dir ./converted --to fbx");
    println!();
    println!("  # Convert FBX to GLB with embedded textures");
    println!("  convert --embed-textures model.fbx model.glb");
    println!();
    println!("  # Recursive conversion with custom output directory");
    println!("  convert --batch ./assets --output-dir ./build --to gltf --recursive");
}

/// 打印使用信息
fn print_usage() {
    println!("USAGE:");
    println!("  convert [OPTIONS] <INPUT> [OUTPUT]");
    println!("  convert --batch [OPTIONS] <INPUT_DIR> --output-dir <OUTPUT_DIR>");
    println!();
    println!("Run 'convert --help' for more information.");
}
