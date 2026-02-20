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

const props = defineProps<{
  isLoading?: boolean
}>()

const { beadsPath, setPath, clearPath, isCustomPath } = useBeadsPath()
const { favorites, sortedFavorites, sortMode, hasReordered, addFavorite, removeFavorite, isFavorite, reorderFavorites, setSortMode, resetSortOrder } = useFavorites()

const favoritesListRef = ref<HTMLElement | null>(null)
let sortableInstance: Sortable | null = null

// Apply counter-zoom on favorites list before SortableJS calculates coordinates
const resetZoomOnPointerDown = (e: PointerEvent) => {
  const target = e.target as HTMLElement
  if (!target.closest('.drag-handle') || !favoritesListRef.value) return
  const zoomableContent = document.getElementById('zoomable-content')
  const parentZoom = parseFloat(zoomableContent?.style.zoom || '100')
  if (parentZoom !== 100) {
    favoritesListRef.value.style.zoom = `${10000 / parentZoom}%`
  }
}

const restoreZoom = () => {
  if (favoritesListRef.value) {
    favoritesListRef.value.style.zoom = ''
  }
}

const initSortable = () => {
  if (sortableInstance) {
    sortableInstance.destroy()
    sortableInstance = null
  }
  if (!favoritesListRef.value) return

  // Listen on pointerdown to reset zoom before SortableJS kicks in
  favoritesListRef.value.addEventListener('pointerdown', resetZoomOnPointerDown)
  // Restore zoom if user releases without dragging
  favoritesListRef.value.addEventListener('pointerup', restoreZoom)

  sortableInstance = Sortable.create(favoritesListRef.value, {
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
      // Build reordered favorites array from paths
      const currentFavs = sortedFavorites.value
      const reordered = newOrder
        .map(path => currentFavs.find(f => f.path === path))
        .filter((f): f is NonNullable<typeof f> => !!f)
      if (reordered.length === currentFavs.length) {
        reorderFavorites(reordered)
      }
    },
  })
}

const isFavoritesCollapsed = useLocalStorage('beads:favoritesCollapsed', false)

onMounted(initSortable)
onBeforeUnmount(() => {
  favoritesListRef.value?.removeEventListener('pointerdown', resetZoomOnPointerDown)
  favoritesListRef.value?.removeEventListener('pointerup', restoreZoom)
  sortableInstance?.destroy()
})

// Re-init when list becomes visible, update disabled state
watch(() => props.isLoading, () => {
  if (sortableInstance) {
    sortableInstance.option('disabled', !!props.isLoading)
  }
})

// Re-init sortable when favorites list becomes visible or content changes
watch([isFavoritesCollapsed, () => favorites.value.length], () => {
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
const favoriteToRemove = ref<string | null>(null)

// Expose isPickerOpen to parent components
defineExpose({ isPickerOpen })

const handleSelectFolder = (path: string) => {
  setPath(path)
  emit('change')
}

const handleSelectFavorite = (path: string) => {
  // Don't allow switching while loading
  if (props.isLoading) return
  setPath(path)
  emit('change')
}

const handleToggleFavorite = () => {
  if (isFavorite(beadsPath.value)) {
    // Show confirmation dialog
    favoriteToRemove.value = beadsPath.value
    isRemoveDialogOpen.value = true
  } else {
    addFavorite(beadsPath.value)
  }
}

const handleRemoveFavorite = (path: string, event: Event) => {
  event.stopPropagation()
  // Show confirmation dialog
  favoriteToRemove.value = path
  isRemoveDialogOpen.value = true
}

const confirmRemoveFavorite = () => {
  if (!favoriteToRemove.value) return

  const pathToRemove = favoriteToRemove.value
  const isCurrentPath = beadsPath.value === pathToRemove
  const isLastFavorite = favorites.value.length === 1

  // Remove the favorite
  removeFavorite(pathToRemove)

  // If we removed the current path
  if (isCurrentPath) {
    if (isLastFavorite) {
      // Last favorite removed - clear path to show onboarding
      clearPath()
      emit('reset')
    } else {
      // Switch to another favorite
      const remainingFavorite = favorites.value.find(f => f.path !== pathToRemove)
      if (remainingFavorite) {
        setPath(remainingFavorite.path)
        emit('change')
      }
    }
  }

  // Close dialog
  isRemoveDialogOpen.value = false
  favoriteToRemove.value = null
}

const favoriteToRemoveName = computed(() => {
  if (!favoriteToRemove.value) return ''
  return getFolderName(favoriteToRemove.value)
})

const currentIsFavorite = computed(() => isFavorite(beadsPath.value))
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

      <template v-if="isCustomPath">
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger as-child>
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7"
                @click="handleToggleFavorite"
              >
                <svg
                  class="w-3 h-3"
                  :class="currentIsFavorite ? 'text-yellow-500 fill-yellow-500' : 'text-muted-foreground'"
                  viewBox="0 0 24 24"
                  :fill="currentIsFavorite ? 'currentColor' : 'none'"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
                </svg>
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              {{ currentIsFavorite ? 'Remove selected from favorites' : 'Add to favorites' }}
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      </template>

    </div>

    <!-- Favorites -->
    <div v-if="favorites.length > 0" class="space-y-1">
      <div class="flex items-center gap-2 w-full">
        <button
          class="flex items-center gap-2 text-[10px] text-muted-foreground hover:text-foreground transition-colors flex-1"
          @click="isFavoritesCollapsed = !isFavoritesCollapsed"
        >
          <svg
            class="w-3 h-3 transition-transform"
            :class="{ '-rotate-90': isFavoritesCollapsed }"
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
          <span class="uppercase tracking-wide">Favorites</span>
          <span class="ml-auto">({{ favorites.length }})</span>
        </button>
        <!-- Sort mode toggle (hidden when collapsed) -->
        <template v-if="!isFavoritesCollapsed">
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
        <template v-if="hasReordered && !isFavoritesCollapsed">
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
      <div v-show="!isFavoritesCollapsed" ref="favoritesListRef" class="flex flex-col gap-1">
        <div v-for="fav in sortedFavorites" :key="fav.path" :data-path="fav.path" class="relative group">
          <Button
            :variant="beadsPath === fav.path ? 'default' : 'ghost'"
            size="sm"
            class="h-7 justify-start text-xs gap-0 w-full pr-6"
            :class="{ 'opacity-50 cursor-wait': isLoading && beadsPath !== fav.path }"
            :disabled="isLoading"
            @click="handleSelectFavorite(fav.path)"
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
            <!-- Loading spinner for active favorite -->
            <svg
              v-if="isLoading && beadsPath === fav.path"
              class="w-3 h-3 shrink-0 animate-spin mr-1"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <circle cx="12" cy="12" r="10" stroke-opacity="0.25" />
              <path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round" />
            </svg>
            <!-- Star icon when not loading -->
            <svg
              v-else
              class="w-3 h-3 shrink-0 mr-1"
              :class="beadsPath === fav.path ? 'text-yellow-300 fill-yellow-300' : 'text-yellow-500 fill-yellow-500'"
              viewBox="0 0 24 24"
            >
              <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
            </svg>
            <span class="truncate flex-1 text-left">{{ fav.name }}</span>
          </Button>
          <!-- Remove button - outside Button to avoid click capture -->
          <button
            v-if="!isLoading"
            class="absolute right-0.5 top-1/2 -translate-y-1/2 p-1 rounded opacity-0 group-hover:opacity-100 transition-opacity text-muted-foreground hover:text-destructive"
            @click.stop.prevent="handleRemoveFavorite(fav.path, $event)"
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

    <!-- Remove Favorite Confirmation Dialog -->
    <ConfirmDialog
      v-model:open="isRemoveDialogOpen"
      title="Remove from favorites"
      :description="`Are you sure you want to remove '${favoriteToRemoveName}' from your favorites?`"
      confirm-text="Remove"
      cancel-text="Cancel"
      variant="destructive"
      @confirm="confirmRemoveFavorite"
    />
  </div>
</template>
