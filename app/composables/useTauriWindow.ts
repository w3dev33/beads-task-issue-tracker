let windowModule: typeof import('@tauri-apps/api/window') | null = null

// Pre-load the Tauri window module
if (import.meta.client) {
  import('@tauri-apps/api/window').then(mod => {
    windowModule = mod
  }).catch(() => {
    // Not in Tauri environment
  })
}

export function useTauriWindow() {
  const startDragging = () => {
    if (windowModule) {
      windowModule.getCurrentWindow().startDragging()
    }
  }

  return {
    startDragging,
  }
}
