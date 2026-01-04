import { useState, useCallback } from 'react';
import { ToastType, ToastItem } from './ToastContainer';

export interface UseToastReturn {
  toasts: ToastItem[];
  toast: (message: string, type?: ToastType, duration?: number) => void;
  success: (message: string, duration?: number) => void;
  error: (message: string, duration?: number) => void;
  info: (message: string, duration?: number) => void;
  warning: (message: string, duration?: number) => void;
  closeToast: (id: string) => void;
}

export const useToast = (): UseToastReturn => {
  const [toasts, setToasts] = useState<ToastItem[]>([]);

  const closeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((toast) => toast.id !== id));
  }, []);

  const toast = useCallback(
    (message: string, type: ToastType = 'info', duration: number = 3000) => {
      const id = Math.random().toString(36).substring(2, 9);
      const newToast: ToastItem = {
        id,
        type,
        message,
        duration,
        onClose: closeToast,
      };

      setToasts((prev) => [...prev, newToast]);

      // Auto-remove after duration (plus animation time)
      setTimeout(() => {
        closeToast(id);
      }, duration + 300);
    },
    [closeToast]
  );

  const success = useCallback(
    (message: string, duration?: number) => {
      toast(message, 'success', duration);
    },
    [toast]
  );

  const error = useCallback(
    (message: string, duration?: number) => {
      toast(message, 'error', duration);
    },
    [toast]
  );

  const info = useCallback(
    (message: string, duration?: number) => {
      toast(message, 'info', duration);
    },
    [toast]
  );

  const warning = useCallback(
    (message: string, duration?: number) => {
      toast(message, 'warning', duration);
    },
    [toast]
  );

  return {
    toasts,
    toast,
    success,
    error,
    info,
    warning,
    closeToast,
  };
};

export default useToast;
