/**
 * Unit tests for Select component
 * Tests options, onChange, error states, and accessibility
 */

import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Select } from './Select';

const mockOptions = [
  { value: '1', label: 'Option 1' },
  { value: '2', label: 'Option 2' },
  { value: '3', label: 'Option 3', disabled: true },
];

describe('Select', () => {
  it('should render with label', () => {
    render(<Select label="Choose option" options={mockOptions} />);
    expect(screen.getByLabelText('Choose option')).toBeInTheDocument();
  });

  it('should render all options', () => {
    render(<Select label="Select" options={mockOptions} />);
    expect(screen.getByRole('option', { name: 'Option 1' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Option 2' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Option 3' })).toBeInTheDocument();
  });

  it('should render placeholder option', () => {
    render(<Select label="Select" options={mockOptions} placeholder="Select an option" />);
    expect(screen.getByRole('option', { name: 'Select an option' })).toBeInTheDocument();
  });

  it('should handle option selection', async () => {
    const user = userEvent.setup();
    render(<Select label="Select" options={mockOptions} />);
    const select = screen.getByLabelText('Select') as HTMLSelectElement;

    await user.selectOptions(select, '2');
    expect(select.value).toBe('2');
  });

  it('should call onChange handler', async () => {
    const handleChange = vi.fn();
    const user = userEvent.setup();

    render(<Select label="Select" options={mockOptions} onChange={handleChange} />);
    const select = screen.getByLabelText('Select');

    await user.selectOptions(select, '1');
    expect(handleChange).toHaveBeenCalledTimes(1);
  });

  it('should display error message', () => {
    render(<Select label="Select" options={mockOptions} error="This field is required" />);
    expect(screen.getByText('This field is required')).toBeInTheDocument();
  });

  it('should display helper text', () => {
    render(<Select label="Select" options={mockOptions} helperText="Choose wisely" />);
    expect(screen.getByText('Choose wisely')).toBeInTheDocument();
  });

  it('should be disabled when disabled prop is true', () => {
    render(<Select label="Disabled select" options={mockOptions} disabled />);
    const select = screen.getByLabelText('Disabled select');
    expect(select).toBeDisabled();
  });

  it('should disable specific options', () => {
    render(<Select label="Select" options={mockOptions} />);
    const option3 = screen.getByRole('option', { name: 'Option 3' }) as HTMLOptionElement;
    expect(option3.disabled).toBe(true);
  });

  it('should apply fullWidth class', () => {
    const { container } = render(<Select label="Full width" options={mockOptions} fullWidth />);
    const wrapper = container.firstChild as HTMLElement;
    expect(wrapper).toHaveClass('w-full');
  });

  it('should support required attribute', () => {
    render(<Select label="Required field" options={mockOptions} required />);
    const select = screen.getByLabelText('Required field');
    expect(select).toBeRequired();
  });

  it('should associate label with select via htmlFor', () => {
    render(<Select label="Associated select" options={mockOptions} id="test-select" />);
    const select = screen.getByLabelText('Associated select');
    expect(select).toHaveAttribute('id', 'test-select');
  });

  it('should support default value', () => {
    render(<Select label="Select" options={mockOptions} defaultValue="2" />);
    const select = screen.getByLabelText('Select') as HTMLSelectElement;
    expect(select.value).toBe('2');
  });

  it('should render chevron icon', () => {
    const { container } = render(<Select label="Select" options={mockOptions} />);
    // ChevronDown icon from lucide-react
    expect(container.querySelector('svg')).toBeInTheDocument();
  });

  it('should handle empty options array', () => {
    render(<Select label="Empty select" options={[]} />);
    const select = screen.getByLabelText('Empty select');
    expect(select).toBeInTheDocument();
  });
});
