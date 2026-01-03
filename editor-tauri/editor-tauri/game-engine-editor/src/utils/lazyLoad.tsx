import { lazy, ComponentType, Suspense, SuspenseProps } from 'react';
import { ErrorBoundary } from 'react-error-boundary';

/**
 * 错误回退组件
 */
function ErrorFallback({
  error,
  resetErrorBoundary,
}: {
  error: Error;
  resetErrorBoundary: () => void;
}) {
  return (
    <div className="flex items-center justify-center h-full bg-slate-900">
      <div className="text-center p-6">
        <div className="text-red-400 text-4xl mb-4">⚠️</div>
        <h3 className="text-lg font-semibold text-slate-200 mb-2">加载失败</h3>
        <p className="text-sm text-slate-400 mb-4">{error.message}</p>
        <button
          onClick={resetErrorBoundary}
          className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded transition-colors"
        >
          重试
        </button>
      </div>
    </div>
  );
}

/**
 * 创建懒加载组件包装器
 * @param importFn 动态导入函数
 * @param fallback 加载时的占位组件
 * @param componentName 组件名称（用于错误处理）
 */
export function createLazyComponent<T extends ComponentType<any>>(
  importFn: () => Promise<{ default: T }>,
  fallback?: SuspenseProps['fallback'],
  componentName?: string
) {
  const LazyComponent = lazy(importFn);

  return function WrappedLazyComponent(props: any) {
    return (
      <ErrorBoundary
        FallbackComponent={ErrorFallback}
        onReset={() => {
          // 清除缓存并重新加载
          window.location.reload();
        }}
        resetKeys={[componentName]}
      >
        <Suspense fallback={fallback || <DefaultLoadingSpinner />}>
          <LazyComponent {...props} />
        </Suspense>
      </ErrorBoundary>
    );
  };
}

/**
 * 默认加载动画组件
 */
export function DefaultLoadingSpinner() {
  return (
    <div className="flex items-center justify-center h-full bg-slate-900">
      <div className="flex flex-col items-center gap-4">
        <div className="w-12 h-12 border-4 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
        <p className="text-sm text-slate-400">加载中...</p>
      </div>
    </div>
  );
}

/**
 * 创建带自定义加载动画的懒加载组件
 */
export function createLazyComponentWithCustomLoading<T extends ComponentType<any>>(
  importFn: () => Promise<{ default: T }>,
  LoadingComponent: ComponentType<any>
) {
  const LazyComponent = lazy(importFn);

  return function WrappedLazyComponent(props: any) {
    return (
      <ErrorBoundary FallbackComponent={ErrorFallback}>
        <Suspense fallback={<LoadingComponent />}>
          <LazyComponent {...props} />
        </Suspense>
      </ErrorBoundary>
    );
  };
}

/**
 * 预加载组件（不渲染，只是加载到缓存）
 */
export function preloadComponent<T extends ComponentType<any>>(
  importFn: () => Promise<{ default: T }>
): Promise<void> {
  return importFn().then(() => {
    // 组件已加载到模块缓存
  });
}

/**
 * 批量预加载多个组件
 */
export async function preloadComponents(
  importFns: Array<() => Promise<any>>
): Promise<void> {
  await Promise.all(importFns.map((fn) => fn()));
}

/**
 * 延迟预加载（在空闲时预加载）
 */
export function idlePreload(importFn: () => Promise<any>, timeout = 2000): void {
  if ('requestIdleCallback' in window) {
    (window as any).requestIdleCallback(
      () => {
        importFn();
      },
      { timeout }
    );
  } else {
    // 降级方案：使用 setTimeout
    setTimeout(() => importFn(), timeout);
  }
}

/**
 * 创建带有超时的懒加载组件
 */
export function createLazyComponentWithTimeout<T extends ComponentType<any>>(
  importFn: () => Promise<{ default: T }>,
  timeoutMs = 10000,
  fallback?: SuspenseProps['fallback']
) {
  const LazyComponent = lazy(
    () =>
      new Promise<{ default: T }>((resolve, reject) => {
        const timer = setTimeout(() => {
          reject(new Error(`组件加载超时 (${timeoutMs}ms)`));
        }, timeoutMs);

        importFn()
          .then(resolve)
          .catch(reject)
          .finally(() => clearTimeout(timer));
      })
  );

  return function WrappedLazyComponent(props: any) {
    return (
      <ErrorBoundary FallbackComponent={ErrorFallback}>
        <Suspense fallback={fallback || <DefaultLoadingSpinner />}>
          <LazyComponent {...props} />
        </Suspense>
      </ErrorBoundary>
    );
  };
}
