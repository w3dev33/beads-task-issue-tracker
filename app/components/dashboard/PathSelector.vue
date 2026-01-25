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

const props = defineProps<{
  isLoading?: boolean
}>()

const { beadsPath, setPath, clearPath, isCustomPath } = useBeadsPath()
const { favorites, addFavorite, removeFavorite, isFavorite } = useFavorites()

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
  return favoriteToRemove.value.split('/').pop() || favoriteToRemove.value
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
        Browse
      </Button>

      <TooltipProvider v-if="isCustomPath">
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

    </div>

    <!-- Favorites -->
    <div v-if="favorites.length > 0" class="space-y-1">
      <span class="text-[10px] text-muted-foreground uppercase tracking-wide">Favorites</span>
      <div class="flex flex-col gap-1">
        <TooltipProvider v-for="fav in favorites" :key="fav.path">
          <Tooltip>
            <TooltipTrigger as-child>
              <Button
                :variant="beadsPath === fav.path ? 'default' : 'ghost'"
                size="sm"
                class="h-7 justify-start text-xs gap-2 group w-full"
                :class="{ 'opacity-50 cursor-wait': isLoading && beadsPath !== fav.path }"
                :disabled="isLoading"
                @click="handleSelectFavorite(fav.path)"
              >
                <!-- Loading spinner for active favorite -->
                <svg
                  v-if="isLoading && beadsPath === fav.path"
                  class="w-3 h-3 shrink-0 animate-spin"
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
                  class="w-3 h-3 shrink-0"
                  :class="beadsPath === fav.path ? 'text-yellow-300 fill-yellow-300' : 'text-yellow-500 fill-yellow-500'"
                  viewBox="0 0 24 24"
                >
                  <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
                </svg>
                <span class="truncate flex-1 text-left">{{ fav.name }}</span>
                <svg
                  v-if="!isLoading"
                  class="w-3 h-3 shrink-0 text-muted-foreground hover:text-destructive opacity-0 group-hover:opacity-100 transition-opacity"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  @click="handleRemoveFavorite(fav.path, $event)"
                >
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </Button>
            </TooltipTrigger>
            <TooltipContent side="right">
              {{ fav.path }}
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
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
