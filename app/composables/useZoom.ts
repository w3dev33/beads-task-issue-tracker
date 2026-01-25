export function useZoom() {
  const zoomLevel = useLocalStorage('beads:zoomLevel', 120)
  const minZoom = 75
  const maxZoom = 150
  const step = 5

  const zoomIn = () => {
    if (zoomLevel.value < maxZoom) {
      zoomLevel.value = Math.min(zoomLevel.value + step, maxZoom)
      applyZoom()
    }
  }

  const zoomOut = () => {
    if (zoomLevel.value > minZoom) {
      zoomLevel.value = Math.max(zoomLevel.value - step, minZoom)
      applyZoom()
    }
  }

  const resetZoom = () => {
    zoomLevel.value = 100
    applyZoom()
  }

  const applyZoom = () => {
    if (import.meta.client) {
      // Clear any zoom on document element (cleanup from previous approach)
      document.documentElement.style.zoom = ''

      const zoomableContent = document.getElementById('zoomable-content')
      if (zoomableContent) {
        zoomableContent.style.zoom = `${zoomLevel.value}%`
      }
    }
  }

  const canZoomIn = computed(() => zoomLevel.value < maxZoom)
  const canZoomOut = computed(() => zoomLevel.value > minZoom)

  // Initialize on mount
  onMounted(() => {
    applyZoom()
  })

  return {
    zoomLevel,
    zoomIn,
    zoomOut,
    resetZoom,
    canZoomIn,
    canZoomOut,
  }
}
