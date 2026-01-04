import React from 'react';
import { useToast } from './useToast';
import ToastContainer from './ToastContainer';

/**
 * Example component demonstrating how to use the Toast notification system
 *
 * Usage in your app:
 * 1. Wrap your root component with ToastContainer
 * 2. Use useToast hook in any component
 * 3. Call toast methods to show notifications
 */
export const ToastExample: React.FC = () => {
  const { toasts, toast, success, error, info, warning } = useToast();

  return (
    <div style={{ padding: '20px' }}>
      <ToastContainer toasts={toasts} onClose={(id) => console.log('Closed:', id)} />

      <h2>Toast Notification System Demo</h2>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '10px', maxWidth: '300px' }}>
        <button onClick={() => success('Operation completed successfully!')}>
          Show Success Toast
        </button>

        <button onClick={() => error('An error occurred while processing your request.')}>
          Show Error Toast
        </button>

        <button onClick={() => info('You have a new message.')}>
          Show Info Toast
        </button>

        <button onClick={() => warning('Please review your settings before continuing.')}>
          Show Warning Toast
        </button>

        <button onClick={() => toast('Custom toast message', 'info', 5000)}>
          Show Custom Duration Toast (5s)
        </button>

        <button
          onClick={() => {
            success('First toast');
            setTimeout(() => error('Second toast'), 500);
            setTimeout(() => warning('Third toast'), 1000);
          }}
        >
          Show Multiple Toasts
        </button>
      </div>
    </div>
  );
};

export default ToastExample;
