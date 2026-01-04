/**
 * Border Radius System - Design Tokens
 *
 * Consistent border radius values for rounded corners.
 * Optimized for modern, clean UI design.
 */

// Border radius tokens
const borderRadius = {
  'none': '0',
  'xs': '0.125rem',   // 2px - subtle rounding
  'sm': '0.25rem',    // 4px - small rounding
  'md': '0.375rem',   // 6px - medium rounding
  'lg': '0.5rem',     // 8px - large rounding
  'xl': '0.75rem',    // 12px - extra large rounding
  '2xl': '1rem',      // 16px - extra extra large rounding
  '3xl': '1.5rem',    // 24px - very large rounding
  'full': '9999px',   // pill shape
} as const;

// Component-specific border radius presets
const componentRadius = {
  // Input components
  'input': borderRadius.md,
  'select': borderRadius.md,
  'textarea': borderRadius.md,
  'checkbox': borderRadius.xs,
  'radio': '9999px', // Always circular

  // Buttons
  'button': borderRadius.md,
  'button-sm': borderRadius.sm,
  'button-lg': borderRadius.lg,
  'button-pill': borderRadius.full,
  'button-circle': borderRadius.full,

  // Cards and panels
  'card': borderRadius.lg,
  'panel': borderRadius.md,
  'popover': borderRadius.lg,
  'tooltip': borderRadius.md,

  // Navigation
  'nav-item': borderRadius.md,
  'tab': borderRadius.md,
  'breadcrumb': borderRadius.sm,

  // Feedback components
  'badge': borderRadius.full,
  'tag': borderRadius.full,
  'alert': borderRadius.md,
  'toast': borderRadius.lg,

  // Media
  'avatar': borderRadius.full,
  'avatar-sm': borderRadius.full,
  'avatar-lg': borderRadius.full,
  'image': borderRadius.md,
  'image-thumbnail': borderRadius.sm,

  // Modals and overlays
  'modal': borderRadius.xl,
  'drawer': borderRadius.xl,
  'dropdown': borderRadius.lg,

  // Other components
  'progress': borderRadius.full,
  'skeleton': borderRadius.md,
  'code-block': borderRadius.md,
} as const;

// Border width tokens
const borderWidth = {
  '0': '0',
  '1': '1px',
  '2': '2px',
  '3': '3px',
  '4': '4px',
  '8': '8px',
} as const;

// Export all border radius tokens
export const borderRadiusTokens = {
  borderRadius,
  componentRadius,
  borderWidth,
} as const;

// Export individual token groups for convenience
export { borderRadius, componentRadius, borderWidth };

// Type definitions
export type BorderRadius = typeof borderRadius;
export type ComponentRadius = typeof componentRadius;
export type BorderWidth = typeof borderWidth;
