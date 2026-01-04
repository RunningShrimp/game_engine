/**
 * Toast Notification System - Type Definitions
 *
 * This file contains all TypeScript type definitions for the Toast system.
 * These types are exported from the main index file.
 */

/**
 * The four types of toast notifications available
 */
export type ToastType = 'success' | 'error' | 'info' | 'warning';

/**
 * Props for individual Toast component
 */
export interface ToastProps {
  /** Unique identifier for the toast */
  id: string;
  /** Type of toast to display */
  type: ToastType;
  /** Message content to display */
  message: string;
  /** Duration in milliseconds before auto-dismiss (default: 3000) */
  duration?: number;
  /** Callback function when toast is closed */
  onClose: (id: string) => void;
}

/**
 * Extended toast item used in the container
 */
export interface ToastItem extends ToastProps {
  id: string;
}

/**
 * Props for the Toast container component
 */
export interface ToastContainerProps {
  /** Array of active toast items to display */
  toasts: ToastItem[];
  /** Callback function when a toast is closed */
  onClose: (id: string) => void;
}

/**
 * Return type of the useToast hook
 */
export interface UseToastReturn {
  /** Array of currently active toasts */
  toasts: ToastItem[];

  /** Show a toast with specified type and duration */
  toast: (message: string, type?: ToastType, duration?: number) => void;

  /** Show a success toast (green) */
  success: (message: string, duration?: number) => void;

  /** Show an error toast (red) */
  error: (message: string, duration?: number) => void;

  /** Show an info toast (blue) */
  info: (message: string, duration?: number) => void;

  /** Show a warning toast (yellow) */
  warning: (message: string, duration?: number) => void;

  /** Manually close a toast by ID */
  closeToast: (id: string) => void;
}

/**
 * Configuration options for toast behavior
 */
export interface ToastConfig {
  /** Default duration for all toasts (milliseconds) */
  defaultDuration?: number;
  /** Maximum number of toasts to show at once */
  maxToasts?: number;
  /** Position of toast container */
  position?: 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left' | 'top-center' | 'bottom-center';
}

/**
 * Toast animation variants
 */
export type ToastAnimation = 'slide' | 'fade' | 'scale';

/**
 * Internal toast state (not exposed in API)
 */
export interface ToastState extends ToastItem {
  isVisible: boolean;
  isExiting: boolean;
  createdAt: number;
}
