/**
 * Accessibility Utilities
 *
 * Provides helper functions for implementing accessibility features
 * including focus management, screen reader announcements, and ID generation.
 */

/**
 * Generate a unique ID for accessibility attributes
 *
 * @param prefix - Optional prefix for the ID
 * @returns A unique ID string
 *
 * @example
 * ```ts
 * const id = generateId('button'); // 'button-1234567890'
 * ```
 */
export function generateId(prefix: string = 'id'): string {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * ID generator for creating consistent IDs across renders
 */
export class IdGenerator {
  private prefix: string;
  private counter: number;

  constructor(prefix: string = 'id') {
    this.prefix = prefix;
    this.counter = 0;
  }

  /**
   * Generate a new unique ID
   */
  generate(): string {
    return `${this.prefix}-${this.counter++}`;
  }

  /**
   * Reset the counter
   */
  reset(): void {
    this.counter = 0;
  }
}

/**
 * Announce a message to screen readers using an ARIA live region
 *
 * @param message - The message to announce
 * @param priority - The priority level ('polite' or 'assertive')
 * @param duration - Optional duration in milliseconds after which to clear the message
 *
 * @example
 * ```ts
 * announceToScreenReader('Entity created successfully', 'polite');
 * announceToScreenReader('Error occurred', 'assertive');
 * ```
 */
export function announceToScreenReader(
  message: string,
  priority: 'polite' | 'assertive' = 'polite',
  duration?: number
): void {
  // Get or create the live region
  let liveRegion = document.getElementById(`a11y-live-region-${priority}`);

  if (!liveRegion) {
    liveRegion = document.createElement('div');
    liveRegion.id = `a11y-live-region-${priority}`;
    liveRegion.setAttribute('role', 'status');
    liveRegion.setAttribute('aria-live', priority);
    liveRegion.setAttribute('aria-atomic', 'true');

    // Hide the live region visually
    Object.assign(liveRegion.style, {
      position: 'absolute',
      left: '-10000px',
      width: '1px',
      height: '1px',
      overflow: 'hidden',
    });

    document.body.appendChild(liveRegion);
  }

  // Clear any existing content to ensure the message is announced
  liveRegion.textContent = '';

  // Use setTimeout to ensure the screen reader detects the change
  setTimeout(() => {
    if (liveRegion) {
      liveRegion.textContent = message;
    }
  }, 100);

  // Clear the message after a duration if specified
  if (duration) {
    setTimeout(() => {
      if (liveRegion) {
        liveRegion.textContent = '';
      }
    }, duration);
  }
}

/**
 * Create a live region element for announcements
 *
 * @param priority - The priority level ('polite' or 'assertive')
 * @returns The live region element
 */
export function createLiveRegion(
  priority: 'polite' | 'assertive' = 'polite'
): HTMLElement {
  const liveRegion = document.createElement('div');
  liveRegion.id = `a11y-live-region-${priority}`;
  liveRegion.setAttribute('role', 'status');
  liveRegion.setAttribute('aria-live', priority);
  liveRegion.setAttribute('aria-atomic', 'true');

  Object.assign(liveRegion.style, {
    position: 'absolute',
    left: '-10000px',
    width: '1px',
    height: '1px',
    overflow: 'hidden',
  });

  document.body.appendChild(liveRegion);

  return liveRegion;
}

/**
 * Set focus to an element with retry logic
 *
 * @param element - The element to focus
 * @param options - Configuration options
 * @returns true if focus was successful, false otherwise
 *
 * @example
 * ```ts
 * const button = document.getElementById('my-button');
 * if (button) {
 *   setFocus(button);
 * }
 * ```
 */
export function setFocus(
  element: HTMLElement,
  options: {
    preventScroll?: boolean;
    retryCount?: number;
    retryDelay?: number;
  } = {}
): boolean {
  const {
    preventScroll = false,
    retryCount = 3,
    retryDelay = 50,
  } = options;

  let attempts = 0;

  const attemptFocus = (): boolean => {
    try {
      element.focus({ preventScroll });
      return document.activeElement === element;
    } catch (error) {
      console.warn('Failed to focus element:', error);
      return false;
    }
  };

  const tryFocusWithRetry = (): boolean => {
    if (attemptFocus()) {
      return true;
    }

    attempts++;

    if (attempts < retryCount) {
      setTimeout(tryFocusWithRetry, retryDelay);
    }

    return false;
  };

  return tryFocusWithRetry();
}

/**
 * Set focus to the first focusable element in a container
 *
 * @param container - The container element
 * @param selector - Optional custom selector for focusable elements
 * @returns The focused element or null
 *
 * @example
 * ```ts
 * const container = document.getElementById('modal');
 * if (container) {
 *   const focused = setFocusToFirst(container);
 * }
 * ```
 */
export function setFocusToFirst(
  container: HTMLElement,
  selector?: string
): HTMLElement | null {
  const focusableSelector =
    selector ||
    [
      'a[href]',
      'button:not([disabled])',
      'textarea:not([disabled])',
      'input:not([disabled])',
      'select:not([disabled])',
      '[tabindex]:not([tabindex="-1"])',
    ].join(', ');

  const focusableElements = Array.from(
    container.querySelectorAll<HTMLElement>(focusableSelector)
  ).filter(
    (element) =>
      getComputedStyle(element).display !== 'none' &&
      getComputedStyle(element).visibility !== 'hidden'
  );

  if (focusableElements.length > 0) {
    const first = focusableElements[0];
    setFocus(first);
    return first;
  }

  return null;
}

/**
 * Check if an element is focusable
 *
 * @param element - The element to check
 * @returns true if the element is focusable
 */
export function isFocusable(element: HTMLElement): boolean {
  if (element.disabled || element.getAttribute('aria-hidden') === 'true') {
    return false;
  }

  const style = getComputedStyle(element);
  if (style.display === 'none' || style.visibility === 'hidden') {
    return false;
  }

  const focusableTags = ['BUTTON', 'INPUT', 'SELECT', 'TEXTAREA', 'A'];
  const isFocusableTag =
    focusableTags.includes(element.tagName) ||
    element.getAttribute('tabindex') !== null;

  return isFocusableTag;
}

/**
 * Get all focusable elements within a container
 *
 * @param container - The container element
 * @param options - Configuration options
 * @returns Array of focusable elements
 */
export function getFocusableElements(
  container: HTMLElement,
  options: {
    exclude?: HTMLElement[];
    include?: string[];
  } = {}
): HTMLElement[] {
  const { exclude = [], include = [] } = options;

  const focusableSelectors = [
    'a[href]',
    'button:not([disabled])',
    'textarea:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
    '[contenteditable="true"]',
    ...include,
  ].join(', ');

  const focusableElements = Array.from(
    container.querySelectorAll<HTMLElement>(focusableSelectors)
  );

  return focusableElements.filter(
    (element) =>
      !exclude.includes(element) &&
      getComputedStyle(element).display !== 'none' &&
      getComputedStyle(element).visibility !== 'hidden'
  );
}

/**
 * Trap tab key focus within a container
 *
 * @param event - The keyboard event
 * @param container - The container element
 */
export function trapTabKey(
  event: KeyboardEvent,
  container: HTMLElement
): void {
  if (event.key !== 'Tab') return;

  const focusableElements = getFocusableElements(container);
  if (focusableElements.length === 0) return;

  const firstElement = focusableElements[0];
  const lastElement = focusableElements[focusableElements.length - 1];

  if (event.shiftKey) {
    if (document.activeElement === firstElement) {
      event.preventDefault();
      lastElement.focus();
    }
  } else {
    if (document.activeElement === lastElement) {
      event.preventDefault();
      firstElement.focus();
    }
  }
}

/**
 * Check if an element is visible
 *
 * @param element - The element to check
 * @returns true if the element is visible
 */
export function isVisible(element: HTMLElement): boolean {
  const style = getComputedStyle(element);
  return (
    style.display !== 'none' &&
    style.visibility !== 'hidden' &&
    style.opacity !== '0'
  );
}

/**
 * Get the previous and next focusable elements
 *
 * @param container - The container element
 * @param currentElement - The currently focused element
 * @returns Object with previous and next elements
 */
export function getAdjacentFocusableElements(
  container: HTMLElement,
  currentElement: HTMLElement
): {
  previous: HTMLElement | null;
  next: HTMLElement | null;
} {
  const focusableElements = getFocusableElements(container);
  const currentIndex = focusableElements.indexOf(currentElement);

  if (currentIndex === -1) {
    return { previous: null, next: null };
  }

  return {
    previous: focusableElements[currentIndex - 1] || null,
    next: focusableElements[currentIndex + 1] || null,
  };
}

/**
 * Move focus to the next or previous element
 *
 * @param direction - The direction to move ('next' or 'previous')
 * @param container - The container element
 * @param currentElement - The currently focused element
 * @returns The newly focused element or null
 */
export function moveFocus(
  direction: 'next' | 'previous',
  container: HTMLElement,
  currentElement: HTMLElement
): HTMLElement | null {
  const { previous, next } = getAdjacentFocusableElements(
    container,
    currentElement
  );

  const targetElement = direction === 'next' ? next : previous;

  if (targetElement) {
    setFocus(targetElement);
    return targetElement;
  }

  return null;
}

/**
 * Pause screen reader announcements
 *
 * Useful during rapid DOM updates to prevent announcement spam
 */
export function pauseAnnouncements(): void {
  const liveRegions = document.querySelectorAll('[aria-live]');
  liveRegions.forEach((region) => {
    region.setAttribute('data-aria-live-paused', region.getAttribute('aria-live') || '');
    region.setAttribute('aria-live', 'off');
  });
}

/**
 * Resume screen reader announcements
 */
export function resumeAnnouncements(): void {
  const liveRegions = document.querySelectorAll('[data-aria-live-paused]');
  liveRegions.forEach((region) => {
    const originalValue = region.getAttribute('data-aria-live-paused');
    if (originalValue) {
      region.setAttribute('aria-live', originalValue);
      region.removeAttribute('data-aria-live-paused');
    }
  });
}

/**
 * Check if color contrast meets WCAG AA standards
 *
 * @param foreground - Foreground color (hex or rgb)
 * @param background - Background color (hex or rgb)
 * @param largeText - Whether the text is large (>18pt or >14pt bold)
 * @returns true if contrast meets WCAG AA standards
 */
export function checkContrast(
  foreground: string,
  background: string,
  largeText: boolean = false
): boolean {
  const hexToRgb = (hex: string): { r: number; g: number; b: number } => {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result
      ? {
          r: parseInt(result[1], 16),
          g: parseInt(result[2], 16),
          b: parseInt(result[3], 16),
        }
      : { r: 0, g: 0, b: 0 };
  };

  const getLuminance = (r: number, g: number, b: number): number => {
    const a = [r, g, b].map((v) => {
      v /= 255;
      return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
    });
    return a[0] * 0.2126 + a[1] * 0.7152 + a[2] * 0.0722;
  };

  const fg = foreground.startsWith('#') ? hexToRgb(foreground) : { r: 0, g: 0, b: 0 };
  const bg = background.startsWith('#') ? hexToRgb(background) : { r: 255, g: 255, b: 255 };

  const fgLuminance = getLuminance(fg.r, fg.g, fg.b);
  const bgLuminance = getLuminance(bg.r, bg.g, bg.b);

  const lighter = Math.max(fgLuminance, bgLuminance);
  const darker = Math.min(fgLuminance, bgLuminance);

  const contrastRatio = (lighter + 0.05) / (darker + 0.05);

  // WCAG AA requires 4.5:1 for normal text, 3:1 for large text
  const minimumRatio = largeText ? 3 : 4.5;

  return contrastRatio >= minimumRatio;
}

/**
 * Add a global keyboard event listener for accessibility shortcuts
 *
 * @param handler - The event handler function
 * @returns Cleanup function to remove the listener
 */
export function addGlobalKeyboardListener(
  handler: (event: KeyboardEvent) => void
): () => void {
  document.addEventListener('keydown', handler);

  return () => {
    document.removeEventListener('keydown', handler);
  };
}

export default {
  generateId,
  IdGenerator,
  announceToScreenReader,
  createLiveRegion,
  setFocus,
  setFocusToFirst,
  isFocusable,
  getFocusableElements,
  trapTabKey,
  isVisible,
  getAdjacentFocusableElements,
  moveFocus,
  pauseAnnouncements,
  resumeAnnouncements,
  checkContrast,
  addGlobalKeyboardListener,
};
