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
import { setCliBinaryPath, checkExternalHealth, trackerCheckBeadsSource, trackerMigrateFromBeads } from '~/utils/bd-api'
import type { BeadsSourceInfo, TrackerMigrationResult } from '~/utils/bd-api'
import { useBackendMode } from '~/composables/useBackendMode'

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

// Backend mode (also drives CLI client selection for br/bd)
const { backendMode, setMode: setBackendModeValue, ensureTrackerInit } = useBackendMode()
const isSwitchingBackend = ref(false)
const backendWarning = ref<string | null>(null)
const backendResult = ref<{ success: boolean; message: string } | null>(null)

async function selectBackend(mode: string) {
  if (mode === backendMode.value) return

  isSwitchingBackend.value = true
  backendWarning.value = null
  backendResult.value = null
  try {
    if (mode === 'built-in') {
      await ensureTrackerInit()
    } else if (backendMode.value === 'built-in') {
      backendWarning.value = 'Issues in .tracker/ won\'t be visible with this backend'
    }

    // For br/bd modes, also update the CLI binary path
    if (mode === 'br' || mode === 'bd') {
      try {
        const version = await setCliBinaryPath(mode)
        const { setBinary } = useCliClient()
        setBinary(mode)
        backendResult.value = { success: true, message: version }
      } catch (error) {
        backendResult.value = {
          success: false,
          message: error instanceof Error ? error.message : String(error),
        }
      }
    }

    await setBackendModeValue(mode)
  } catch (error) {
    backendWarning.value = error instanceof Error ? error.message : String(error)
  } finally {
    isSwitchingBackend.value = false
  }
}

// Migration from .beads/ to .tracker/
const beadsSource = ref<BeadsSourceInfo | null>(null)
const isMigrating = ref(false)
const migrationResult = ref<TrackerMigrationResult | null>(null)
const migrationError = ref<string | null>(null)

async function checkBeadsSource() {
  try {
    beadsSource.value = await trackerCheckBeadsSource()
  } catch {
    beadsSource.value = null
  }
}

async function runMigration() {
  isMigrating.value = true
  migrationResult.value = null
  migrationError.value = null
  try {
    migrationResult.value = await trackerMigrateFromBeads()
  } catch (error) {
    migrationError.value = error instanceof Error ? error.message : String(error)
  } finally {
    isMigrating.value = false
  }
}

// Probe settings (dev-only — hidden in production until probe is a public feature)
const isDev = import.meta.dev
const probeEnabled = useLocalStorage('beads:probeEnabled', false)
const dataSourceUrl = useLocalStorage('beads:dataSourceUrl', 'http://localhost:9100')
const isTesting = ref(false)
const healthResult = ref<boolean | null>(null)

// Reset results when dialog opens
watch(open, (isOpen) => {
  if (isOpen) {
    backendResult.value = null
    backendWarning.value = null
    healthResult.value = null
    migrationResult.value = null
    migrationError.value = null
    checkBeadsSource()
  }
})

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
          Configure theme and backend engine for issue management.
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

        <!-- Backend Selector -->
        <div class="space-y-3">
          <Label>Backend Engine</Label>
          <div class="grid grid-cols-3 gap-3">
            <!-- Built-in option -->
            <button
              class="relative flex flex-col items-start gap-1.5 rounded-lg border-2 p-3 text-left transition-colors"
              :class="backendMode === 'built-in'
                ? 'border-primary bg-primary/5'
                : 'border-muted hover:border-muted-foreground/25 hover:bg-muted/50'"
              :disabled="isSwitchingBackend"
              @click="selectBackend('built-in')"
            >
              <div class="flex items-center gap-2">
                <div
                  class="flex h-5 w-5 items-center justify-center rounded-full border-2 transition-colors"
                  :class="backendMode === 'built-in' ? 'border-primary' : 'border-muted-foreground/40'"
                >
                  <div
                    v-if="backendMode === 'built-in'"
                    class="h-2.5 w-2.5 rounded-full bg-primary"
                  />
                </div>
                <span class="font-semibold text-sm">Built-in</span>
              </div>
              <p class="text-xs text-muted-foreground pl-7">
                SQLite engine (no CLI needed)
              </p>
            </button>

            <!-- br option -->
            <button
              class="relative flex flex-col items-start gap-1.5 rounded-lg border-2 p-3 text-left transition-colors"
              :class="backendMode === 'br'
                ? 'border-primary bg-primary/5'
                : 'border-muted hover:border-muted-foreground/25 hover:bg-muted/50'"
              :disabled="isSwitchingBackend"
              @click="selectBackend('br')"
            >
              <div class="flex items-center gap-2">
                <div
                  class="flex h-5 w-5 items-center justify-center rounded-full border-2 transition-colors"
                  :class="backendMode === 'br' ? 'border-primary' : 'border-muted-foreground/40'"
                >
                  <div
                    v-if="backendMode === 'br'"
                    class="h-2.5 w-2.5 rounded-full bg-primary"
                  />
                </div>
                <span class="font-mono font-semibold text-sm">br</span>
              </div>
              <p class="text-xs text-muted-foreground pl-7">
                Beads Rust CLI
              </p>
            </button>

            <!-- bd option (legacy) -->
            <button
              class="relative flex flex-col items-start gap-1.5 rounded-lg border-2 p-3 text-left transition-colors"
              :class="backendMode === 'bd'
                ? 'border-primary bg-primary/5'
                : 'border-muted hover:border-muted-foreground/25 hover:bg-muted/50'"
              :disabled="isSwitchingBackend"
              @click="selectBackend('bd')"
            >
              <div class="flex items-center gap-2">
                <div
                  class="flex h-5 w-5 items-center justify-center rounded-full border-2 transition-colors"
                  :class="backendMode === 'bd' ? 'border-primary' : 'border-muted-foreground/40'"
                >
                  <div
                    v-if="backendMode === 'bd'"
                    class="h-2.5 w-2.5 rounded-full bg-primary"
                  />
                </div>
                <span class="font-mono font-semibold text-sm">bd</span>
              </div>
              <p class="text-xs text-muted-foreground pl-7">
                Legacy Go CLI (0.49.x)
              </p>
            </button>
          </div>

          <!-- Backend switching spinner -->
          <div v-if="isSwitchingBackend" class="flex items-center gap-2 text-sm text-muted-foreground">
            <svg class="animate-spin h-3 w-3" viewBox="0 0 24 24" fill="none">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
            </svg>
            Switching backend...
          </div>

          <!-- Backend warning -->
          <div v-if="backendWarning" class="flex items-start gap-2 p-2 rounded-md text-sm bg-amber-500/10 text-amber-600 dark:text-amber-400">
            <svg class="w-4 h-4 shrink-0 mt-0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
              <line x1="12" y1="9" x2="12" y2="13" />
              <line x1="12" y1="17" x2="12.01" y2="17" />
            </svg>
            <span class="text-xs">{{ backendWarning }}</span>
          </div>

          <!-- CLI version result (shown after switching to br/bd) -->
          <div v-if="backendResult" class="flex items-start gap-2 p-2 rounded-md text-sm" :class="backendResult.success ? 'bg-green-500/10 text-green-600 dark:text-green-400' : 'bg-destructive/10 text-destructive'">
            <svg v-if="backendResult.success" class="w-4 h-4 shrink-0 mt-0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="20 6 9 17 4 12" />
            </svg>
            <svg v-else class="w-4 h-4 shrink-0 mt-0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <line x1="15" y1="9" x2="9" y2="15" />
              <line x1="9" y1="9" x2="15" y2="15" />
            </svg>
            <span class="font-mono text-xs break-all">{{ backendResult.message }}</span>
          </div>
        </div>

        <!-- Migration from .beads/ -->
        <div v-if="backendMode === 'built-in' && beadsSource?.has_jsonl" class="space-y-3">
          <Label>Migrate from .beads/</Label>
          <p class="text-xs text-muted-foreground">
            Import {{ beadsSource.issue_count }} issues, attachments, and config from your existing .beads/ data into the built-in tracker.
          </p>

          <Button
            size="sm"
            variant="outline"
            :disabled="isMigrating || migrationResult !== null"
            @click="runMigration"
          >
            <svg v-if="isMigrating" class="animate-spin h-3 w-3 mr-1.5" viewBox="0 0 24 24" fill="none">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
            </svg>
            <svg v-else-if="migrationResult" class="w-3.5 h-3.5 mr-1.5 text-green-600 dark:text-green-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="20 6 9 17 4 12" />
            </svg>
            {{ isMigrating ? 'Migrating...' : migrationResult ? 'Migration complete' : 'Start migration' }}
          </Button>

          <!-- Migration result -->
          <div v-if="migrationResult" class="space-y-1.5 p-2 rounded-md text-xs bg-green-500/10 text-green-600 dark:text-green-400">
            <div class="font-medium">Migration successful</div>
            <div>Issues: {{ migrationResult.issues.inserted }} imported, {{ migrationResult.issues.updated }} updated, {{ migrationResult.issues.skipped }} skipped</div>
            <div v-if="migrationResult.attachments_copied > 0 || migrationResult.attachments_skipped > 0">
              Attachments: {{ migrationResult.attachments_copied }} copied, {{ migrationResult.attachments_skipped }} skipped
            </div>
            <div v-if="migrationResult.config_migrated">Config migrated</div>
            <div v-if="migrationResult.warnings.length > 0" class="text-amber-600 dark:text-amber-400">
              <div v-for="(w, i) in migrationResult.warnings" :key="i">{{ w }}</div>
            </div>
          </div>

          <!-- Migration error -->
          <div v-if="migrationError" class="flex items-start gap-2 p-2 rounded-md text-sm bg-destructive/10 text-destructive">
            <svg class="w-4 h-4 shrink-0 mt-0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <line x1="15" y1="9" x2="9" y2="15" />
              <line x1="9" y1="9" x2="15" y2="15" />
            </svg>
            <span class="text-xs">{{ migrationError }}</span>
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

      </div>
    </DialogContent>
  </Dialog>
</template>
