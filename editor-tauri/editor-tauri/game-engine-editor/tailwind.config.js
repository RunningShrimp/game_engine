/**
 * Tailwind CSS Configuration
 *
 * This configuration imports and extends design tokens from the theme system.
 * All design values are centralized in src/theme/ for consistency.
 *
 * @see src/theme/README.md for documentation
 */
import { colors, typography, spacing, borderRadius, shadows, zIndex, transitions, breakpoints } from './src/theme/index.js';

/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      // Color system from design tokens
      colors: {
        // Neutral colors
        neutral: colors.neutral,

        // Semantic color palettes
        primary: colors.primary,
        secondary: colors.secondary,
        success: colors.success,
        error: colors.error,
        warning: colors.warning,
        info: colors.info,

        // Semantic aliases for convenience
        background: colors.semantic.background,
        text: colors.semantic.text,
        border: colors.semantic.border,
      },

      // Typography system
      fontFamily: {
        sans: [typography.fontFamily.sans],
        mono: [typography.fontFamily.mono],
        serif: [typography.fontFamily.serif],
      },

      fontSize: typography.fontSize,
      fontWeight: typography.fontWeight,
      lineHeight: typography.lineHeight,
      letterSpacing: typography.letterSpacing,

      // Spacing system (4px base unit)
      spacing: spacing.space,

      // Border radius system
      borderRadius: borderRadius.borderRadius,

      // Shadow system
      boxShadow: shadows.shadows,

      // Z-index layering
      zIndex: zIndex.zIndex,

      // Transition system
      transitionDuration: transitions.duration,
      transitionTimingFunction: transitions.easing,
      transitionDelay: transitions.delay,

      // Responsive breakpoints
      screens: breakpoints.breakpoints,

      // Animation system
      animation: {
        // Fade animations
        'fade-in': 'fadeIn 0.3s ease-in-out',
        'fade-out': 'fadeOut 0.3s ease-in-out',
        'fade-in-up': 'fadeInUp 0.4s ease-out',
        'fade-in-down': 'fadeInDown 0.4s ease-out',

        // Slide animations
        'slide-in-left': 'slideInFromLeft 0.3s ease-out',
        'slide-in-right': 'slideInFromRight 0.3s ease-out',
        'slide-in-top': 'slideInFromTop 0.3s ease-out',
        'slide-in-bottom': 'slideInFromBottom 0.3s ease-out',

        // Scale animations
        'scale-in': 'scaleIn 0.3s ease-out',
        'scale-out': 'scaleOut 0.3s ease-out',
        'scale-in-bounce': 'scaleInBounce 0.5s ease-out',

        // Shimmer and pulse
        'shimmer': 'shimmer 2s infinite linear',
        'pulse-custom': 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',

        // Spin animations
        'spin-slow': 'spin 3s linear infinite',
        'spin-medium': 'spin 1s linear infinite',
        'spin-fast': 'spin 0.5s linear infinite',

        // Bounce animations
        'bounce-custom': 'bounce 1s infinite',
        'bounce-in': 'bounceIn 0.5s ease-out',
      },
      keyframes: {
        // Fade keyframes
        fadeIn: {
          from: { opacity: '0' },
          to: { opacity: '1' },
        },
        fadeOut: {
          from: { opacity: '1' },
          to: { opacity: '0' },
        },
        fadeInUp: {
          from: { opacity: '0', transform: 'translateY(20px)' },
          to: { opacity: '1', transform: 'translateY(0)' },
        },
        fadeInDown: {
          from: { opacity: '0', transform: 'translateY(-20px)' },
          to: { opacity: '1', transform: 'translateY(0)' },
        },

        // Slide keyframes
        slideInFromLeft: {
          from: { transform: 'translateX(-100%)', opacity: '0' },
          to: { transform: 'translateX(0)', opacity: '1' },
        },
        slideInFromRight: {
          from: { transform: 'translateX(100%)', opacity: '0' },
          to: { transform: 'translateX(0)', opacity: '1' },
        },
        slideInFromTop: {
          from: { transform: 'translateY(-100%)', opacity: '0' },
          to: { transform: 'translateY(0)', opacity: '1' },
        },
        slideInFromBottom: {
          from: { transform: 'translateY(100%)', opacity: '0' },
          to: { transform: 'translateY(0)', opacity: '1' },
        },

        // Scale keyframes
        scaleIn: {
          from: { transform: 'scale(0.9)', opacity: '0' },
          to: { transform: 'scale(1)', opacity: '1' },
        },
        scaleOut: {
          from: { transform: 'scale(1)', opacity: '1' },
          to: { transform: 'scale(0.9)', opacity: '0' },
        },
        scaleInBounce: {
          '0%': { transform: 'scale(0.3)', opacity: '0' },
          '50%': { transform: 'scale(1.05)' },
          '70%': { transform: 'scale(0.9)' },
          '100%': { transform: 'scale(1)', opacity: '1' },
        },

        // Shimmer keyframe
        shimmer: {
          '0%': { backgroundPosition: '-1000px 0' },
          '100%': { backgroundPosition: '1000px 0' },
        },

        // Pulse keyframe
        pulse: {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.5' },
        },

        // Bounce keyframes
        bounce: {
          '0%, 100%': { transform: 'translateY(0)' },
          '50%': { transform: 'translateY(-10px)' },
        },
        bounceIn: {
          '0%': { transform: 'scale(0.3)', opacity: '0' },
          '50%': { transform: 'scale(1.05)', opacity: '1' },
          '70%': { transform: 'scale(0.9)' },
          '100%': { transform: 'scale(1)' },
        },
      },
    },
  },
  plugins: [],
}
