/**
 * 代码分割和懒加载测试
 * 验证懒加载组件是否正常工作
 */

import { createLazyComponent } from './lazyLoad';
import { MaterialEditorLoadingSkeleton } from '../components/loading';

// 测试懒加载组件创建
const TestLazyComponent = createLazyComponent(
  () => new Promise<{ default: any }>((resolve) => {
    // 模拟组件加载
    setTimeout(() => {
      resolve({
        default: function TestComponent() {
          return 'Test Component Loaded';
        }
      });
    }, 100);
  }),
  <MaterialEditorLoadingSkeleton />
);

export { TestLazyComponent };

// 测试预加载功能
export async function testPreload() {
  console.log('[Lazy Load Test] Testing component preload...');

  try {
    const result = await import('../components/MaterialEditor');
    console.log('[Lazy Load Test] MaterialEditor loaded successfully');
    return true;
  } catch (error) {
    console.error('[Lazy Load Test] Failed to load MaterialEditor:', error);
    return false;
  }
}

// 测试路由预加载
export async function testRoutePreload() {
  console.log('[Lazy Load Test] Testing route preload...');

  const routes = [
    () => import('../components/MaterialEditor'),
    () => import('../components/BehaviorEditor'),
    () => import('../components/Timeline'),
    () => import('../components/AssetBrowser'),
    () => import('../components/PerformanceDashboard'),
  ];

  try {
    await Promise.all(routes.map(route => route()));
    console.log('[Lazy Load Test] All routes loaded successfully');
    return true;
  } catch (error) {
    console.error('[Lazy Load Test] Failed to load routes:', error);
    return false;
  }
}
