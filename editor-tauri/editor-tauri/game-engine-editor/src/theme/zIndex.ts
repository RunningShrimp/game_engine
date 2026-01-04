/**
 * Z-Index System - Design Tokens
 *
 * Layer hierarchy for stacked elements.
 * Ensures consistent stacking context across the application.
 */

// Z-index scale
const zIndex = {
  // Base layers
  'base': '0',
  'raised': '10',

  // Dropdown and popover layer
  'dropdown': '1000',
  'sticky': '1020',
  'popover': '1030',
  'tooltip': '1040',

  // Modal and overlay layers
  'modal-backdrop': '1050',
  'modal': '1060',
  'drawer': '1070',

  // Feedback layers
  'toast': '1080',
  'notification': '1090',

  // Highest layer
  'max': '9999',
} as const;

// Component-specific z-index presets
const componentZIndex = {
  // Navigation
  'header': '50',
  'sidebar': '60',
  'subheader': '55',

  // Interactive elements
  'dropdown-menu': zIndex.dropdown,
  'select-menu': zIndex.dropdown,
  'autocomplete': zIndex.dropdown,
  'context-menu': zIndex.dropdown,

  // Overlays
  'popover': zIndex.popover,
  'tooltip': zIndex.tooltip,
  'hint': zIndex.tooltip,

  // Modals and drawers
  'modal-backdrop': zIndex['modal-backdrop'],
  'modal-content': zIndex.modal,
  'drawer-backdrop': zIndex['modal-backdrop'],
  'drawer-content': zIndex.drawer,

  // Feedback
  'toast-container': zIndex.toast,
  'toast-item': zIndex.toast,
  'notification': zIndex.notification,

  // Special cases
  'loading-overlay': '100',
  'screenshot-overlay': '9998',
} as const;

// Z-index layers grouped by category
const layers = {
  // Background and base content
  'background': {
    min: '0',
    max: '49',
  },

  // Navigation and structural elements
  'navigation': {
    min: '50',
    max: '999',
  },

  // Dropdowns and overlays
  'overlay': {
    min: '1000',
    max: '1049',
  },

  // Modals and dialogs
  'modal': {
    min: '1050',
    max: '1079',
  },

  // Feedback and notifications
  'feedback': {
    min: '1080',
    max: '9999',
  },
} as const;

// Export all z-index tokens
export const zIndexTokens = {
  zIndex,
  componentZIndex,
  layers,
} as const;

// Export individual token groups for convenience
export { zIndex, componentZIndex, layers };

// Type definitions
export type ZIndex = typeof zIndex;
export type ComponentZIndex = typeof componentZIndex;
export type Layers = typeof layers;
