import { lazy, Suspense } from 'react';
import { createBrowserRouter, Navigate } from 'react-router-dom';
import { Layout } from './components/layout/Layout';
import { DeviceListPage } from './pages/DeviceListPage';
import { NotFoundPage } from './pages/NotFoundPage';
import { PairingDetailsPage } from './pages/PairingDetailsPage';
import PolicyBuilderPage from './pages/PolicyBuilderPage';

// Code splitting: Lazy load chart-heavy screens
// These pages use recharts (~50 kB), so they are split into separate chunks
const StreamDashboardPage = lazy(() => import('./pages/StreamDashboardPage'));
const MetricsHubPage = lazy(() => import('./pages/MetricsHubPage'));

/**
 * Loading fallback component for lazy-loaded routes
 */
const SuspenseFallback = () => (
  <div className="flex items-center justify-center h-64">
    <div className="text-center">
      <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-primary mb-3"></div>
      <div className="text-sm text-text-secondary">Loading...</div>
    </div>
  </div>
);

/**
 * React Router configuration for HoneyLink UI
 * Implements routing structure from spec/ui/wireframes.md
 *
 * Code Splitting Strategy:
 * - WF-01/02/04: Eagerly loaded (no charts, smaller bundle)
 * - WF-03/05: Lazy loaded (recharts dependency, ~50 kB each)
 */
export const router = createBrowserRouter([
  {
    path: '/',
    element: <Layout />,
    children: [
      {
        index: true,
        element: <Navigate to="/devices" replace />,
      },
      {
        path: 'devices',
        element: <DeviceListPage />,
      },
      {
        path: 'devices/:deviceId/pair',
        element: <PairingDetailsPage />,
      },
      {
        path: 'streams',
        element: (
          <Suspense fallback={<SuspenseFallback />}>
            <StreamDashboardPage />
          </Suspense>
        ),
      },
      {
        path: 'policies',
        element: <PolicyBuilderPage />,
      },
      {
        path: 'metrics',
        element: (
          <Suspense fallback={<SuspenseFallback />}>
            <MetricsHubPage />
          </Suspense>
        ),
      },
      {
        path: '*',
        element: <NotFoundPage />,
      },
    ],
  },
]);
