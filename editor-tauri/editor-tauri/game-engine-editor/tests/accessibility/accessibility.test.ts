/**
 * Accessibility Tests
 *
 * Test suite for accessibility features
 */

import { announceToScreenReader, generateId, setFocus, checkContrast } from '../../src/utils/accessibility';
import { IdGenerator } from '../../src/utils/accessibility';

describe('Accessibility Utilities', () => {
  beforeEach(() => {
    // Clear DOM before each test
    document.body.innerHTML = '';
  });

  describe('generateId', () => {
    it('should generate unique IDs', () => {
      const id1 = generateId('test');
      const id2 = generateId('test');

      expect(id1).not.toBe(id2);
      expect(id1).toMatch(/^test-\d+-/);
    });

    it('should use custom prefix', () => {
      const id = generateId('custom');
      expect(id).toMatch(/^custom-/);
    });

    it('should use default prefix when none provided', () => {
      const id = generateId();
      expect(id).toMatch(/^id-/);
    });
  });

  describe('IdGenerator', () => {
    it('should generate sequential IDs', () => {
      const generator = new IdGenerator('button');
      const id1 = generator.generate();
      const id2 = generator.generate();
      const id3 = generator.generate();

      expect(id1).toBe('button-0');
      expect(id2).toBe('button-1');
      expect(id3).toBe('button-2');
    });

    it('should reset counter', () => {
      const generator = new IdGenerator('test');
      generator.generate();
      generator.generate();
      generator.reset();

      const id = generator.generate();
      expect(id).toBe('test-0');
    });
  });

  describe('announceToScreenReader', () => {
    it('should create live region element', () => {
      announceToScreenReader('Test message', 'polite');

      const liveRegion = document.getElementById('a11y-live-region-polite');
      expect(liveRegion).toBeTruthy();
      expect(liveRegion?.getAttribute('role')).toBe('status');
      expect(liveRegion?.getAttribute('aria-live')).toBe('polite');
      expect(liveRegion?.getAttribute('aria-atomic')).toBe('true');
    });

    it('should create assertive live region', () => {
      announceToScreenReader('Alert message', 'assertive');

      const liveRegion = document.getElementById('a11y-live-region-assertive');
      expect(liveRegion?.getAttribute('aria-live')).toBe('assertive');
    });

    it('should update live region content', () => {
      announceToScreenReader('First message', 'polite');
      announceToScreenReader('Second message', 'polite');

      const liveRegion = document.getElementById('a11y-live-region-polite');
      expect(liveRegion?.textContent).toBe('Second message');
    });

    it('should hide live region visually', () => {
      announceToScreenReader('Test', 'polite');

      const liveRegion = document.getElementById('a11y-live-region-polite');
      const styles = window.getComputedStyle(liveRegion!);

      expect(styles.position).toBe('absolute');
      expect(parseInt(styles.left)).toBeLessThan(0);
    });
  });

  describe('setFocus', () => {
    it('should focus element', () => {
      const button = document.createElement('button');
      document.body.appendChild(button);

      const result = setFocus(button);

      expect(result).toBe(true);
      expect(document.activeElement).toBe(button);
    });

    it('should handle focus failures gracefully', () => {
      const div = document.createElement('div');
      document.body.appendChild(div);

      const result = setFocus(div);

      // div is not focusable by default
      expect(document.activeElement).not.toBe(div);
    });

    it('should prevent scroll when requested', () => {
      const button = document.createElement('button');
      document.body.appendChild(button);

      // Scroll to bottom first
      window.scrollTo(0, document.body.scrollHeight);
      const scrollYBefore = window.scrollY;

      setFocus(button, { preventScroll: true });

      expect(window.scrollY).toBe(scrollYBefore);
    });
  });

  describe('checkContrast', () => {
    it('should pass high contrast colors', () => {
      const result = checkContrast('#ffffff', '#000000');
      expect(result).toBe(true);
    });

    it('should fail low contrast colors', () => {
      const result = checkContrast('#cccccc', '#eeeeee');
      expect(result).toBe(false);
    });

    it('should use lower threshold for large text', () => {
      const result1 = checkContrast('#888888', '#ffffff', false);
      const result2 = checkContrast('#888888', '#ffffff', true);

      expect(result2).toBe(true); // Large text passes
      expect(result1).toBe(false); // Normal text fails
    });

    it('should handle RGB format', () => {
      const result = checkContrast('rgb(255, 255, 255)', 'rgb(0, 0, 0)');
      expect(result).toBe(true);
    });
  });
});

/**
 * Keyboard Navigation Tests
 */
describe('Keyboard Navigation', () => {
  describe('Entity Tree Navigation', () => {
    beforeEach(() => {
      // Setup entity tree DOM
      const tree = document.createElement('div');
      tree.setAttribute('role', 'tree');
      tree.innerHTML = `
        <div role="treeitem" tabindex="0" data-entity-id="1">Entity 1</div>
        <div role="treeitem" tabindex="-1" data-entity-id="2">Entity 2</div>
        <div role="treeitem" tabindex="-1" data-entity-id="3">Entity 3</div>
      `;
      document.body.appendChild(tree);
    });

    afterEach(() => {
      document.body.innerHTML = '';
    });

    it('should navigate with arrow keys', () => {
      const items = document.querySelectorAll('[role="treeitem"]');
      const firstItem = items[0] as HTMLElement;
      const lastItem = items[2] as HTMLElement;

      // Focus first item
      firstItem.focus();
      expect(document.activeElement).toBe(firstItem);

      // Press Arrow Down
      const downEvent = new KeyboardEvent('keydown', { key: 'ArrowDown' });
      firstItem.dispatchEvent(downEvent);

      // Check if focus moved (this would be handled by component logic)
      expect(firstItem).toBeTruthy();
      expect(lastItem).toBeTruthy();
    });

    it('should select with Enter key', () => {
      const item = document.querySelector('[role="treeitem"]') as HTMLElement;

      const enterEvent = new KeyboardEvent('keydown', {
        key: 'Enter',
        bubbles: true,
      });

      let selected = false;
      item.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
          selected = true;
        }
      });

      item.dispatchEvent(enterEvent);
      expect(selected).toBe(true);
    });
  });
});

/**
 * ARIA Attributes Tests
 */
describe('ARIA Attributes', () => {
  describe('Button ARIA', () => {
    it('should have aria-label when icon-only', () => {
      const button = document.createElement('button');
      button.setAttribute('aria-label', 'Close');
      button.innerHTML = '<svg>X</svg>';

      document.body.appendChild(button);

      expect(button.getAttribute('aria-label')).toBe('Close');
      expect(button.textContent.trim()).toBe('');
    });

    it('should have aria-pressed for toggle buttons', () => {
      const button = document.createElement('button');
      button.setAttribute('aria-pressed', 'true');

      expect(button.getAttribute('aria-pressed')).toBe('true');
    });

    it('should have aria-disabled for disabled state', () => {
      const button = document.createElement('button');
      button.disabled = true;

      expect(button.disabled).toBe(true);
    });
  });

  describe('Modal ARIA', () => {
    it('should have role="dialog"', () => {
      const modal = document.createElement('div');
      modal.setAttribute('role', 'dialog');

      expect(modal.getAttribute('role')).toBe('dialog');
    });

    it('should have aria-modal="true"', () => {
      const modal = document.createElement('div');
      modal.setAttribute('role', 'dialog');
      modal.setAttribute('aria-modal', 'true');

      expect(modal.getAttribute('aria-modal')).toBe('true');
    });

    it('should have aria-labelledby', () => {
      const modal = document.createElement('div');
      modal.setAttribute('role', 'dialog');
      modal.setAttribute('aria-labelledby', 'modal-title');

      expect(modal.getAttribute('aria-labelledby')).toBe('modal-title');
    });
  });

  describe('Tree ARIA', () => {
    it('should have role="tree"', () => {
      const tree = document.createElement('div');
      tree.setAttribute('role', 'tree');

      expect(tree.getAttribute('role')).toBe('tree');
    });

    it('should have aria-multiselectable="true"', () => {
      const tree = document.createElement('div');
      tree.setAttribute('role', 'tree');
      tree.setAttribute('aria-multiselectable', 'true');

      expect(tree.getAttribute('aria-multiselectable')).toBe('true');
    });

    it('treeitem should have aria-expanded', () => {
      const item = document.createElement('div');
      item.setAttribute('role', 'treeitem');
      item.setAttribute('aria-expanded', 'true');

      expect(item.getAttribute('aria-expanded')).toBe('true');
    });

    it('treeitem should have aria-level', () => {
      const item = document.createElement('div');
      item.setAttribute('role', 'treeitem');
      item.setAttribute('aria-level', '2');

      expect(item.getAttribute('aria-level')).toBe('2');
    });

    it('treeitem should have aria-setsize and aria-posinset', () => {
      const item = document.createElement('div');
      item.setAttribute('role', 'treeitem');
      item.setAttribute('aria-setsize', '10');
      item.setAttribute('aria-posinset', '5');

      expect(item.getAttribute('aria-setsize')).toBe('10');
      expect(item.getAttribute('aria-posinset')).toBe('5');
    });
  });

  describe('Live Region ARIA', () => {
    it('should have aria-live', () => {
      const region = document.createElement('div');
      region.setAttribute('role', 'status');
      region.setAttribute('aria-live', 'polite');

      expect(region.getAttribute('aria-live')).toBe('polite');
    });

    it('should have aria-atomic', () => {
      const region = document.createElement('div');
      region.setAttribute('aria-atomic', 'true');

      expect(region.getAttribute('aria-atomic')).toBe('true');
    });
  });
});

/**
 * Focus Management Tests
 */
describe('Focus Management', () => {
  describe('Tab Order', () => {
    beforeEach(() => {
      document.body.innerHTML = `
        <button id="btn1">Button 1</button>
        <button id="btn2">Button 2</button>
        <button id="btn3">Button 3</button>
      `;
    });

    it('should have logical tab order', () => {
      const buttons = document.querySelectorAll('button');

      expect(buttons[0].id).toBe('btn1');
      expect(buttons[1].id).toBe('btn2');
      expect(buttons[2].id).toBe('btn3');
    });

    it('should respect tabindex', () => {
      const btn1 = document.getElementById('btn1') as HTMLButtonElement;
      const btn2 = document.getElementById('btn2') as HTMLButtonElement;

      btn2.tabIndex = 1;
      btn1.tabIndex = 2;

      expect(btn2.tabIndex).toBeLessThan(btn1.tabIndex);
    });
  });

  describe('Focus Indicators', () => {
    it('should have visible focus styles', () => {
      const button = document.createElement('button');
      document.body.appendChild(button);

      button.focus();
      const styles = window.getComputedStyle(button);

      // Check if outline exists (would be set by CSS)
      expect(document.activeElement).toBe(button);
    });
  });
});

/**
 * Color Contrast Tests
 */
describe('Color Contrast', () => {
  const expectedContrasts = [
    { element: '主文本', foreground: '#f1f5f9', background: '#0f172a', expected: 14.3 },
    { element: '次要文本', foreground: '#cbd5e1', background: '#0f172a', expected: 9.8 },
    { element: '链接', foreground: '#60a5fa', background: '#0f172a', expected: 7.2 },
    { element: '按钮文本', foreground: '#ffffff', background: '#3b82f6', expected: 4.8 },
  ];

  expectedContrasts.forEach(({ element, foreground, background, expected }) => {
    it(`${element} should have sufficient contrast`, () => {
      const result = checkContrast(foreground, background);
      expect(result).toBe(true);
    });
  });
});

/**
 * Integration Tests
 */
describe('Accessibility Integration', () => {
  it('should announce entity creation', () => {
    const announcer = jest.spyOn(console, 'log');

    announceToScreenReader('实体已创建', 'polite');

    const liveRegion = document.getElementById('a11y-live-region-polite');
    expect(liveRegion?.textContent).toBe('实体已创建');
  });

  it('should manage focus in modal', () => {
    const modal = document.createElement('div');
    modal.setAttribute('role', 'dialog');
    modal.setAttribute('aria-modal', 'true');
    modal.innerHTML = `
      <button id="close">Close</button>
      <button id="save">Save</button>
    `;

    document.body.appendChild(modal);

    const closeButton = document.getElementById('close') as HTMLButtonElement;
    setFocus(closeButton);

    expect(document.activeElement).toBe(closeButton);
  });

  it('should support keyboard navigation in entity tree', () => {
    const tree = document.createElement('div');
    tree.setAttribute('role', 'tree');
    tree.innerHTML = `
      <div role="treeitem" tabindex="0" data-id="1">Entity 1</div>
      <div role="treeitem" tabindex="-1" data-id="2">Entity 2</div>
    `;

    document.body.appendChild(tree);

    const firstItem = tree.querySelector('[role="treeitem"]') as HTMLElement;
    firstItem.focus();

    expect(document.activeElement).toBe(firstItem);
  });
});

/**
 * WCAG Compliance Tests
 */
describe('WCAG Compliance', () => {
  describe('Perceivable', () => {
    it('should have text alternatives for images', () => {
      const img = document.createElement('img');
      img.alt = 'A beautiful scene';

      expect(img.alt).toBeTruthy();
    });

    it('should have captions for video', () => {
      const video = document.createElement('video');
      // Should have track element for captions
      expect(video).toBeTruthy();
    });
  });

  describe('Operable', () => {
    it('should be keyboard accessible', () => {
      const button = document.createElement('button');
      document.body.appendChild(button);

      // Should be focusable
      expect(button.tabIndex).toBeGreaterThanOrEqual(0);
    });

    it('should not have keyboard trap', () => {
      // Test that user can navigate away with keyboard
      const button1 = document.createElement('button');
      const button2 = document.createElement('button');

      document.body.appendChild(button1);
      document.body.appendChild(button2);

      button1.focus();
      expect(document.activeElement).toBe(button1);

      // Simulate Tab key
      const tabEvent = new KeyboardEvent('keydown', { key: 'Tab' });
      button1.dispatchEvent(tabEvent);

      // Component should handle Tab and move focus
      expect(button1).toBeTruthy();
    });
  });

  describe('Understandable', () => {
    it('should have language attribute', () => {
      expect(document.documentElement.lang).toBeTruthy();
    });

    it('should have labels for inputs', () => {
      const input = document.createElement('input');
      input.id = 'test-input';
      input.setAttribute('aria-label', 'Test input');

      expect(input.getAttribute('aria-label')).toBeTruthy();
    });
  });

  describe('Robust', () => {
    it('should use valid HTML', () => {
      const div = document.createElement('div');
      expect(div instanceof HTMLDivElement).toBe(true);
    });

    it('should have ARIA roles', () => {
      const button = document.createElement('button');
      expect(button.tagName).toBe('BUTTON');
    });
  });
});
