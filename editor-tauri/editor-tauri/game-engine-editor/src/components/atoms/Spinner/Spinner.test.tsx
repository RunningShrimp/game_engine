import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Spinner } from './index';

describe('Spinner Component', () => {
  it('renders spinner', () => {
    const { container } = render(<Spinner />);
    const spinner = container.querySelector('[role="status"]');
    expect(spinner).toBeInTheDocument();
  });

  it('applies size classes', () => {
    const { container } = render(<Spinner size="lg" />);
    const spinner = container.firstChild as HTMLElement;
    expect(spinner).toHaveClass('h-8');
    expect(spinner).toHaveClass('w-8');
  });

  it('applies color classes', () => {
    const { container } = render(<Spinner color="white" />);
    const spinner = container.firstChild as HTMLElement;
    expect(spinner).toHaveClass('border-white');
  });

  it('has accessibility label', () => {
    render(<Spinner />);
    expect(screen.getByLabelText('Loading')).toBeInTheDocument();
  });

  it('contains screen reader text', () => {
    render(<Spinner />);
    expect(screen.getByText('Loading...')).toBeInTheDocument();
    const srText = screen.getByText('Loading...');
    expect(srText).toHaveClass('sr-only');
  });

  it('applies custom className', () => {
    const { container } = render(<Spinner className="custom-spinner" />);
    const spinner = container.firstChild as HTMLElement;
    expect(spinner).toHaveClass('custom-spinner');
  });

  it('has rounded-full class', () => {
    const { container } = render(<Spinner />);
    const spinner = container.firstChild as HTMLElement;
    expect(spinner).toHaveClass('rounded-full');
  });
});
