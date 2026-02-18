<script setup lang="ts">
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog'
import { Button } from '~/components/ui/button'
import { Input } from '~/components/ui/input'
import { ScrollArea } from '~/components/ui/scroll-area'
import { Badge } from '~/components/ui/badge'
import { Separator } from '~/components/ui/separator'
import { fsList, type DirectoryEntry } from '~/utils/bd-api'
import { getParentPath, getFolderName } from '~/utils/path'

const props = defineProps<{
  open: boolean
  currentPath: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  select: [path: string]
}>()

const { favorites, addFavorite, isFavorite } = useFavorites()

const currentPath = ref(props.currentPath || '~')
const pathInput = ref('')
const entries = ref<DirectoryEntry[]>([])
const hasBeads = ref(false)
const usesDolt = ref(false)
const isLoading = ref(false)
const error = ref<string | null>(null)

// Watch for dialog open to load initial path
watch(() => props.open, (isOpen) => {
  if (isOpen) {
    // Use home if currentPath is empty or "."
    const initialPath = props.currentPath && props.currentPath !== '.' ? props.currentPath : '~'
    currentPath.value = initialPath
    loadDirectory(initialPath)
  }
})

// Watch currentPath to update input (immediate so pathInput is set on mount)
watch(currentPath, (path) => {
  pathInput.value = path
}, { immediate: true })

// Watch prop changes so opening from a different favorite updates the path
watch(() => props.currentPath, (newPath) => {
  if (props.open && newPath && newPath !== '.') {
    currentPath.value = newPath
    loadDirectory(newPath)
  }
})

const loadDirectory = async (path: string) => {
  isLoading.value = true
  error.value = null

  try {
    const data = await fsList(path)
    currentPath.value = data.currentPath
    entries.value = data.entries
    hasBeads.value = data.hasBeads
    usesDolt.value = data.usesDolt
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load directory'
    entries.value = []
    hasBeads.value = false
    usesDolt.value = false
  } finally {
    isLoading.value = false
  }
}

const navigateTo = (path: string) => {
  loadDirectory(path)
}

const navigateUp = () => {
  loadDirectory(getParentPath(currentPath.value))
}

const navigateHome = () => {
  loadDirectory('~')
}

const handlePathInput = () => {
  if (pathInput.value) {
    loadDirectory(pathInput.value)
  }
}

const handleSelect = () => {
  emit('select', currentPath.value)
  emit('update:open', false)
}

const handleCancel = () => {
  emit('update:open', false)
}

const handleAddToFavorites = () => {
  addFavorite(currentPath.value)
  // Also select the folder and close the dialog for better UX
  emit('select', currentPath.value)
  emit('update:open', false)
}

// Get folder name from path
const currentFolderName = computed(() => {
  return getFolderName(currentPath.value) || '/'
})

const isCurrentFavorite = computed(() => isFavorite(currentPath.value))
</script>

<template>
  <Dialog :open="open" @update:open="$emit('update:open', $event)">
    <DialogContent class="max-w-4xl max-h-[80vh] flex flex-col">
      <DialogHeader>
        <DialogTitle>Select Beads Project Folder</DialogTitle>
        <DialogDescription>
          Navigate to a folder containing a Beads database (.beads folder)
        </DialogDescription>
      </DialogHeader>

      <div class="flex-1 flex flex-col gap-4 overflow-hidden">
        <!-- Navigation bar -->
        <div class="flex items-center gap-2">
          <Button variant="outline" size="icon" class="shrink-0" @click="navigateHome" title="Home">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
              <polyline points="9 22 9 12 15 12 15 22" />
            </svg>
          </Button>
          <Button variant="outline" size="icon" class="shrink-0" @click="navigateUp" title="Parent folder">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="18 15 12 9 6 15" />
            </svg>
          </Button>
          <div class="flex-1 flex items-center gap-2">
            <Input
              v-model="pathInput"
              dir="rtl"
              class="flex-1 font-mono text-sm h-9"
              placeholder="/path/to/folder"
              @keyup.enter="handlePathInput"
            />
            <Button variant="outline" size="icon" class="shrink-0" @click="handlePathInput" title="Go">
              <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="9 18 15 12 9 6" />
              </svg>
            </Button>
          </div>
        </div>

        <Separator />

        <!-- Current selection info -->
        <div class="flex items-center justify-between gap-4 px-1">
          <div class="flex items-center gap-2 min-w-0">
            <svg
              class="w-5 h-5 shrink-0"
              :class="hasBeads ? 'text-green-500' : 'text-muted-foreground'"
              viewBox="0 0 24 24"
              fill="currentColor"
            >
              <path d="M20 6h-8l-2-2H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2z" />
            </svg>
            <span class="font-medium truncate">{{ currentFolderName }}</span>
          </div>
          <div class="flex items-center gap-2">
            <Badge v-if="hasBeads" class="bg-green-600 text-white shrink-0">
              Beads Project
            </Badge>
            <Badge v-if="usesDolt" variant="outline" class="text-[#29E3C1] border-[#29E3C1]/50 shrink-0 px-1.5">
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
            </Badge>
            <Badge v-if="isCurrentFavorite" variant="outline" class="text-yellow-500 border-yellow-500/50">
              <svg class="w-3 h-3 mr-1 fill-yellow-500" viewBox="0 0 24 24">
                <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
              </svg>
              Favorite
            </Badge>
          </div>
        </div>

        <!-- Error message -->
        <div v-if="error" class="px-3 py-2 text-sm text-destructive bg-destructive/10 rounded">
          {{ error }}
        </div>

        <!-- Directory listing -->
        <div class="flex-1 min-h-0 border border-border rounded-md overflow-hidden">
          <ScrollArea class="h-[280px]">
            <div v-if="isLoading" class="flex items-center justify-center py-12">
              <span class="text-muted-foreground">Loading...</span>
            </div>

            <div v-else-if="entries.length === 0" class="flex items-center justify-center py-12">
              <span class="text-muted-foreground">No subfolders</span>
            </div>

            <div v-else class="divide-y divide-border">
              <button
                v-for="entry in entries"
                :key="entry.path"
                class="w-full flex items-center gap-3 px-4 py-3 hover:bg-secondary/50 transition-colors text-left"
                @click="navigateTo(entry.path)"
              >
                <svg
                  class="w-5 h-5 shrink-0"
                  :class="entry.hasBeads ? 'text-green-500' : 'text-muted-foreground'"
                  viewBox="0 0 24 24"
                  :fill="entry.hasBeads ? 'currentColor' : 'none'"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
                </svg>
                <span class="flex-1 truncate">{{ entry.name }}</span>
                <Badge v-if="entry.hasBeads" variant="outline" class="text-green-500 border-green-500/50 text-xs">
                  beads
                </Badge>
                <Badge v-if="entry.usesDolt" variant="outline" class="text-[#29E3C1] border-[#29E3C1]/50 text-xs px-1">
                  <svg class="w-6 h-2.5" viewBox="0 0 163 56" fill="none">
                    <path d="M28.87 7.0459V45.8632C28.8654 46.7997 28.498 47.6965 27.8476 48.3591C27.1971 49.0217 26.316 49.3964 25.3957 49.402H10.4953C9.5713 49.402 8.68489 49.0298 8.0299 48.3666C7.3749 47.7035 7.00462 46.8034 7 45.8632V24.7722C7.00462 23.832 7.3749 22.9319 8.0299 22.2688C8.68489 21.6056 9.5713 21.2334 10.4953 21.2334H22.2115" stroke="currentColor" stroke-width="12.6599" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M156.3 49.4019H145.283" stroke="currentColor" stroke-width="12.6599" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M156.026 21.5259H134.174" stroke="currentColor" stroke-width="12.6599" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M145.336 7.0498V49.4024" stroke="currentColor" stroke-width="12.6599" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M72.2752 7.68311H59.049C56.6669 7.68311 54.7358 9.64808 54.7358 12.072V44.8074C54.7358 47.2313 56.6669 49.1963 59.049 49.1963H72.2752C74.6573 49.1963 76.5884 47.2313 76.5884 44.8074V12.072C76.5884 9.64808 74.6573 7.68311 72.2752 7.68311Z" stroke="currentColor" stroke-width="12.6599" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M119.586 49.4019H99.418" stroke="currentColor" stroke-width="12.6599" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M110.344 7.0498V49.4024" stroke="currentColor" stroke-width="12.6599" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M109.884 7H98.7939" stroke="currentColor" stroke-width="12.6599" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                </Badge>
                <svg class="w-4 h-4 text-muted-foreground shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="9 18 15 12 9 6" />
                </svg>
              </button>
            </div>
          </ScrollArea>
        </div>
      </div>

      <DialogFooter class="mt-4">
        <Button variant="outline" @click="handleCancel">
          Cancel
        </Button>

        <!-- If already a favorite, show "Open" button -->
        <Button v-if="isCurrentFavorite" @click="handleSelect">
          <svg class="w-4 h-4 mr-2" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
          </svg>
          Open
        </Button>

        <!-- If not a favorite, show "Add to Favorites" button (disabled if no .beads) -->
        <Button v-else :disabled="!hasBeads" @click="handleAddToFavorites">
          <svg
            class="w-4 h-4 mr-2"
            :class="hasBeads ? 'text-yellow-500' : ''"
            viewBox="0 0 24 24"
            :fill="hasBeads ? 'currentColor' : 'none'"
            stroke="currentColor"
            stroke-width="2"
          >
            <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
          </svg>
          Add to Favorites
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
