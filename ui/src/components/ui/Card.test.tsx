/**
 * Unit tests for Card component
 * Tests variants, padding, hover states, and children rendering
 */

import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { Card } from './Card';

describe('Card', () => {
  it('should render children correctly', () => {
    render(
      <Card>
        <h2>Card Title</h2>
        <p>Card content</p>
      </Card>
    );

    expect(screen.getByText('Card Title')).toBeInTheDocument();
    expect(screen.getByText('Card content')).toBeInTheDocument();
  });

  it('should apply default styles', () => {
    const { container } = render(<Card>Default card</Card>);
    const card = container.firstChild as HTMLElement;
    expect(card).toHaveClass('bg-surface', 'rounded-card', 'shadow-card');
  });

  it('should apply dark mode classes', () => {
    const { container } = render(<Card>Dark mode card</Card>);
    const card = container.firstChild as HTMLElement;
    expect(card).toHaveClass('dark:bg-surface-dark', 'dark:shadow-card-dark');
  });

  it('should apply custom className', () => {
    const { container } = render(<Card className="custom-card">Custom</Card>);
    const card = container.firstChild as HTMLElement;
    expect(card).toHaveClass('custom-card');
  });

  it('should apply default padding', () => {
    const { container } = render(<Card>Padded card</Card>);
    const card = container.firstChild as HTMLElement;
    expect(card).toHaveClass('p-3');
  });

  it('should apply custom padding', () => {
    const { container } = render(<Card padding="sm">Small padding</Card>);
    const card = container.firstChild as HTMLElement;
    expect(card).toHaveClass('p-2');
  });

  it('should apply no padding', () => {
    const { container } = render(<Card padding="none">No padding</Card>);
    const card = container.firstChild as HTMLElement;
    expect(card).not.toHaveClass('p-0', 'p-2', 'p-3', 'p-4');
  });

  it('should be hoverable', () => {
    const { container } = render(<Card hoverable>Hoverable card</Card>);
    const card = container.firstChild as HTMLElement;
    expect(card).toHaveClass('hover:-translate-y-1', 'cursor-pointer');
  });

  it('should render complex nested content', () => {
    render(
      <Card>
        <header>
          <h1>Header</h1>
        </header>
        <main>
          <p>Main content</p>
        </main>
        <footer>
          <button>Action</button>
        </footer>
      </Card>
    );

    expect(screen.getByText('Header')).toBeInTheDocument();
    expect(screen.getByText('Main content')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /action/i })).toBeInTheDocument();
  });
});
