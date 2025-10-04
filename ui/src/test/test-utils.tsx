/**
 * Test utilities for wrapping components with providers
 * Provides React Query and i18n context for testing
 */

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, RenderOptions } from '@testing-library/react';
import { ReactElement, ReactNode } from 'react';
import { I18nextProvider } from 'react-i18next';
import { BrowserRouter } from 'react-router-dom';
import i18n from '../i18n';

/**
 * Create a new QueryClient for each test to ensure isolation
 * Disables retries and logging for faster, quieter tests
 */
export function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false, // Disable retries in tests
        gcTime: Infinity, // Never garbage collect cache during tests
      },
      mutations: {
        retry: false,
      },
    },
  });
}

/**
 * All providers wrapper for component testing
 * Includes React Query, i18n, and Router context
 */
interface AllProvidersProps {
  children: ReactNode;
  queryClient?: QueryClient;
}

export function AllProviders({ children, queryClient }: AllProvidersProps) {
  const testQueryClient = queryClient || createTestQueryClient();

  return (
    <QueryClientProvider client={testQueryClient}>
      <I18nextProvider i18n={i18n}>
        <BrowserRouter>{children}</BrowserRouter>
      </I18nextProvider>
    </QueryClientProvider>
  );
}

/**
 * Custom render function that wraps components with all necessary providers
 * Usage: renderWithProviders(<MyComponent />)
 */
interface CustomRenderOptions extends Omit<RenderOptions, 'wrapper'> {
  queryClient?: QueryClient;
}

export function renderWithProviders(
  ui: ReactElement,
  { queryClient, ...renderOptions }: CustomRenderOptions = {}
) {
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <AllProviders queryClient={queryClient}>{children}</AllProviders>
  );

  return render(ui, { wrapper: Wrapper, ...renderOptions });
}

/**
 * Wait for async React Query operations to complete
 * Useful for testing hooks that trigger queries
 */
export async function waitForQueryToSettle(queryClient: QueryClient) {
  await queryClient.refetchQueries();
  // Small delay to let React update
  await new Promise((resolve) => setTimeout(resolve, 0));
}

// Re-export everything from testing-library
export * from '@testing-library/react';
export { default as userEvent } from '@testing-library/user-event';
