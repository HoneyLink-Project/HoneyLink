/**
 * Axios HTTP client for Control Plane API
 * 
 * Base configuration:
 * - Base URL: http://localhost:3000 (dev), configurable via env
 * - JWT authentication: Authorization header
 * - W3C Trace Context: traceparent header
 * - Request/Response interceptors for logging and error handling
 */

import axios, { AxiosError } from 'axios';

// ============================================================================
// Environment Configuration
// ============================================================================

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000';

// ============================================================================
// Axios Instance Creation
// ============================================================================

export const apiClient = axios.create({
  baseURL: API_BASE_URL,
  timeout: 10000, // 10s timeout
  headers: {
    'Content-Type': 'application/json',
  },
});

// ============================================================================
// Request Interceptor (JWT + Trace Context)
// ============================================================================

apiClient.interceptors.request.use(
  (config) => {
    // JWT token (from localStorage or auth store)
    const token = localStorage.getItem('authToken');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }

    // W3C Trace Context (traceparent header)
    // Format: 00-<trace-id>-<span-id>-<flags>
    const traceId = generateTraceId();
    const spanId = generateSpanId();
    config.headers['traceparent'] = `00-${traceId}-${spanId}-01`;

    return config;
  },
  (error) => {
    console.error('[API Request Error]', error);
    return Promise.reject(error);
  }
);

// ============================================================================
// Response Interceptor (Error Handling)
// ============================================================================

apiClient.interceptors.response.use(
  (response) => response,
  (error: AxiosError) => {
    // Log error details
    console.error('[API Response Error]', {
      status: error.response?.status,
      statusText: error.response?.statusText,
      data: error.response?.data,
      url: error.config?.url,
    });

    // Handle specific error codes
    if (error.response?.status === 401) {
      // Unauthorized: Clear token and redirect to login
      localStorage.removeItem('authToken');
      // TODO Part 4: Redirect to login page via router
      console.warn('[API] 401 Unauthorized - Token cleared');
    }

    if (error.response?.status === 403) {
      // Forbidden: Insufficient permissions
      console.warn('[API] 403 Forbidden - Access denied');
    }

    if (error.response?.status === 500) {
      // Server error: Show toast notification (Part 4)
      console.error('[API] 500 Internal Server Error');
    }

    return Promise.reject(error);
  }
);

// ============================================================================
// Helper Functions (Trace Context Generation)
// ============================================================================

/**
 * Generate random trace ID (32 hex chars)
 * Format: 128-bit identifier
 */
function generateTraceId(): string {
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  return Array.from(bytes)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
}

/**
 * Generate random span ID (16 hex chars)
 * Format: 64-bit identifier
 */
function generateSpanId(): string {
  const bytes = new Uint8Array(8);
  crypto.getRandomValues(bytes);
  return Array.from(bytes)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
}

// ============================================================================
// Export Types (for consumer convenience)
// ============================================================================

export type { AxiosError } from 'axios';
