// Sidebar resize logic — extracted from index.vue
// Singleton pattern: refs at module level, shared across all callers

const isLeftSidebarOpen = useLocalStorage('beads:leftSidebar', true)
const isRightSidebarOpen = useLocalStorage('beads:rightSidebar', true)
const leftSidebarWidth = useLocalStorage('beads:leftSidebarWidth', 360)
const rightSidebarWidth = useLocalStorage('beads:rightSidebarWidth', 484)

const isResizingLeft = ref(false)
const isResizingRight = ref(false)
const startX = ref(0)
const startWidth = ref(0)

const onResizeLeft = (e: MouseEvent) => {
  if (!isResizingLeft.value) return
  const diff = e.clientX - startX.value
  const newWidth = Math.min(Math.max(startWidth.value + diff, 280), 500)
  leftSidebarWidth.value = newWidth
}

const stopResizeLeft = () => {
  isResizingLeft.value = false
  document.removeEventListener('mousemove', onResizeLeft)
  document.removeEventListener('mouseup', stopResizeLeft)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
  document.body.style.webkitUserSelect = ''
}

const startResizeLeft = (e: MouseEvent) => {
  e.preventDefault()
  window.getSelection()?.removeAllRanges()
  isResizingLeft.value = true
  startX.value = e.clientX
  startWidth.value = leftSidebarWidth.value
  document.addEventListener('mousemove', onResizeLeft)
  document.addEventListener('mouseup', stopResizeLeft)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  document.body.style.webkitUserSelect = 'none'
}

const onResizeRight = (e: MouseEvent) => {
  if (!isResizingRight.value) return
  const diff = startX.value - e.clientX
  const newWidth = Math.min(Math.max(startWidth.value + diff, 300), 800)
  rightSidebarWidth.value = newWidth
}

const stopResizeRight = () => {
  isResizingRight.value = false
  document.removeEventListener('mousemove', onResizeRight)
  document.removeEventListener('mouseup', stopResizeRight)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
  document.body.style.webkitUserSelect = ''
}

const startResizeRight = (e: MouseEvent) => {
  e.preventDefault()
  window.getSelection()?.removeAllRanges()
  isResizingRight.value = true
  startX.value = e.clientX
  startWidth.value = rightSidebarWidth.value
  document.addEventListener('mousemove', onResizeRight)
  document.addEventListener('mouseup', stopResizeRight)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  document.body.style.webkitUserSelect = 'none'
}

const isResizing = computed(() => isResizingLeft.value || isResizingRight.value)

export function useSidebarResize() {
  return {
    isLeftSidebarOpen,
    isRightSidebarOpen,
    leftSidebarWidth: readonly(leftSidebarWidth),
    rightSidebarWidth: readonly(rightSidebarWidth),
    isResizing: readonly(isResizing),
    startResizeLeft,
    startResizeRight,
  }
}
