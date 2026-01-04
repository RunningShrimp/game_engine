import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Avatar } from './index';

describe('Avatar Component', () => {
  it('renders avatar with image', () => {
    render(<Avatar src="/avatar.png" alt="User avatar" />);
    const img = screen.getByAltText('User avatar');
    expect(img).toBeInTheDocument();
    expect(img.tagName).toBe('IMG');
  });

  it('renders initials fallback', () => {
    render(<Avatar initials="JD" />);
    expect(screen.getByText('JD')).toBeInTheDocument();
  });

  it('generates initials from full name', () => {
    render(<Avatar initials="John Doe" />);
    expect(screen.getByText('JD')).toBeInTheDocument();
  });

  it('applies size classes', () => {
    render(<Avatar initials="AB" size="lg" />);
    expect(screen.getByText('AB').parentElement).toHaveClass('h-12');
  });

  it('applies shape classes', () => {
    const { container } = render(<Avatar initials="AB" shape="square" />);
    expect(container.firstChild).toHaveClass('rounded-md');
  });

  it('shows status indicator', () => {
    render(<Avatar initials="AB" status="online" />);
    const avatar = screen.getByRole('img');
    expect(avatar.querySelector('.bg-success')).toBeInTheDocument();
  });

  it('handles click events', () => {
    const handleClick = jest.fn();
    render(<Avatar initials="AB" onClick={handleClick} />);
    const avatar = screen.getByRole('img');
    avatar.click();
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('has proper accessibility attributes', () => {
    render(<Avatar initials="AB" alt="John Doe" />);
    expect(screen.getByRole('img', { name: 'John Doe' })).toBeInTheDocument();
  });
});
