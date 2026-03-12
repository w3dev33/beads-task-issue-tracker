let windowModule: typeof import('@tauri-apps/api/window') | null = null
let setTitlePermissionDeniedLogged = false

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
      windowModule.getCurrentWindow().startDragging().catch(() => {
        // Ignore drag failures in unsupported environments.
      })
    }
  }

  const setWindowTitle = (title: string) => {
    if (windowModule) {
      windowModule.getCurrentWindow().setTitle(title).catch((error) => {
        // Some capability profiles may deny changing title; do not break app render.
        if (!setTitlePermissionDeniedLogged) {
          setTitlePermissionDeniedLogged = true
          console.warn('Unable to set window title:', error)
        }
      })
    }
  }

  return {
    startDragging,
    setWindowTitle,
  }
}
