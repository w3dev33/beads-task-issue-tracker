import { setBackendMode, trackerDetect, trackerInit } from '~/utils/bd-api'
import { useProjectStorage } from '~/composables/useProjectStorage'

export function useBackendMode() {
  const backendMode = useProjectStorage<string>('backendMode', 'br')

  async function setMode(mode: string) {
    await setBackendMode(mode)
    backendMode.value = mode
  }

  async function syncFromStorage() {
    await setBackendMode(backendMode.value)
  }

  async function ensureTrackerInit(cwd?: string) {
    const exists = await trackerDetect(cwd)
    if (!exists) {
      await trackerInit(cwd)
    }
  }

  return { backendMode, setMode, syncFromStorage, ensureTrackerInit }
}
