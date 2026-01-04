import React from 'react';
import { createPortal } from 'react-dom';
import Toast, { ToastProps } from './Toast';
import styles from './Toast.module.css';

export interface ToastItem extends ToastProps {
  id: string;
}

export interface ToastContainerProps {
  toasts: ToastItem[];
  onClose: (id: string) => void;
}

const ToastContainer: React.FC<ToastContainerProps> = ({ toasts, onClose }) => {
  if (typeof window === 'undefined') return null;

  return createPortal(
    <div className={styles.container}>
      {toasts.map((toast) => (
        <Toast
          key={toast.id}
          id={toast.id}
          type={toast.type}
          message={toast.message}
          duration={toast.duration}
          onClose={onClose}
        />
      ))}
    </div>,
    document.body
  );
};

export default ToastContainer;
