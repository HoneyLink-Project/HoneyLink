import { useAppStore } from '@/stores/appStore';
import { Menu, Moon, Sun } from 'lucide-react';

/**
 * Header component with logo, theme toggle, and sidebar toggle
 */
export const Header = () => {
  const { theme, toggleTheme, toggleSidebar } = useAppStore();

  return (
    <header className="sticky top-0 z-50 border-b border-gray-200 bg-white dark:border-gray-700 dark:bg-gray-800">
      <div className="flex h-16 items-center justify-between px-4">
        {/* Left section: Sidebar toggle + Logo */}
        <div className="flex items-center gap-4">
          <button
            onClick={toggleSidebar}
            className="rounded-lg p-2 hover:bg-gray-100 dark:hover:bg-gray-700"
            aria-label="Toggle sidebar"
          >
            <Menu className="h-5 w-5" />
          </button>

          <div className="flex items-center gap-2">
            <div className="h-8 w-8 rounded-lg bg-amber-500" />
            <span className="text-xl font-bold text-gray-900 dark:text-white">
              HoneyLink
            </span>
          </div>
        </div>

        {/* Right section: Theme toggle */}
        <div className="flex items-center gap-2">
          <button
            onClick={toggleTheme}
            className="rounded-lg p-2 hover:bg-gray-100 dark:hover:bg-gray-700"
            aria-label={`Switch to ${theme === 'light' ? 'dark' : 'light'} mode`}
          >
            {theme === 'light' ? (
              <Moon className="h-5 w-5" />
            ) : (
              <Sun className="h-5 w-5" />
            )}
          </button>
        </div>
      </div>
    </header>
  );
};
