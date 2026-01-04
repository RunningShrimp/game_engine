# Design Tokens System

A comprehensive design token system that provides consistent styling values across the entire application. Design tokens are the visual design atoms of the design system — specifically, they are named entities that store visual design attributes.

## 📁 Structure

```
src/theme/
├── colors.ts          # Color system (palettes, semantic colors)
├── typography.ts      # Typography system (fonts, sizes, weights)
├── spacing.ts         # Spacing system (margins, padding, gaps)
├── borderRadius.ts    # Border radius system
├── shadows.ts         # Shadow system (elevation)
├── zIndex.ts          # Z-index layering system
├── transitions.ts     # Transition and animation system
├── breakpoints.ts     # Responsive breakpoints
├── index.ts           # Unified export
└── README.md          # This file
```

## 🎨 Usage

### Importing Design Tokens

```tsx
// Import specific token modules
import { colors, typography, spacing } from '@/theme';

// Or import everything
import * as theme from '@/theme';
```

### Using with Tailwind CSS

The design tokens are automatically integrated with Tailwind CSS. You can use them in your components:

```tsx
<div className="
  bg-primary-500
  text-neutral-900
  p-4
  rounded-lg
  shadow-md
">
  Content
</div>
```

### Using in TypeScript/JavaScript

```tsx
import { colors, typography, spacing } from '@/theme';

const MyComponent = () => (
  <div style={{
    backgroundColor: colors.primary[500],
    color: colors.semantic.text.primary,
    fontSize: typography.fontSize.base,
    padding: spacing.space[4],
    borderRadius: borderRadius.borderRadius.lg,
  }}>
    Content
  </div>
);
```

### Using CSS Variables

For runtime theme switching, use CSS variables defined in `src/styles/variables.css`:

```tsx
<div style={{
  color: 'var(--color-primary-500)',
  padding: 'var(--spacing-4)',
}}>
  Content
</div>
```

## 📚 Token Reference

### Colors (`colors.ts`)

Complete color system with semantic tokens:

- **Neutral**: Gray scale from 50-950
- **Primary**: Blue-based brand colors (50-950)
- **Secondary**: Purple-based accent colors (50-950)
- **Success**: Green colors for positive feedback (50-950)
- **Error**: Red colors for errors/destructive (50-950)
- **Warning**: Yellow/amber for cautions (50-950)
- **Info**: Light blue for information (50-950)
- **Semantic**: Background, text, and border colors
- **Special**: Code syntax and chart colors

```tsx
import { colors } from '@/theme';

// Usage
const bgColor = colors.primary[500];
const textColor = colors.semantic.text.primary;
const errorColor = colors.error[600];
```

### Typography (`typography.ts`)

Font system with families, sizes, weights, and presets:

- **Font Family**: sans, mono, serif
- **Font Size**: From 10px (overline) to 60px (display-xl)
- **Font Weight**: 100-900
- **Line Height**: Relative values from 1-2
- **Letter Spacing**: From -0.05em to 0.1em
- **Presets**: Predefined combinations for common elements

```tsx
import { typography } from '@/theme';

// Usage
const fontFamily = typography.fontFamily.sans;
const fontSize = typography.fontSize.base;
const fontWeight = typography.fontWeight.medium;
const headingStyle = typography.presets.h1;
```

### Spacing (`spacing.ts`)

Consistent spacing scale based on 4px base unit:

- **Space**: 0-96 (0 to 384px)
- **Spacing Presets**: Common padding/margin combinations
- **Container**: Max-widths for different layouts

```tsx
import { spacing } from '@/theme';

// Usage
const padding = spacing.space[4]; // 16px
const margin = spacing.spacing['item-gap']; // 16px
const maxWidth = spacing.container.lg; // 512px
```

### Border Radius (`borderRadius.ts`)

Rounded corner values:

- **Radius**: none, xs, sm, md, lg, xl, 2xl, 3xl, full
- **Component Presets**: Predefined radius for specific components
- **Border Width**: 0-8px

```tsx
import { borderRadius } from '@/theme';

// Usage
const radius = borderRadius.borderRadius.lg; // 8px
const buttonRadius = borderRadius.componentRadius.button;
```

### Shadows (`shadows.ts`)

Elevation shadows for depth:

- **Shadows**: none, sm, md, lg, xl, 2xl, inner
- **Component Presets**: Shadows for buttons, cards, modals, etc.
- **Ring**: Focus ring shadows

```tsx
import { shadows } from '@/theme';

// Usage
const shadow = shadows.shadows.md;
const cardShadow = shadows.componentShadows.card;
```

### Z-Index (`zIndex.ts`)

Layering hierarchy for stacked elements:

- **Base Layers**: base (0), raised (10)
- **Overlays**: dropdown (1000), popover (1030), tooltip (1040)
- **Modals**: modal-backdrop (1050), modal (1060), drawer (1070)
- **Feedback**: toast (1080), notification (1090)

```tsx
import { zIndex } from '@/theme';

// Usage
const modalZIndex = zIndex.zIndex.modal; // '1060'
const toastZIndex = zIndex.componentZIndex['toast-item'];
```

### Transitions (`transitions.ts`)

Animation and transition tokens:

- **Duration**: 100ms-1000ms
- **Easing**: Linear, standard, and custom bezier curves
- **Delay**: none, short, normal, long
- **Transitions**: Preset combinations for common use cases
- **Component Presets**: Button, input, modal transitions

```tsx
import { transitions } from '@/theme';

// Usage
const duration = transitions.duration.normal; // '300ms'
const easing = transitions.easing.smooth;
const transition = transitions.transitions.base;
```

### Breakpoints (`breakpoints.ts`)

Responsive design breakpoints:

- **Breakpoints**: xs (375px), sm (640px), md (768px), lg (1024px), xl (1280px), 2xl (1536px), 3xl (1920px)
- **Screens**: Mobile, tablet, desktop ranges
- **Containers**: Max-widths for responsive layouts
- **Media Queries**: Predefined query strings

```tsx
import { breakpoints } from '@/theme';

// Usage
const tabletBreakpoint = breakpoints.breakpoints.md; // '768px'
const mobileQuery = breakpoints.mediaQuery['down-sm'];
```

## 🎯 Best Practices

### 1. Use Design Tokens Consistently

**❌ Bad:**
```tsx
<div style={{ padding: '16px', color: '#0ea5e9' }}>
```

**✅ Good:**
```tsx
<div style={{ padding: spacing.space[4], color: colors.primary[500] }}>
// or
<div className="p-4 text-primary-500">
```

### 2. Prefer Semantic Colors

**❌ Bad:**
```tsx
const errorColor = colors.error[500]; // Too specific
```

**✅ Good:**
```tsx
const statusColor = hasError ? colors.semantic.border.error : colors.semantic.border.focus;
```

### 3. Use Presets When Available

**❌ Bad:**
```tsx
const style = {
  fontFamily: typography.fontFamily.sans,
  fontSize: typography.fontSize.h3,
  fontWeight: typography.fontWeight.semibold,
  lineHeight: typography.lineHeight.snug,
};
```

**✅ Good:**
```tsx
const style = typography.presets.h3;
```

### 4. Consider Accessibility

All color combinations follow WCAG AA guidelines (4.5:1 contrast ratio). When using custom color combinations, verify accessibility:

```tsx
// Good - semantic colors ensure proper contrast
<div style={{ color: colors.semantic.text.primary, backgroundColor: colors.semantic.background.DEFAULT }}>
```

## 🌙 Dark Mode Support

The design system supports dark mode through CSS variables. See `src/styles/variables.css` for implementation:

```tsx
// Toggle dark mode
document.documentElement.classList.toggle('dark');
```

Dark mode colors are automatically applied via `@media (prefers-color-scheme: dark)` or manual `.dark` class.

## 🔧 Extending the System

### Adding New Colors

Edit `src/theme/colors.ts`:

```tsx
const myCustomPalette = {
  50: '#...',
  500: '#...',
  900: '#...',
} as const;

export const colors = {
  // ...existing colors
  custom: myCustomPalette,
};
```

### Adding New Spacing

Edit `src/theme/spacing.ts`:

```tsx
const space = {
  // ...existing values
  '128': '32rem',
};
```

### Adding New Breakpoints

Edit `src/theme/breakpoints.ts`:

```tsx
const breakpoints = {
  // ...existing breakpoints
  '4xl': '2560px',
};
```

## 📦 Integration with Tailwind

The `tailwind.config.js` imports design tokens and makes them available in Tailwind classes:

```js
import { colors, typography, spacing } from './src/theme/index.js';

export default {
  theme: {
    extend: {
      colors: {
        primary: colors.primary,
        // ...
      },
      spacing: spacing.space,
      // ...
    },
  },
};
```

## 📖 Resources

- [Design Tokens Best Practices](https://css-tricks.com/what-are-design-tokens/)
- [Tailwind CSS Customization](https://tailwindcss.com/docs/configuration)
- [Accessibility Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)

## 🤝 Contributing

When modifying design tokens:

1. Update the corresponding token file
2. Update this README if adding new categories
3. Ensure TypeScript types are exported
4. Update Tailwind config if needed
5. Test in both light and dark modes
6. Verify accessibility compliance

---

**Last Updated:** January 2026
**Version:** 1.0.0
