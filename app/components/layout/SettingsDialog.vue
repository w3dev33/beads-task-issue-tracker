<script setup lang="ts">
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog'
import { Input } from '~/components/ui/input'
import { Label } from '~/components/ui/label'
import { Button } from '~/components/ui/button'
import { getCliBinaryPath, setCliBinaryPath, validateCliBinary } from '~/utils/bd-api'

const open = defineModel<boolean>('open', { default: false })

const binaryPath = ref('')
const savedPath = ref('')
const testResult = ref<{ success: boolean; message: string } | null>(null)
const isTesting = ref(false)
const isSaving = ref(false)

// Load current setting when dialog opens
watch(open, async (isOpen) => {
  if (isOpen) {
    try {
      const current = await getCliBinaryPath()
      binaryPath.value = current
      savedPath.value = current
      testResult.value = null
    } catch {
      binaryPath.value = 'bd'
      savedPath.value = 'bd'
    }
  }
})

const hasChanges = computed(() => binaryPath.value !== savedPath.value)

async function handleTest() {
  isTesting.value = true
  testResult.value = null
  try {
    const version = await validateCliBinary(binaryPath.value || 'bd')
    testResult.value = { success: true, message: version }
  } catch (error) {
    testResult.value = {
      success: false,
      message: error instanceof Error ? error.message : String(error),
    }
  } finally {
    isTesting.value = false
  }
}

async function handleSave() {
  isSaving.value = true
  testResult.value = null
  try {
    const version = await setCliBinaryPath(binaryPath.value || 'bd')
    savedPath.value = binaryPath.value || 'bd'
    if (!binaryPath.value) binaryPath.value = 'bd'
    testResult.value = { success: true, message: `Saved! ${version}` }
  } catch (error) {
    testResult.value = {
      success: false,
      message: error instanceof Error ? error.message : String(error),
    }
  } finally {
    isSaving.value = false
  }
}

function handleReset() {
  binaryPath.value = 'bd'
  testResult.value = null
}
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>Settings</DialogTitle>
        <DialogDescription>
          Configure the CLI binary used for issue management.
        </DialogDescription>
      </DialogHeader>

      <div class="space-y-4 pt-2">
        <!-- CLI Binary Path -->
        <div class="space-y-2">
          <Label for="cli-binary">CLI Binary</Label>
          <div class="flex gap-2">
            <Input
              id="cli-binary"
              v-model="binaryPath"
              placeholder="bd"
              class="flex-1 font-mono text-sm"
              @keydown.enter="handleTest"
            />
            <Button
              variant="outline"
              size="sm"
              :disabled="isTesting || isSaving"
              class="shrink-0"
              @click="handleTest"
            >
              <svg v-if="isTesting" class="animate-spin -ml-1 mr-1 h-3 w-3" viewBox="0 0 24 24" fill="none">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
              </svg>
              Test
            </Button>
          </div>
          <p class="text-xs text-muted-foreground">
            Name or full path of a bd-compatible CLI binary (e.g., <code class="bg-muted px-1 rounded">bd</code>, <code class="bg-muted px-1 rounded">br</code>, <code class="bg-muted px-1 rounded">/usr/local/bin/br</code>).
          </p>
          <p class="text-xs text-muted-foreground">
            The alternative CLI must be fully compatible with the bd command interface.
          </p>
        </div>

        <!-- Test/Validation Result -->
        <div v-if="testResult" class="flex items-start gap-2 p-2 rounded-md text-sm" :class="testResult.success ? 'bg-green-500/10 text-green-600 dark:text-green-400' : 'bg-destructive/10 text-destructive'">
          <svg v-if="testResult.success" class="w-4 h-4 shrink-0 mt-0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20 6 9 17 4 12" />
          </svg>
          <svg v-else class="w-4 h-4 shrink-0 mt-0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" />
            <line x1="15" y1="9" x2="9" y2="15" />
            <line x1="9" y1="9" x2="15" y2="15" />
          </svg>
          <span class="font-mono text-xs break-all">{{ testResult.message }}</span>
        </div>

        <!-- Action buttons -->
        <div class="flex items-center justify-between pt-2">
          <Button
            variant="ghost"
            size="sm"
            :disabled="binaryPath === 'bd'"
            @click="handleReset"
          >
            Reset to default
          </Button>
          <Button
            size="sm"
            :disabled="!hasChanges || isSaving || isTesting"
            @click="handleSave"
          >
            <svg v-if="isSaving" class="animate-spin -ml-1 mr-1 h-3 w-3" viewBox="0 0 24 24" fill="none">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
            </svg>
            Save
          </Button>
        </div>
      </div>
    </DialogContent>
  </Dialog>
</template>
