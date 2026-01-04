/**
 * Text Atom Component Tests
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Text } from './Text';

describe('Text Component', () => {
  describe('Rendering', () => {
    it('renders children correctly', () => {
      render(<Text>Hello World</Text>);
      expect(screen.getByText('Hello World')).toBeInTheDocument();
    });

    it('renders correct HTML tag for variant', () => {
      const { container: h1 } = render(<Text variant="h1">Heading</Text>);
      const { container: body } = render(<Text variant="body">Text</Text>);

      expect(h1.querySelector('h1')).toBeInTheDocument();
      expect(body.querySelector('p')).toBeInTheDocument();
    });

    it('renders custom tag when "as" prop is provided', () => {
      render(<Text as="span">Custom Tag</Text>);
      expect(screen.getByText('Custom Tag').tagName).toBe('SPAN');
    });
  });

  describe('Variants', () => {
    it('applies h1 variant styles', () => {
      render(<Text variant="h1">Heading</Text>);
      const heading = screen.getByText('Heading');
      expect(heading).toHaveClass('h1');
    });

    it('applies body variant styles', () => {
      render(<Text variant="body">Body text</Text>);
      const text = screen.getByText('Body text');
      expect(text).toHaveClass('body');
    });
  });

  describe('Truncation', () => {
    it('applies truncate class when truncate is true', () => {
      render(
        <Text truncate>
          This is a very long text that should be truncated with ellipsis
        </Text>
      );
      const text = screen.getByText(/This is a very long text/);
      expect(text).toHaveClass('truncate');
    });

    it('applies multiline class when maxLines is specified', () => {
      render(<Text maxLines={2}>Text content</Text>);
      const text = screen.getByText('Text content');
      expect(text).toHaveClass('multiline');
    });
  });
});
