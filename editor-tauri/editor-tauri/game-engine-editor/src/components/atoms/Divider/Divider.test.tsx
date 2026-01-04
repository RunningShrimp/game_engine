import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Divider } from './index';

describe('Divider Component', () => {
  it('renders horizontal divider by default', () => {
    const { container } = render(<Divider />);
    const divider = container.querySelector('div');
    expect(divider).toHaveClass('w-full');
    expect(divider).toHaveClass('border-t');
  });

  it('renders vertical divider', () => {
    const { container } = render(<Divider orientation="vertical" />);
    const divider = container.querySelector('div');
    expect(divider).toHaveClass('h-full');
    expect(divider).toHaveClass('border-l');
  });

  it('renders with children content', () => {
    render(<Divider>Section Title</Divider>);
    expect(screen.getByText('Section Title')).toBeInTheDocument();
  });

  it('applies label styling when label prop is true', () => {
    render(<Divider label>Important Section</Divider>);
    const text = screen.getByText('Important Section');
    expect(text).toHaveClass('font-medium');
  });

  it('has proper accessibility role', () => {
    const { container } = render(<Divider />);
    const divider = container.querySelector('div');
    expect(divider).toHaveAttribute('role', 'separator');
  });

  it('has proper aria-orientation', () => {
    const { container } = render(<Divider orientation="vertical" />);
    const divider = container.querySelector('div');
    expect(divider).toHaveAttribute('aria-orientation', 'vertical');
  });
});
