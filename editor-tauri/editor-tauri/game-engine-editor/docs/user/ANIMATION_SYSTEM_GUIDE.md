# Animation and Style System Guide

## Overview

This guide explains the unified animation and style system for the Game Engine Editor. The system provides reusable animations, transitions, and utility classes that can be used throughout the application.

## File Structure

```
src/
├── styles/
│   └── animations.css     # Main animation definitions
├── App.tsx                # Application entry point with animations imported
└── tailwind.config.js     # Tailwind configuration with custom animations
```

## Installation

The animation system is already integrated into the project. The CSS file is imported in `App.tsx`:

```tsx
import './styles/animations.css';
```

## Available Animations

### 1. Fade Animations

#### CSS Classes
- `.animate-fade-in` - Basic fade in effect
- `.animate-fade-out` - Basic fade out effect
- `.animate-fade-in-up` - Fade in with slide up
- `.animate-fade-in-down` - Fade in with slide down

#### Tailwind Classes
- `animate-fade-in`
- `animate-fade-out`
- `animate-fade-in-up`
- `animate-fade-in-down`

#### Usage Examples

```tsx
// Using CSS class
<div className="animate-fade-in">Content fades in</div>

// Using Tailwind
<div className="animate-fade-in-up">Content fades in from bottom</div>

// With delay
<div className="animate-fade-in delay-300">Delayed fade in</div>
```

### 2. Slide Animations

#### CSS Classes
- `.animate-slide-in-left` - Slide in from left
- `.animate-slide-in-right` - Slide in from right
- `.animate-slide-in-top` - Slide in from top
- `.animate-slide-in-bottom` - Slide in from bottom

#### Tailwind Classes
- `animate-slide-in-left`
- `animate-slide-in-right`
- `animate-slide-in-top`
- `animate-slide-in-bottom`

#### Usage Examples

```tsx
// Side panel sliding in
<div className="animate-slide-in-left">Left panel</div>

// Bottom panel sliding up
<div className="animate-slide-in-bottom">Bottom panel</div>
```

### 3. Scale Animations

#### CSS Classes
- `.animate-scale-in` - Simple scale in
- `.animate-scale-out` - Scale out (reverse)
- `.animate-scale-in-bounce` - Scale in with bounce effect

#### Tailwind Classes
- `animate-scale-in`
- `animate-scale-out`
- `animate-scale-in-bounce`

#### Usage Examples

```tsx
// Modal appearing
<div className="animate-scale-in modal-enter">
  <Modal>Content</Modal>
</div>

// Bouncy entrance
<div className="animate-scale-in-bounce">Bouncy content</div>
```

### 4. Shimmer and Pulse

#### CSS Classes
- `.animate-shimmer` - Loading shimmer effect
- `.animate-pulse-custom` - Custom pulse animation
- `.skeleton` - Skeleton loading state (light)
- `.skeleton-dark` - Skeleton loading state (dark)

#### Tailwind Classes
- `animate-shimmer`
- `animate-pulse-custom`

#### Usage Examples

```tsx
// Loading state
<div className="animate-shimmer">Loading...</div>

// Skeleton placeholder
<div className="skeleton h-4 w-full"></div>

// Dark skeleton
<div className="skeleton-dark h-4 w-full"></div>

// Pulsing indicator
<div className="animate-pulse-custom">Live</div>
```

### 5. Spin Animations

#### CSS Classes
- `.animate-spin-slow` - Slow rotation (3s)
- `.animate-spin-medium` - Medium rotation (1s)
- `.animate-spin-fast` - Fast rotation (0.5s)

#### Tailwind Classes
- `animate-spin-slow`
- `animate-spin-medium`
- `animate-spin-fast`

#### Usage Examples

```tsx
// Loading spinner
<div className="animate-spin-medium">
  <LoadingIcon />
</div>

// Slow rotating icon
<div className="animate-spin-slow">
  <SettingsIcon />
</div>
```

### 6. Bounce Animations

#### CSS Classes
- `.animate-bounce-custom` - Continuous bounce
- `.animate-bounce-in` - Bounce entrance

#### Tailwind Classes
- `animate-bounce-custom`
- `animate-bounce-in`

#### Usage Examples

```tsx
// Attention-grabbing button
<button className="animate-bounce-custom">Click me!</button>

// Bouncy entrance
<div className="animate-bounce-in">Hello!</div>
```

## Transition Utilities

### Speed Classes

- `.transition-fast` - Fast transitions (150ms)
- `.transition-smooth` - Smooth transitions (300ms) - **Default**
- `.transition-medium` - Medium transitions (250ms)
- `.transition-slow` - Slow transitions (500ms)

### Property-Specific Transitions

- `.transition-colors-smooth` - Color changes only
- `.transition-opacity-smooth` - Opacity changes only
- `.transition-transform-smooth` - Transform changes only

### Usage Examples

```tsx
// Smooth all transitions
<button className="transition-smooth hover:bg-blue-500">
  Hover me
</button>

// Fast hover effect
<div className="transition-fast hover:scale-105">
  Fast hover
</div>

// Color transition only
<div className="transition-colors-smooth hover:text-red-500">
  Color change only
</div>
```

## Hover Effects

### Available Classes

- `.hover-lift` - Lifts element up slightly on hover with shadow
- `.hover-scale` - Scales element up on hover
- `.hover-glow` - Adds glowing shadow on hover

### Usage Examples

```tsx
// Lift effect
<div className="hover-lift bg-white p-4">
  Card content
</div>

// Scale effect
<button className="hover-scale">
  Click me
</button>

// Glow effect
<div className="hover-glow">
  Glowing on hover
</div>
```

## Animation Modifiers

### Delays

- `.delay-100` - 100ms delay
- `.delay-200` - 200ms delay
- `.delay-300` - 300ms delay
- `.delay-500` - 500ms delay
- `.delay-700` - 700ms delay
- `.delay-1000` - 1000ms delay

### Durations

- `.duration-150` - 150ms animation
- `.duration-300` - 300ms animation
- `.duration-500` - 500ms animation
- `.duration-700` - 700ms animation
- `.duration-1000` - 1000ms animation

### Easing Functions

- `.ease-linear` - Linear easing
- `.ease-in` - Ease in
- `.ease-out` - Ease out
- `.ease-in-out` - Ease in and out
- `.ease-bounce` - Bouncy easing

### States

- `.animate-paused` - Pause animation
- `.animate-running` - Resume animation

### Usage Examples

```tsx
// Staggered list items
<ul className="stagger-in">
  <li>Item 1</li>  // 0ms delay
  <li>Item 2</li>  // 50ms delay
  <li>Item 3</li>  // 100ms delay
</ul>

// Custom duration and delay
<div className="animate-fade-in duration-700 delay-500">
  Delayed fade
</div>

// Pause on hover
<div className="animate-spin-slow hover:animate-paused">
  Pauses on hover
</div>
```

## Specialized Animations

### Modal Animations

```tsx
// Enter animation
<div className="modal-enter animate-scale-in">
  <Modal>Content</Modal>
</div>

// Exit animation
<div className="modal-exit animate-scale-out">
  <Modal>Closing</Modal>
</div>

// Backdrop animation
<div className="modal-backdrop-enter animate-fade-in">
  Backdrop
</div>
```

### Panel Animations

```tsx
// Slide in from left
<div className="panel-slide-in-left animate-slide-in-left">
  Side Panel
</div>

// Slide in from right
<div className="panel-slide-in-right animate-slide-in-right">
  Right Panel
</div>
```

### Notification Animations

```tsx
// Notification appearing
<div className="notification-enter animate-slide-in-top">
  Notification message
</div>

// Notification disappearing
<div className="notification-exit">
  Dismissing...
</div>
```

### Tooltip Animations

```tsx
// Tooltip appearing
<div className="tooltip-enter animate-scale-in">
  Tooltip content
</div>

// Tooltip disappearing
<div className="tooltip-exit animate-scale-out">
  Closing...
</div>
```

## Performance Optimizations

### GPU Acceleration

For smooth animations, especially on mobile:

```tsx
<div className="gpu-accelerated animate-fade-in">
  GPU accelerated content
</div>
```

### Will Change Hints

Tell the browser what will animate:

```tsx
// Transform animations
<div className="will-change-transform animate-scale-in">
  Content
</div>

// Opacity animations
<div className="will-change-opacity animate-fade-in">
  Content
</div>
```

## Accessibility

### Reduced Motion Support

The system automatically respects the user's `prefers-reduced-motion` setting. All animations are disabled or significantly slowed when this preference is detected.

### Best Practices

1. **Don't over-animate**: Use animations purposefully
2. **Respect preferences**: The system handles reduced motion automatically
3. **Provide feedback**: Use animations to indicate state changes
4. **Keep it performant**: Use GPU-accelerated properties (transform, opacity)

## Real-World Examples

### Example 1: Loading State

```tsx
function LoadingCard() {
  return (
    <div className="bg-white p-4 rounded-lg">
      <div className="skeleton h-4 w-3/4 mb-2"></div>
      <div className="skeleton h-4 w-1/2 mb-2"></div>
      <div className="skeleton h-32 w-full"></div>
    </div>
  );
}
```

### Example 2: Animated List

```tsx
function AnimatedList({ items }) {
  return (
    <ul className="stagger-in">
      {items.map((item) => (
        <li key={item.id} className="animate-fade-in-up hover-lift bg-white p-2">
          {item.name}
        </li>
      ))}
    </ul>
  );
}
```

### Example 3: Modal with Animation

```tsx
function AnimatedModal({ isOpen, onClose }) {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50">
      <div className="modal-backdrop-enter animate-fade-in absolute inset-0 bg-black/50" onClick={onClose} />
      <div className="modal-enter animate-scale-in relative bg-white rounded-lg p-6 m-auto max-w-md">
        <h2>Modal Title</h2>
        <p>Modal content</p>
        <button onClick={onClose} className="transition-smooth hover-lift">
          Close
        </button>
      </div>
    </div>
  );
}
```

### Example 4: Button with Hover Effects

```tsx
function AnimatedButton({ children, onClick }) {
  return (
    <button
      onClick={onClick}
      className="transition-smooth hover-lift hover-scale bg-blue-500 text-white px-4 py-2 rounded"
    >
      {children}
    </button>
  );
}
```

### Example 5: Status Indicator

```tsx
function StatusIndicator({ isPlaying }) {
  return (
    <span className={isPlaying ? 'text-green-400 animate-pulse-custom' : 'text-slate-400'}>
      {isPlaying ? '● Playing' : '○ Stopped'}
    </span>
  );
}
```

## Customization

### Adding New Animations

To add custom animations, edit `tailwind.config.js`:

```javascript
animation: {
  'my-custom': 'myCustom 1s ease-in-out',
},
keyframes: {
  myCustom: {
    '0%': { transform: 'scale(0)' },
    '100%': { transform: 'scale(1)' },
  },
}
```

### Adding New Transitions

Edit `src/styles/animations.css` to add custom transitions:

```css
.transition-custom {
  transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}
```

## Troubleshooting

### Animations Not Working

1. Check if the CSS file is imported in `App.tsx`
2. Verify Tailwind configuration is correct
3. Check for CSS specificity issues
4. Ensure no conflicting styles

### Performance Issues

1. Use `will-change` sparingly
2. Prefer `transform` and `opacity` for animations
3. Use GPU acceleration classes
4. Limit the number of animated elements
5. Consider using `requestIdleCallback` for non-critical animations

### Browser Compatibility

All animations use standard CSS properties and are supported in:
- Chrome/Edge 88+
- Firefox 85+
- Safari 14+

For older browsers, animations will gracefully degrade.

## Best Practices

1. **Use meaningful animations**: Animations should enhance UX, not distract
2. **Keep it consistent**: Use the same animations for similar interactions
3. **Test performance**: Profile animations on lower-end devices
4. **Accessibility first**: Always respect motion preferences
5. **Document usage**: Comment complex animation sequences

## Resources

- [Tailwind CSS Animation Documentation](https://tailwindcss.com/docs/animation)
- [CSS Animation Best Practices](https://web.dev/animations-guide/)
- [Accessibility for Animations](https://www.w3.org/WAI/WCAG21/Understanding/animation-from-interactions)
