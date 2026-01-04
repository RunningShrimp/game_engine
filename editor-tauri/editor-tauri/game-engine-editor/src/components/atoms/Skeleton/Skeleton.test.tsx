import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Skeleton, CardSkeleton, TableSkeleton } from './index';

describe('Skeleton Component', () => {
  it('renders skeleton element', () => {
    const { container } = render(<Skeleton />);
    const skeleton = container.firstChild as HTMLElement;
    expect(skeleton).toBeInTheDocument();
  });

  it('applies variant classes', () => {
    const { container } = render(<Skeleton variant="circular" />);
    const skeleton = container.firstChild as HTMLElement;
    expect(skeleton).toHaveClass('rounded-full');
  });

  it('applies custom width and height', () => {
    const { container } = render(<Skeleton width="100px" height="50px" />);
    const skeleton = container.firstChild as HTMLElement;
    expect(skeleton.style.width).toBe('100px');
    expect(skeleton.style.height).toBe('50px');
  });

  it('has aria-hidden attribute', () => {
    const { container } = render(<Skeleton />);
    const skeleton = container.firstChild as HTMLElement;
    expect(skeleton).toHaveAttribute('aria-hidden', 'true');
  });

  it('applies animate class by default', () => {
    const { container } = render(<Skeleton />);
    const skeleton = container.firstChild as HTMLElement;
    expect(skeleton).toHaveClass('animate-pulse');
  });

  it('does not animate when animate is false', () => {
    const { container } = render(<Skeleton animate={false} />);
    const skeleton = container.firstChild as HTMLElement;
    expect(skeleton).not.toHaveClass('animate-pulse');
  });

  it('renders multiple lines for text variant', () => {
    const { container } = render(<Skeleton variant="text" lines={3} />);
    const skeletons = container.querySelectorAll('.bg-muted');
    expect(skeletons).toHaveLength(3);
  });
});

describe('CardSkeleton Component', () => {
  it('renders card skeleton with avatar and text', () => {
    render(<CardSkeleton />);
    expect(screen.getAllByText('')[0]).toBeInTheDocument();
  });

  it('applies custom className', () => {
    const { container } = render(<CardSkeleton className="custom-card" />);
    expect(container.firstChild).toHaveClass('custom-card');
  });
});

describe('TableSkeleton Component', () => {
  it('renders table with correct number of rows and columns', () => {
    const { container } = render(<TableSkeleton rows={3} columns={4} />);
    const skeletons = container.querySelectorAll('.bg-muted');
    expect(skeletons).toHaveLength(12); // 3 rows * 4 columns
  });

  it('uses default rows and columns', () => {
    const { container } = render(<TableSkeleton />);
    const skeletons = container.querySelectorAll('.bg-muted');
    expect(skeletons).toHaveLength(20); // 5 rows * 4 columns (default)
  });
});
