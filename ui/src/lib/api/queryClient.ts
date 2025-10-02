import { QueryClient } from '@tanstack/react-query';
import { ApiError } from './client';

/**
 * TanStack Query client configuration
 * Implements retry logic and error handling
 */

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // Retry configuration
      retry: (failureCount, error) => {
        // Don't retry on 4xx errors (client errors)
        if (error instanceof ApiError && error.statusCode >= 400 && error.statusCode < 500) {
          return false;
        }
        // Retry up to 3 times for 5xx errors
        return failureCount < 3;
      },
      retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),

      // Stale time: 5 minutes
      staleTime: 5 * 60 * 1000,

      // Cache time: 10 minutes
      gcTime: 10 * 60 * 1000,

      // Refetch on window focus
      refetchOnWindowFocus: true,

      // Refetch on reconnect
      refetchOnReconnect: true,
    },
    mutations: {
      // Retry mutations once for 5xx errors only
      retry: (failureCount, error) => {
        if (error instanceof ApiError && error.statusCode >= 500) {
          return failureCount < 1;
        }
        return false;
      },
    },
  },
});
