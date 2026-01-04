import { useEffect, useRef } from 'react';

/**
 * Focus Trap Hook
 *
 * Traps focus within a specified element, ensuring keyboard navigation
 * remains contained. Essential for modals, dialogs, and dropdowns.
 *
 * @param isActive - Whether the focus trap is active
 * @param options - Configuration options
 */
export interface UseFocusTrapOptions {
  /**
   * Elements that should be excluded from the focus trap
   */
  excludeElements?: HTMLElement[];

  /**
   * Whether to automatically focus the first element on activation
   * @default true
   */
  autoFocus?: boolean;

  /**
   * Whether to return focus to the previously focused element on deactivation
   * @default true
   */
  restoreFocus?: boolean;

  /**
   * Additional elements that should be considered focusable
   */
  additionalFocusableElements?: HTMLElement[];
}

/**
 * Hook for trapping focus within a container element
 *
 * @example
 * ```tsx
 * const modalRef = useRef<HTMLDivElement>(null);
 * useFocusTrap(isModalOpen, { containerRef: modalRef });
 *
 * return (
 *   <div ref={modalRef} role="dialog" aria-modal="true">
 *     Modal content
 *   </div>
 * );
 * ```
 */
export function useFocusTrap(
  isActive: boolean,
  options: UseFocusTrapOptions = {}
) {
  const {
    excludeElements = [],
    autoFocus = true,
    restoreFocus = true,
    additionalFocusableElements = [],
  } = options;

  const containerRef = useRef<HTMLElement | null>(null);
  const previousActiveElement = useRef<HTMLElement | null>(null);

  /**
   * Get all focusable elements within the container
   */
  const getFocusableElements = (): HTMLElement[] => {
    if (!containerRef.current) return [];

    const focusableSelectors = [
      'a[href]',
      'button:not([disabled])',
      'textarea:not([disabled])',
      'input:not([disabled])',
      'select:not([disabled])',
      '[tabindex]:not([tabindex="-1"])',
      '[contenteditable="true"]',
    ].join(', ');

    const focusableElements = Array.from(
      containerRef.current.querySelectorAll<HTMLElement>(focusableSelectors)
    );

    // Filter out excluded elements
    return focusableElements.filter(
      (element) =>
        !excludeElements.includes(element) &&
        getComputedStyle(element).display !== 'none' &&
        getComputedStyle(element).visibility !== 'hidden'
    );
  };

  /**
   * Get the first and last focusable elements
   */
  const getBoundaryElements = (): {
    first: HTMLElement | null;
    last: HTMLElement | null;
  } => {
    const focusableElements = [
      ...getFocusableElements(),
      ...additionalFocusableElements,
    ].filter((element) =>
      containerRef.current?.contains(element)
    );

    return {
      first: focusableElements[0] || null,
      last: focusableElements[focusableElements.length - 1] || null,
    };
  };

  /**
   * Focus the first focusable element
   */
  const focusFirstElement = () => {
    const { first } = getBoundaryElements();
    if (first) {
      first.focus();
    }
  };

  /**
   * Handle keyboard navigation
   */
  const handleKeyDown = (event: KeyboardEvent) => {
    if (!isActive || !containerRef.current) return;

    if (event.key !== 'Tab') return;

    const { first, last } = getBoundaryElements();

    if (!first || !last) return;

    // Shift + Tab
    if (event.shiftKey) {
      if (document.activeElement === first) {
        event.preventDefault();
        last.focus();
      }
    }
    // Tab
    else {
      if (document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    }
  };

  /**
   * Activate focus trap
   */
  useEffect(() => {
    if (!isActive) return;

    // Store the currently focused element
    if (restoreFocus) {
      previousActiveElement.current = document.activeElement as HTMLElement;
    }

    // Focus the first element
    if (autoFocus) {
      // Use setTimeout to ensure DOM is ready
      const timeoutId = setTimeout(() => {
        focusFirstElement();
      }, 0);

      return () => clearTimeout(timeoutId);
    }
  }, [isActive, autoFocus, restoreFocus]);

  /**
   * Restore focus when deactivated
   */
  useEffect(() => {
    return () => {
      if (restoreFocus && previousActiveElement.current) {
        previousActiveElement.current.focus();
      }
    };
  }, [restoreFocus]);

  /**
   * Attach keyboard event listener
   */
  useEffect(() => {
    if (!isActive) return;

    document.addEventListener('keydown', handleKeyDown);

    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [isActive]);

  return {
    containerRef,
    focusFirstElement,
    getFocusableElements,
  };
}

/**
 * Hook for trapping focus with a custom container ref
 *
 * @example
 * ```tsx
 * const modalRef = useRef<HTMLDivElement>(null);
 * useFocusTrapRef(isModalOpen, modalRef);
 * ```
 */
export function useFocusTrapRef(
  isActive: boolean,
  ref: React.RefObject<HTMLElement>,
  options: UseFocusTrapOptions = {}
) {
  const {
    excludeElements = [],
    autoFocus = true,
    restoreFocus = true,
    additionalFocusableElements = [],
  } = options;

  const previousActiveElement = useRef<HTMLElement | null>(null);

  const getFocusableElements = (): HTMLElement[] => {
    if (!ref.current) return [];

    const focusableSelectors = [
      'a[href]',
      'button:not([disabled])',
      'textarea:not([disabled])',
      'input:not([disabled])',
      'select:not([disabled])',
      '[tabindex]:not([tabindex="-1"])',
      '[contenteditable="true"]',
    ].join(', ');

    const focusableElements = Array.from(
      ref.current.querySelectorAll<HTMLElement>(focusableSelectors)
    );

    return focusableElements.filter(
      (element) =>
        !excludeElements.includes(element) &&
        getComputedStyle(element).display !== 'none' &&
        getComputedStyle(element).visibility !== 'hidden'
    );
  };

  const getBoundaryElements = (): {
    first: HTMLElement | null;
    last: HTMLElement | null;
  } => {
    const focusableElements = [
      ...getFocusableElements(),
      ...additionalFocusableElements,
    ].filter((element) => ref.current?.contains(element));

    return {
      first: focusableElements[0] || null,
      last: focusableElements[focusableElements.length - 1] || null,
    };
  };

  const focusFirstElement = () => {
    const { first } = getBoundaryElements();
    if (first) {
      first.focus();
    }
  };

  const handleKeyDown = (event: KeyboardEvent) => {
    if (!isActive || !ref.current) return;

    if (event.key !== 'Tab') return;

    const { first, last } = getBoundaryElements();

    if (!first || !last) return;

    if (event.shiftKey) {
      if (document.activeElement === first) {
        event.preventDefault();
        last.focus();
      }
    } else {
      if (document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    }
  };

  useEffect(() => {
    if (!isActive) return;

    if (restoreFocus) {
      previousActiveElement.current = document.activeElement as HTMLElement;
    }

    if (autoFocus) {
      const timeoutId = setTimeout(() => {
        focusFirstElement();
      }, 0);

      return () => clearTimeout(timeoutId);
    }
  }, [isActive, autoFocus, restoreFocus]);

  useEffect(() => {
    if (!isActive) return;

    document.addEventListener('keydown', handleKeyDown);

    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      if (restoreFocus && previousActiveElement.current) {
        previousActiveElement.current.focus();
      }
    };
  }, [isActive, restoreFocus]);

  return {
    focusFirstElement,
    getFocusableElements,
  };
}

export default useFocusTrap;
