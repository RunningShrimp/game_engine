# UI Component Library

A comprehensive collection of reusable UI components for the Game Engine Editor application.

## Installation

The components are located in `src/components/ui/`. All required dependencies have been added to `package.json`:

```bash
npm install
```

## Available Components

### Button

A versatile button component with multiple variants, sizes, and loading states.

**Features:**
- 5 variants: `primary`, `secondary`, `outline`, `ghost`, `danger`
- 5 sizes: `xs`, `sm`, `md`, `lg`, `xl`
- Loading state with integrated spinner
- Full width option
- Ref forwarding
- Full TypeScript support

**Usage:**
```tsx
import { Button } from '@/components/ui';

// Basic button
<Button onClick={handleClick}>Click me</Button>

// With variant and size
<Button variant="outline" size="lg">Large Outline Button</Button>

// Loading state
<Button isLoading>Loading...</Button>

// Full width
<Button variant="danger" fullWidth>Delete</Button>

// With ref
const buttonRef = useRef<HTMLButtonElement>(null);
<Button ref={buttonRef}>With Ref</Button>
```

**Props:**
- `variant?: ButtonVariant` - Visual style (default: 'primary')
- `size?: ButtonSize` - Button size (default: 'md')
- `isLoading?: boolean` - Show loading spinner (default: false)
- `disabled?: boolean` - Disable button (default: false)
- `fullWidth?: boolean` - Take full width (default: false)
- `className?: string` - Additional CSS classes
- All standard HTML button attributes

---

### Spinner

A loading spinner component with configurable size and color.

**Features:**
- 5 sizes: `xs`, `sm`, `md`, `lg`, `xl`
- 4 colors: `primary`, `secondary`, `white`, `currentColor`
- 3 speed options: `slow`, `normal`, `fast`
- Accessible with ARIA labels

**Usage:**
```tsx
import { Spinner } from '@/components/ui';

// Basic spinner
<Spinner />

// With size and color
<Spinner size="lg" color="white" />

// Custom speed
<Spinner size="md" speed="fast" />

// In a loading overlay
<div className="flex justify-center p-8">
  <Spinner size="xl" />
</div>
```

**Props:**
- `size?: SpinnerSize` - Spinner size (default: 'md')
- `color?: SpinnerColor` - Spinner color (default: 'primary')
- `speed?: 'slow' | 'normal' | 'fast'` - Animation speed (default: 'normal')
- `className?: string` - Additional CSS classes

---

### Skeleton

Placeholder components for content that is loading.

**Features:**
- 3 variants: `text`, `rectangular`, `circular`
- Multiple text lines support
- Animated shimmer effect
- Pre-configured Card and Table skeletons
- Accessible (hidden from screen readers)

**Usage:**
```tsx
import { Skeleton, CardSkeleton, TableSkeleton } from '@/components/ui';

// Text skeleton
<Skeleton variant="text" width="100%" height={20} />

// Circular avatar skeleton
<Skeleton variant="circular" width={40} height={40} />

// Rectangular card skeleton
<Skeleton variant="rectangular" width="100%" height={200} />

// Multiple text lines
<Skeleton variant="text" lines={3} />

// Pre-configured card skeleton
<CardSkeleton />

// Pre-configured table skeleton
<TableSkeleton rows={5} columns={4} />

// Disable animation
<Skeleton variant="text" animate={false} />
```

**Props:**
- `variant?: SkeletonVariant` - Shape variant (default: 'rectangular')
- `width?: string` - Width (e.g., '100px', '50%')
- `height?: string` - Height (e.g., '20px', '100%')
- `lines?: number` - Number of lines for text variant (default: 1)
- `animate?: boolean` - Enable shimmer animation (default: true)
- `className?: string` - Additional CSS classes

---

### EmptyState

A component to display empty states with optional actions.

**Features:**
- 3 sizes: `sm`, `md`, `lg`
- Icon support
- Action button support
- Pre-configured variants (NoData, NoSearchResults, Error)
- Responsive design

**Usage:**
```tsx
import { EmptyState, NoDataEmptyState, NoSearchResultsEmptyState } from '@/components/ui';
import { FolderOpenIcon } from 'lucide-react';

// Custom empty state
<EmptyState
  icon={<FolderOpenIcon />}
  title="No documents found"
  description="Create your first document to get started."
  action={<Button onClick={handleCreate}>Create Document</Button>}
/>

// Small size
<EmptyState
  size="sm"
  title="No results"
  description="Try adjusting your search filters."
/>

// Pre-configured variants
<NoDataEmptyState onAction={handleAddData} actionLabel="Add Data" />
<NoSearchResultsEmptyState />
<ErrorEmptyState onRetry={handleRetry} />
```

**Props:**
- `icon?: React.ReactNode` - Icon or illustration
- `title: string` - Main heading text
- `description?: string` - Descriptive text
- `action?: React.ReactNode` - Action button
- `size?: 'sm' | 'md' | 'lg'` - Size variant (default: 'md')
- `className?: string` - Additional CSS classes

---

## Utility Functions

### cn()

A utility function to merge Tailwind CSS classes with proper precedence.

**Usage:**
```tsx
import { cn } from '@/utils/cn';

// Basic usage
cn('px-4 py-2', 'px-6') // Returns: 'py-2 px-6'

// Conditional classes
cn('text-red-500', isActive && 'text-blue-500')

// Object syntax
cn({ 'bg-blue-500': isActive }, 'px-4')

// With components
<Button className={cn('mt-4', customClassName)} />
```

---

## Best Practices

### 1. Importing Components

Always import from the index file for cleaner imports:

```tsx
// Good
import { Button, Spinner, Skeleton } from '@/components/ui';

// Avoid
import { Button } from '@/components/ui/Button';
```

### 2. TypeScript Support

All components have full TypeScript support. Use the exported types:

```tsx
import type { ButtonProps, ButtonVariant } from '@/components/ui';

const createButton = (props: ButtonProps) => {
  return <Button {...props} />;
};
```

### 3. Accessibility

All components follow accessibility best practices:
- Proper ARIA labels
- Keyboard navigation support
- Screen reader friendly
- Focus management

### 4. Styling

Use the `className` prop to extend component styles:

```tsx
<Button className="mt-4 shadow-lg">Custom Styled Button</Button>
```

### 5. Ref Forwarding

All components support ref forwarding:

```tsx
const buttonRef = useRef<HTMLButtonElement>(null);
<Button ref={buttonRef}>With Ref</Button>
```

---

## Tailwind Configuration

Custom animations and utilities are already configured in `tailwind.config.js`:

- `spin-slow`: 3s spin animation
- `spin-medium`: 1s spin animation
- `spin-fast`: 0.5s spin animation
- `animate-pulse`: Pulse animation for skeletons
- And many more...

---

## Browser Support

All components support modern browsers:
- Chrome/Edge (latest)
- Firefox (latest)
- Safari (latest)
- WebKit-based browsers

---

## Performance

Components are optimized for performance:
- Minimal re-renders
- Efficient prop handling
- Proper React.memo usage where appropriate
- Lazy loading support

---

## Future Enhancements

Planned components for future releases:
- Input/TextField
- Select/Dropdown
- Modal/Dialog
- Tabs
- Tooltip
- Toast notifications
- Progress indicators
- And more...

---

## Contributing

When adding new components:

1. Follow the established patterns
2. Include TypeScript types
3. Add JSDoc documentation
4. Export from `index.ts`
5. Update this README
6. Include usage examples
7. Ensure accessibility
8. Add responsive design support

---

## License

Part of the Game Engine Editor project.
