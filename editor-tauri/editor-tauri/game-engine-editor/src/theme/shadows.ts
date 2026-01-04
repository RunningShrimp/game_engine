/**
 * Shadow System - Design Tokens
 *
 * Elevation shadows for depth and hierarchy.
 * Uses multiple box-shadows for smooth, realistic effects.
 */

// Shadow tokens
const shadows = {
  // No shadow
  'none': 'none',

  // Small shadows - for subtle elevation
  'sm': '0 1px 2px 0 rgb(0 0 0 / 0.05)',

  // Default shadow - for buttons, cards, etc.
  'md': [
    '0 4px 6px -1px rgb(0 0 0 / 0.1)',
    '0 2px 4px -2px rgb(0 0 0 / 0.1)',
  ].join(', '),

  // Large shadow - for dropdowns, popovers
  'lg': [
    '0 10px 15px -3px rgb(0 0 0 / 0.1)',
    '0 4px 6px -4px rgb(0 0 0 / 0.1)',
  ].join(', '),

  // Extra large shadow - for modals, drawers
  'xl': [
    '0 20px 25px -5px rgb(0 0 0 / 0.1)',
    '0 8px 10px -6px rgb(0 0 0 / 0.1)',
  ].join(', '),

  // 2XL shadow - for highest elevation
  '2xl': '0 25px 50px -12px rgb(0 0 0 / 0.25)',

  // Inner shadow - for pressed states
  'inner': 'inset 0 2px 4px 0 rgb(0 0 0 / 0.05)',

  // Colored shadows for semantic purposes
  'primary-sm': '0 1px 3px 0 rgb(14 165 233 / 0.3)',
  'primary-md': '0 4px 6px -1px rgb(14 165 233 / 0.3)',
  'success-sm': '0 1px 3px 0 rgb(34 197 94 / 0.3)',
  'success-md': '0 4px 6px -1px rgb(34 197 94 / 0.3)',
  'error-sm': '0 1px 3px 0 rgb(239 68 68 / 0.3)',
  'error-md': '0 4px 6px -1px rgb(239 68 68 / 0.3)',
  'warning-sm': '0 1px 3px 0 rgb(245 158 11 / 0.3)',
  'warning-md': '0 4px 6px -1px rgb(245 158 11 / 0.3)',
} as const;

// Component-specific shadow presets
const componentShadows = {
  // Interactive elements
  'button': '0 1px 2px 0 rgb(0 0 0 / 0.05)',
  'button-hover': '0 4px 6px -1px rgb(0 0 0 / 0.1)',
  'button-active': 'inset 0 2px 4px 0 rgb(0 0 0 / 0.05)',

  // Cards and containers
  'card': [
    '0 4px 6px -1px rgb(0 0 0 / 0.1)',
    '0 2px 4px -2px rgb(0 0 0 / 0.1)',
  ].join(', '),
  'card-hover': [
    '0 10px 15px -3px rgb(0 0 0 / 0.1)',
    '0 4px 6px -4px rgb(0 0 0 / 0.1)',
  ].join(', '),

  // Overlays
  'dropdown': [
    '0 10px 15px -3px rgb(0 0 0 / 0.1)',
    '0 4px 6px -4px rgb(0 0 0 / 0.1)',
  ].join(', '),
  'modal': [
    '0 20px 25px -5px rgb(0 0 0 / 0.1)',
    '0 8px 10px -6px rgb(0 0 0 / 0.1)',
  ].join(', '),
  'tooltip': '0 10px 15px -3px rgb(0 0 0 / 0.1)',

  // Navigation
  'header': '0 1px 3px 0 rgb(0 0 0 / 0.1)',
  'sidebar': '0 0 20px 0 rgb(0 0 0 / 0.1)',

  // Feedback elements
  'toast': [
    '0 10px 15px -3px rgb(0 0 0 / 0.1)',
    '0 4px 6px -4px rgb(0 0 0 / 0.1)',
  ].join(', '),
  'alert': '0 1px 2px 0 rgb(0 0 0 / 0.05)',

  // Floating elements
  'fab': [
    '0 10px 15px -3px rgb(0 0 0 / 0.1)',
    '0 4px 6px -4px rgb(0 0 0 / 0.1)',
  ].join(', '),
} as const;

// Ring shadows (for focus states)
const ring = {
  'sm': '0 0 0 1px rgb(14 165 233 / 0.5)',
  'md': '0 0 0 3px rgb(14 165 233 / 0.5)',
  'lg': '0 0 0 5px rgb(14 165 233 / 0.5)',
  'xl': '0 0 0 7px rgb(14 165 233 / 0.5)',
} as const;

// Export all shadow tokens
export const shadowTokens = {
  shadows,
  componentShadows,
  ring,
} as const;

// Export individual token groups for convenience
export { shadows, componentShadows, ring };

// Type definitions
export type Shadows = typeof shadows;
export type ComponentShadows = typeof componentShadows;
export type Ring = typeof ring;
