/**
 * Spacing System - Design Tokens
 *
 * Consistent spacing scale based on 4px base unit.
 * Follows a geometric progression for harmonious spacing.
 */

// Spacing scale (in rem, based on 16px base = 1rem)
// Base unit: 0.25rem (4px)
const space = {
  '0': '0',
  'px': '1px',     // 1px
  '0.5': '0.125rem',   // 2px
  '1': '0.25rem',      // 4px
  '1.5': '0.375rem',   // 6px
  '2': '0.5rem',       // 8px
  '2.5': '0.625rem',   // 10px
  '3': '0.75rem',      // 12px
  '3.5': '0.875rem',   // 14px
  '4': '1rem',         // 16px
  '5': '1.25rem',      // 20px
  '6': '1.5rem',       // 24px
  '7': '1.75rem',      // 28px
  '8': '2rem',         // 32px
  '9': '2.25rem',      // 36px
  '10': '2.5rem',      // 40px
  '11': '2.75rem',     // 44px
  '12': '3rem',        // 48px
  '14': '3.5rem',      // 56px
  '16': '4rem',        // 64px
  '20': '5rem',        // 80px
  '24': '6rem',        // 96px
  '28': '7rem',        // 112px
  '32': '8rem',        // 128px
  '36': '9rem',        // 144px
  '40': '10rem',       // 160px
  '44': '11rem',       // 176px
  '48': '12rem',       // 192px
  '52': '13rem',       // 208px
  '56': '14rem',       // 224px
  '60': '15rem',       // 240px
  '64': '16rem',       // 256px
  '72': '18rem',       // 288px
  '80': '20rem',       // 320px
  '96': '24rem',       // 384px
} as const;

// Common spacing presets for specific use cases
const spacing = {
  // Component padding
  'input-padding': '0.5rem 0.75rem',      // 8px 12px
  'button-padding': '0.5rem 1rem',        // 8px 16px
  'button-padding-sm': '0.375rem 0.75rem', // 6px 12px
  'button-padding-lg': '0.75rem 1.5rem',   // 12px 24px
  'card-padding': '1.5rem',                // 24px

  // Layout spacing
  'section-gap': '4rem',                   // 64px
  'content-gap': '2rem',                   // 32px
  'item-gap': '1rem',                      // 16px
  'tight-gap': '0.5rem',                   // 8px

  // Grid and layout
  'container-padding': '2rem',             // 32px
  'grid-gap': '1.5rem',                    // 24px
  'flex-gap': '1rem',                      // 16px

  // Margin presets
  'element-margin': '1.5rem',              // 24px
  'section-margin': '3rem',                // 48px
  'page-margin': '4rem',                   // 64px
} as const;

// Container sizes (max-width)
const container = {
  'xs': '20rem',    // 320px
  'sm': '24rem',    // 384px
  'md': '28rem',    // 448px
  'lg': '32rem',    // 512px
  'xl': '36rem',    // 576px
  '2xl': '42rem',   // 672px
  '3xl': '48rem',   // 768px
  '4xl': '56rem',   // 896px
  '5xl': '64rem',   // 1024px
  '6xl': '72rem',   // 1152px
  '7xl': '80rem',   // 1280px
  'full': '100%',
} as const;

// Export all spacing tokens
export const spacingTokens = {
  space,
  spacing,
  container,
} as const;

// Export individual token groups for convenience
export { space, spacing, container };

// Type definitions
export type Space = typeof space;
export type SpacingPresets = typeof spacing;
export type Container = typeof container;
