<script setup lang="ts">
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog'
import { Label } from '~/components/ui/label'
import { Button } from '~/components/ui/button'
import { getCliBinaryPath, setCliBinaryPath, checkExternalHealth } from '~/utils/bd-api'
import type { ThemeDefinition } from '~/composables/useTheme'

const open = defineModel<boolean>('open', { default: false })

const { theme: activeTheme, themes, setTheme } = useTheme()

// SVG icons for theme cards
const themeIconPaths: Record<string, string> = {
  sun: 'M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42',
  moon: 'M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z',
  square: '', // uses rect element instead
  zap: 'M13 2L3 14h9l-1 10 10-12h-9l1-10z',
}
// Sun needs a separate circle
const sunCircle = { cx: 12, cy: 12, r: 5 }

const selectedClient = ref<'bd' | 'br'>('bd')
const isSwitching = ref(false)
const switchResult = ref<{ success: boolean; message: string } | null>(null)

// Probe settings (dev-only — hidden in production until probe is a public feature)
const isDev = import.meta.dev
const probeEnabled = useLocalStorage('beads:probeEnabled', false)
const dataSourceUrl = useLocalStorage('beads:dataSourceUrl', 'http://localhost:9100')
const isTesting = ref(false)
const healthResult = ref<boolean | null>(null)

// Load current setting when dialog opens
watch(open, async (isOpen) => {
  if (isOpen) {
    try {
      const current = await getCliBinaryPath()
      selectedClient.value = current === 'br' ? 'br' : 'bd'
      switchResult.value = null
    } catch {
      selectedClient.value = 'bd'
    }
    healthResult.value = null
  }
})

async function selectClient(client: 'bd' | 'br') {
  if (client === selectedClient.value) return

  isSwitching.value = true
  switchResult.value = null
  try {
    const version = await setCliBinaryPath(client)
    selectedClient.value = client
    switchResult.value = { success: true, message: version }
  } catch (error) {
    switchResult.value = {
      success: false,
      message: error instanceof Error ? error.message : String(error),
    }
  } finally {
    isSwitching.value = false
  }
}

async function testConnection() {
  isTesting.value = true
  healthResult.value = null
  try {
    healthResult.value = await checkExternalHealth(dataSourceUrl.value)
  } catch {
    healthResult.value = false
  } finally {
    isTesting.value = false
  }
}
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="sm:max-w-lg">
      <DialogHeader>
        <DialogTitle>Settings</DialogTitle>
        <DialogDescription>
          Choose which CLI client to use for issue management.
        </DialogDescription>
      </DialogHeader>

      <div class="space-y-6 pt-2">
        <!-- Theme Selector -->
        <div class="space-y-3">
          <Label>Theme</Label>
          <div class="grid grid-cols-4 gap-3">
            <button
              v-for="t in themes"
              :key="t.id"
              class="relative flex flex-col items-center gap-1.5 rounded-lg border-2 p-3 text-center transition-colors"
              :class="activeTheme === t.id
                ? 'border-primary bg-primary/5'
                : 'border-muted hover:border-muted-foreground/25 hover:bg-muted/50'"
              @click="setTheme(t.id)"
            >
              <div class="flex items-center justify-center h-8 w-8">
                <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <circle v-if="t.icon === 'sun'" v-bind="sunCircle" />
                  <rect v-if="t.icon === 'square'" x="3" y="3" width="18" height="18" rx="2" />
                  <path v-if="themeIconPaths[t.icon]" :d="themeIconPaths[t.icon]" />
                </svg>
              </div>
              <span class="text-xs font-medium">{{ t.label }}</span>
              <div
                class="absolute top-1.5 right-1.5 h-2 w-2 rounded-full transition-colors"
                :class="activeTheme === t.id ? 'bg-primary' : 'bg-transparent'"
              />
            </button>
          </div>
        </div>

        <!-- CLI Client Selector -->
        <div class="space-y-3">
          <Label>CLI Client</Label>
          <div class="grid grid-cols-2 gap-3">
            <!-- bd option -->
            <button
              class="relative flex flex-col items-start gap-1.5 rounded-lg border-2 p-3 text-left transition-colors"
              :class="selectedClient === 'bd'
                ? 'border-primary bg-primary/5'
                : 'border-muted hover:border-muted-foreground/25 hover:bg-muted/50'"
              :disabled="isSwitching"
              @click="selectClient('bd')"
            >
              <div class="flex items-center gap-2">
                <div
                  class="flex h-5 w-5 items-center justify-center rounded-full border-2 transition-colors"
                  :class="selectedClient === 'bd' ? 'border-primary' : 'border-muted-foreground/40'"
                >
                  <div
                    v-if="selectedClient === 'bd'"
                    class="h-2.5 w-2.5 rounded-full bg-primary"
                  />
                </div>
                <span class="font-mono font-semibold text-sm">bd</span>
              </div>
              <p class="text-xs text-muted-foreground pl-7">
                Original Beads CLI (Go)
              </p>
            </button>

            <!-- br option -->
            <button
              class="relative flex flex-col items-start gap-1.5 rounded-lg border-2 p-3 text-left transition-colors"
              :class="selectedClient === 'br'
                ? 'border-primary bg-primary/5'
                : 'border-muted hover:border-muted-foreground/25 hover:bg-muted/50'"
              :disabled="isSwitching"
              @click="selectClient('br')"
            >
              <div class="flex items-center gap-2">
                <div
                  class="flex h-5 w-5 items-center justify-center rounded-full border-2 transition-colors"
                  :class="selectedClient === 'br' ? 'border-primary' : 'border-muted-foreground/40'"
                >
                  <div
                    v-if="selectedClient === 'br'"
                    class="h-2.5 w-2.5 rounded-full bg-primary"
                  />
                </div>
                <span class="font-mono font-semibold text-sm">br</span>
              </div>
              <p class="text-xs text-muted-foreground pl-7">
                Beads Rust (SQLite + JSONL)
              </p>
            </button>
          </div>
        </div>

        <!-- Probe Toggle (dev-only until probe is a public feature) -->
        <div v-if="isDev" class="space-y-3">
          <div class="flex items-center justify-between">
            <Label>Probe (monitoring broadcast)</Label>
            <button
              class="relative inline-flex h-5 w-9 items-center rounded-full transition-colors"
              :class="probeEnabled ? 'bg-primary' : 'bg-muted-foreground/30'"
              @click="probeEnabled = !probeEnabled; healthResult = null"
            >
              <span
                class="inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform"
                :class="probeEnabled ? 'translate-x-4.5' : 'translate-x-0.5'"
              />
            </button>
          </div>
          <p class="text-xs text-muted-foreground">
            When enabled, registers projects with the probe for external monitoring.
          </p>

          <!-- URL input + Test connection (visible only when probe enabled) -->
          <div v-if="probeEnabled" class="space-y-2">
            <div class="flex gap-2">
              <input
                v-model="dataSourceUrl"
                type="text"
                placeholder="http://localhost:9100"
                class="flex-1 rounded-md border border-input bg-background px-3 py-1.5 text-sm font-mono shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
              />
              <Button
                size="sm"
                variant="outline"
                :disabled="isTesting"
                @click="testConnection"
              >
                <svg v-if="isTesting" class="animate-spin h-3 w-3 mr-1" viewBox="0 0 24 24" fill="none">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
                Test connection
              </Button>
            </div>

            <!-- Health check result -->
            <div v-if="healthResult !== null" class="flex items-center gap-1.5 text-xs">
              <svg v-if="healthResult" class="w-3.5 h-3.5 text-green-600 dark:text-green-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="20 6 9 17 4 12" />
              </svg>
              <svg v-else class="w-3.5 h-3.5 text-destructive" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10" />
                <line x1="15" y1="9" x2="9" y2="15" />
                <line x1="9" y1="9" x2="15" y2="15" />
              </svg>
              <span :class="healthResult ? 'text-green-600 dark:text-green-400' : 'text-destructive'">
                {{ healthResult ? 'Connected' : 'Disconnected' }}
              </span>
            </div>
          </div>
        </div>

        <!-- Switching spinner -->
        <div v-if="isSwitching" class="flex items-center gap-2 text-sm text-muted-foreground">
          <svg class="animate-spin h-3 w-3" viewBox="0 0 24 24" fill="none">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
          </svg>
          Switching client...
        </div>

        <!-- Result -->
        <div v-if="switchResult" class="flex items-start gap-2 p-2 rounded-md text-sm" :class="switchResult.success ? 'bg-green-500/10 text-green-600 dark:text-green-400' : 'bg-destructive/10 text-destructive'">
          <svg v-if="switchResult.success" class="w-4 h-4 shrink-0 mt-0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20 6 9 17 4 12" />
          </svg>
          <svg v-else class="w-4 h-4 shrink-0 mt-0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" />
            <line x1="15" y1="9" x2="9" y2="15" />
            <line x1="9" y1="9" x2="15" y2="15" />
          </svg>
          <span class="font-mono text-xs break-all">{{ switchResult.message }}</span>
        </div>
      </div>
    </DialogContent>
  </Dialog>
</template>
