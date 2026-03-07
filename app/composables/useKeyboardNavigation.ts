import { type ComputedRef, ref, watch, nextTick } from 'vue'

interface UseKeyboardNavigationOptions {
  itemIds: ComputedRef<string[]>
  onSelect?: (id: string) => void
  onAction?: (id: string) => void
  dataAttribute?: string
}

export function useKeyboardNavigation(options: UseKeyboardNavigationOptions) {
  const { itemIds, onSelect, onAction, dataAttribute = 'data-issue-id' } = options

  const focusedId = ref<string | null>(null)

  const setFocused = (id: string | null) => {
    focusedId.value = id
  }

  const isFocused = (id: string) => focusedId.value === id

  const scrollToFocused = (id: string) => {
    nextTick(() => {
      const escaped = typeof CSS !== 'undefined' && CSS.escape ? CSS.escape(id) : id
      const el = document.querySelector(`[${dataAttribute}="${escaped}"]`)
      el?.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
    })
  }

  const handleKeydown = (event: KeyboardEvent) => {
    // Don't interfere with typing in form elements
    const tag = (event.target as HTMLElement)?.tagName
    if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return

    const ids = itemIds.value
    if (ids.length === 0) return

    switch (event.key) {
      case 'ArrowDown': {
        event.preventDefault()
        if (focusedId.value === null) {
          focusedId.value = ids[0]!
        } else {
          const idx = ids.indexOf(focusedId.value)
          if (idx < ids.length - 1) {
            focusedId.value = ids[idx + 1]!
          }
        }
        scrollToFocused(focusedId.value!)
        break
      }
      case 'ArrowUp': {
        event.preventDefault()
        if (focusedId.value === null) {
          focusedId.value = ids[ids.length - 1]!
        } else {
          const idx = ids.indexOf(focusedId.value)
          if (idx > 0) {
            focusedId.value = ids[idx - 1]!
          }
        }
        scrollToFocused(focusedId.value!)
        break
      }
      case 'Enter': {
        if (focusedId.value && onSelect) {
          event.preventDefault()
          onSelect(focusedId.value)
        }
        break
      }
      case ' ': {
        if (focusedId.value && onAction) {
          event.preventDefault()
          onAction(focusedId.value)
        }
        break
      }
    }
  }

  // Revalidate focusedId when list changes
  watch(itemIds, (ids) => {
    if (focusedId.value && !ids.includes(focusedId.value)) {
      focusedId.value = ids.length > 0 ? ids[0]! : null
    }
  })

  return { focusedId, setFocused, handleKeydown, isFocused }
}
