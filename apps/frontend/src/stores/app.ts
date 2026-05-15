import { create } from 'zustand'

interface AppStore {
  sidebarOpen: boolean
  darkMode: boolean
  toggleSidebar: () => void
  toggleDarkMode: () => void
}

function applyDarkMode(dark: boolean) {
  document.documentElement.classList.toggle('dark', dark)
  document.documentElement.classList.toggle('light', !dark)
}

// Init from system preference or default to dark
const initialDark = window.matchMedia('(prefers-color-scheme: dark)').matches ?? true
applyDarkMode(initialDark)

export const useAppStore = create<AppStore>((set) => ({
  sidebarOpen: true,
  darkMode: initialDark,
  toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),
  toggleDarkMode: () =>
    set((s) => {
      const next = !s.darkMode
      applyDarkMode(next)
      return { darkMode: next }
    }),
}))
