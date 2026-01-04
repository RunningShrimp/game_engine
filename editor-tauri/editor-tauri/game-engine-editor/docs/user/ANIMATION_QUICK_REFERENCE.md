# Animation System - Quick Reference

## Most Common Animations

```tsx
// Fade In
<div className="animate-fade-in">Content</div>

// Slide In
<div className="animate-slide-in-left">Side Panel</div>
<div className="animate-slide-in-bottom">Bottom Panel</div>

// Scale In (Modals)
<div className="animate-scale-in modal-enter">Modal</div>

// Loading
<div className="animate-shimmer h-4 w-full"></div>
<div className="animate-spin-medium">Spinner</div>

// Status Indicator
<span className="animate-pulse-custom text-green-400">● Live</span>
```

## Transitions

```tsx
// Speeds
transition-fast     // 150ms
transition-smooth   // 300ms (default)
transition-slow     // 500ms

// Hover Effects
hover-lift    // Lift + shadow
hover-scale   // Scale up
hover-glow    // Glow effect
```

## Modifiers

```tsx
// Delays
delay-100, delay-200, delay-300, delay-500

// Durations
duration-150, duration-300, duration-500, duration-700

// Easing
ease-in, ease-out, ease-in-out, ease-bounce
```

## Specialized

```tsx
// Modals
modal-enter    animate-scale-in
modal-exit     animate-scale-out

// Panels
panel-slide-in-left/right/top/bottom

// Notifications
notification-enter   animate-slide-in-top
notification-exit    slide-out-top

// Tooltips
tooltip-enter   animate-scale-in
tooltip-exit    animate-scale-out
```

## Examples

### Button with Hover
```tsx
<button className="transition-smooth hover-lift bg-blue-500 px-4 py-2">
  Click Me
</button>
```

### Loading Card
```tsx
<div className="bg-white p-4 rounded">
  <div className="skeleton h-4 w-3/4 mb-2"></div>
  <div className="skeleton h-32 w-full"></div>
</div>
```

### Animated List
```tsx
<ul className="stagger-in">
  {items.map(item => (
    <li key={item.id} className="animate-fade-in-up hover-lift">
      {item.name}
    </li>
  ))}
</ul>
```

### Status Badge
```tsx
<span className={`${
  isActive
    ? 'animate-pulse-custom text-green-400'
    : 'text-slate-400'
}`}>
  {isActive ? '● Active' : '○ Inactive'}
</span>
```

## Performance Tips

```tsx
// GPU Acceleration
<div className="gpu-accelerated animate-fade-in">Content</div>

// Hint to browser
<div className="will-change-transform animate-scale-in">Content</div>
<div className="will-change-opacity animate-fade-in">Content</div>
```

## Accessibility

Animations automatically respect `prefers-reduced-motion` setting.

## Quick Testing

```tsx
import { AnimationDemo } from './components/AnimationDemo';

<AnimationDemo />
```

## File Locations

- **CSS**: `/src/styles/animations.css`
- **Demo**: `/src/components/AnimationDemo.tsx`
- **Guide**: `/ANIMATION_SYSTEM_GUIDE.md`
- **Tailwind**: `tailwind.config.js`

## Need Help?

See complete documentation: `ANIMATION_SYSTEM_GUIDE.md`
