/**
 * 预加载策略工具
 * 用于优化路由和组件的加载时机
 */

import { preloadComponent, idlePreload } from './lazyLoad';

/**
 * 路由预加载映射表
 */
const routePreloadMap: Record<string, () => Promise<any>> = {
  '/material-editor': () => import('../components/MaterialEditor'),
  '/behavior-editor': () => import('../components/BehaviorEditor'),
  '/timeline': () => import('../components/Timeline'),
  '/asset-browser': () => import('../components/AssetBrowser'),
  '/performance-dashboard': () => import('../components/PerformanceDashboard'),
};

/**
 * 预加载指定路由
 * @param routePath 路由路径
 */
export async function preloadRoute(routePath: string): Promise<void> {
  const importFn = routePreloadMap[routePath];
  if (importFn) {
    try {
      await preloadComponent(importFn);
      console.log(`[Preload] Successfully loaded: ${routePath}`);
    } catch (error) {
      console.error(`[Preload] Failed to load: ${routePath}`, error);
    }
  }
}

/**
 * 批量预加载多个路由
 * @param routePaths 路由路径数组
 */
export async function preloadRoutes(routePaths: string[]): Promise<void> {
  const importFns = routePaths
    .map((path) => routePreloadMap[path])
    .filter(Boolean);

  try {
    await Promise.all(importFns.map((fn) => fn()));
    console.log(`[Preload] Successfully loaded ${importFns.length} routes`);
  } catch (error) {
    console.error('[Preload] Failed to load routes', error);
  }
}

/**
 * 在鼠标悬停时预加载路由
 * 用于导航链接的 hover 优化
 */
export function setupPreloadOnHover(): void {
  document.querySelectorAll('[data-preload]').forEach((element) => {
    element.addEventListener('mouseenter', () => {
      const route = element.getAttribute('data-preload');
      if (route) {
        idlePreload(() => preloadRoute(route), 1000);
      }
    });
  });
}

/**
 * 在空闲时预加载关键路由
 */
export function setupIdlePreload(): void {
  // 预加载最常用的路由
  const criticalRoutes = ['/material-editor', '/behavior-editor'];

  if ('requestIdleCallback' in window) {
    (window as any).requestIdleCallback(
      () => {
        console.log('[Preload] Starting idle preload...');
        preloadRoutes(criticalRoutes);
      },
      { timeout: 3000 }
    );
  } else {
    setTimeout(() => {
      console.log('[Preload] Starting idle preload...');
      preloadRoutes(criticalRoutes);
    }, 3000);
  }
}

/**
 * 基于用户行为的智能预加载
 * 根据用户访问频率预测下一个可能访问的路由
 */
class RoutePredictor {
  private routeHistory: string[] = [];
  private maxHistorySize = 10;

  recordVisit(routePath: string): void {
    this.routeHistory.push(routePath);
    if (this.routeHistory.length > this.maxHistorySize) {
      this.routeHistory.shift();
    }
  }

  predictNextRoute(): string | null {
    if (this.routeHistory.length < 2) {
      return null;
    }

    // 简单的马尔可夫链预测：基于最后访问的路由预测下一个
    const lastRoute = this.routeHistory[this.routeHistory.length - 1];
    const commonPatterns: Record<string, string> = {
      '/material-editor': '/behavior-editor',
      '/behavior-editor': '/timeline',
      '/timeline': '/asset-browser',
    };

    return commonPatterns[lastRoute] || null;
  }
}

const routePredictor = new RoutePredictor();

/**
 * 记录路由访问并预加载预测的路由
 */
export function recordRouteAndPreload(routePath: string): void {
  routePredictor.recordVisit(routePath);

  const predictedRoute = routePredictor.predictNextRoute();
  if (predictedRoute) {
    idlePreload(() => preloadRoute(predictedRoute!), 2000);
  }
}

/**
 * 网络信息感知的预加载
 * 根据网络连接质量决定是否预加载
 */
export function shouldPreload(): boolean {
  if ('connection' in navigator) {
    const conn = (navigator as any).connection;
    const effectiveType = conn.effectiveType; // '4g', '3g', '2g', 'slow-2g'
    const saveData = conn.saveData; // 用户是否开启了省流量模式

    // 如果开启了省流量模式或网络较慢，不预加载
    if (saveData || effectiveType === '2g' || effectiveType === 'slow-2g') {
      return false;
    }
  }

  return true;
}

/**
 * 智能预加载：结合网络状态和空闲时间
 */
export function smartPreload(routePath: string): void {
  if (!shouldPreload()) {
    console.log('[Preload] Skipped due to network constraints');
    return;
  }

  idlePreload(() => preloadRoute(routePath), 2000);
}

/**
 * 预加载图片资源
 */
export function preloadImages(imageUrls: string[]): void {
  imageUrls.forEach((url) => {
    const link = document.createElement('link');
    link.rel = 'preload';
    link.as = 'image';
    link.href = url;
    document.head.appendChild(link);
  });
}

/**
 * 预加载字体
 */
export function preloadFonts(fontUrls: string[]): void {
  fontUrls.forEach((url) => {
    const link = document.createElement('link');
    link.rel = 'preload';
    link.as = 'font';
    link.type = 'font/woff2';
    link.crossOrigin = 'anonymous';
    link.href = url;
    document.head.appendChild(link);
  });
}

/**
 * 设置组件级别的预加载
 * 在特定组件可见时预加载其他组件
 */
export function setupComponentVisibilityPreload(): void {
  if ('IntersectionObserver' in window) {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            const target = entry.target as HTMLElement;
            const preloadTarget = target.getAttribute('data-preload-component');
            if (preloadTarget) {
              idlePreload(() => preloadRoute(preloadTarget), 1000);
              observer.unobserve(target);
            }
          }
        });
      },
      { rootMargin: '50px' }
    );

    document.querySelectorAll('[data-preload-component]').forEach((element) => {
      observer.observe(element);
    });
  }
}

/**
 * 初始化所有预加载策略
 */
export function initPreloadStrategies(): void {
  // 延迟初始化，避免影响首屏加载
  setTimeout(() => {
    setupPreloadOnHover();
    setupIdlePreload();
    setupComponentVisibilityPreload();
    console.log('[Preload] All preload strategies initialized');
  }, 1000);
}
