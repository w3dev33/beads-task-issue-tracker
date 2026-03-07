import { ref, watch } from 'vue'

// Global cache to share refs across composable instances
const cache = new Map<string, Ref<unknown>>()

export function useLocalStorage<T>(key: string, defaultValue: T) {
  // Return cached ref if already created
  if (cache.has(key)) {
    return cache.get(key) as Ref<T>
  }

  const storedValue = ref<T>(defaultValue) as Ref<T>

  // Load initial value from localStorage
  if (import.meta.client) {
    const stored = localStorage.getItem(key)
    if (stored) {
      try {
        storedValue.value = JSON.parse(stored)
      } catch {
        storedValue.value = defaultValue
      }
    }
  }

  // Watch for changes and persist to localStorage
  watch(
    storedValue,
    (newValue) => {
      if (import.meta.client) {
        localStorage.setItem(key, JSON.stringify(newValue))
      }
    },
    { deep: true }
  )

  // Cache the ref for future calls
  cache.set(key, storedValue as Ref<unknown>)

  return storedValue
}
