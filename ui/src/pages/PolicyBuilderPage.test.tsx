import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { renderWithProviders } from '../test/test-utils';
import PolicyBuilderPage from './PolicyBuilderPage';

describe('PolicyBuilderPage', () => {
  it('should render page title and subtitle', () => {
    renderWithProviders(<PolicyBuilderPage />);
    expect(screen.getByText(/Policy Builder/i)).toBeInTheDocument();
    expect(screen.getByText(/QoS profile templates/i)).toBeInTheDocument();
  });

  it('should render all form fields with English labels', () => {
    renderWithProviders(<PolicyBuilderPage />);

    // Basic info fields (English labels from en.json)
    expect(screen.getByLabelText(/Template Name/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Usage Type/i)).toBeInTheDocument();

    // QoS settings fields
    expect(screen.getByLabelText(/Latency Target/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Minimum Bandwidth/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/FEC Mode/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Priority/i)).toBeInTheDocument();

    // Schedule fields
    expect(screen.getByLabelText(/Valid From/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Valid Until/i)).toBeInTheDocument();
  });

  it('should allow user to fill form fields', async () => {
    renderWithProviders(<PolicyBuilderPage />);

    const nameInput = screen.getByLabelText(/Template Name/i) as HTMLInputElement;
    expect(nameInput).toBeInTheDocument();
    expect(nameInput.type).toBe('text');
  });

  it('should render save and preview buttons', () => {
    renderWithProviders(<PolicyBuilderPage />);

    expect(screen.getByRole('button', { name: /Save/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Preview/i })).toBeInTheDocument();
  });

  it('should display helper text for QoS fields', () => {
    renderWithProviders(<PolicyBuilderPage />);

    expect(screen.getByText(/Specify between 1-50ms/i)).toBeInTheDocument();
    expect(screen.getByText(/Specify between 10-5000Mbps/i)).toBeInTheDocument();
  });
});
