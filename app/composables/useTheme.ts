export interface ThemeDefinition {
  id: string
  label: string
  icon: 'sun' | 'moon' | 'square' | 'zap'
  baseMode: 'light' | 'dark'
  showBadgeIcons: boolean
}

const THEMES: ThemeDefinition[] = [
  { id: 'light', label: 'Classic Light', icon: 'sun', baseMode: 'light', showBadgeIcons: false },
  { id: 'dark', label: 'Classic Dark', icon: 'moon', baseMode: 'dark', showBadgeIcons: false },
  { id: 'flat', label: 'Dark Flat', icon: 'square', baseMode: 'dark', showBadgeIcons: false },
  { id: 'neon', label: 'Neon', icon: 'zap', baseMode: 'dark', showBadgeIcons: false },
]

export function useTheme() {
  const theme = useLocalStorage('beads:theme', 'dark')

  // Migration from old localStorage key
  if (import.meta.client && !localStorage.getItem('beads:theme')) {
    const oldDarkMode = localStorage.getItem('beads:darkMode')
    if (oldDarkMode !== null) {
      theme.value = oldDarkMode === 'true' ? 'dark' : 'light'
      localStorage.removeItem('beads:darkMode')
    }
  }

  const themes = THEMES

  const currentTheme = computed((): ThemeDefinition => {
    const found = THEMES.find(t => t.id === theme.value)
    return found ?? THEMES[1]! // fallback to dark theme
  })

  const isDark = computed(() => currentTheme.value.baseMode === 'dark')

  const showBadgeIcons = computed(() => currentTheme.value.showBadgeIcons)

  const setTheme = (id: string) => {
    theme.value = id
    updateHtmlClass()
  }

  const cycleTheme = () => {
    const currentIndex = THEMES.findIndex(t => t.id === theme.value)
    const nextIndex = (currentIndex + 1) % THEMES.length
    setTheme(THEMES[nextIndex]!.id)
  }

  const updateHtmlClass = () => {
    if (import.meta.client) {
      document.documentElement.classList.toggle('dark', isDark.value)
      document.documentElement.setAttribute('data-theme', theme.value)
    }
  }

  // Initialize on mount
  onMounted(() => {
    updateHtmlClass()
  })

  return {
    theme,
    themes,
    currentTheme,
    isDark,
    showBadgeIcons,
    setTheme,
    cycleTheme,
  }
}
