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
import { getCliBinaryPath, setCliBinaryPath } from '~/utils/bd-api'

const open = defineModel<boolean>('open', { default: false })

const selectedClient = ref<'bd' | 'br'>('bd')
const isSwitching = ref(false)
const switchResult = ref<{ success: boolean; message: string } | null>(null)

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
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>Settings</DialogTitle>
        <DialogDescription>
          Choose which CLI client to use for issue management.
        </DialogDescription>
      </DialogHeader>

      <div class="space-y-4 pt-2">
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
