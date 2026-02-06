<script setup lang="ts">
import { Button } from '~/components/ui/button'
import { readLogs, clearLogs, exportLogs as exportLogsApi, getLogPath, getBdVersion, getVerboseLogging, setVerboseLogging } from '~/utils/bd-api'

const { isSyncing: isForceSyncing, forceSync, syncMessage, lastSyncSuccess } = useSyncStatus()

const props = defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits<{
  'update:isOpen': [value: boolean]
}>()

const logs = ref('')
const logPath = ref('')
const bdVersion = ref('')
const isAutoRefresh = ref(true)
const isLoading = ref(false)
const isVerbose = ref(false)
let refreshInterval: ReturnType<typeof setInterval> | null = null

const logContainerRef = ref<HTMLDivElement | null>(null)

// Resizable panel
const panelHeight = ref(250)
const isResizing = ref(false)
const startY = ref(0)
const startHeight = ref(0)

const minHeight = 150
const maxHeightPercent = 0.5 // 50% of screen

const startResize = (e: MouseEvent) => {
  e.preventDefault()
  isResizing.value = true
  startY.value = e.clientY
  startHeight.value = panelHeight.value
  document.addEventListener('mousemove', onResize)
  document.addEventListener('mouseup', stopResize)
  document.body.style.cursor = 'row-resize'
  document.body.style.userSelect = 'none'
}

const onResize = (e: MouseEvent) => {
  if (!isResizing.value) return
  const maxHeight = window.innerHeight * maxHeightPercent
  const diff = startY.value - e.clientY
  const newHeight = Math.min(Math.max(startHeight.value + diff, minHeight), maxHeight)
  panelHeight.value = newHeight
}

const stopResize = () => {
  isResizing.value = false
  document.removeEventListener('mousemove', onResize)
  document.removeEventListener('mouseup', stopResize)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
}

const scrollToBottom = () => {
  if (logContainerRef.value) {
    logContainerRef.value.scrollTop = logContainerRef.value.scrollHeight
  }
}

// Colorize log tags
const colorizedLogs = computed(() => {
  if (!logs.value) return ''

  // Escape HTML first
  let escaped = logs.value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')

  // Colorize tags
  escaped = escaped.replace(/\[ERROR\]/g, '<span class="text-red-500 font-semibold">[ERROR]</span>')
  escaped = escaped.replace(/\[WARN\]/g, '<span class="text-amber-500 font-semibold">[WARN]</span>')
  escaped = escaped.replace(/\[DEBUG\]/g, '<span class="text-slate-400">[DEBUG]</span>')
  escaped = escaped.replace(/\[INFO\]/g, '<span class="text-blue-400">[INFO]</span>')

  // Colorize command tags
  escaped = escaped.replace(/\[bd_list\]/g, '<span class="text-cyan-400">[bd_list]</span>')
  escaped = escaped.replace(/\[bd_ready\]/g, '<span class="text-cyan-400">[bd_ready]</span>')
  escaped = escaped.replace(/\[bd_show\]/g, '<span class="text-cyan-400">[bd_show]</span>')
  escaped = escaped.replace(/\[bd_sync\]/g, '<span class="text-cyan-400">[bd_sync]</span>')
  escaped = escaped.replace(/\[bd\]/g, '<span class="text-blue-400">[bd]</span>')
  escaped = escaped.replace(/\[sync\]/g, '<span class="text-green-400">[sync]</span>')
  escaped = escaped.replace(/\[startup\]/g, '<span class="text-purple-400">[startup]</span>')
  escaped = escaped.replace(/\[debug\]/g, '<span class="text-amber-400">[debug]</span>')
  escaped = escaped.replace(/\[frontend\]/g, '<span class="text-orange-400 font-semibold">[frontend]</span>')
  escaped = escaped.replace(/\[unhandled\]/g, '<span class="text-red-400 font-semibold">[unhandled]</span>')
  escaped = escaped.replace(/\[unhandled-rejection\]/g, '<span class="text-red-400 font-semibold">[unhandled-rejection]</span>')

  return escaped
})

const fetchLogs = async () => {
  try {
    logs.value = await readLogs(300) // Last 300 lines
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
  refreshInterval = setInterval(fetchLogs, 2000)
}

const stopAutoRefresh = () => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
    refreshInterval = null
  }
}

const toggleVerbose = async () => {
  isVerbose.value = !isVerbose.value
  await setVerboseLogging(isVerbose.value)
}

const exportedPath = ref('')

const exportLogs = async () => {
  try {
    const path = await exportLogsApi()
    if (path) {
      exportedPath.value = path
      // Clear message after 5 seconds
      setTimeout(() => {
        exportedPath.value = ''
      }, 5000)
    }
  } catch (e) {
    console.error('Failed to export logs:', e)
  }
}

const close = () => {
  emit('update:isOpen', false)
}

// Start/stop auto-refresh when panel opens/closes
watch(() => props.isOpen, async (isOpen) => {
  if (isOpen) {
    bdVersion.value = await getBdVersion()
    logPath.value = await getLogPath()
    isVerbose.value = await getVerboseLogging()
    if (isAutoRefresh.value) {
      startAutoRefresh()
    } else {
      fetchLogs()
    }
  } else {
    stopAutoRefresh()
  }
}, { immediate: true })

onUnmounted(() => {
  stopAutoRefresh()
})
</script>

<template>
  <div
    v-if="isOpen"
    class="border-t border-border bg-card flex flex-col relative"
    :style="{ height: `${panelHeight}px` }"
  >
    <!-- Resize handle -->
    <div
      class="absolute top-0 left-0 right-0 h-1 cursor-row-resize hover:bg-primary/50 transition-colors z-10"
      @mousedown="startResize"
    />

    <!-- Header -->
    <div class="flex items-center justify-between gap-2 px-4 py-2 border-b border-border bg-muted/30">
      <div class="flex items-center gap-2">
        <span class="text-sm font-medium">Debug Logs</span>

        <Button
          variant="outline"
          size="sm"
          class="h-7 px-2"
          :class="isAutoRefresh ? 'border-green-500 text-green-500' : ''"
          @click="toggleAutoRefresh"
        >
          <svg
            class="w-3.5 h-3.5 mr-1"
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

        <Button variant="outline" size="sm" class="h-7 px-2" :disabled="isAutoRefresh" @click="fetchLogs">
          Refresh
        </Button>

        <Button variant="outline" size="sm" class="h-7 px-2" @click="scrollToBottom">
          Bottom
        </Button>

        <Button
          variant="outline"
          size="sm"
          class="h-7 px-2"
          :class="isVerbose ? 'border-amber-500 text-amber-500' : ''"
          @click="toggleVerbose"
        >
          Verbose {{ isVerbose ? 'ON' : 'OFF' }}
        </Button>

        <Button
          variant="outline"
          size="sm"
          class="h-7 px-2 text-destructive border-destructive/50 hover:bg-destructive hover:text-destructive-foreground"
          :disabled="isLoading"
          @click="handleClearLogs"
        >
          Clear
        </Button>

        <Button
          v-if="logs"
          variant="outline"
          size="sm"
          class="h-7 px-2"
          @click="exportLogs"
        >
          Export
        </Button>
        <span v-if="exportedPath" class="text-xs text-green-500 truncate max-w-[300px]" :title="exportedPath">{{ exportedPath }}</span>

        <div class="w-px h-4 bg-border mx-2" />

        <Button
          variant="outline"
          size="sm"
          class="h-7 px-2"
          :class="isForceSyncing ? 'border-primary text-primary' : ''"
          :disabled="isForceSyncing"
          @click="forceSync"
        >
          <svg
            :class="['w-3.5 h-3.5 mr-1', isForceSyncing ? 'animate-pulse' : '']"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242" />
            <path d="M12 12v9" />
            <path d="m8 17 4 4 4-4" />
          </svg>
          {{ isForceSyncing ? 'Syncing...' : 'Force Sync' }}
        </Button>
        <span v-if="syncMessage && lastSyncSuccess" class="text-green-500 text-xs ml-1">{{ syncMessage }}</span>
      </div>

      <div class="flex items-center gap-2">
        <span class="text-xs font-medium text-foreground">{{ bdVersion }}</span>
        <span class="text-muted-foreground">|</span>
        <span class="text-xs text-muted-foreground truncate max-w-[300px]">{{ logPath }}</span>
        <Button variant="outline" size="sm" class="h-7 w-7 p-0" @click="close">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6 6 18" />
            <path d="m6 6 12 12" />
          </svg>
        </Button>
      </div>
    </div>

    <!-- Log content -->
    <div ref="logContainerRef" class="flex-1 overflow-auto bg-muted/10">
      <pre v-if="logs" class="p-3 text-[11px] font-mono whitespace-pre-wrap break-all leading-relaxed" v-html="colorizedLogs"></pre>
      <pre v-else class="p-3 text-[11px] font-mono text-muted-foreground">No logs yet...</pre>
    </div>
  </div>
</template>
