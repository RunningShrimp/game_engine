# Toast Integration Guide

## Step 1: Import and Setup in Your Main App

Add the ToastContainer to your main App component:

```tsx
// src/App.tsx or your main entry point
import React from 'react';
import { ToastContainer, useToast } from './components/Toast';

function App() {
  const { toasts, closeToast } = useToast();

  return (
    <>
      {/* Your existing app content */}
      <MainApplication />

      {/* Toast Container - Add this at the end */}
      <ToastContainer toasts={toasts} onClose={closeToast} />
    </>
  );
}

export default App;
```

## Step 2: Use Toasts in Any Component

```tsx
// src/features/SaveButton.tsx
import React from 'react';
import { useToast } from '../../components/Toast';

export const SaveButton: React.FC = () => {
  const { success, error } = useToast();

  const handleSave = async () => {
    try {
      // Your save logic
      await saveDocument();
      success('Document saved successfully!');
    } catch (err) {
      error('Failed to save document. Please try again.');
    }
  };

  return <button onClick={handleSave}>Save</button>;
};
```

## Step 3: Alternative - Context Provider Approach

If you prefer a more centralized approach, create a Toast Context:

```tsx
// src/contexts/ToastContext.tsx
import React, { createContext, useContext } from 'react';
import { useToast, UseToastReturn } from '../components/Toast';

const ToastContext = createContext<UseToastReturn | null>(null);

export const ToastProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const toastHook = useToast();

  return (
    <ToastContext.Provider value={toastHook}>
      {children}
    </ToastContext.Provider>
  );
};

export const useToastContext = () => {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error('useToastContext must be used within ToastProvider');
  }
  return context;
};
```

Then wrap your app:

```tsx
// src/App.tsx
import { ToastProvider } from './contexts/ToastContext';
import { ToastContainer } from './components/Toast';

function App() {
  const { toasts, closeToast } = useToast();

  return (
    <ToastProvider>
      <MainApplication />
      <ToastContainer toasts={toasts} onClose={closeToast} />
    </ToastProvider>
  );
}
```

And use anywhere:

```tsx
import { useToastContext } from '../../contexts/ToastContext';

function MyComponent() {
  const { success, error } = useToastContext();
  // ... use as before
}
```

## Common Use Cases

### Form Submission

```tsx
const handleSubmit = async (e: FormEvent) => {
  e.preventDefault();

  if (!validate()) {
    warning('Please fill in all required fields');
    return;
  }

  try {
    await submitForm(formData);
    success('Form submitted successfully!');
    resetForm();
  } catch (err) {
    error('Submission failed. Please try again.');
  }
};
```

### API Calls

```tsx
const fetchData = async () => {
  info('Loading data...');

  try {
    const data = await api.getData();
    success(`Loaded ${data.length} items`);
  } catch (err) {
    error('Failed to load data');
  }
};
```

### File Operations

```tsx
const handleFileUpload = async (file: File) => {
  info('Uploading file...');

  try {
    await uploadFile(file);
    success('File uploaded successfully!');
  } catch (err) {
    error('Upload failed. Please try again.');
  }
};
```

### Copy to Clipboard

```tsx
const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text);
    success('Copied to clipboard!');
  } catch (err) {
    error('Failed to copy');
  }
};
```

## Tips and Best Practices

1. **Meaningful Messages**: Be clear and specific
   - ✅ "Settings saved successfully"
   - ❌ "Done"

2. **Appropriate Duration**: Adjust based on message length
   - Short messages: 2000-3000ms
   - Long messages: 4000-6000ms
   - Important alerts: 8000-10000ms

3. **Use Correct Type**: Match type to severity
   - Success: Completed actions, successful saves
   - Error: Failures, validation errors
   - Warning: Potential issues, cautions
   - Info: Neutral information, status updates

4. **Don't Overuse**: Too many toasts can be annoying
   - Limit to 1-2 toasts per action
   - Consider using inline errors for forms

5. **Accessibility**: The system is already accessible
   - No additional work needed for screen readers
   - Auto-dismiss reduces manual interaction needed
