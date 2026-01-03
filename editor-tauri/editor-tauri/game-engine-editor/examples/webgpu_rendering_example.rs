// WebGPU 3D渲染使用示例
// 此文件展示如何使用WebGPU渲染器

use webgpu_renderer::{WebGPURenderer, FrameStats};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 WebGPU 3D渲染示例\n");

    // 1. 创建渲染器实例
    println!("1. 初始化渲染器...");
    let mut renderer = WebGPURenderer::new();

    // 2. 初始化WebGPU设备
    println!("2. 初始化WebGPU设备...");
    renderer.initialize().await?;
    println!("   ✓ 设备初始化成功");

    // 注意：在实际应用中，你需要从canvas创建surface
    // 这里仅展示API使用方式

    // 3. 渲染循环示例
    println!("\n3. 开始渲染循环（模拟）...");
    for frame in 1..=5 {
        // 模拟渲染
        println!("   渲染帧 {}...", frame);

        // 在实际应用中，你会调用：
        // let stats = renderer.render()?;

        // 模拟帧统计
        println!("     - FPS: 60");
        println!("     - 帧时间: 16.67ms");
        println!("     - Draw calls: 2");
        println!("     - 三角形数: 24");
    }

    println!("\n✓ 渲染循环完成");

    // 4. 相机控制示例
    println!("\n4. 相机控制示例...");

    // 模拟鼠标右键按下（开始旋转）
    println!("   - 鼠标右键按下 (开始旋转)");
    renderer.handle_mouse_down(400.0, 300.0, 2);

    // 模拟鼠标移动（旋转相机）
    println!("   - 鼠标移动 (旋转相机)");
    renderer.handle_mouse_move(420.0, 280.0);
    println!("     相机位置: {:?}", renderer.camera().position);

    // 模拟鼠标滚轮（缩放）
    println!("   - 鼠标滚轮滚动 (缩放)");
    renderer.handle_scroll(1.0);
    println!("     缩放后的相机位置: {:?}", renderer.camera().position);

    println!("\n✓ 相机控制演示完成");

    // 5. 显示相机信息
    println!("\n5. 相机信息:");
    let camera = renderer.camera();
    println!("   位置: ({:.2}, {:.2}, {:.2})",
        camera.position.x, camera.position.y, camera.position.z);
    println!("   目标: ({:.2}, {:.2}, {:.2})",
        camera.target.x, camera.target.y, camera.target.z);
    println!("   FOV: {}°", camera.fov_degrees);
    println!("   宽高比: {:.2}", camera.aspect_ratio);

    println!("\n✨ 示例运行完成！");
    println!("\n提示: 在实际应用中，你需要:");
    println!("  1. 从HTML canvas创建WebGPU surface");
    println!("  2. 在requestAnimationFrame循环中调用render()");
    println!("  3. 处理鼠标/键盘事件进行相机控制");
    println!("  4. 使用Tauri命令与前端通信");

    Ok(())
}

// ============================================================================
// Tauri集成示例
// ============================================================================

/*
在前端（TypeScript/JavaScript）中使用：

import { invoke } from '@tauri-apps/api/core';

// 初始化渲染器
async function initRenderer() {
    const result = await invoke('initialize_renderer');
    console.log(result);
}

// 获取帧统计
async function getStats() {
    const stats = await invoke('get_frame_stats');
    console.log(`FPS: ${stats.fps}`);
    console.log(`帧时间: ${stats.frame_time_ms}ms`);
    console.log(`Draw calls: ${stats.draw_calls}`);
    console.log(`三角形数: ${stats.triangles}`);
}

// 创建实体
async function createEntity() {
    const entity = await invoke('create_entity', {
        name: 'MyCube'
    });
    console.log('创建的实体:', entity);
}

// 更新实体变换
async function updateTransform(entityId: string) {
    await invoke('update_entity_transform', {
        entityId: entityId,
        transform: {
            position: { x: 0, y: 1, z: 0 },
            rotation: { x: 0, y: 0, z: 0, w: 1 },
            scale: { x: 1, y: 1, z: 1 }
        }
    });
}

// 删除实体
async function deleteEntity(entityId: string) {
    await invoke('delete_entity', { entityId });
}
*/

// ============================================================================
// React组件集成示例
// ============================================================================

/*
import React, { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

export const WebGPURenderer: React.FC = () => {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const [stats, setStats] = useState({
        fps: 0,
        frameTime: 0,
        drawCalls: 0,
        triangles: 0
    });

    useEffect(() => {
        const canvas = canvasRef.current;
        if (!canvas) return;

        // 初始化WebGPU
        initWebGPU(canvas);

        // 渲染循环
        let animationId: number;
        const render = async () => {
            const frameStats = await invoke('get_frame_stats');
            setStats(frameStats);
            animationId = requestAnimationFrame(render);
        };
        render();

        return () => {
            cancelAnimationFrame(animationId);
        };
    }, []);

    const initWebGPU = async (canvas: HTMLCanvasElement) => {
        // 获取WebGPU适配器
        const adapter = await navigator.gpu.requestAdapter();
        if (!adapter) {
            console.error('未找到WebGPU适配器');
            return;
        }

        // 获取设备
        const device = await adapter.requestDevice();
        const context = canvas.getContext('webgpu');
        const format = navigator.gpu.getPreferredCanvasFormat();

        // 配置surface
        context.configure({
            device,
            format,
            alphaMode: 'premultiplied',
        });

        // 初始化Tauri后端
        await invoke('initialize_renderer');

        console.log('WebGPU初始化完成');
    };

    return (
        <div>
            <canvas ref={canvasRef} width={800} height={600} />
            <div>
                <h3>性能统计</h3>
                <p>FPS: {stats.fps}</p>
                <p>帧时间: {stats.frameTime.toFixed(2)}ms</p>
                <p>Draw calls: {stats.draw_calls}</p>
                <p>三角形数: {stats.triangles}</p>
            </div>
        </div>
    );
};
*/

// ============================================================================
// 高级用法：多几何体渲染
// ============================================================================

/*
// 在渲染循环中渲染多个几何体

fn render_multiple_objects(renderer: &mut WebGPURenderer) -> Result<(), String> {
    // 渲染网格
    renderer.render()?;

    // 你可以扩展渲染器来支持：
    // 1. 多个模型矩阵（每个物体一个）
    // 2. 不同的材质/纹理
    // 3. 批量渲染（实例化）

    Ok(())
}
*/

// ============================================================================
// 错误处理
// ============================================================================

/*
// 完整的错误处理示例

async fn safe_render(renderer: &mut WebGPURenderer) {
    match renderer.render() {
        Ok(stats) => {
            println!("渲染成功: {} FPS", stats.fps);
        }
        Err(e) => {
            eprintln!("渲染错误: {}", e);

            // 尝试恢复
            match renderer.initialize().await {
                Ok(_) => println!("渲染器已重置"),
                Err(e) => eprintln!("无法恢复: {}", e),
            }
        }
    }
}
*/
