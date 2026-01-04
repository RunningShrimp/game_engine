/**
 * Breakpoints System - Design Tokens
 *
 * Responsive design breakpoints for different screen sizes.
 * Follows mobile-first approach.
 */

// Breakpoint tokens (in pixels)
const breakpoints = {
  // Extra small devices (phones)
  'xs': '375px',

  // Small devices (landscape phones)
  'sm': '640px',

  // Medium devices (tablets)
  'md': '768px',

  // Large devices (desktops)
  'lg': '1024px',

  // Extra large devices (large desktops)
  'xl': '1280px',

  // 2XL devices (very large desktops)
  '2xl': '1536px',

  // 3XL devices (ultra-wide monitors)
  '3xl': '1920px',
} as const;

// Screen size categories
const screens = {
  // Mobile devices
  'mobile': {
    min: '0px',
    max: '639px',
  },

  // Tablet devices
  'tablet': {
    min: '640px',
    max: '1023px',
  },

  // Desktop devices
  'desktop': {
    min: '1024px',
    max: '1535px',
  },

  // Large desktop
  'large-desktop': {
    min: '1536px',
    max: '1919px',
  },

  // Ultra-wide monitors
  'ultra-wide': {
    min: '1920px',
  },
} as const;

// Container max-widths for responsive layouts
const containers = {
  'mobile': '100%',
  'sm': '640px',
  'md': '768px',
  'lg': '1024px',
  'xl': '1280px',
  '2xl': '1536px',
  'full': '100%',
} as const;

// Common responsive patterns
const responsive = {
  // Grid columns per breakpoint
  'grid-columns': {
    mobile: 1,
    sm: 2,
    md: 2,
    lg: 3,
    xl: 4,
    '2xl': 4,
  },

  // Container padding per breakpoint
  'container-padding': {
    mobile: '1rem',
    sm: '1.5rem',
    md: '2rem',
    lg: '2rem',
    xl: '2rem',
    '2xl': '2rem',
  },

  // Font size scaling
  'font-scale': {
    mobile: 1,
    sm: 1,
    md: 1.05,
    lg: 1.1,
    xl: 1.15,
    '2xl': 1.2,
  },

  // Gap spacing per breakpoint
  'gap': {
    mobile: '0.75rem',
    sm: '1rem',
    md: '1.5rem',
    lg: '2rem',
    xl: '2rem',
    '2xl': '2rem',
  },
} as const;

// Media query helpers
const mediaQuery = {
  // Min-width queries (mobile-first)
  'up-sm': '@media (min-width: 640px)',
  'up-md': '@media (min-width: 768px)',
  'up-lg': '@media (min-width: 1024px)',
  'up-xl': '@media (min-width: 1280px)',
  'up-2xl': '@media (min-width: 1536px)',

  // Max-width queries (desktop-first)
  'down-sm': '@media (max-width: 639px)',
  'down-md': '@media (max-width: 767px)',
  'down-lg': '@media (max-width: 1023px)',
  'down-xl': '@media (max-width: 1279px)',
  'down-2xl': '@media (max-width: 1535px)',

  // Range queries (specific breakpoints)
  'only-sm': '@media (min-width: 640px) and (max-width: 767px)',
  'only-md': '@media (min-width: 768px) and (max-width: 1023px)',
  'only-lg': '@media (min-width: 1024px) and (max-width: 1279px)',
  'only-xl': '@media (min-width: 1280px) and (max-width: 1535px)',

  // Orientation
  'portrait': '@media (orientation: portrait)',
  'landscape': '@media (orientation: landscape)',

  // High DPI displays
  'retina': '@media (-webkit-min-device-pixel-ratio: 2), (min-resolution: 192dpi)',

  // Print styles
  'print': '@media print',
} as const;

// Export all breakpoint tokens
export const breakpointTokens = {
  breakpoints,
  screens,
  containers,
  responsive,
  mediaQuery,
} as const;

// Export individual token groups for convenience
export { breakpoints, screens, containers, responsive, mediaQuery };

// Type definitions
export type Breakpoints = typeof breakpoints;
export type Screens = typeof screens;
export type Containers = typeof containers;
export type Responsive = typeof responsive;
export type MediaQuery = typeof mediaQuery;
