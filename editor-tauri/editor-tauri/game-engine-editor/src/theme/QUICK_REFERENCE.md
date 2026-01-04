# Design Tokens - Quick Reference

Quick lookup guide for commonly used design tokens.

## 🎨 Colors

### Semantic Text Colors
```tsx
colors.semantic.text.primary    // #171717 - Main text
colors.semantic.text.secondary  // #525252 - Secondary text
colors.semantic.text.disabled   // #a3a3a3 - Disabled text
colors.semantic.text.link       // #0284c7 - Links
```

### Semantic Background Colors
```tsx
colors.semantic.background.DEFAULT  // #fafafa - Default bg
colors.semantic.background.paper    // #ffffff - Card/panel bg
colors.semantic.background.overlay  // rgba(0,0,0,0.5) - Modal overlay
```

### Primary Color Scale
```tsx
colors.primary[50]   // #f0f9ff - Lightest
colors.primary[100]
colors.primary[200]
colors.primary[300]
colors.primary[400]
colors.primary[500]  // #0ea5e9 - Main brand color
colors.primary[600]  // #0284c7 - Hover state
colors.primary[700]
colors.primary[800]
colors.primary[900]  // #0c4a6e - Darkest
```

### Semantic Status Colors
```tsx
colors.success[500]  // #22c55e - Success
colors.error[500]    // #ef4444 - Error
colors.warning[500]  // #f59e0b - Warning
colors.info[500]     // #3b82f6 - Info
```

## ✏️ Typography

### Font Families
```tsx
typography.fontFamily.sans  // Inter, system-ui, sans-serif
typography.fontFamily.mono  // JetBrains Mono, monospace
typography.fontFamily.serif // Georgia, serif
```

### Font Sizes
```tsx
typography.fontSize.xs      // 0.75rem (12px)
typography.fontSize.sm      // 0.875rem (14px)
typography.fontSize.base    // 1rem (16px) - Default
typography.fontSize.lg      // 1.125rem (18px)
typography.fontSize.xl      // 1.25rem (20px)
typography.fontSize['2xl']  // 1.5rem (24px)
typography.fontSize['3xl']  // 1.875rem (30px)
```

### Font Weights
```tsx
typography.fontWeight.normal    // 400
typography.fontWeight.medium    // 500
typography.fontWeight.semibold  // 600
typography.fontWeight.bold      // 700
```

### Typography Presets
```tsx
typography.presets.h1       // Display heading 1
typography.presets.h2       // Display heading 2
typography.presets.h3       // Display heading 3
typography.presets.body     // Body text
typography.presets.button   // Button text
typography.presets.code     // Code/monospace
```

## 📏 Spacing

### Space Scale (4px base unit)
```tsx
spacing.space[0]   // 0
spacing.space[1]   // 0.25rem (4px)
spacing.space[2]   // 0.5rem (8px)
spacing.space[3]   // 0.75rem (12px)
spacing.space[4]   // 1rem (16px) - Most common
spacing.space[6]   // 1.5rem (24px)
spacing.space[8]   // 2rem (32px)
spacing.space[12]  // 3rem (48px)
spacing.space[16]  // 4rem (64px)
```

### Spacing Presets
```tsx
spacing.spacing['input-padding']      // 0.5rem 0.75rem
spacing.spacing['button-padding']     // 0.5rem 1rem
spacing.spacing['card-padding']       // 1.5rem
spacing.spacing['section-gap']       // 4rem
spacing.spacing['content-gap']       // 2rem
spacing.spacing['item-gap']          // 1rem
```

## 🔘 Border Radius

### Radius Values
```tsx
borderRadius.borderRadius.none  // 0
borderRadius.borderRadius.sm    // 0.25rem (4px)
borderRadius.borderRadius.md    // 0.375rem (6px)
borderRadius.borderRadius.lg    // 0.5rem (8px)
borderRadius.borderRadius.xl    // 0.75rem (12px)
borderRadius.borderRadius.full  // 9999px (pill/circle)
```

### Component Presets
```tsx
borderRadius.componentRadius.button  // 0.375rem
borderRadius.componentRadius.input   // 0.375rem
borderRadius.componentRadius.card    // 0.5rem
borderRadius.componentRadius.badge   // 9999px (pill)
borderRadius.componentRadius.avatar  // 9999px (circle)
```

## 🌑 Shadows

### Shadow Levels
```tsx
shadows.shadows.none  // none
shadows.shadows.sm    // Subtle
shadows.shadows.md    // Default
shadows.shadows.lg    // Large
shadows.shadows.xl    // Extra large
shadows.shadows.inner // Inset shadow
```

### Component Presets
```tsx
shadows.componentShadows.button       // Hover shadow
shadows.componentShadows.card         // Card elevation
shadows.componentShadows.modal        // Modal elevation
shadows.componentShadows.dropdown     // Dropdown shadow
shadows.componentShadows.tooltip      // Tooltip shadow
```

## 📊 Z-Index

### Layer Hierarchy
```tsx
zIndex.zIndex.base           // 0 - Default
zIndex.zIndex.raised         // 10 - Slightly elevated
zIndex.zIndex.dropdown       // 1000 - Dropdowns
zIndex.zIndex.popover        // 1030 - Popovers
zIndex.zIndex.tooltip        // 1040 - Tooltips
zIndex.zIndex.modal          // 1060 - Modals
zIndex.zIndex.toast          // 1080 - Toasts
```

## ⚡ Transitions

### Durations
```tsx
transitions.duration.instant  // 100ms - Instant
transitions.duration.fast     // 150ms - Fast
transitions.duration.base     // 200ms - Default
transitions.duration.normal   // 300ms - Normal
transitions.duration.slow     // 500ms - Slow
```

### Easing Functions
```tsx
transitions.easing.smooth     // cubic-bezier(0.4, 0, 0.2, 1)
transitions.easing.bounce     // cubic-bezier(0.68, -0.55, 0.265, 1.55)
transitions.easing.ease       // ease
```

### Component Transitions
```tsx
transitions.componentTransitions.button  // Button hover/active
transitions.componentTransitions.input   // Input focus
transitions.componentTransitions.card    // Card hover
transitions.componentTransitions.modal   // Modal enter/leave
```

## 📱 Breakpoints

### Responsive Breakpoints
```tsx
breakpoints.breakpoints.xs   // 375px - Extra small (phones)
breakpoints.breakpoints.sm   // 640px - Small (landscape phones)
breakpoints.breakpoints.md   // 768px - Medium (tablets)
breakpoints.breakpoints.lg   // 1024px - Large (desktops)
breakpoints.breakpoints.xl   // 1280px - Extra large
breakpoints.breakpoints['2xl'] // 1536px - 2XL
```

### Container Max-Widths
```tsx
spacing.container.sm   // 640px
spacing.container.md   // 768px
spacing.container.lg   // 1024px
spacing.container.xl   // 1280px
spacing.container.full // 100%
```

## 🎯 Common Component Patterns

### Button
```tsx
const buttonStyle = {
  ...typography.presets.button,
  padding: spacing.spacing['button-padding'],
  borderRadius: borderRadius.componentRadius.button,
  backgroundColor: colors.primary[500],
  color: '#ffffff',
  border: 'none',
  cursor: 'pointer',
  transition: transitions.componentTransitions.button,
};
```

### Input
```tsx
const inputStyle = {
  ...typography.presets.body,
  padding: spacing.spacing['input-padding'],
  borderRadius: borderRadius.componentRadius.input,
  border: `1px solid ${colors.semantic.border.DEFAULT}`,
  fontFamily: typography.fontFamily.sans,
  transition: transitions.componentTransitions.input,
};
```

### Card
```tsx
const cardStyle = {
  padding: spacing.spacing['card-padding'],
  borderRadius: borderRadius.componentRadius.card,
  backgroundColor: colors.semantic.background.paper,
  boxShadow: shadows.componentShadows.card,
  border: `1px solid ${colors.semantic.border.DEFAULT}`,
};
```

### Alert/Notification
```tsx
const alertStyle = {
  padding: spacing.space[4],
  borderRadius: borderRadius.borderRadius.md,
  backgroundColor: colors.error[50],
  border: `1px solid ${colors.semantic.border.error}`,
  color: colors.semantic.text.primary,
};
```

### Badge/Tag
```tsx
const badgeStyle = {
  ...typography.presets.caption,
  padding: `${spacing.space[1]} ${spacing.space[3]}`,
  borderRadius: borderRadius.componentRadius.badge,
  backgroundColor: colors.primary[100],
  color: colors.primary[700],
  fontWeight: typography.fontWeight.medium,
};
```

## 🌙 Dark Mode

CSS Variables for dark mode:

```css
/* Toggle dark mode */
document.documentElement.classList.toggle('dark');

/* Access dark mode colors */
color: var(--color-text-primary);
background: var(--color-background-paper);
```

## 📦 Tailwind Integration

Design tokens are automatically available as Tailwind classes:

```tsx
// Colors
<div className="bg-primary-500 text-neutral-900">
<div className="border-error-500">

// Spacing
<div className="p-4 m-2 gap-4">

// Typography
<div className="text-base font-medium">
<div className="text-lg font-bold">

// Border Radius
<div className="rounded-lg rounded-full">

// Shadows
<div className="shadow-md shadow-lg">
```

## 🔍 Type Safety

All design tokens are fully typed:

```tsx
import type { ColorTokens, TypographyPresets, Space } from '@/theme';

const myColor: ColorTokens = colors;
const myStyle: TypographyPresets = typography.presets.h1;
```

---

**Pro Tip:** Use the design tokens consistently throughout your application. Avoid hardcoded values to maintain design system consistency.
