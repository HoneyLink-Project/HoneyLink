/**
 * Unit tests for Input component
 * Tests validation, error states, accessibility, and user interaction
 */

import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Search } from 'lucide-react';
import { describe, expect, it, vi } from 'vitest';
import { Input } from './Input';

describe('Input', () => {
  it('should render with label', () => {
    render(<Input label="Username" />);
    expect(screen.getByLabelText('Username')).toBeInTheDocument();
  });

  it('should render without label', () => {
    render(<Input placeholder="Enter text" />);
    expect(screen.getByPlaceholderText('Enter text')).toBeInTheDocument();
  });

  it('should display error message', () => {
    render(<Input label="Email" error="Invalid email address" />);
    expect(screen.getByText('Invalid email address')).toBeInTheDocument();
  });

  it('should display helper text', () => {
    render(<Input label="Password" helperText="Must be at least 8 characters" />);
    expect(screen.getByText('Must be at least 8 characters')).toBeInTheDocument();
  });

  it('should render with icon', () => {
    render(<Input label="Search" icon={<Search data-testid="search-icon" />} />);
    expect(screen.getByTestId('search-icon')).toBeInTheDocument();
  });

  it('should handle user input', async () => {
    const user = userEvent.setup();
    render(<Input label="Name" />);
    const input = screen.getByLabelText('Name') as HTMLInputElement;

    await user.type(input, 'John Doe');
    expect(input.value).toBe('John Doe');
  });

  it('should call onChange handler', async () => {
    const handleChange = vi.fn();
    const user = userEvent.setup();

    render(<Input label="Email" onChange={handleChange} />);
    const input = screen.getByLabelText('Email');

    await user.type(input, 'test');
    expect(handleChange).toHaveBeenCalledTimes(4); // once per character
  });

  it('should be disabled when disabled prop is true', () => {
    render(<Input label="Disabled input" disabled />);
    const input = screen.getByLabelText('Disabled input');
    expect(input).toBeDisabled();
  });

  it('should apply fullWidth class', () => {
    const { container } = render(<Input label="Full width" fullWidth />);
    const wrapper = container.firstChild as HTMLElement;
    expect(wrapper).toHaveClass('w-full');
  });

  it('should support type attribute', () => {
    render(<Input label="Password" type="password" />);
    const input = screen.getByLabelText('Password');
    expect(input).toHaveAttribute('type', 'password');
  });

  it('should support required attribute', () => {
    render(<Input label="Required field" required />);
    const input = screen.getByLabelText('Required field');
    expect(input).toBeRequired();
  });

  it('should have proper accessibility attributes', () => {
    render(<Input label="Accessible input" error="Error message" />);
    const input = screen.getByLabelText('Accessible input');
    expect(input).toHaveAccessibleDescription();
  });

  it('should associate label with input via htmlFor', () => {
    render(<Input label="Associated input" id="test-input" />);
    const input = screen.getByLabelText('Associated input');
    expect(input).toHaveAttribute('id', 'test-input');
  });

  it('should support placeholder', () => {
    render(<Input label="Email" placeholder="example@domain.com" />);
    expect(screen.getByPlaceholderText('example@domain.com')).toBeInTheDocument();
  });

  it('should display error icon when error is present', () => {
    const { container } = render(<Input label="Field" error="Error occurred" />);
    // Error icon is AlertCircle from lucide-react
    expect(container.querySelector('svg')).toBeInTheDocument();
  });
});
