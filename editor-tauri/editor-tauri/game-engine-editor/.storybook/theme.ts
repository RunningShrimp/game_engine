import { create } from '@storybook/theming/create';

/**
 * Custom Storybook Theme
 *
 * Customizes the Storybook UI to match the Game Engine Editor brand.
 */
export const Themes = {
  light: create({
    base: 'light',
    brandTitle: 'Game Engine Editor',
    brandUrl: '/',
    brandTarget: '_self',
    // Primary colors
    colorPrimary: '#6366f1',
    colorSecondary: '#8b5cf6',

    // UI
    appBg: '#ffffff',
    appContentBg: '#f9fafb',
    appBorderColor: '#e5e7eb',
    appBorderRadius: 8,

    // Typography
    fontBase: '"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    fontCode: '"Fira Code", "Consolas", monospace',

    // Text colors
    textColor: '#111827',
    textInverseColor: '#ffffff',

    // Toolbar defaults
    barTextColor: '#6b7280',
    barSelectedColor: '#6366f1',
    barBg: '#ffffff',

    // Form colors
    inputBg: '#ffffff',
    inputBorder: '#d1d5db',
    inputTextColor: '#111827',

    // Utility colors
    boolColor: '#10b981',
    negativeColor: '#ef4444',
    positiveColor: '#10b981',
    warningColor: '#f59e0b',
  }),

  dark: create({
    base: 'dark',
    brandTitle: 'Game Engine Editor',
    brandUrl: '/',
    brandTarget: '_self',
    // Primary colors
    colorPrimary: '#818cf8',
    colorSecondary: '#a78bfa',

    // UI
    appBg: '#111827',
    appContentBg: '#1f2937',
    appBorderColor: '#374151',
    appBorderRadius: 8,

    // Typography
    fontBase: '"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    fontCode: '"Fira Code", "Consolas", monospace',

    // Text colors
    textColor: '#f9fafb',
    textInverseColor: '#111827',

    // Toolbar defaults
    barTextColor: '#9ca3af',
    barSelectedColor: '#818cf8',
    barBg: '#1f2937',

    // Form colors
    inputBg: '#374151',
    inputBorder: '#4b5563',
    inputTextColor: '#f9fafb',

    // Utility colors
    boolColor: '#34d399',
    negativeColor: '#f87171',
    positiveColor: '#34d399',
    warningColor: '#fbbf24',
  }),
};

export default Themes.light;
