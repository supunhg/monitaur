import { create } from 'zustand'

interface AppStore {
  sidebarOpen: boolean
  darkMode: boolean
  toggleSidebar: () => void
  toggleDarkMode: () => void
}

export const useAppStore = create<AppStore>((set) => ({
  sidebarOpen: true,
  darkMode: true,
  toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),
  toggleDarkMode: () => set((s) => ({ darkMode: !s.darkMode })),
}))
