<script setup lang="ts">
import { Button } from '~/components/ui/button'
import { readLogs, clearLogs, exportLogs as exportLogsApi, getLogPath, getBdVersion, getLoggingEnabled, setLoggingEnabled, getVerboseLogging, setVerboseLogging, checkBdCliUpdate, fsExists, type BdCliUpdateInfo } from '~/utils/bd-api'
import { openUrl } from '~/utils/open-url'

const { isSyncing: isForceSyncing, forceSync, syncMessage, lastSyncSuccess } = useSyncStatus()
const { beadsPath } = useBeadsPath()

const props = defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits<{
  'update:isOpen': [value: boolean]
}>()

const logs = ref('')
const logPath = ref('')
const bdVersion = ref('')
const projectUsesDolt = ref(false)
const isAutoRefresh = ref(true)
const isLoading = ref(false)
const isVerbose = ref(false)
const bdCliUpdate = ref<BdCliUpdateInfo | null>(null)
let refreshInterval: ReturnType<typeof setInterval> | null = null

const logContainerRef = ref<HTMLDivElement | null>(null)
const isUserAtBottom = ref(true)

const SCROLL_THRESHOLD = 30 // px from bottom to consider "at bottom"

const onScroll = () => {
  if (!logContainerRef.value) return
  const el = logContainerRef.value
  isUserAtBottom.value = el.scrollHeight - el.scrollTop - el.clientHeight < SCROLL_THRESHOLD
}

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
    isUserAtBottom.value = true
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
  escaped = escaped.replace(/\[bd_check_changed\]/g, '<span class="text-green-400">[bd_check_changed]</span>')
  escaped = escaped.replace(/\[bd_poll_data\]/g, '<span class="text-cyan-400">[bd_poll_data]</span>')
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
    if (isUserAtBottom.value) {
      nextTick(() => {
        scrollToBottom()
      })
    }
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
// Also enable backend logging when panel is open (LOGGING_ENABLED flag)
watch(() => props.isOpen, async (isOpen) => {
  if (isOpen) {
    bdVersion.value = await getBdVersion()
    logPath.value = await getLogPath()
    isVerbose.value = await getVerboseLogging()
    // Check if current project uses Dolt backend
    if (beadsPath.value && beadsPath.value !== '.') {
      fsExists(`${beadsPath.value}/.beads/.dolt`).then((exists) => { projectUsesDolt.value = exists }).catch(() => { projectUsesDolt.value = false })
    } else {
      projectUsesDolt.value = false
    }
    // Check for bd CLI updates (non-blocking)
    checkBdCliUpdate().then((info) => { bdCliUpdate.value = info }).catch(() => {})
    // Enable backend logging so log_info!/log_debug! macros produce output
    await setLoggingEnabled(true)
    if (isAutoRefresh.value) {
      startAutoRefresh()
    } else {
      fetchLogs()
    }
  } else {
    stopAutoRefresh()
    // Disable backend logging to save resources when panel is closed
    await setLoggingEnabled(false)
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
        <button
          v-if="bdCliUpdate?.hasUpdate"
          class="flex items-center gap-1.5 text-xs font-medium text-foreground hover:text-green-500 transition-colors cursor-pointer"
          :title="`Update available: v${bdCliUpdate.latestVersion} — click to view`"
          @click="openUrl(bdCliUpdate.releaseUrl)"
        >
          {{ bdVersion }}
          <span class="relative flex h-2.5 w-2.5">
            <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
            <span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-green-500"></span>
          </span>
        </button>
        <button
          v-else-if="bdCliUpdate"
          class="text-xs font-medium text-foreground hover:text-primary transition-colors cursor-pointer"
          title="View bd CLI releases"
          @click="openUrl(bdCliUpdate.releaseUrl)"
        >
          {{ bdVersion }}
        </button>
        <span v-else class="text-xs font-medium text-foreground">{{ bdVersion }}</span>
        <span v-if="projectUsesDolt" class="text-[#29E3C1] flex items-center" title="This project uses the Dolt backend">
          <svg class="w-8 h-3" viewBox="0 0 163 56" fill="none">
            <path d="M28.87 7.0459V45.8632C28.8654 46.7997 28.498 47.6965 27.8476 48.3591C27.1971 49.0217 26.316 49.3964 25.3957 49.402H10.4953C9.5713 49.402 8.68489 49.0298 8.0299 48.3666C7.3749 47.7035 7.00462 46.8034 7 45.8632V24.7722C7.00462 23.832 7.3749 22.9319 8.0299 22.2688C8.68489 21.6056 9.5713 21.2334 10.4953 21.2334H22.2115" stroke="currentColor" stroke-width="12.6599" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M156.3 49.4019H145.283" stroke="currentColor" stroke-width="12.6599" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M156.026 21.5259H134.174" stroke="currentColor" stroke-width="12.6599" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M145.336 7.0498V49.4024" stroke="currentColor" stroke-width="12.6599" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M72.2752 7.68311H59.049C56.6669 7.68311 54.7358 9.64808 54.7358 12.072V44.8074C54.7358 47.2313 56.6669 49.1963 59.049 49.1963H72.2752C74.6573 49.1963 76.5884 47.2313 76.5884 44.8074V12.072C76.5884 9.64808 74.6573 7.68311 72.2752 7.68311Z" stroke="currentColor" stroke-width="12.6599" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M119.586 49.4019H99.418" stroke="currentColor" stroke-width="12.6599" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M110.344 7.0498V49.4024" stroke="currentColor" stroke-width="12.6599" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M109.884 7H98.7939" stroke="currentColor" stroke-width="12.6599" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </span>
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
    <div ref="logContainerRef" class="flex-1 overflow-auto bg-muted/10" @scroll="onScroll">
      <pre v-if="logs" class="p-3 text-[11px] font-mono whitespace-pre-wrap break-all leading-relaxed" v-html="colorizedLogs"></pre>
      <pre v-else class="p-3 text-[11px] font-mono text-muted-foreground">No logs yet...</pre>
    </div>
  </div>
</template>
