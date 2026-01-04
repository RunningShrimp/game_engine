import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Spacer } from './index';

describe('Spacer Component', () => {
  it('renders spacer element', () => {
    const { container } = render(<Spacer />);
    const spacer = container.firstChild as HTMLElement;
    expect(spacer).toBeInTheDocument();
  });

  it('applies size classes', () => {
    const { container } = render(<Spacer size="lg" />);
    const spacer = container.firstChild as HTMLElement;
    expect(spacer).toHaveClass('size-6');
  });

  it('applies custom size style', () => {
    const { container } = render(<Spacer size="custom" value={32} />);
    const spacer = container.firstChild as HTMLElement;
    expect(spacer.style.width).toBe('32px');
  });

  it('applies grow class', () => {
    const { container } = render(<Spacer grow />);
    const spacer = container.firstChild as HTMLElement;
    expect(spacer).toHaveClass('flex-grow');
  });

  it('renders horizontal axis by default', () => {
    const { container } = render(<Spacer />);
    const spacer = container.firstChild as HTMLElement;
    expect(spacer).toHaveClass('w-full');
  });

  it('renders vertical axis', () => {
    const { container } = render(<Spacer axis="vertical" />);
    const spacer = container.firstChild as HTMLElement;
    expect(spacer).toHaveClass('h-full');
  });

  it('has aria-hidden attribute', () => {
    const { container } = render(<Spacer />);
    const spacer = container.firstChild as HTMLElement;
    expect(spacer).toHaveAttribute('aria-hidden', 'true');
  });
});
