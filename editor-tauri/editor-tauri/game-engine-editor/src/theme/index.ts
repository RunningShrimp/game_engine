/**
 * Design Tokens System - Unified Export
 *
 * Complete design system tokens for consistent styling across the application.
 * Includes colors, typography, spacing, and more.
 *
 * @example
 * ```tsx
 * import { colors, typography, spacing } from '@/theme';
 *
 * const MyComponent = () => (
 *   <div style={{
 *     color: colors.semantic.text.primary,
 *     fontSize: typography.fontSize.base,
 *     padding: spacing.space[4],
 *   }}>
 *     Content
 *   </div>
 * );
 * ```
 */

// Export all design token modules
export { colors } from './colors';
export { typography } from './typography';
export { spacingTokens as spacing } from './spacing';
export { borderRadiusTokens as borderRadius } from './borderRadius';
export { shadowTokens as shadows } from './shadows';
export { zIndexTokens as zIndex } from './zIndex';
export { transitionTokens as transitions } from './transitions';
export { breakpointTokens as breakpoints } from './breakpoints';

// Export individual token categories for convenience
export type {
  ColorPalette,
  ColorTokens,
  SemanticColors,
} from './colors';

export type {
  FontFamily,
  FontSize,
  FontWeight,
  LineHeight,
  LetterSpacing,
  TypographyPresets,
} from './typography';

export type {
  Space,
  SpacingPresets,
  Container,
} from './spacing';

export type {
  BorderRadius,
  ComponentRadius,
  BorderWidth,
} from './borderRadius';

export type {
  Shadows,
  ComponentShadows,
  Ring,
} from './shadows';

export type {
  ZIndex,
  ComponentZIndex,
  Layers,
} from './zIndex';

export type {
  Duration,
  Easing,
  Delay,
  Transitions,
  ComponentTransitions,
  Animations,
} from './transitions';

export type {
  Breakpoints,
  Screens,
  Containers,
  Responsive,
  MediaQuery,
} from './breakpoints';

// Create a unified theme object for Tailwind config
export const theme = {
  colors: {
    neutral: {},
    primary: {},
    secondary: {},
    success: {},
    error: {},
    warning: {},
    info: {},
  },
  fontFamily: {},
  fontSize: {},
  fontWeight: {},
  lineHeight: {},
  letterSpacing: {},
  spacing: {},
  borderRadius: {},
  boxShadow: {},
  zIndex: {},
  transitionDuration: {},
  transitionTimingFunction: {},
  screens: {},
};

// Type for the complete theme
export type Theme = typeof theme;
