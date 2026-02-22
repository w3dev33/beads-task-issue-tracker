<script setup lang="ts">
import { Button } from '~/components/ui/button'
import { Badge } from '~/components/ui/badge'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '~/components/ui/tooltip'
import ConfirmDialog from '~/components/ui/confirm-dialog/ConfirmDialog.vue'
import FolderPicker from './FolderPicker.vue'
import Sortable from 'sortablejs'
import { getFolderName } from '~/utils/path'
import { listProbeProjects, registerOrExposeProject, patchProbeProject, probeUnregisterProject, getExternalUrl } from '~/utils/bd-api'
import type { ProbeProject } from '~/utils/probe-adapter'

const props = defineProps<{
  isLoading?: boolean
}>()

const { beadsPath, setPath, clearPath } = useBeadsPath()
const { projects, sortedProjects, sortMode, hasReordered, removeProject, reorderProjects, setSortMode, resetSortOrder } = useProjects()

const projectsListRef = ref<HTMLElement | null>(null)
let sortableInstance: Sortable | null = null

// Apply counter-zoom on projects list before SortableJS calculates coordinates
const resetZoomOnPointerDown = (e: PointerEvent) => {
  const target = e.target as HTMLElement
  if (!target.closest('.drag-handle') || !projectsListRef.value) return
  const zoomableContent = document.getElementById('zoomable-content')
  const parentZoom = parseFloat(zoomableContent?.style.zoom || '100')
  if (parentZoom !== 100) {
    projectsListRef.value.style.zoom = `${10000 / parentZoom}%`
  }
}

const restoreZoom = () => {
  if (projectsListRef.value) {
    projectsListRef.value.style.zoom = ''
  }
}

const initSortable = () => {
  if (sortableInstance) {
    sortableInstance.destroy()
    sortableInstance = null
  }
  if (!projectsListRef.value) return

  // Listen on pointerdown to reset zoom before SortableJS kicks in
  projectsListRef.value.addEventListener('pointerdown', resetZoomOnPointerDown)
  // Restore zoom if user releases without dragging
  projectsListRef.value.addEventListener('pointerup', restoreZoom)

  sortableInstance = Sortable.create(projectsListRef.value, {
    handle: '.drag-handle',
    animation: 200,
    ghostClass: 'opacity-30',
    forceFallback: true,
    fallbackClass: 'sortable-fallback',
    fallbackOnBody: true,
    disabled: !!props.isLoading,
    onEnd: (evt) => {
      restoreZoom()
      if (evt.oldIndex == null || evt.newIndex == null || evt.oldIndex === evt.newIndex) return
      // Read new order from DOM data attributes before reverting
      const container = evt.from
      const newOrder: string[] = []
      for (const child of container.children) {
        const path = (child as HTMLElement).dataset.path
        if (path) newOrder.push(path)
      }
      // Revert DOM change so Vue can handle re-rendering via reactivity
      container.removeChild(evt.item)
      const refNode = container.children[evt.oldIndex] || null
      container.insertBefore(evt.item, refNode)
      // Build reordered projects array from paths
      const currentProjs = sortedProjects.value
      const reordered = newOrder
        .map(path => currentProjs.find(f => f.path === path))
        .filter((f): f is NonNullable<typeof f> => !!f)
      if (reordered.length === currentProjs.length) {
        reorderProjects(reordered)
      }
    },
  })
}

const isProjectsCollapsed = useLocalStorage('beads:favoritesCollapsed', false)

onMounted(initSortable)
onBeforeUnmount(() => {
  projectsListRef.value?.removeEventListener('pointerdown', resetZoomOnPointerDown)
  projectsListRef.value?.removeEventListener('pointerup', restoreZoom)
  sortableInstance?.destroy()
})

// Re-init when list becomes visible, update disabled state
watch(() => props.isLoading, () => {
  if (sortableInstance) {
    sortableInstance.option('disabled', !!props.isLoading)
  }
})

// Re-init sortable when projects list becomes visible or content changes
watch([isProjectsCollapsed, () => projects.value.length], () => {
  nextTick(initSortable)
})

const toggleSortMode = () => {
  const cycle: Record<string, 'alpha' | 'alpha-desc' | 'manual'> = {
    alpha: 'alpha-desc',
    'alpha-desc': 'manual',
    manual: 'alpha',
  }
  setSortMode(cycle[sortMode.value] ?? 'alpha')
}

const emit = defineEmits<{
  change: []
  reset: []
}>()

const isPickerOpen = ref(false)
const isRemoveDialogOpen = ref(false)
const projectToRemove = ref<string | null>(null)

// Whether the currently selected project is exposed to the probe
const isCurrentExposed = computed(() => {
  if (!probeEnabled.value || !beadsPath.value) return false
  return isExposed(beadsPath.value)
})

// Expose to parent components
defineExpose({ isPickerOpen, isCurrentExposed })

const handleSelectFolder = (path: string) => {
  setPath(path)
  emit('change')
}

const handleSelectProject = (path: string) => {
  // Don't allow switching while loading
  if (props.isLoading) return
  setPath(path)
  emit('change')
}

const handleRemoveProject = (path: string, event: Event) => {
  event.stopPropagation()
  // Show confirmation dialog
  projectToRemove.value = path
  isRemoveDialogOpen.value = true
}

const confirmRemoveProject = () => {
  if (!projectToRemove.value) return

  const pathToRemove = projectToRemove.value
  const isCurrentPath = beadsPath.value === pathToRemove
  const isLastProject = projects.value.length === 1

  // Unregister from probe before removing (fire-and-forget)
  probeUnregisterProject(pathToRemove)

  // Remove the project
  removeProject(pathToRemove)

  // If we removed the current path
  if (isCurrentPath) {
    if (isLastProject) {
      // Last project removed - clear path to show onboarding
      clearPath()
      emit('reset')
    } else {
      // Switch to another project
      const remainingProject = projects.value.find(f => f.path !== pathToRemove)
      if (remainingProject) {
        setPath(remainingProject.path)
        emit('change')
      }
    }
  }

  // Close dialog
  isRemoveDialogOpen.value = false
  projectToRemove.value = null
}

const projectToRemoveName = computed(() => {
  if (!projectToRemove.value) return ''
  return getFolderName(projectToRemove.value)
})

// ============================================================================
// Probe Expose Toggle (per-project)
// ============================================================================

const probeProjects = ref<ProbeProject[]>([])
const isDev = import.meta.dev
const probeEnabled = isDev ? useLocalStorage('beads:probeEnabled', false) : ref(false)
const togglingPath = ref<string | null>(null)
const isExposeDialogOpen = ref(false)
const exposeTargetPath = ref<string | null>(null)

const exposeDialogTitle = computed(() => {
  if (!exposeTargetPath.value) return ''
  return isExposed(exposeTargetPath.value) ? 'Remove from monitoring' : 'Expose to monitoring'
})

const exposeDialogDescription = computed(() => {
  if (!exposeTargetPath.value) return ''
  const name = getFolderName(exposeTargetPath.value)
  return isExposed(exposeTargetPath.value)
    ? `Are you sure you want to remove '${name}' from monitoring?`
    : `Are you sure you want to expose '${name}' to monitoring?`
})

function getProbeProjectForPath(projPath: string): ProbeProject | undefined {
  // Match by checking if the probe path corresponds to the project path
  const beadsPath = projPath.endsWith('.beads') ? projPath : `${projPath}/.beads`
  return probeProjects.value.find(p => p.path === beadsPath || p.path === projPath)
}

function isExposed(projPath: string): boolean {
  const proj = getProbeProjectForPath(projPath)
  return proj?.expose === true
}

function isRegistered(projPath: string): boolean {
  return !!getProbeProjectForPath(projPath)
}

async function refreshProbeProjects() {
  if (!probeEnabled.value) return
  try {
    probeProjects.value = await listProbeProjects(getExternalUrl())
  } catch {
    probeProjects.value = []
  }
}

function requestToggleExpose(projPath: string) {
  exposeTargetPath.value = projPath
  isExposeDialogOpen.value = true
}

async function confirmToggleExpose() {
  const projPath = exposeTargetPath.value
  if (!projPath) return

  isExposeDialogOpen.value = false
  const baseUrl = getExternalUrl()
  if (!baseUrl) return

  togglingPath.value = projPath
  try {
    const beadsPath = projPath.endsWith('.beads') ? projPath : `${projPath}/.beads`
    const existing = getProbeProjectForPath(projPath)

    if (existing) {
      // Already registered — toggle expose
      await patchProbeProject(baseUrl, existing.name, { expose: !existing.expose })
    } else {
      // Not registered — register with expose: true
      await registerOrExposeProject(baseUrl, beadsPath, true)
    }

    await refreshProbeProjects()
  } catch {
    // Silently fail — user can retry
  } finally {
    togglingPath.value = null
    exposeTargetPath.value = null
  }
}

// Refresh probe projects when probe is enabled
watch(probeEnabled, (active) => {
  if (active) refreshProbeProjects()
}, { immediate: true })

// Also refresh when projects change (new project added)
watch(() => projects.value.length, () => {
  if (probeEnabled.value) refreshProbeProjects()
})
</script>

<template>
  <div class="space-y-2">
    <!-- Action buttons -->
    <div class="flex items-center gap-1">
      <Button variant="outline" size="sm" class="flex-1 h-7 text-xs" @click="isPickerOpen = true">
        <svg class="w-3 h-3 mr-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
        </svg>
        Select Project
      </Button>

    </div>

    <!-- Projects -->
    <div v-if="projects.length > 0" class="space-y-1">
      <div class="flex items-center gap-2 w-full">
        <button
          class="flex items-center gap-2 text-[10px] text-muted-foreground hover:text-foreground transition-colors flex-1"
          @click="isProjectsCollapsed = !isProjectsCollapsed"
        >
          <svg
            class="w-3 h-3 transition-transform"
            :class="{ '-rotate-90': isProjectsCollapsed }"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <polyline points="6 9 12 15 18 9" />
          </svg>
          <span class="uppercase tracking-wide">Projects</span>
          <span class="ml-auto">({{ projects.length }})</span>
        </button>
        <!-- Sort mode toggle (hidden when collapsed) -->
        <template v-if="!isProjectsCollapsed">
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger as-child>
                <button
                  class="text-muted-foreground hover:text-foreground transition-colors p-0.5 rounded"
                  @click.stop="toggleSortMode"
                >
                  <!-- A-Z ascending icon -->
                  <svg v-if="sortMode === 'alpha'" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M3 6h7" /><path d="M3 12h5" /><path d="M3 18h3" />
                    <path d="M17 18V6" /><path d="M14 9l3-3 3 3" />
                  </svg>
                  <!-- Z-A descending icon -->
                  <svg v-else-if="sortMode === 'alpha-desc'" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M3 6h3" /><path d="M3 12h5" /><path d="M3 18h7" />
                    <path d="M17 6v12" /><path d="M14 15l3 3 3-3" />
                  </svg>
                  <!-- Grip icon for manual mode -->
                  <svg v-else class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="9" cy="6" r="1" fill="currentColor" /><circle cx="15" cy="6" r="1" fill="currentColor" />
                    <circle cx="9" cy="12" r="1" fill="currentColor" /><circle cx="15" cy="12" r="1" fill="currentColor" />
                    <circle cx="9" cy="18" r="1" fill="currentColor" /><circle cx="15" cy="18" r="1" fill="currentColor" />
                  </svg>
                </button>
              </TooltipTrigger>
              <TooltipContent>
                {{ sortMode === 'alpha' ? 'A-Z (click for Z-A)' : sortMode === 'alpha-desc' ? 'Z-A (click for manual)' : 'Manual (click for A-Z)' }}
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </template>
        <!-- Reset to A-Z (hidden when collapsed) -->
        <template v-if="hasReordered && !isProjectsCollapsed">
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger as-child>
                <button
                  class="text-muted-foreground hover:text-foreground transition-colors p-0.5 rounded"
                  @click.stop="resetSortOrder"
                >
                  <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M3 12a9 9 0 1 0 9-9" /><polyline points="3 3 3 7 7 7" /><path d="M3 3l4 4" />
                  </svg>
                </button>
              </TooltipTrigger>
              <TooltipContent>Reset to A-Z</TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </template>
      </div>
      <div v-show="!isProjectsCollapsed" ref="projectsListRef" class="flex flex-col gap-1">
        <div v-for="proj in sortedProjects" :key="proj.path" :data-path="proj.path" class="relative group">
          <Button
            :variant="beadsPath === proj.path ? 'default' : 'ghost'"
            size="sm"
            class="h-7 justify-start text-xs gap-0 w-full pr-6"
            :class="{ 'opacity-50 cursor-wait': isLoading && beadsPath !== proj.path }"
            :disabled="isLoading"
            @click="handleSelectProject(proj.path)"
          >
            <!-- Drag handle -->
            <span
              v-if="!isLoading"
              class="drag-handle cursor-grab active:cursor-grabbing opacity-0 group-hover:opacity-60 transition-opacity mr-1 shrink-0"
            >
              <svg class="w-2.5 h-3" viewBox="0 0 24 24" fill="currentColor">
                <circle cx="8" cy="4" r="2" /><circle cx="16" cy="4" r="2" />
                <circle cx="8" cy="12" r="2" /><circle cx="16" cy="12" r="2" />
                <circle cx="8" cy="20" r="2" /><circle cx="16" cy="20" r="2" />
              </svg>
            </span>
            <!-- Loading spinner for active project -->
            <svg
              v-if="isLoading && beadsPath === proj.path"
              class="w-3 h-3 shrink-0 animate-spin mr-1"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <circle cx="12" cy="12" r="10" stroke-opacity="0.25" />
              <path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round" />
            </svg>
            <!-- Probe expose toggle (external mode) -->
            <button
              v-if="probeEnabled"
              class="shrink-0 mr-1 p-0 rounded transition-colors"
              :class="isExposed(proj.path)
                ? 'text-green-500'
                : 'text-muted-foreground/40 hover:text-green-500'"
              :disabled="togglingPath === proj.path"
              @click.stop.prevent="requestToggleExpose(proj.path)"
              :title="isExposed(proj.path) ? 'Exposed to monitoring (click to disable)' : 'Not exposed (click to expose)'"
            >
              <svg v-if="togglingPath === proj.path" class="w-3.5 h-3.5 animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10" stroke-opacity="0.25" />
                <path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round" />
              </svg>
              <svg v-else class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="10" />
                <line x1="2" y1="12" x2="22" y2="12" />
                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
              </svg>
            </button>
            <!-- Folder icon (when probe disabled) -->
            <svg
              v-else
              class="w-3 h-3 shrink-0 mr-1 text-muted-foreground"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            </svg>
            <span class="truncate flex-1 text-left">{{ proj.name }}</span>
          </Button>
          <!-- Remove button - outside Button to avoid click capture -->
          <button
            v-if="!isLoading"
            class="absolute right-0.5 top-1/2 -translate-y-1/2 p-1 rounded opacity-0 group-hover:opacity-100 transition-opacity text-muted-foreground hover:text-destructive"
            @click.stop.prevent="handleRemoveProject(proj.path, $event)"
          >
            <svg
              class="w-3.5 h-3.5"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Folder Picker Dialog -->
    <FolderPicker
      v-model:open="isPickerOpen"
      :current-path="beadsPath"
      @select="handleSelectFolder"
    />

    <!-- Remove Project Confirmation Dialog -->
    <ConfirmDialog
      v-model:open="isRemoveDialogOpen"
      title="Remove project"
      :description="`Are you sure you want to remove '${projectToRemoveName}' from your projects?`"
      confirm-text="Remove"
      cancel-text="Cancel"
      variant="destructive"
      @confirm="confirmRemoveProject"
    />

    <!-- Probe Expose Confirmation Dialog -->
    <ConfirmDialog
      v-model:open="isExposeDialogOpen"
      :title="exposeDialogTitle"
      :description="exposeDialogDescription"
      confirm-text="Confirm"
      cancel-text="Cancel"
      @confirm="confirmToggleExpose"
    />
  </div>
</template>
