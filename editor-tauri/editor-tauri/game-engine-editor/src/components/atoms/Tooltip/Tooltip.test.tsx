import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Tooltip } from './index';

describe('Tooltip Component', () => {
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it('renders trigger element', () => {
    render(
      <Tooltip content="Tooltip content">
        <button>Hover me</button>
      </Tooltip>
    );
    expect(screen.getByRole('button')).toBeInTheDocument();
  });

  it('shows tooltip on hover after delay', async () => {
    render(
      <Tooltip content="Tooltip content">
        <button>Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByRole('button');
    fireEvent.mouseEnter(trigger);

    jest.advanceTimersByTime(200);

    await waitFor(() => {
      expect(screen.getByText('Tooltip content')).toBeInTheDocument();
    });
  });

  it('hides tooltip on mouse leave', async () => {
    render(
      <Tooltip content="Tooltip content">
        <button>Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByRole('button');
    fireEvent.mouseEnter(trigger);
    jest.advanceTimersByTime(200);

    fireEvent.mouseLeave(trigger);

    await waitFor(() => {
      expect(screen.queryByText('Tooltip content')).not.toBeInTheDocument();
    });
  });

  it('does not show tooltip when disabled', () => {
    render(
      <Tooltip content="Tooltip content" disabled>
        <button>Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByRole('button');
    fireEvent.mouseEnter(trigger);
    jest.advanceTimersByTime(200);

    expect(screen.queryByText('Tooltip content')).not.toBeInTheDocument();
  });

  it('respects custom delay', async () => {
    render(
      <Tooltip content="Tooltip content" delay={500}>
        <button>Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByRole('button');
    fireEvent.mouseEnter(trigger);
    jest.advanceTimersByTime(300);

    expect(screen.queryByText('Tooltip content')).not.toBeInTheDocument();

    jest.advanceTimersByTime(200);

    await waitFor(() => {
      expect(screen.getByText('Tooltip content')).toBeInTheDocument();
    });
  });

  it('applies aria-describedby when visible', async () => {
    render(
      <Tooltip content="Tooltip content">
        <button>Hover me</button>
      </Tooltip>
    );

    const trigger = screen.getByRole('button');
    fireEvent.mouseEnter(trigger);
    jest.advanceTimersByTime(200);

    await waitFor(() => {
      expect(trigger).toHaveAttribute('aria-describedby', 'tooltip-content');
    });
  });
});
