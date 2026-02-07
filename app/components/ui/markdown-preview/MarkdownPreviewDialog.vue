<script setup lang="ts">
import { ChevronLeft, ChevronRight, Pencil, Save, X } from 'lucide-vue-next'
import {
  Dialog,
  DialogContent,
  DialogTitle,
} from '~/components/ui/dialog'
import { Button } from '~/components/ui/button'
import { ScrollArea } from '~/components/ui/scroll-area'
import { renderMarkdown } from '~/utils/markdown'
import { openUrl } from '~/utils/open-url'

const props = defineProps<{
  open: boolean
  markdownContent: string
  markdownTitle: string
  isLoading?: boolean
  hasMultipleFiles?: boolean
  canGoNext?: boolean
  canGoPrev?: boolean
  fileCounter?: string
  isEditMode?: boolean
  editedContent?: string
  isSaving?: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  next: []
  prev: []
  'toggle-edit': []
  save: []
  'cancel-edit': []
  'update:editedContent': [value: string]
}>()

const renderedHtml = computed(() => renderMarkdown(props.markdownContent))

// Handle link clicks inside rendered markdown
const handleContentClick = (e: MouseEvent) => {
  const target = (e.target as HTMLElement).closest('[data-external-link]') as HTMLAnchorElement | null
  if (target?.href) {
    e.preventDefault()
    openUrl(target.href)
  }
}

// Ref for the contenteditable element
const editorRef = ref<HTMLElement | null>(null)

// Set initial content once when entering edit mode (no reactive v-text binding)
watch(() => props.isEditMode, async (editing) => {
  if (editing) {
    await nextTick()
    if (editorRef.value) {
      editorRef.value.textContent = props.editedContent ?? ''
    }
  }
})

// Handle contenteditable input — update parent without re-rendering the div
const handleEditInput = (e: Event) => {
  const target = e.target as HTMLElement
  emit('update:editedContent', target.innerText)
}

// Keyboard navigation (disabled in edit mode)
const handleKeydown = (e: KeyboardEvent) => {
  if (!props.open || props.isEditMode) return

  if (e.key === 'ArrowRight' && props.canGoNext) {
    e.preventDefault()
    emit('next')
  } else if (e.key === 'ArrowLeft' && props.canGoPrev) {
    e.preventDefault()
    emit('prev')
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="!w-[min(68rem,calc(100vw-6rem))] !h-[calc(100vh-5rem)] !max-w-none p-4 flex flex-col">
      <div class="flex items-center gap-2 mb-2">
        <DialogTitle class="text-sm font-medium">{{ markdownTitle }}</DialogTitle>
        <span v-if="hasMultipleFiles && !isEditMode" class="text-xs text-muted-foreground">
          {{ fileCounter }}
        </span>
        <div class="ml-auto mr-8 flex items-center gap-1.5">
          <template v-if="isEditMode">
            <Button
              size="sm"
              class="h-7 gap-1.5 text-xs"
              :disabled="isSaving"
              @click="emit('save')"
            >
              <Save class="w-3.5 h-3.5" />
              {{ isSaving ? 'Saving...' : 'Save' }}
            </Button>
            <Button
              variant="outline"
              size="sm"
              class="h-7 gap-1.5 text-xs"
              :disabled="isSaving"
              @click="emit('cancel-edit')"
            >
              <X class="w-3.5 h-3.5" />
              Cancel
            </Button>
          </template>
          <Button
            v-else
            variant="outline"
            size="sm"
            class="h-7 gap-1.5 text-xs"
            @click="emit('toggle-edit')"
          >
            <Pencil class="w-3.5 h-3.5" />
            Edit
          </Button>
        </div>
      </div>
      <div class="relative flex-1 min-h-0 flex">
        <!-- Previous button (hidden in edit mode) -->
        <Button
          v-if="hasMultipleFiles && !isEditMode"
          variant="outline"
          size="icon"
          class="absolute left-2 top-1/2 -translate-y-1/2 z-10 w-10 h-10 rounded-full bg-background/80 border border-border shadow-md hover:bg-background disabled:opacity-30 disabled:cursor-not-allowed"
          :disabled="!canGoPrev"
          @click="emit('prev')"
        >
          <ChevronLeft class="w-5 h-5" />
        </Button>

        <ScrollArea class="flex-1 min-h-0">
          <div v-if="isLoading" class="flex items-center justify-center h-32 text-muted-foreground">
            Loading...
          </div>
          <!-- Edit mode: contentEditable with raw markdown -->
          <div
            v-else-if="isEditMode"
            ref="editorRef"
            contenteditable="true"
            class="max-w-5xl mx-auto px-8 py-4 font-mono text-sm whitespace-pre-wrap outline-none min-h-full"
            @input="handleEditInput"
          />
          <!-- Read mode: rendered HTML -->
          <div
            v-else
            class="markdown-reader max-w-5xl mx-auto px-8 py-4"
            :class="{ 'px-18': hasMultipleFiles }"
            @click="handleContentClick"
            v-html="renderedHtml"
          />
        </ScrollArea>

        <!-- Next button (hidden in edit mode) -->
        <Button
          v-if="hasMultipleFiles && !isEditMode"
          variant="outline"
          size="icon"
          class="absolute right-2 top-1/2 -translate-y-1/2 z-10 w-10 h-10 rounded-full bg-background/80 border border-border shadow-md hover:bg-background disabled:opacity-30 disabled:cursor-not-allowed"
          :disabled="!canGoNext"
          @click="emit('next')"
        >
          <ChevronRight class="w-5 h-5" />
        </Button>
      </div>
    </DialogContent>
  </Dialog>
</template>
