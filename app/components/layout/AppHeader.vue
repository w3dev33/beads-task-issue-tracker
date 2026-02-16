<script setup lang="ts">
import { Button } from '~/components/ui/button'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '~/components/ui/tooltip'

const props = defineProps<{
  favoriteName?: string
  editContext?: string
  editId?: string
  showRefresh?: boolean
}>()

const emit = defineEmits<{
  refresh: []
}>()

// Title: show favorite name if selected, otherwise default app title
const displayTitle = computed(() => props.favoriteName || 'Beads Task-Issue Tracker')

const { isDark, currentTheme, cycleTheme } = useTheme()
const { zoomLevel, zoomIn, zoomOut, resetZoom, canZoomIn, canZoomOut } = useZoom()
const { startDragging } = useTauriWindow()

// Handle window dragging via Tauri API
const handleMouseDown = (event: MouseEvent) => {
  // Only handle left click
  if (event.button !== 0) return

  // Don't start dragging if click is inside a no-drag zone (buttons, inputs, etc.)
  const target = event.target as HTMLElement
  if (target.closest('.app-no-drag')) return

  startDragging()
}

const handleZoomOut = (event: MouseEvent) => {
  if (event.altKey) {
    resetZoom()
  } else {
    zoomOut()
  }
}

const handleZoomIn = (event: MouseEvent) => {
  if (event.altKey) {
    resetZoom()
  } else {
    zoomIn()
  }
}
</script>

<template>
  <!-- macOS: pl-20 leaves space for traffic lights, mousedown triggers Tauri window dragging -->
  <header
    class="flex items-center justify-center pl-20 pr-4 py-3 border-b border-border bg-card relative app-drag-region"
    data-tauri-drag-region
    @mousedown="handleMouseDown"
  >
    <!-- Centered title with icon - pointer-events-none to allow drag through -->
    <div v-if="!editContext" class="flex items-center gap-3 pointer-events-none">
      <svg
        class="w-8 h-8"
        viewBox="0 0 24 24"
        fill="none"
        stroke-width="2"
      >
        <!-- Curved lines from center to center (circles drawn on top will hide the ends) -->
        <path d="M 4 5 Q 3 12 12 18" stroke="currentColor" class="text-muted-foreground" />
        <path d="M 12 18 Q 21 12 20 5" stroke="currentColor" class="text-muted-foreground" />
        <path d="M 20 5 Q 12 1 4 5" stroke="currentColor" class="text-muted-foreground" />
        <circle cx="4" cy="5" r="3" fill="#22c55e" />
        <circle cx="20" cy="5" r="3" fill="#eab308" />
        <circle cx="12" cy="18" r="3" fill="#ef4444" />
      </svg>
      <h1 class="text-lg font-semibold text-foreground leading-tight">{{ displayTitle }}</h1>
    </div>

    <!-- Edit context (centered, replaces title) - pointer-events-none to allow drag through -->
    <div v-else class="flex items-center gap-3 pointer-events-none">
      <span class="text-sm font-medium uppercase">
        <span class="text-foreground">{{ editContext }}</span>
        <span v-if="editId" class="text-sky-400 ml-1">{{ editId }}</span>
      </span>
    </div>

    <!-- Zoom and Theme controls (right, absolute positioned) - no-drag to keep buttons clickable -->
    <div class="absolute right-4 flex items-center gap-1 app-no-drag">
      <!-- Zoom controls -->
      <Tooltip>
        <TooltipTrigger as-child>
          <Button
            variant="ghost"
            size="icon"
            class="h-8 w-8"
            :disabled="!canZoomOut && zoomLevel === 100"
            @click="handleZoomOut"
          >
            <svg
              class="w-4 h-4"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <circle cx="11" cy="11" r="8" />
              <line x1="21" y1="21" x2="16.65" y2="16.65" />
              <line x1="8" y1="11" x2="14" y2="11" />
            </svg>
          </Button>
        </TooltipTrigger>
        <TooltipContent>Zoom out (⌥ click to reset)</TooltipContent>
      </Tooltip>

      <span class="text-xs text-muted-foreground w-10 text-center tabular-nums">{{ zoomLevel }}%</span>

      <Tooltip>
        <TooltipTrigger as-child>
          <Button
            variant="ghost"
            size="icon"
            class="h-8 w-8"
            :disabled="!canZoomIn && zoomLevel === 100"
            @click="handleZoomIn"
          >
            <svg
              class="w-4 h-4"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <circle cx="11" cy="11" r="8" />
              <line x1="21" y1="21" x2="16.65" y2="16.65" />
              <line x1="11" y1="8" x2="11" y2="14" />
              <line x1="8" y1="11" x2="14" y2="11" />
            </svg>
          </Button>
        </TooltipTrigger>
        <TooltipContent>Zoom in (⌥ click to reset)</TooltipContent>
      </Tooltip>

      <!-- Refresh button -->
      <Tooltip v-if="showRefresh">
        <TooltipTrigger as-child>
          <Button
            variant="ghost"
            size="icon"
            class="h-8 w-8"
            @click="emit('refresh')"
          >
            <svg
              class="w-4 h-4"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" />
              <path d="M21 3v5h-5" />
            </svg>
          </Button>
        </TooltipTrigger>
        <TooltipContent>Refresh</TooltipContent>
      </Tooltip>

      <!-- Theme toggle (cycles through themes) -->
      <Tooltip>
        <TooltipTrigger as-child>
          <Button
            variant="ghost"
            size="icon"
            class="h-8 w-8"
            @click="cycleTheme"
          >
            <!-- Sun icon (light theme) -->
            <svg
              v-if="currentTheme.icon === 'sun'"
              class="w-4 h-4"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <circle cx="12" cy="12" r="5" />
              <line x1="12" y1="1" x2="12" y2="3" />
              <line x1="12" y1="21" x2="12" y2="23" />
              <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
              <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
              <line x1="1" y1="12" x2="3" y2="12" />
              <line x1="21" y1="12" x2="23" y2="12" />
              <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
              <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
            </svg>
            <!-- Moon icon (dark theme) -->
            <svg
              v-else-if="currentTheme.icon === 'moon'"
              class="w-4 h-4"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
            </svg>
            <!-- Square icon (flat theme) -->
            <svg
              v-else-if="currentTheme.icon === 'square'"
              class="w-4 h-4"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <rect x="3" y="3" width="18" height="18" rx="2" />
            </svg>
            <!-- Zap icon (neon theme) -->
            <svg
              v-else
              class="w-4 h-4"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M13 2L3 14h9l-1 10 10-12h-9l1-10z" />
            </svg>
          </Button>
        </TooltipTrigger>
        <TooltipContent>{{ currentTheme.label }}</TooltipContent>
      </Tooltip>
    </div>
  </header>
</template>
