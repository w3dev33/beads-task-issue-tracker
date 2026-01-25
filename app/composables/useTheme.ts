export function useTheme() {
  const isDark = useLocalStorage('beads:darkMode', true)

  const toggleTheme = () => {
    isDark.value = !isDark.value
    updateHtmlClass()
  }

  const updateHtmlClass = () => {
    if (import.meta.client) {
      if (isDark.value) {
        document.documentElement.classList.add('dark')
      } else {
        document.documentElement.classList.remove('dark')
      }
    }
  }

  // Initialize on mount
  onMounted(() => {
    updateHtmlClass()
  })

  return {
    isDark,
    toggleTheme,
  }
}
