import { useAppStore } from '@/stores/appStore';
import { Outlet } from 'react-router-dom';
import { Header } from './Header';
import { Sidebar } from './Sidebar';

/**
 * Main layout component with header, sidebar, and content area
 * Implements responsive layout with collapsible sidebar
 */
export const Layout = () => {
  const sidebarOpen = useAppStore((state) => state.sidebarOpen);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <Header />

      <div className="flex">
        <Sidebar isOpen={sidebarOpen} />

        <main
          className={`flex-1 transition-all duration-300 ${
            sidebarOpen ? 'ml-64' : 'ml-0'
          }`}
        >
          <div className="container mx-auto px-4 py-6">
            <Outlet />
          </div>
        </main>
      </div>
    </div>
  );
};
