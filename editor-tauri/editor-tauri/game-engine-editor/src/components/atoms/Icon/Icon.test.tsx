import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Icon } from './index';

describe('Icon Component', () => {
  it('renders icon correctly', () => {
    render(<Icon name="Play" />);
    expect(screen.getByRole('img')).toBeInTheDocument();
  });

  it('applies custom size', () => {
    render(<Icon name="Play" size={32} data-testid="icon" />);
    const icon = screen.getByTestId('icon');
    expect(icon).toHaveAttribute('width', '32');
  });

  it('applies custom className', () => {
    render(<Icon name="Play" className="custom-class" />);
    expect(screen.getByRole('img')).toHaveClass('custom-class');
  });

  it('has accessibility label when provided', () => {
    render(<Icon name="Play" label="Play video" />);
    expect(screen.getByLabelText('Play video')).toBeInTheDocument();
  });

  it('is hidden from screen readers when no label provided', () => {
    render(<Icon name="Play" />);
    expect(screen.getByRole('img')).toHaveAttribute('aria-hidden', 'true');
  });

  it('warns when icon name does not exist', () => {
    const consoleSpy = jest.spyOn(console, 'warn').mockImplementation();
    render(<Icon name="NonExistentIcon" />);
    expect(consoleSpy).toHaveBeenCalledWith(expect.stringContaining('not found'));
    consoleSpy.mockRestore();
  });
});
