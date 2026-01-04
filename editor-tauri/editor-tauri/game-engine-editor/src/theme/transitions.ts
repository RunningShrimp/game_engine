/**
 * Transition System - Design Tokens
 *
 * Duration, timing functions, and presets for smooth animations.
 * Optimized for natural, pleasing motion.
 */

// Transition duration tokens (in milliseconds)
const duration = {
  'instant': '100ms',
  'fast': '150ms',
  'base': '200ms',
  'normal': '300ms',
  'slow': '500ms',
  'slower': '700ms',
  'slowest': '1000ms',
} as const;

// Easing/timing function tokens
const easing = {
  // Linear
  'linear': 'linear',

  // Standard easing
  'ease': 'ease',
  'ease-in': 'ease-in',
  'ease-out': 'ease-out',
  'ease-in-out': 'ease-in-out',

  // Custom bezier curves
  'bounce': 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
  'elastic': 'cubic-bezier(0.87, 0, 0.13, 1)',

  // Smooth curves
  'smooth': 'cubic-bezier(0.4, 0, 0.2, 1)',
  'smooth-in': 'cubic-bezier(0.4, 0, 1, 1)',
  'smooth-out': 'cubic-bezier(0, 0, 0.2, 1)',

  // Material design curves
  'decelerate': 'cubic-bezier(0.0, 0.0, 0.2, 1)',
  'accelerate': 'cubic-bezier(0.4, 0.0, 1, 1)',
  'sharp': 'cubic-bezier(0.4, 0.0, 0.6, 1)',
} as const;

// Transition delay tokens
const delay = {
  'none': '0ms',
  'short': '100ms',
  'normal': '200ms',
  'long': '300ms',
} as const;

// Common transition presets
const transitions = {
  // Instant transitions (hover, focus)
  'instant': 'all 100ms cubic-bezier(0.4, 0, 0.2, 1)',

  // Fast transitions (buttons, links)
  'fast': 'all 150ms cubic-bezier(0.4, 0, 0.2, 1)',

  // Default transitions (inputs, cards)
  'base': 'all 200ms cubic-bezier(0.4, 0, 0.2, 1)',

  // Normal transitions (dropdowns, modals)
  'normal': 'all 300ms cubic-bezier(0.4, 0, 0.2, 1)',

  // Slow transitions (page transitions)
  'slow': 'all 500ms cubic-bezier(0.4, 0, 0.2, 1)',

  // Property-specific transitions
  'fade': 'opacity 300ms cubic-bezier(0.4, 0, 0.2, 1)',
  'slide': 'transform 300ms cubic-bezier(0.4, 0, 0.2, 1)',
  'scale': 'transform 200ms cubic-bezier(0.4, 0, 0.2, 1)',
  'colors': 'background-color 200ms cubic-bezier(0.4, 0, 0.2, 1), color 200ms cubic-bezier(0.4, 0, 0.2, 1)',
  'shadows': 'box-shadow 200ms cubic-bezier(0.4, 0, 0.2, 1)',
} as const;

// Component-specific transition presets
const componentTransitions = {
  // Buttons
  'button': 'all 150ms cubic-bezier(0.4, 0, 0.2, 1)',
  'button-hover': 'background-color 150ms cubic-bezier(0.4, 0, 0.2, 1), border-color 150ms cubic-bezier(0.4, 0, 0.2, 1)',

  // Inputs
  'input': 'border-color 150ms cubic-bezier(0.4, 0, 0.2, 1), box-shadow 150ms cubic-bezier(0.4, 0, 0.2, 1)',
  'input-focus': 'border-color 200ms cubic-bezier(0.4, 0, 0.2, 1), box-shadow 200ms cubic-bezier(0.4, 0, 0.2, 1)',

  // Cards
  'card': 'transform 200ms cubic-bezier(0.4, 0, 0.2, 1), box-shadow 200ms cubic-bezier(0.4, 0, 0.2, 1)',
  'card-hover': 'transform 250ms cubic-bezier(0.4, 0, 0.2, 1), box-shadow 250ms cubic-bezier(0.4, 0, 0.2, 1)',

  // Modals and overlays
  'modal': 'opacity 300ms cubic-bezier(0.4, 0, 0.2, 1), transform 300ms cubic-bezier(0.4, 0, 0.2, 1)',
  'modal-backdrop': 'opacity 300ms cubic-bezier(0.4, 0, 0.2, 1)',
  'dropdown': 'opacity 200ms cubic-bezier(0.4, 0, 0.2, 1), transform 200ms cubic-bezier(0.4, 0, 0.2, 1)',
  'tooltip': 'opacity 150ms cubic-bezier(0.4, 0, 0.2, 1), transform 150ms cubic-bezier(0.4, 0, 0.2, 1)',

  // Navigation
  'drawer': 'transform 300ms cubic-bezier(0.4, 0, 0.2, 1)',
  'sidebar': 'transform 250ms cubic-bezier(0.4, 0, 0.2, 1)',

  // Feedback
  'toast': 'all 300ms cubic-bezier(0.4, 0, 0.2, 1)',
  'alert': 'opacity 200ms cubic-bezier(0.4, 0, 0.2, 1)',
  'progress': 'width 300ms cubic-bezier(0.4, 0, 0.2, 1)',

  // Tabs
  'tab': 'color 150ms cubic-bezier(0.4, 0, 0.2, 1), border-color 150ms cubic-bezier(0.4, 0, 0.2, 1)',

  // Accordions
  'accordion': 'height 300ms cubic-bezier(0.4, 0, 0.2, 1), opacity 300ms cubic-bezier(0.4, 0, 0.2, 1)',
} as const;

// Animation timing combinations
const animations = {
  // Fade animations
  'fade-in': 'opacity 300ms cubic-bezier(0.4, 0, 0.2, 1)',
  'fade-out': 'opacity 200ms cubic-bezier(0.4, 0, 0.2, 1)',

  // Slide animations
  'slide-in': 'transform 300ms cubic-bezier(0.4, 0, 0.2, 1)',
  'slide-out': 'transform 250ms cubic-bezier(0.4, 0, 0.2, 1)',

  // Scale animations
  'scale-in': 'transform 200ms cubic-bezier(0.4, 0, 0.2, 1)',
  'scale-out': 'transform 150ms cubic-bezier(0.4, 0, 0.2, 1)',

  // Bounce animation
  'bounce': 'transform 500ms cubic-bezier(0.68, -0.55, 0.265, 1.55)',
} as const;

// Export all transition tokens
export const transitionTokens = {
  duration,
  easing,
  delay,
  transitions,
  componentTransitions,
  animations,
} as const;

// Export individual token groups for convenience
export { duration, easing, delay, transitions, componentTransitions, animations };

// Type definitions
export type Duration = typeof duration;
export type Easing = typeof easing;
export type Delay = typeof delay;
export type Transitions = typeof transitions;
export type ComponentTransitions = typeof componentTransitions;
export type Animations = typeof animations;
