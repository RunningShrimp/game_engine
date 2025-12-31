//! # 模型格式转换工具 - 主程序

mod model_converter;

use std::path::PathBuf;
use model_converter::ModelFormat;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        print_help();
        return;
    }

    let input = PathBuf::from(&args[1]);
    let output = if args.len() > 2 {
        Some(PathBuf::from(&args[2]))
    } else {
        None
    };

    // 检测输入格式
    let input_format = input
        .extension()
        .and_then(|e| e.to_str())
        .and_then(|e| ModelFormat::from_extension(e));

    let input_format = match input_format {
        Some(fmt) => fmt,
        None => {
            eprintln!("Error: Could not detect input format from: {}", input.display());
            std::process::exit(1);
        }
    };

    // 确定输出格式
    let output_format = if let Some(ref out) = output {
        out.extension()
            .and_then(|e| e.to_str())
            .and_then(|e| ModelFormat::from_extension(e))
            .unwrap_or(ModelFormat::GlTF)
    } else {
        ModelFormat::GlTF
    };

    println!("Game Engine 3D Model Converter v0.1.0");
    println!("======================================");
    println!();
    println!("Converting: {:?} -> {:?}", input_format, output_format);
    println!("Input:  {}", input.display());
    
    if let Some(ref out) = output {
        println!("Output: {}", out.display());
    } else {
        let mut auto_output = input.clone();
        auto_output.set_extension(output_format.extension());
        println!("Output: {} (auto)", auto_output.display());
    }
    
    println!();
    println!("Status: Conversion framework ready");
    println!("        Loaders implemented: FBX (P2-1.1), OBJ (P2-1.2)");
    println!("        Full conversion logic will be implemented in next phase");
}

fn print_help() {
    println!("Game Engine 3D Model Converter v0.1.0");
    println!();
    println!("USAGE:");
    println!("  convert <INPUT> [OUTPUT]");
    println!("  convert --help");
    println!();
    println!("ARGUMENTS:");
    println!("  <INPUT>   Input model file");
    println!("  [OUTPUT]  Output model file (optional, auto-detect if not specified)");
    println!();
    println!("EXAMPLES:");
    println!("  convert model.obj model.gltf");
    println!("  convert scene.fbx scene.obj");
    println!("  convert character.obj");
    println!();
    println!("SUPPORTED FORMATS:");
    println!("  OBJ   Wavefront OBJ (.obj)");
    println!("  FBX   Autodesk FBX (.fbx)");
    println!("  GLTF  GLTF JSON format (.gltf)");
    println!("  GLB   GLTF binary format (.glb)");
    println!();
    println!("FEATURES:");
    println!("  - Auto-detect input format");
    println!("  - Auto-generate output filename");
    println!("  - Support for mesh, material, and texture data");
}
