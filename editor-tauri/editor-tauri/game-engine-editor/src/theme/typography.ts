/**
 * Typography System - Design Tokens
 *
 * Font families, sizes, weights, line heights, and letter spacing.
 * Optimized for readability and visual hierarchy.
 */

// Font family tokens
const fontFamily = {
  // Primary font for body text
  sans: [
    'Inter',
    '-apple-system',
    'BlinkMacSystemFont',
    'Segoe UI',
    'Roboto',
    'Helvetica Neue',
    'Arial',
    'sans-serif',
  ].join(', '),

  // Monospace font for code
  mono: [
    'JetBrains Mono',
    'Fira Code',
    'Consolas',
    'Monaco',
    'Courier New',
    'monospace',
  ].join(', '),

  // Serif font for special cases
  serif: [
    'Georgia',
    'Cambria',
    'Times New Roman',
    'Times',
    'serif',
  ].join(', '),
} as const;

// Font size tokens (in rem, based on 16px base)
const fontSize = {
  // Display sizes - for large headings
  'display-xl': '3.75rem', // 60px
  'display-lg': '3rem',    // 48px
  'display-md': '2.25rem', // 36px
  'display-sm': '1.875rem', // 30px

  // Heading sizes
  'h1': '2.25rem',   // 36px
  'h2': '1.875rem',  // 30px
  'h3': '1.5rem',    // 24px
  'h4': '1.25rem',   // 20px
  'h5': '1.125rem',  // 18px
  'h6': '1rem',      // 16px

  // Body sizes
  'xl': '1.25rem',   // 20px
  'lg': '1.125rem',  // 18px
  'base': '1rem',    // 16px (default)
  'sm': '0.875rem',  // 14px
  'xs': '0.75rem',   // 12px

  // Special sizes
  'caption': '0.75rem',  // 12px - for captions and labels
  'overline': '0.625rem', // 10px - for overline text
  'button': '0.875rem',   // 14px - for button text
  'code': '0.875rem',     // 14px - for inline code
} as const;

// Font weight tokens
const fontWeight = {
  thin: '100',
  extralight: '200',
  light: '300',
  normal: '400',
  medium: '500',
  semibold: '600',
  bold: '700',
  extrabold: '800',
  black: '900',
} as const;

// Line height tokens (unitless for relative to font size)
const lineHeight = {
  none: '1',
  tight: '1.25',
  snug: '1.375',
  normal: '1.5',
  relaxed: '1.625',
  loose: '2',
} as const;

// Letter spacing tokens (in rem)
const letterSpacing = {
  tighter: '-0.05em',
  tight: '-0.025em',
  normal: '0',
  wide: '0.025em',
  wider: '0.05em',
  widest: '0.1em',
} as const;

// Typography presets - common combinations
const presets = {
  // Display headings
  'display-xl': {
    fontFamily: fontFamily.sans,
    fontSize: fontSize['display-xl'],
    fontWeight: fontWeight.bold,
    lineHeight: lineHeight.tight,
    letterSpacing: letterSpacing.tighter,
  },
  'display-lg': {
    fontFamily: fontFamily.sans,
    fontSize: fontSize['display-lg'],
    fontWeight: fontWeight.bold,
    lineHeight: lineHeight.tight,
    letterSpacing: letterSpacing.tighter,
  },
  'display-md': {
    fontFamily: fontFamily.sans,
    fontSize: fontSize['display-md'],
    fontWeight: fontWeight.semibold,
    lineHeight: lineHeight.snug,
    letterSpacing: letterSpacing.tight,
  },

  // Headings
  'h1': {
    fontFamily: fontFamily.sans,
    fontSize: fontSize.h1,
    fontWeight: fontWeight.bold,
    lineHeight: lineHeight.tight,
  },
  'h2': {
    fontFamily: fontFamily.sans,
    fontSize: fontSize.h2,
    fontWeight: fontWeight.semibold,
    lineHeight: lineHeight.tight,
  },
  'h3': {
    fontFamily: fontFamily.sans,
    fontSize: fontSize.h3,
    fontWeight: fontWeight.semibold,
    lineHeight: lineHeight.snug,
  },
  'h4': {
    fontFamily: fontFamily.sans,
    fontSize: fontSize.h4,
    fontWeight: fontWeight.semibold,
    lineHeight: lineHeight.snug,
  },

  // Body text
  'body-lg': {
    fontFamily: fontFamily.sans,
    fontSize: fontSize.lg,
    fontWeight: fontWeight.normal,
    lineHeight: lineHeight.relaxed,
  },
  'body': {
    fontFamily: fontFamily.sans,
    fontSize: fontSize.base,
    fontWeight: fontWeight.normal,
    lineHeight: lineHeight.normal,
  },
  'body-sm': {
    fontFamily: fontFamily.sans,
    fontSize: fontSize.sm,
    fontWeight: fontWeight.normal,
    lineHeight: lineHeight.normal,
  },

  // UI elements
  'button': {
    fontFamily: fontFamily.sans,
    fontSize: fontSize.button,
    fontWeight: fontWeight.medium,
    lineHeight: lineHeight.normal,
    letterSpacing: letterSpacing.wide,
  },
  'caption': {
    fontFamily: fontFamily.sans,
    fontSize: fontSize.caption,
    fontWeight: fontWeight.normal,
    lineHeight: lineHeight.normal,
  },
  'overline': {
    fontFamily: fontFamily.sans,
    fontSize: fontSize.overline,
    fontWeight: fontWeight.medium,
    lineHeight: lineHeight.normal,
    letterSpacing: letterSpacing.wider,
    textTransform: 'uppercase',
  },

  // Code
  'code': {
    fontFamily: fontFamily.mono,
    fontSize: fontSize.code,
    fontWeight: fontWeight.normal,
    lineHeight: lineHeight.normal,
  },
  'code-inline': {
    fontFamily: fontFamily.mono,
    fontSize: fontSize.sm,
    fontWeight: fontWeight.normal,
    lineHeight: lineHeight.normal,
  },
} as const;

// Export all typography tokens
export const typography = {
  fontFamily,
  fontSize,
  fontWeight,
  lineHeight,
  letterSpacing,
  presets,
} as const;

// Export individual token groups for convenience
export { fontFamily, fontSize, fontWeight, lineHeight, letterSpacing, presets };

// Export type definitions
export type FontFamily = typeof fontFamily;
export type FontSize = typeof fontSize;
export type FontWeight = typeof fontWeight;
export type LineHeight = typeof lineHeight;
export type LetterSpacing = typeof letterSpacing;
export type TypographyPresets = typeof presets;
