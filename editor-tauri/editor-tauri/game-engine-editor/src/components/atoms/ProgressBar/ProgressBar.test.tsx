import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { ProgressBar } from './index';

describe('ProgressBar Component', () => {
  it('renders progress bar', () => {
    const { container } = render(<ProgressBar value={50} />);
    const progressbar = container.querySelector('[role="progressbar"]');
    expect(progressbar).toBeInTheDocument();
  });

  it('displays correct progress value', () => {
    const { container } = render(<ProgressBar value={75} />);
    const progressbar = container.querySelector('[role="progressbar"]') as HTMLElement;
    expect(progressbar).toHaveAttribute('aria-valuenow', '75');
  });

  it('clamps value to 0-100 range', () => {
    const { container: container1 } = render(<ProgressBar value={150} />);
    const { container: container2 } = render(<ProgressBar value={-50} />);

    const progressbar1 = container1.querySelector('[role="progressbar"]') as HTMLElement;
    const progressbar2 = container2.querySelector('[role="progressbar"]') as HTMLElement;

    expect(progressbar1).toHaveAttribute('aria-valuenow', '100');
    expect(progressbar2).toHaveAttribute('aria-valuenow', '0');
  });

  it('applies variant classes', () => {
    const { container } = render(<ProgressBar value={50} variant="success" />);
    const progressFill = container.querySelector('.bg-success');
    expect(progressFill).toBeInTheDocument();
  });

  it('applies size classes', () => {
    const { container } = render(<ProgressBar value={50} size="lg" />);
    const progressFill = container.querySelector('.h-3');
    expect(progressFill).toBeInTheDocument();
  });

  it('shows percentage label when showLabel is true', () => {
    render(<ProgressBar value={75} showLabel />);
    expect(screen.getByText('75%')).toBeInTheDocument();
  });

  it('shows custom label when provided', () => {
    render(<ProgressBar value={50} label="Loading..." />);
    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });

  it('applies striped class when striped is true', () => {
    const { container } = render(<ProgressBar value={50} striped />);
    const progressFill = container.firstChild as HTMLElement;
    const innerBar = progressFill.querySelector('[role="progressbar"]') as HTMLElement;
    expect(innerBar.style.background).toContain('linear-gradient');
  });

  it('has proper accessibility attributes', () => {
    const { container } = render(<ProgressBar value={50} ariaLabel="Download progress" />);
    const progressbar = container.querySelector('[role="progressbar"]') as HTMLElement;
    expect(progressbar).toHaveAttribute('aria-label', 'Download progress');
    expect(progressbar).toHaveAttribute('aria-valuemin', '0');
    expect(progressbar).toHaveAttribute('aria-valuemax', '100');
  });

  it('respects custom max value', () => {
    const { container } = render(<ProgressBar value={75} max={150} />);
    const progressbar = container.querySelector('[role="progressbar"]') as HTMLElement;
    expect(progressbar).toHaveAttribute('aria-valuemax', '150');
  });
});
