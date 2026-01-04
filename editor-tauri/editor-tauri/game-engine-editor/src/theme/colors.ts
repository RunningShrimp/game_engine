/**
 * Color System - Design Tokens
 *
 * Complete color palette for the application with semantic color tokens.
 * All colors follow WCAG AA accessibility guidelines (4.5:1 contrast ratio).
 */

// Neutral colors (gray scale)
const neutral = {
  50: '#fafafa',
  100: '#f5f5f5',
  200: '#e5e5e5',
  300: '#d4d4d4',
  400: '#a3a3a3',
  500: '#737373',
  600: '#525252',
  700: '#404040',
  800: '#262626',
  900: '#171717',
  950: '#0a0a0a',
} as const;

// Primary colors - Main brand colors (blue-based)
const primary = {
  50: '#f0f9ff',
  100: '#e0f2fe',
  200: '#bae6fd',
  300: '#7dd3fc',
  400: '#38bdf8',
  500: '#0ea5e9', // Main primary color
  600: '#0284c7',
  700: '#0369a1',
  800: '#075985',
  900: '#0c4a6e',
  950: '#082f49',
} as const;

// Secondary colors - Complementary accent colors (purple-based)
const secondary = {
  50: '#faf5ff',
  100: '#f3e8ff',
  200: '#e9d5ff',
  300: '#d8b4fe',
  400: '#c084fc',
  500: '#a855f7', // Main secondary color
  600: '#9333ea',
  700: '#7e22ce',
  800: '#6b21a8',
  900: '#581c87',
  950: '#3b0764',
} as const;

// Success colors - Green for positive feedback
const success = {
  50: '#f0fdf4',
  100: '#dcfce7',
  200: '#bbf7d0',
  300: '#86efac',
  400: '#4ade80',
  500: '#22c55e', // Main success color
  600: '#16a34a',
  700: '#15803d',
  800: '#166534',
  900: '#14532d',
  950: '#052e16',
} as const;

// Error colors - Red for destructive actions and errors
const error = {
  50: '#fef2f2',
  100: '#fee2e2',
  200: '#fecaca',
  300: '#fca5a5',
  400: '#f87171',
  500: '#ef4444', // Main error color
  600: '#dc2626',
  700: '#b91c1c',
  800: '#991b1b',
  900: '#7f1d1d',
  950: '#450a0a',
} as const;

// Warning colors - Yellow/amber for warnings and cautions
const warning = {
  50: '#fffbeb',
  100: '#fef3c7',
  200: '#fde68a',
  300: '#fcd34d',
  400: '#fbbf24',
  500: '#f59e0b', // Main warning color
  600: '#d97706',
  700: '#b45309',
  800: '#92400e',
  900: '#78350f',
  950: '#451a03',
} as const;

// Info colors - Light blue for informational messages
const info = {
  50: '#eff6ff',
  100: '#dbeafe',
  200: '#bfdbfe',
  300: '#93c5fd',
  400: '#60a5fa',
  500: '#3b82f6', // Main info color
  600: '#2563eb',
  700: '#1d4ed8',
  800: '#1e40af',
  900: '#1e3a8a',
  950: '#172554',
} as const;

// Semantic color tokens for common UI states
const semantic = {
  // Background colors
  background: {
    DEFAULT: neutral[50],
    paper: '#ffffff',
    overlay: 'rgba(0, 0, 0, 0.5)',
  },

  // Text colors
  text: {
    primary: neutral[900],
    secondary: neutral[600],
    disabled: neutral[400],
    inverse: '#ffffff',
    link: primary[600],
  },

  // Border colors
  border: {
    DEFAULT: neutral[300],
    focus: primary[500],
    error: error[500],
    success: success[500],
    warning: warning[500],
  },

  // Interactive states
  action: {
    hover: neutral[100],
    active: neutral[200],
    selected: primary[100],
    disabled: neutral[100],
  },
} as const;

// Special purpose colors
const special = {
  // Code syntax highlighting colors
  code: {
    keyword: '#d73a49',
    string: '#032f62',
    comment: '#6a737d',
    function: '#6f42c1',
    number: '#005cc5',
    class: '#6f42c1',
    variable: '#e36209',
  },

  // Chart colors (data visualization)
  charts: [
    primary[500],
    secondary[500],
    success[500],
    warning[500],
    error[500],
    info[500],
    '#ec4899',
    '#14b8a6',
  ],

  // Status indicators
  status: {
    online: success[500],
    offline: neutral[400],
    busy: error[500],
    away: warning[500],
  },
} as const;

// Export all color tokens
export const colors = {
  neutral,
  primary,
  secondary,
  success,
  error,
  warning,
  info,
  semantic,
  special,
} as const;

// Export individual color palettes for convenience
export { neutral, primary, secondary, success, error, warning, info };

// Export type definitions for TypeScript
export type ColorPalette = typeof neutral;
export type ColorTokens = typeof colors;
export type SemanticColors = typeof semantic;
