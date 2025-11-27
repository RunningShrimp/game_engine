import { useEffect, useRef, useState } from 'react';
import { Card } from './ui/card';
import { Box } from 'lucide-react';

// WebGPU类型扩展
declare global {
  interface Navigator {
    gpu?: any;
  }
}

interface Renderer3DProps {
  width?: number;
  height?: number;
  className?: string;
}

export default function Renderer3D({ width, height, className }: Renderer3DProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [isWebGPUSupported, setIsWebGPUSupported] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const initRenderer = async () => {
      if (!canvasRef.current) return;

      // 检查WebGPU支持
      if (!('gpu' in navigator)) {
        setError('WebGPU不支持，请使用支持WebGPU的浏览器');
        setIsWebGPUSupported(false);
        return;
      }

      try {
        const adapter = await navigator.gpu.requestAdapter();
        if (!adapter) {
          setError('无法获取GPU适配器');
          return;
        }

        const device = await adapter.requestDevice();
        const context = canvasRef.current.getContext('webgpu') as any;
        
        if (!context) {
          setError('无法获取WebGPU上下文');
          return;
        }

        const presentationFormat = (navigator.gpu as any).getPreferredCanvasFormat();
        context.configure({
          device,
          format: presentationFormat,
          alphaMode: 'premultiplied',
        });

        setIsWebGPUSupported(true);

        // 简单的渲染循环
        const render = () => {
          const commandEncoder = device.createCommandEncoder();
          const textureView = context.getCurrentTexture().createView();

          const renderPassDescriptor = {
            colorAttachments: [
              {
                view: textureView,
                clearValue: { r: 0.1, g: 0.1, b: 0.15, a: 1.0 },
                loadOp: 'clear',
                storeOp: 'store',
              },
            ],
          };

          const passEncoder = commandEncoder.beginRenderPass(renderPassDescriptor);
          passEncoder.end();

          device.queue.submit([commandEncoder.finish()]);
          requestAnimationFrame(render);
        };

        render();
      } catch (err) {
        console.error('WebGPU初始化失败:', err);
        setError(`WebGPU初始化失败: ${err instanceof Error ? err.message : String(err)}`);
        setIsWebGPUSupported(false);
      }
    };

    initRenderer();
  }, []);

  if (error || !isWebGPUSupported) {
    return (
      <div className={`flex items-center justify-center bg-background ${className}`}>
        <Card className="p-8 text-center bg-card/80 backdrop-blur-md max-w-md">
          <div className="w-16 h-16 mx-auto mb-4 rounded-lg bg-primary/20 flex items-center justify-center">
            <Box className="w-8 h-8 text-primary" />
          </div>
          <h3 className="text-lg font-semibold mb-2">3D渲染器</h3>
          {error ? (
            <p className="text-sm text-destructive mb-4">{error}</p>
          ) : (
            <p className="text-sm text-muted-foreground mb-4">
              正在初始化WebGPU渲染器...
            </p>
          )}
          <div className="text-xs text-muted-foreground space-y-1">
            <div>提示: 需要Chrome 113+或Edge 113+</div>
            <div>并启用WebGPU实验性功能</div>
          </div>
        </Card>
      </div>
    );
  }

  return (
    <canvas
      ref={canvasRef}
      width={width || 800}
      height={height || 600}
      className={className}
      style={{ display: 'block', width: '100%', height: '100%' }}
    />
  );
}
