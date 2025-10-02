import axios, { AxiosError, AxiosInstance, InternalAxiosRequestConfig } from 'axios';

/**
 * API client configuration for HoneyLink Control Plane API
 * Implements authentication, error handling, and tracing context
 */

// API base URL from environment variable
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000/api/v1';

// JWT token storage key
const TOKEN_STORAGE_KEY = 'honeylink_auth_token';

/**
 * Error response from API (matches backend/src/error.rs)
 */
export interface ApiErrorResponse {
  error_code: string;
  message: string;
  trace_id?: string;
}

/**
 * Custom API error class
 */
export class ApiError extends Error {
  constructor(
    public statusCode: number,
    public errorCode: string,
    public traceId?: string,
    message?: string
  ) {
    super(message || errorCode);
    this.name = 'ApiError';
  }
}

/**
 * Create axios instance with default configuration
 */
const createApiClient = (): AxiosInstance => {
  const client = axios.create({
    baseURL: API_BASE_URL,
    timeout: 30000,
    headers: {
      'Content-Type': 'application/json',
    },
  });

  // Request interceptor: Add auth token and trace context
  client.interceptors.request.use(
    (config: InternalAxiosRequestConfig) => {
      // Add JWT token from localStorage
      const token = localStorage.getItem(TOKEN_STORAGE_KEY);
      if (token && config.headers) {
        config.headers.Authorization = `Bearer ${token}`;
      }

      // Add W3C traceparent header (simplified version)
      const traceId = generateTraceId();
      if (config.headers) {
        config.headers.traceparent = `00-${traceId}-${generateSpanId()}-01`;
      }

      return config;
    },
    (error) => Promise.reject(error)
  );

  // Response interceptor: Handle errors
  client.interceptors.response.use(
    (response) => response,
    (error: AxiosError<ApiErrorResponse>) => {
      if (error.response?.data) {
        const { error_code, message, trace_id } = error.response.data;
        throw new ApiError(
          error.response.status,
          error_code,
          trace_id,
          message
        );
      }
      throw error;
    }
  );

  return client;
};

/**
 * Generate random trace ID (32 hex characters)
 */
const generateTraceId = (): string => {
  return Array.from({ length: 32 }, () =>
    Math.floor(Math.random() * 16).toString(16)
  ).join('');
};

/**
 * Generate random span ID (16 hex characters)
 */
const generateSpanId = (): string => {
  return Array.from({ length: 16 }, () =>
    Math.floor(Math.random() * 16).toString(16)
  ).join('');
};

/**
 * Store JWT token
 */
export const setAuthToken = (token: string): void => {
  localStorage.setItem(TOKEN_STORAGE_KEY, token);
};

/**
 * Clear JWT token
 */
export const clearAuthToken = (): void => {
  localStorage.removeItem(TOKEN_STORAGE_KEY);
};

/**
 * Get current JWT token
 */
export const getAuthToken = (): string | null => {
  return localStorage.getItem(TOKEN_STORAGE_KEY);
};

/**
 * Singleton API client instance
 */
export const apiClient = createApiClient();
