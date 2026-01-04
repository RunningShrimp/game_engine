# Toast Notification System

A complete, production-ready toast notification system for React applications with TypeScript support.

## Features

- ✨ **4 Toast Types**: Success, Error, Info, Warning
- ⏱️ **Auto-dismiss**: Configurable duration (default: 3000ms)
- 🎨 **Beautiful Animations**: Smooth slide-in/slide-out animations
- 📱 **Responsive Design**: Mobile-friendly with adaptive layout
- 🌓 **Dark Mode Support**: Automatically respects system preferences
- 🎯 **Portal Rendering**: Renders to body for proper z-index stacking
- 📦 **Stackable**: Multiple toasts can be displayed simultaneously
- ♿ **Accessible**: ARIA attributes for screen readers
- 🎭 **TypeScript**: Full type safety
- 🎨 **CSS Modules**: Scoped styles to prevent conflicts

## Installation

The system is already installed in your project at:
`src/components/Toast/`

## Quick Start

### 1. Add ToastContainer to your App

Wrap your root component with `ToastContainer`:

```tsx
import React from 'react';
import { ToastContainer, useToast } from './components/Toast';

function App() {
  const { toasts, closeToast } = useToast();

  return (
    <>
      <YourMainComponent />
      <ToastContainer toasts={toasts} onClose={closeToast} />
    </>
  );
}

export default App;
```

### 2. Use the useToast Hook in Components

```tsx
import React from 'react';
import { useToast } from './components/Toast';

function MyComponent() {
  const { success, error, info, warning } = useToast();

  const handleSave = async () => {
    try {
      await saveData();
      success('Data saved successfully!');
    } catch (err) {
      error('Failed to save data. Please try again.');
    }
  };

  return <button onClick={handleSave}>Save</button>;
}
```

## API Reference

### useToast Hook

```typescript
const {
  toasts,        // Array of active toasts
  toast,         // Generic toast method
  success,       // Success toast shortcut
  error,         // Error toast shortcut
  info,          // Info toast shortcut
  warning,       // Warning toast shortcut
  closeToast,    // Manually close a toast
} = useToast();
```

#### Methods

##### `toast(message, type?, duration?)`

Shows a toast notification.

```typescript
toast('Hello World', 'info', 3000);
```

**Parameters:**
- `message` (string): The message to display
- `type` (ToastType, optional): 'success' | 'error' | 'info' | 'warning' (default: 'info')
- `duration` (number, optional): Duration in milliseconds (default: 3000)

##### `success(message, duration?)`

Shows a success toast.

```typescript
success('Operation completed!');
```

##### `error(message, duration?)`

Shows an error toast.

```typescript
error('Something went wrong!');
```

##### `info(message, duration?)`

Shows an info toast.

```typescript
info('New message received');
```

##### `warning(message, duration?)`

Shows a warning toast.

```typescript
warning('Please review your input');
```

##### `closeToast(id)`

Manually closes a toast by ID.

```typescript
closeToast('toast-id-123');
```

## Component Props

### ToastContainer

```typescript
interface ToastContainerProps {
  toasts: ToastItem[];      // Array of toast items
  onClose: (id: string) => void; // Callback when toast closes
}
```

### Toast

```typescript
interface ToastProps {
  id: string;
  type: ToastType;
  message: string;
  duration?: number;
  onClose: (id: string) => void;
}
```

## Usage Examples

### Basic Usage

```tsx
import { useToast } from './components/Toast';

function Component() {
  const { success } = useToast();

  return (
    <button onClick={() => success('Done!')}>
      Click Me
    </button>
  );
}
```

### Custom Duration

```tsx
const { toast } = useToast();

// Show for 5 seconds
toast('Long operation completed', 'success', 5000);

// Show for 10 seconds
toast('Very important message', 'warning', 10000);
```

### Multiple Toasts

```tsx
const { success, error, info } = useToast();

const showAll = () => {
  success('Step 1 completed');
  setTimeout(() => info('Step 2 in progress'), 500);
  setTimeout(() => error('Step 3 failed'), 1000);
};
```

### Conditional Toasts

```tsx
const { success, error } = useToast();

const handleSubmit = async (data: FormData) => {
  try {
    const result = await submitData(data);
    if (result.success) {
      success('Form submitted successfully!');
    } else {
      error(result.message);
    }
  } catch (err) {
    error('Network error. Please try again.');
  }
};
```

### With Form Validation

```tsx
const { warning } = useToast();

const validateForm = (values: FormValues) => {
  if (!values.email) {
    warning('Email is required');
    return false;
  }
  if (!values.email.includes('@')) {
    error('Invalid email format');
    return false;
  }
  return true;
};
```

## Styling

The system uses CSS Modules for scoped styling. Styles are defined in `Toast.module.css`.

### Customization

To customize the appearance, edit `Toast.module.css`:

```css
/* Change position */
.container {
  top: 20px;
  right: 20px;
  /* Or try:
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  */
}

/* Change colors */
.toast.success {
  background: #your-color;
  border-left-color: #your-border-color;
}

/* Change animation */
@keyframes slideIn {
  from {
    transform: translateX(100%);
  }
  to {
    transform: translateX(0);
  }
}
```

## Accessibility

The Toast system includes accessibility features:

- **ARIA Live Region**: Uses `role="alert"` and `aria-live="polite"`
- **Close Button**: Includes `aria-label` for screen readers
- **Keyboard Accessible**: Close button can be focused and activated
- **Auto-dismiss**: Reduces need for manual interaction

## Browser Support

- Chrome/Edge (latest)
- Firefox (latest)
- Safari (latest)
- Mobile browsers (iOS Safari, Chrome Mobile)

## Performance

- **Portal Rendering**: Uses React Portal for optimal DOM placement
- **CSS Animations**: Hardware-accelerated transforms
- **Auto-cleanup**: Removes toasts from DOM after animation
- **Minimal Re-renders**: Optimized with useCallback hooks

## TypeScript Support

Full TypeScript support with exported types:

```typescript
import type {
  ToastType,
  ToastProps,
  ToastItem,
  ToastContainerProps,
  UseToastReturn
} from './components/Toast';
```

## Testing

Example test with React Testing Library:

```tsx
import { render, screen, waitFor } from '@testing-library/react';
import { useToast } from './components/Toast';

function TestComponent() {
  const { toasts, success } = useToast();
  return (
    <>
      <button onClick={() => success('Test')}>Show Toast</button>
      <ToastContainer toasts={toasts} onClose={() => {}} />
    </>
  );
}

test('shows success toast', async () => {
  render(<TestComponent />);
  fireEvent.click(screen.getByText('Show Toast'));

  await waitFor(() => {
    expect(screen.getByText('Test')).toBeInTheDocument();
  });
});
```

## File Structure

```
Toast/
├── Toast.tsx              # Individual Toast component
├── ToastContainer.tsx     # Container for all toasts
├── useToast.ts           # React hook
├── Toast.module.css      # Component styles
├── index.ts              # Public API exports
├── ToastExample.tsx      # Usage examples
└── README.md             # This file
```

## License

MIT

## Contributing

Feel free to customize and extend this system for your needs!
