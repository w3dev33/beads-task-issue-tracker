<script setup lang="ts">
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog'
import { Button } from '~/components/ui/button'
import { readLogs, clearLogs, getLogPath } from '~/utils/bd-api'

const open = defineModel<boolean>('open', { default: false })

const logs = ref('')
const logPath = ref('')
const isAutoRefresh = ref(true)
const isLoading = ref(false)
let refreshInterval: ReturnType<typeof setInterval> | null = null

const logContainerRef = ref<HTMLDivElement | null>(null)

const scrollToBottom = () => {
  if (logContainerRef.value) {
    logContainerRef.value.scrollTop = logContainerRef.value.scrollHeight
  }
}

const fetchLogs = async () => {
  try {
    logs.value = await readLogs(500) // Last 500 lines
    // Auto-scroll to bottom
    nextTick(() => {
      scrollToBottom()
    })
  } catch (e) {
    console.error('Failed to fetch logs:', e)
  }
}

const handleClearLogs = async () => {
  isLoading.value = true
  try {
    await clearLogs()
    logs.value = ''
  } catch (e) {
    console.error('Failed to clear logs:', e)
  } finally {
    isLoading.value = false
  }
}

const toggleAutoRefresh = () => {
  isAutoRefresh.value = !isAutoRefresh.value
  if (isAutoRefresh.value) {
    startAutoRefresh()
  } else {
    stopAutoRefresh()
  }
}

const startAutoRefresh = () => {
  if (refreshInterval) return
  fetchLogs()
  refreshInterval = setInterval(fetchLogs, 2000) // Refresh every 2 seconds
}

const stopAutoRefresh = () => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
    refreshInterval = null
  }
}

// Start/stop auto-refresh when dialog opens/closes
watch(open, async (isOpen) => {
  if (isOpen) {
    logPath.value = await getLogPath()
    if (isAutoRefresh.value) {
      startAutoRefresh()
    } else {
      fetchLogs()
    }
  } else {
    stopAutoRefresh()
  }
})

onUnmounted(() => {
  stopAutoRefresh()
})
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="sm:max-w-4xl max-h-[80vh] grid grid-rows-[auto_auto_1fr_auto] gap-0 p-0 overflow-hidden">
      <DialogHeader class="p-6 pb-0">
        <DialogTitle class="flex items-center gap-2">
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 20h9" />
            <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
          </svg>
          Debug Logs
        </DialogTitle>
      </DialogHeader>

      <!-- Toolbar -->
      <div class="flex items-center justify-between gap-2 px-6 py-3 border-b border-border">
        <div class="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            :class="isAutoRefresh ? 'bg-green-500/10 border-green-500/50 text-green-600' : ''"
            @click="toggleAutoRefresh"
          >
            <svg
              class="w-4 h-4 mr-1"
              :class="{ 'animate-spin': isAutoRefresh }"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" />
              <path d="M21 3v5h-5" />
            </svg>
            {{ isAutoRefresh ? 'Live' : 'Paused' }}
          </Button>

          <Button variant="outline" size="sm" :disabled="isAutoRefresh" @click="fetchLogs">
            <svg class="w-4 h-4 mr-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" />
              <path d="M21 3v5h-5" />
            </svg>
            Refresh
          </Button>

          <Button variant="outline" size="sm" @click="scrollToBottom">
            <svg class="w-4 h-4 mr-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 5v14" />
              <path d="m19 12-7 7-7-7" />
            </svg>
            Bottom
          </Button>
        </div>

        <div class="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            class="text-destructive hover:bg-destructive hover:text-destructive-foreground"
            :disabled="isLoading"
            @click="handleClearLogs"
          >
            <svg class="w-4 h-4 mr-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3 6 5 6 21 6" />
              <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
            </svg>
            Clear
          </Button>
        </div>
      </div>

      <!-- Log content -->
      <div ref="logContainerRef" class="overflow-auto mx-6 my-3 border border-border rounded-md bg-muted/30">
        <pre class="p-4 text-xs font-mono whitespace-pre-wrap break-all text-muted-foreground">{{ logs || 'No logs yet...' }}</pre>
      </div>

      <!-- Footer with log path -->
      <div class="text-xs text-muted-foreground px-6 pb-4 truncate">
        {{ logPath }}
      </div>
    </DialogContent>
  </Dialog>
</template>
