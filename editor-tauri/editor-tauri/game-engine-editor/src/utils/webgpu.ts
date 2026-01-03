/**
 * WebGPU Integration Layer
 *
 * This module handles the integration between the frontend and WebGPU renderer.
 * It manages WebGPU device initialization, command communication, and frame rendering.
 */

/// <reference types="@webgpu/types" />

import { invoke } from '@tauri-apps/api/core';
import { Transform, Vector3 as Vector3Interface } from '../types/engine';

export interface WebGPUInitOptions {
  canvas: HTMLCanvasElement;
  width: number;
  height: number;
}

export interface WebGPUFrameStats {
  fps: number;
  frameTime: number;
  drawCalls: number;
  triangles: number;
}

export interface SceneData {
  entities: EntityData[];
  camera?: CameraData;
}

export interface EntityData {
  id: string;
  name: string;
  transform: Transform;
  visible: boolean;
  mesh?: string; // Reference to mesh ID
  material?: string; // Reference to material ID
}

export interface CameraData {
  position: Vector3Interface;
  target: Vector3Interface;
  up: Vector3Interface;
  fov: number;
  aspect: number;
  near: number;
  far: number;
}

/**
 * WebGPU Renderer Class
 *
 * Manages WebGPU rendering lifecycle and communication with Rust backend.
 */
export class WebGPURenderer {
  private device: GPUDevice | null = null;
  private context: GPUCanvasContext | null = null;
  private pipeline: GPURenderPipeline | null = null;
  private uniformBuffer: GPUBuffer | null = null;
  private vertexBuffer: GPUBuffer | null = null;
  private indexBuffer: GPUBuffer | null = null;

  private canvas: HTMLCanvasElement;

  // Performance tracking
  private lastFrameTime: number = 0;
  private frameCount: number = 0;
  private fpsUpdateTime: number = 0;
  private currentFps: number = 60;

  // Scene data
  private sceneData: SceneData = {
    entities: [],
    camera: {
      position: { x: 5, y: 5, z: 5 },
      target: { x: 0, y: 0, z: 0 },
      up: { x: 0, y: 1, z: 0 },
      fov: 45,
      aspect: 1,
      near: 0.1,
      far: 100,
    },
  };

  private isInitialized: boolean = false;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
  }

  /**
   * Initialize WebGPU device and rendering resources
   */
  async initialize(): Promise<boolean> {
    try {
      // Check WebGPU support
      if (!navigator.gpu) {
        console.error('WebGPU is not supported in this browser');
        return false;
      }

      // Request GPU adapter
      const adapter = await navigator.gpu.requestAdapter({
        powerPreference: 'high-performance',
      });

      if (!adapter) {
        console.error('Failed to get GPU adapter');
        return false;
      }

      // Request device
      this.device = await adapter.requestDevice({
        requiredFeatures: [],
        requiredLimits: {
          maxBufferSize: adapter.limits.maxBufferSize,
        },
      });

      if (!this.device) {
        console.error('Failed to get GPU device');
        return false;
      }

      // Get canvas context
      this.context = this.canvas.getContext('webgpu') as GPUCanvasContext;

      if (!this.context) {
        console.error('Failed to get WebGPU context');
        return false;
      }

      // Configure context
      this.context.configure({
        device: this.device,
        format: navigator.gpu.getPreferredCanvasFormat(),
        alphaMode: 'premultiplied',
      });

      // Create rendering resources
      await this.createPipeline();
      this.createBuffers();

      this.isInitialized = true;

      // Initialize backend renderer
      try {
        await invoke('initialize_renderer');
      } catch (error) {
        console.warn('Backend renderer initialization failed:', error);
        // Continue with frontend-only rendering
      }

      return true;
    } catch (error) {
      console.error('WebGPU initialization failed:', error);
      return false;
    }
  }

  /**
   * Create render pipeline
   */
  private async createPipeline(): Promise<void> {
    if (!this.device) {
      throw new Error('Device not initialized');
    }

    // Shader code (simplified for now)
    const shaderCode = `
      @vertex
      fn vs_main(
        @location(0) position: vec3<f32>,
        @location(1) normal: vec3<f32>,
      ) -> @builtin(position) vec4<f32> {
        return vec4<f32>(position, 1.0);
      }

      @fragment
      fn fs_main() -> @location(0) vec4<f32> {
        return vec4<f32>(0.3, 0.5, 0.7, 1.0);
      }
    `;

    const shaderModule = this.device.createShaderModule({
      code: shaderCode,
    });

    // Create pipeline
    this.pipeline = this.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: shaderModule,
        entryPoint: 'vs_main',
        buffers: [
          {
            arrayStride: 6 * 4, // 6 floats per vertex (pos + normal)
            stepMode: 'vertex',
            attributes: [
              {
                shaderLocation: 0,
                offset: 0,
                format: 'float32x3',
              },
              {
                shaderLocation: 1,
                offset: 3 * 4,
                format: 'float32x3',
              },
            ],
          },
        ],
      },
      fragment: {
        module: shaderModule,
        entryPoint: 'fs_main',
        targets: [
          {
            format: navigator.gpu.getPreferredCanvasFormat(),
          },
        ],
      },
      primitive: {
        topology: 'triangle-list',
        cullMode: 'back',
      },
      depthStencil: {
        format: 'depth24plus',
        depthWriteEnabled: true,
        depthCompare: 'less',
      },
    });
  }

  /**
   * Create GPU buffers
   */
  private createBuffers(): void {
    if (!this.device) {
      throw new Error('Device not initialized');
    }

    // Create vertex buffer (cube)
    const vertices = new Float32Array([
      // Front face
      -0.5, -0.5,  0.5,  0,  0,  1,
       0.5, -0.5,  0.5,  0,  0,  1,
       0.5,  0.5,  0.5,  0,  0,  1,
      -0.5,  0.5,  0.5,  0,  0,  1,
      // Back face
      -0.5, -0.5, -0.5,  0,  0, -1,
      -0.5,  0.5, -0.5,  0,  0, -1,
       0.5,  0.5, -0.5,  0,  0, -1,
       0.5, -0.5, -0.5,  0,  0, -1,
    ]);

    this.vertexBuffer = this.device.createBuffer({
      size: vertices.byteLength,
      usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
    });

    this.device.queue.writeBuffer(this.vertexBuffer, 0, vertices);

    // Create index buffer
    const indices = new Uint16Array([
      0, 1, 2, 0, 2, 3, // front
      4, 5, 6, 4, 6, 7, // back
      0, 4, 7, 0, 7, 3, // left
      1, 7, 6, 1, 6, 2, // right
      0, 1, 4, 1, 7, 4, // bottom
      3, 2, 6, 3, 6, 5, // top
    ]);

    this.indexBuffer = this.device.createBuffer({
      size: indices.byteLength,
      usage: GPUBufferUsage.INDEX | GPUBufferUsage.COPY_DST,
    });

    this.device.queue.writeBuffer(this.indexBuffer, 0, indices);

    // Create uniform buffer
    this.uniformBuffer = this.device.createBuffer({
      size: 256, // Enough for MVP matrix + other uniforms
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });
  }

  /**
   * Resize renderer
   */
  resize(_width: number, _height: number): void {
    if (this.sceneData.camera) {
      this.sceneData.camera.aspect = _width / _height;
    }
  }

  /**
   * Update scene data
   */
  updateScene(scene: SceneData): void {
    this.sceneData = scene;
  }

  /**
   * Render frame
   */
  render(): WebGPUFrameStats {
    const currentTime = performance.now();

    // Calculate FPS
    this.frameCount++;
    const deltaTime = currentTime - this.lastFrameTime;
    this.fpsUpdateTime += deltaTime;

    if (this.fpsUpdateTime >= 1000) {
      this.currentFps = Math.round((this.frameCount * 1000) / this.fpsUpdateTime);
      this.frameCount = 0;
      this.fpsUpdateTime = 0;
    }

    this.lastFrameTime = currentTime;

    // Actual rendering
    if (this.isInitialized && this.device && this.context && this.pipeline) {
      const commandEncoder = this.device.createCommandEncoder();

      const textureView = this.context.getCurrentTexture().createView();

      const renderPassDescriptor: GPURenderPassDescriptor = {
        colorAttachments: [
          {
            view: textureView,
            clearValue: { r: 0.1, g: 0.1, b: 0.15, a: 1.0 },
            loadOp: 'clear',
            storeOp: 'store',
          },
        ],
        depthStencilAttachment: {
          view: this.context.getCurrentTexture().createView(), // Simplified
          depthClearValue: 1.0,
          depthLoadOp: 'clear',
          depthStoreOp: 'store',
        },
      };

      const passEncoder = commandEncoder.beginRenderPass(renderPassDescriptor);

      passEncoder.setPipeline(this.pipeline);

      if (this.vertexBuffer && this.indexBuffer) {
        passEncoder.setVertexBuffer(0, this.vertexBuffer);
        passEncoder.setIndexBuffer(this.indexBuffer, 'uint16');
        passEncoder.drawIndexed(36);
      }

      passEncoder.end();

      this.device.queue.submit([commandEncoder.finish()]);
    }

    // Return frame stats
    return {
      fps: this.currentFps,
      frameTime: deltaTime,
      drawCalls: 1,
      triangles: 12,
    };
  }

  /**
   * Check if renderer is initialized
   */
  getIsInitialized(): boolean {
    return this.isInitialized;
  }

  /**
   * Get current scene data
   */
  getSceneData(): SceneData {
    return this.sceneData;
  }

  /**
   * Cleanup resources
   */
  cleanup(): void {
    // 销毁GPU缓冲区
    if (this.uniformBuffer) {
      this.uniformBuffer.destroy();
      this.uniformBuffer = null;
    }

    if (this.vertexBuffer) {
      this.vertexBuffer.destroy();
      this.vertexBuffer = null;
    }

    if (this.indexBuffer) {
      this.indexBuffer.destroy();
      this.indexBuffer = null;
    }

    // 释放管线资源
    this.pipeline = null;

    // 释放上下文
    this.context = null;

    // 销毁设备（这会自动释放所有相关资源）
    if (this.device) {
      this.device.destroy();
      this.device = null;
    }

    // 重置状态
    this.isInitialized = false;
    this.lastFrameTime = 0;
    this.frameCount = 0;
    this.currentFps = 60;

    console.log('WebGPU resources cleaned up successfully');
  }
}

/**
 * Create a WebGPU renderer instance
 */
export async function createWebGPURenderer(
  canvas: HTMLCanvasElement,
  _width: number,
  _height: number
): Promise<WebGPURenderer | null> {
  const renderer = new WebGPURenderer(canvas);
  const success = await renderer.initialize();

  return success ? renderer : null;
}
