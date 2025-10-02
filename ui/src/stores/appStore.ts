import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';

/**
 * Global application state store using Zustand
 * Handles theme, user preferences, and UI state
 */

export interface AppState {
  // Theme
  theme: 'light' | 'dark';
  toggleTheme: () => void;

  // User preferences
  locale: 'en' | 'ja' | 'es' | 'zh';
  setLocale: (locale: 'en' | 'ja' | 'es' | 'zh') => void;

  // UI state
  sidebarOpen: boolean;
  toggleSidebar: () => void;

  // Current user role (from JWT claims)
  userRole: 'end_user' | 'admin' | 'sre' | 'developer' | null;
  setUserRole: (role: AppState['userRole']) => void;
}

export const useAppStore = create<AppState>()(
  devtools(
    persist(
      (set) => ({
        // Theme state
        theme: 'light',
        toggleTheme: () =>
          set((state) => ({ theme: state.theme === 'light' ? 'dark' : 'light' })),

        // Locale state
        locale: 'en',
        setLocale: (locale) => set({ locale }),

        // UI state
        sidebarOpen: true,
        toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),

        // User role
        userRole: null,
        setUserRole: (role) => set({ userRole: role }),
      }),
      {
        name: 'honeylink-app-store',
        partialize: (state) => ({
          theme: state.theme,
          locale: state.locale,
          sidebarOpen: state.sidebarOpen,
        }),
      }
    ),
    { name: 'AppStore' }
  )
);
