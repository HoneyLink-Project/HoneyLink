import { useAppStore } from '@/stores/appStore';
import {
    Activity,
    BarChart3,
    GitBranch,
    Shield,
    Smartphone,
} from 'lucide-react';
import { NavLink } from 'react-router-dom';

interface SidebarProps {
  isOpen: boolean;
}

/**
 * Sidebar navigation component
 * Implements navigation from spec/ui/overview.md
 */
export const Sidebar = ({ isOpen }: SidebarProps) => {
  const userRole = useAppStore((state) => state.userRole);

  const navItems = [
    {
      to: '/devices',
      icon: Smartphone,
      label: 'Devices',
      roles: ['end_user', 'admin', 'sre', 'developer'],
    },
    {
      to: '/streams',
      icon: GitBranch,
      label: 'Streams',
      roles: ['admin', 'sre'],
    },
    {
      to: '/policies',
      icon: Shield,
      label: 'Policies',
      roles: ['admin'],
    },
    {
      to: '/metrics',
      icon: BarChart3,
      label: 'Metrics',
      roles: ['admin', 'sre'],
    },
    {
      to: '/audit',
      icon: Activity,
      label: 'Audit',
      roles: ['admin', 'sre'],
    },
  ];

  // Filter nav items by user role
  const visibleNavItems = navItems.filter(
    (item) => !userRole || item.roles.includes(userRole)
  );

  return (
    <aside
      className={`fixed left-0 top-16 h-[calc(100vh-4rem)] border-r border-gray-200 bg-white transition-all duration-300 dark:border-gray-700 dark:bg-gray-800 ${
        isOpen ? 'w-64' : 'w-0'
      } overflow-hidden`}
    >
      <nav className="flex flex-col gap-1 p-4">
        {visibleNavItems.map((item) => {
          const Icon = item.icon;
          return (
            <NavLink
              key={item.to}
              to={item.to}
              className={({ isActive }) =>
                `flex items-center gap-3 rounded-lg px-4 py-3 text-sm font-medium transition-colors ${
                  isActive
                    ? 'bg-amber-100 text-amber-900 dark:bg-amber-900 dark:text-amber-100'
                    : 'text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700'
                }`
              }
            >
              <Icon className="h-5 w-5" />
              <span>{item.label}</span>
            </NavLink>
          );
        })}
      </nav>
    </aside>
  );
};
