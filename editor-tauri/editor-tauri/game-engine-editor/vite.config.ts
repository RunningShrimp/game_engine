import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },

  // 构建优化配置
  build: {
    // 目标浏览器
    target: 'esnext',

    // 启用 CSS 代码分割
    cssCodeSplit: true,

    // Chunk 大小警告阈值 (500KB)
    chunkSizeWarningLimit: 500,

    // Rollup 配置
    rollupOptions: {
      output: {
        // 手动代码分割策略
        manualChunks: (id) => {
          // 第三方库分离
          if (id.includes('node_modules')) {
            // React 核心库
            if (id.includes('react') || id.includes('react-dom')) {
              return 'vendor-react';
            }

            // 图表库
            if (id.includes('recharts')) {
              return 'vendor-charts';
            }

            // 图标库
            if (id.includes('lucide-react')) {
              return 'vendor-icons';
            }

            // WebGPU 类型
            if (id.includes('@webgpu/types')) {
              return 'vendor-webgpu';
            }

            // Tauri API
            if (id.includes('@tauri-apps')) {
              return 'vendor-tauri';
            }

            // 其他第三方库
            return 'vendor';
          }

          // 编辑器组件分离
          if (id.includes('/components/MaterialEditor')) {
            return 'editor-material';
          }

          if (id.includes('/components/BehaviorEditor')) {
            return 'editor-behavior';
          }

          if (id.includes('/components/Timeline')) {
            return 'editor-timeline';
          }

          if (id.includes('/components/AssetBrowser')) {
            return 'editor-assets';
          }

          if (id.includes('/components/PerformanceDashboard')) {
            return 'editor-performance';
          }

          if (id.includes('/components/Viewport')) {
            return 'editor-viewport';
          }

          if (id.includes('/components/EntityTree')) {
            return 'editor-entity-tree';
          }

          if (id.includes('/components/PropertyInspector')) {
            return 'editor-property-inspector';
          }

          if (id.includes('/components/Toolbar')) {
            return 'editor-toolbar';
          }
        },

        // Chunk 文件命名策略
        chunkFileNames: 'assets/js/[name]-[hash].js',
        entryFileNames: 'assets/js/[name]-[hash].js',
        assetFileNames: (assetInfo) => {
          const info = assetInfo.name?.split('.') || [];
          let extType = info[info.length - 1];

          // CSS 文件
          if (extType === 'css') {
            return 'assets/css/[name]-[hash][extname]';
          }

          // 图片文件
          if (/\.(png|jpe?g|gif|svg|webp|avif)$/.test(assetInfo.name || '')) {
            return 'assets/images/[name]-[hash][extname]';
          }

          // 字体文件
          if (/\.(woff2?|eot|ttf|otf)$/.test(assetInfo.name || '')) {
            return 'assets/fonts/[name]-[hash][extname]';
          }

          // 其他资源
          return 'assets/[name]-[hash][extname]';
        },
      },
    },

    // 压缩配置
    minify: 'terser',
    terserOptions: {
      compress: {
        // 删除 console
        drop_console: true,
        // 删除 debugger
        drop_debugger: true,
        // 纯函数优化
        pure_funcs: ['console.log', 'console.info', 'console.debug'],
      },
      format: {
        // 删除注释
        comments: false,
      },
    },

    // Source Map 配置（生产环境关闭）
    sourcemap: false,
  },

  // 依赖预构建配置
  optimizeDeps: {
    include: [
      'react',
      'react-dom',
      'recharts',
      'lucide-react',
    ],
    exclude: ['@webgpu/types'],
  },

  // CSS 配置
  css: {
    modules: {
      // CSS Modules 的 localsConvention
      localsConvention: 'camelCase',
    },
    preprocessorOptions: {
      // 如果需要使用 SCSS/Less 等，可以在这里配置
    },
    devSourcemap: false,
  },
}));
