import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Badge } from './index';

describe('Badge Component', () => {
  it('renders badge with content', () => {
    render(<Badge>Active</Badge>);
    expect(screen.getByText('Active')).toBeInTheDocument();
    expect(screen.getByRole('status')).toBeInTheDocument();
  });

  it('applies variant classes', () => {
    render(<Badge variant="success">Success</Badge>);
    expect(screen.getByText('Success')).toHaveClass('bg-success');
  });

  it('applies size classes', () => {
    render(<Badge size="lg">Large Badge</Badge>);
    expect(screen.getByText('Large Badge')).toHaveClass('text-base');
  });

  it('renders with icon', () => {
    render(
      <Badge icon={<span data-testid="badge-icon">★</span>}>
        Featured
      </Badge>
    );
    expect(screen.getByTestId('badge-icon')).toBeInTheDocument();
  });

  it('renders with dot indicator', () => {
    render(<Badge dot>New</Badge>);
    const badge = screen.getByRole('status');
    expect(badge.querySelector('span[class*="rounded-full"]')).toBeInTheDocument();
  });

  it('has accessibility label when provided', () => {
    render(<Badge label="User status badge">Online</Badge>);
    expect(screen.getByRole('status')).toHaveAttribute('aria-label', 'User status badge');
  });
});
