/**
 * 懒加载组件导出
 * 所有主要编辑器组件的懒加载版本
 */

import { lazy } from 'react';
import {
  createLazyComponent,
  DefaultLoadingSpinner,
} from '../utils/lazyLoad';
import {
  MaterialEditorLoadingSkeleton,
  BehaviorEditorLoadingSkeleton,
  TimelineLoadingSkeleton,
  AssetBrowserLoadingSkeleton,
  PerformanceDashboardLoadingSkeleton,
  AssetStoreLoadingSkeleton,
} from './components/loading';

/**
 * 材质编辑器懒加载组件
 */
export const LazyMaterialEditor = createLazyComponent(
  () => import('./components/MaterialEditor'),
  <MaterialEditorLoadingSkeleton />
);

/**
 * 行为树编辑器懒加载组件
 */
export const LazyBehaviorEditor = createLazyComponent(
  () => import('./components/BehaviorEditor'),
  <BehaviorEditorLoadingSkeleton />
);

/**
 * 时间轴懒加载组件
 */
export const LazyTimeline = createLazyComponent(
  () => import('./components/Timeline'),
  <TimelineLoadingSkeleton />
);

/**
 * 资源浏览器懒加载组件
 */
export const LazyAssetBrowser = createLazyComponent(
  () => import('./components/AssetBrowser'),
  <AssetBrowserLoadingSkeleton />
);

/**
 * 性能仪表板懒加载组件
 */
export const LazyPerformanceDashboard = createLazyComponent(
  () => import('./components/PerformanceDashboard'),
  <PerformanceDashboardLoadingSkeleton />
);

/**
 * 资源商店懒加载组件
 */
export const LazyAssetStore = createLazyComponent(
  () => import('./components/AssetStorePanel'),
  <AssetStoreLoadingSkeleton />
);

/**
 * 视口懒加载组件（核心组件，通常不需要懒加载）
 */
export const LazyViewport = createLazyComponent(
  () => import('./components/Viewport'),
  <DefaultLoadingSpinner />
);

/**
 * 实体树懒加载组件（核心组件，通常不需要懒加载）
 */
export const LazyEntityTree = createLazyComponent(
  () => import('./components/EntityTree'),
  <DefaultLoadingSpinner />
);

/**
 * 属性检查器懒加载组件（核心组件，通常不需要懒加载）
 */
export const LazyPropertyInspector = createLazyComponent(
  () => import('./components/PropertyInspector'),
  <DefaultLoadingSpinner />
);

/**
 * 工具栏懒加载组件（核心组件，通常不需要懒加载）
 */
export const LazyToolbar = createLazyComponent(
  () => import('./components/Toolbar'),
  <DefaultLoadingSpinner />
);

/**
 * 预加载所有编辑器组件
 * 可在应用启动后的空闲时间调用
 */
export async function preloadAllEditors() {
  await Promise.all([
    import('./components/MaterialEditor'),
    import('./components/BehaviorEditor'),
    import('./components/Timeline'),
    import('./components/AssetBrowser'),
    import('./components/PerformanceDashboard'),
    import('./components/AssetStorePanel'),
  ]);
  console.log('[Preload] All editor components loaded');
}
