# Design Tokens Migration Guide

Quick guide to migrating existing components to use the new design token system.

## 🚀 Quick Start

### Step 1: Import Design Tokens
```tsx
import { colors, typography, spacing, borderRadius } from '@/theme';
```

### Step 2: Replace Hardcoded Values

**Before:**
```tsx
<div style={{
  padding: '16px',
  backgroundColor: '#0ea5e9',
  color: '#ffffff',
  borderRadius: '8px',
  fontSize: '14px',
}}>
  Button
</div>
```

**After:**
```tsx
<div style={{
  padding: spacing.space[4],
  backgroundColor: colors.primary[500],
  color: '#ffffff',
  borderRadius: borderRadius.borderRadius.lg,
  fontSize: typography.fontSize.sm,
}}>
  Button
</div>
```

## 📋 Common Migration Patterns

### Colors

**Before:**
```tsx
const primaryColor = '#0ea5e9';
const errorColor = '#ef4444';
const bgColor = '#ffffff';
const textColor = '#171717';
```

**After:**
```tsx
const primaryColor = colors.primary[500];
const errorColor = colors.error[500];
const bgColor = colors.semantic.background.paper;
const textColor = colors.semantic.text.primary;
```

### Typography

**Before:**
```tsx
const headingStyle = {
  fontFamily: 'Inter, sans-serif',
  fontSize: '24px',
  fontWeight: 600,
  lineHeight: 1.25,
};
```

**After:**
```tsx
const headingStyle = typography.presets.h3;
// or
const headingStyle = {
  fontFamily: typography.fontFamily.sans,
  fontSize: typography.fontSize['2xl'],
  fontWeight: typography.fontWeight.semibold,
  lineHeight: typography.lineHeight.tight,
};
```

### Spacing

**Before:**
```tsx
<div style={{
  padding: '16px 24px',
  margin: '0 auto',
  gap: '12px',
}}>
```

**After:**
```tsx
<div style={{
  padding: `${spacing.space[4]} ${spacing.space[6]}`,
  margin: '0 auto',
  gap: spacing.space[3],
}}>
// or use spacing presets
<div style={{
  padding: spacing.spacing['button-padding'],
  gap: spacing.spacing['tight-gap'],
}}>
```

### Border Radius

**Before:**
```tsx
<div style={{ borderRadius: '8px' }}>
<button style={{ borderRadius: '9999px' }}>
```

**After:**
```tsx
<div style={{ borderRadius: borderRadius.borderRadius.lg }}>
<button style={{ borderRadius: borderRadius.borderRadius.full }}>
// or use component presets
<button style={{ borderRadius: borderRadius.componentRadius.button }}>
```

### Shadows

**Before:**
```tsx
<div style={{ boxShadow: '0 4px 6px -1px rgb(0 0 0 / 0.1)' }}>
```

**After:**
```tsx
<div style={{ boxShadow: shadows.shadows.md }}>
// or
<div style={{ boxShadow: shadows.componentShadows.card }}>
```

## 🎨 Component Migration Examples

### Button Component

**Before:**
```tsx
const buttonStyle = {
  padding: '8px 16px',
  backgroundColor: '#0ea5e9',
  color: '#ffffff',
  border: 'none',
  borderRadius: '6px',
  fontSize: '14px',
  fontWeight: 500,
  cursor: 'pointer',
  transition: 'all 150ms ease',
};
```

**After:**
```tsx
import { colors, typography, spacing, borderRadius, transitions } from '@/theme';

const buttonStyle = {
  ...typography.presets.button,
  padding: spacing.spacing['button-padding'],
  backgroundColor: colors.primary[500],
  color: '#ffffff',
  border: 'none',
  borderRadius: borderRadius.componentRadius.button,
  cursor: 'pointer',
  transition: transitions.componentTransitions.button,
};
```

### Input Component

**Before:**
```tsx
const inputStyle = {
  padding: '8px 12px',
  border: '1px solid #d4d4d4',
  borderRadius: '6px',
  fontSize: '16px',
  fontFamily: 'Inter, sans-serif',
};
```

**After:**
```tsx
import { colors, typography, spacing, borderRadius } from '@/theme';

const inputStyle = {
  ...typography.presets.body,
  padding: spacing.spacing['input-padding'],
  border: `1px solid ${colors.semantic.border.DEFAULT}`,
  borderRadius: borderRadius.componentRadius.input,
};
```

### Card Component

**Before:**
```tsx
const cardStyle = {
  padding: '24px',
  backgroundColor: '#ffffff',
  borderRadius: '8px',
  boxShadow: '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
  border: '1px solid #e5e5e5',
};
```

**After:**
```tsx
import { colors, spacing, borderRadius, shadows } from '@/theme';

const cardStyle = {
  padding: spacing.spacing['card-padding'],
  backgroundColor: colors.semantic.background.paper,
  borderRadius: borderRadius.componentRadius.card,
  boxShadow: shadows.componentShadows.card,
  border: `1px solid ${colors.semantic.border.DEFAULT}`,
};
```

### Modal Component

**Before:**
```tsx
const modalStyle = {
  position: 'fixed',
  zIndex: 1060,
  padding: '24px',
  backgroundColor: '#ffffff',
  borderRadius: '12px',
  boxShadow: '0 20px 25px -5px rgb(0 0 0 / 0.1)',
};
```

**After:**
```tsx
import { colors, spacing, borderRadius, shadows, zIndex } from '@/theme';

const modalStyle = {
  position: 'fixed',
  zIndex: zIndex.zIndex.modal,
  padding: spacing.spacing['card-padding'],
  backgroundColor: colors.semantic.background.paper,
  borderRadius: borderRadius.componentRadius.modal,
  boxShadow: shadows.componentShadows.modal,
};
```

## 🔄 Tailwind Migration

### Existing Tailwind Classes

Most existing Tailwind classes will continue to work. The design tokens extend Tailwind with:

**New Color Classes:**
```tsx
// Now available
className="bg-primary-500"
className="text-error-600"
className="border-success-500"
className="bg-neutral-100"
```

**New Spacing Classes:**
```tsx
// Extended spacing scale
className="p-12"  // 3rem
className="gap-14" // 3.5rem
```

**New Border Radius Classes:**
```tsx
// Extended radius scale
className="rounded-2xl"  // 1rem
className="rounded-3xl"  // 1.5rem
```

## 📝 Migration Checklist

### Phase 1: New Development
- [ ] Use design tokens for all new components
- [ ] Import from `@/theme` instead of hardcoding values
- [ ] Use Tailwind classes where appropriate
- [ ] Reference examples.tsx for patterns

### Phase 2: High-Value Components
- [ ] Migrate button components
- [ ] Migrate input components
- [ ] Migrate card components
- [ ] Migrate modal components

### Phase 3: Layout Components
- [ ] Migrate header/navigation
- [ ] Migrate sidebar
- [ ] Migrate footer
- [ ] Migrate grid layouts

### Phase 4: Remaining Components
- [ ] Migrate all remaining components
- [ ] Update inline styles
- [ ] Remove hardcoded values
- [ ] Add TypeScript types

## 🎯 Best Practices

### 1. Use Presets When Available
```tsx
// ✅ Good
typography.presets.h1
spacing.spacing['button-padding']
borderRadius.componentRadius.card

// ❌ Bad
{ fontSize: typography.fontSize['2xl'], fontWeight: typography.fontWeight.bold }
{ padding: `${spacing.space[2]} ${spacing.space[4]}` }
```

### 2. Prefer Semantic Colors
```tsx
// ✅ Good
colors.semantic.text.primary
colors.semantic.border.error
colors.semantic.background.paper

// ❌ Bad
colors.neutral[900]  // Too specific
colors.error[500]    // Less semantic
```

### 3. Use Tailwind Classes for Common Cases
```tsx
// ✅ Good - simple cases
<div className="p-4 bg-primary-500 text-white">

// ✅ Good - complex/dynamic cases
<div style={{ padding: spacing.space[4], backgroundColor: colors.primary[500] }}>
```

### 4. Maintain Type Safety
```tsx
// ✅ Good - typed
import type { ColorTokens } from '@/theme';
const myColor: ColorTokens = colors;

// ❌ Bad - untyped
const myColor = colors;
```

## 🔍 Verification

After migration, verify:

1. **Visual Consistency**
   - Compare before/after screenshots
   - Check color contrast
   - Test responsive breakpoints

2. **Functionality**
   - Test all interactive states
   - Verify dark mode (if implemented)
   - Check hover/focus styles

3. **Code Quality**
   - No hardcoded values remain
   - Proper TypeScript types
   - Consistent token usage

## 📚 Resources

- **Full Documentation**: `src/theme/README.md`
- **Quick Reference**: `src/theme/QUICK_REFERENCE.md`
- **Working Examples**: `src/theme/examples.tsx`
- **CSS Variables**: `src/styles/variables.css`

## 🆘 Troubleshooting

### Issue: Token Not Found
**Solution**: Make sure you're importing from `@/theme`:
```tsx
import { colors } from '@/theme';  // ✅
import { colors } from './theme/colors';  // ❌ Use index.ts instead
```

### Issue: Type Errors
**Solution**: Import types:
```tsx
import type { ColorTokens, TypographyPresets } from '@/theme';
```

### Issue: Tailwind Classes Not Working
**Solution**: Check that token is exported in `tailwind.config.js`

### Issue: Dark Mode Not Applied
**Solution**: Ensure CSS variables are imported in `main.tsx`:
```tsx
import './styles/variables.css';
```

---

**Need Help?** Refer to `src/theme/examples.tsx` for comprehensive examples.
