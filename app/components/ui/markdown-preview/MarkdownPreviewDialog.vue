<script setup lang="ts">
import { ChevronLeft, ChevronRight, ChevronUp, ChevronDown, Pencil, Save, Search, X } from 'lucide-vue-next'
import {
  Dialog,
  DialogContent,
  DialogTitle,
} from '~/components/ui/dialog'
import { Button } from '~/components/ui/button'
import { Badge } from '~/components/ui/badge'
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

// --- Search state ---
const isSearchOpen = ref(false)
const searchQuery = ref('')
const matchCount = ref(0)
const currentMatchIndex = ref(0)
const searchInputRef = ref<HTMLInputElement | null>(null)
const previewContentRef = ref<HTMLElement | null>(null)

let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null

function getSearchContainer(): HTMLElement | null {
  if (props.isEditMode) return editorRef.value
  return previewContentRef.value
}

function escapeRegExp(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

function clearHighlights(container: HTMLElement | null) {
  if (!container) return
  const marks = container.querySelectorAll('mark.search-highlight')
  marks.forEach((mark) => {
    const parent = mark.parentNode
    if (parent) {
      parent.replaceChild(document.createTextNode(mark.textContent || ''), mark)
      parent.normalize()
    }
  })
}

function performSearch() {
  const container = getSearchContainer()
  if (!container) return

  // Clear existing highlights
  clearHighlights(container)
  matchCount.value = 0
  currentMatchIndex.value = 0

  const query = searchQuery.value.trim()
  if (!query) return

  const escapedQuery = escapeRegExp(query)
  const regex = new RegExp(escapedQuery, 'gi')

  // Collect text nodes via TreeWalker
  const walker = document.createTreeWalker(container, NodeFilter.SHOW_TEXT, null)
  const textNodes: Text[] = []
  let node: Text | null
  while ((node = walker.nextNode() as Text | null)) {
    textNodes.push(node)
  }

  let totalMatches = 0

  // Process text nodes in reverse to avoid invalidating earlier references
  for (let i = textNodes.length - 1; i >= 0; i--) {
    const textNode = textNodes[i]!
    const text = textNode.nodeValue || ''
    const matches: { start: number; end: number }[] = []
    let match: RegExpExecArray | null

    regex.lastIndex = 0
    while ((match = regex.exec(text)) !== null) {
      matches.push({ start: match.index, end: match.index + match[0].length })
    }

    if (matches.length === 0) continue

    totalMatches += matches.length
    const parent = textNode.parentNode
    if (!parent) continue

    const fragment = document.createDocumentFragment()
    let lastIndex = 0

    for (const m of matches) {
      // Text before match
      if (m.start > lastIndex) {
        fragment.appendChild(document.createTextNode(text.slice(lastIndex, m.start)))
      }
      // Highlighted match
      const mark = document.createElement('mark')
      mark.className = 'search-highlight'
      mark.textContent = text.slice(m.start, m.end)
      fragment.appendChild(mark)
      lastIndex = m.end
    }

    // Remaining text after last match
    if (lastIndex < text.length) {
      fragment.appendChild(document.createTextNode(text.slice(lastIndex)))
    }

    parent.replaceChild(fragment, textNode)
  }

  matchCount.value = totalMatches

  if (totalMatches > 0) {
    currentMatchIndex.value = 1
    highlightCurrentMatch()
  }
}

function highlightCurrentMatch() {
  const container = getSearchContainer()
  if (!container || matchCount.value === 0) return

  const marks = container.querySelectorAll('mark.search-highlight')

  // Remove current highlight from all
  marks.forEach(m => m.classList.remove('search-highlight-current'))

  // Add current highlight
  const idx = currentMatchIndex.value - 1
  const currentMark = marks[idx]
  if (idx >= 0 && currentMark) {
    currentMark.classList.add('search-highlight-current')
    currentMark.scrollIntoView({ behavior: 'smooth', block: 'center' })
  }
}

function goToNextMatch() {
  if (matchCount.value === 0) return
  currentMatchIndex.value = currentMatchIndex.value >= matchCount.value ? 1 : currentMatchIndex.value + 1
  highlightCurrentMatch()
}

function goToPrevMatch() {
  if (matchCount.value === 0) return
  currentMatchIndex.value = currentMatchIndex.value <= 1 ? matchCount.value : currentMatchIndex.value - 1
  highlightCurrentMatch()
}

function openSearch() {
  isSearchOpen.value = true
  nextTick(() => {
    searchInputRef.value?.focus()
    searchInputRef.value?.select()
  })
}

function closeSearch() {
  isSearchOpen.value = false
  searchQuery.value = ''
  matchCount.value = 0
  currentMatchIndex.value = 0
  clearHighlights(getSearchContainer())
}

function toggleSearch() {
  if (isSearchOpen.value) {
    closeSearch()
  } else {
    openSearch()
  }
}

// Debounced search on query change
watch(searchQuery, () => {
  if (searchDebounceTimer) clearTimeout(searchDebounceTimer)
  searchDebounceTimer = setTimeout(() => {
    performSearch()
  }, 200)
})

// Re-search when content changes (e.g. file navigation while search is open)
watch(() => props.markdownContent, () => {
  if (isSearchOpen.value && searchQuery.value) {
    nextTick(() => performSearch())
  }
})

// Clear search on mode switch
watch(() => props.isEditMode, () => {
  if (isSearchOpen.value) {
    // Clear highlights in old container before mode switches
    clearHighlights(getSearchContainer())
    matchCount.value = 0
    currentMatchIndex.value = 0
    // Re-search in new container after mode switch renders
    nextTick(() => {
      if (searchQuery.value) {
        performSearch()
      }
    })
  }
})

// Clear search when modal closes
watch(() => props.open, (isOpen) => {
  if (!isOpen) {
    closeSearch()
  }
})

// Keyboard navigation
const handleKeydown = (e: KeyboardEvent) => {
  if (!props.open) return

  // Cmd/Ctrl+F: toggle search
  if ((e.metaKey || e.ctrlKey) && e.key === 'f') {
    e.preventDefault()
    toggleSearch()
    return
  }

  // When search input is focused
  if (isSearchOpen.value && document.activeElement === searchInputRef.value) {
    if (e.key === 'Enter' && e.shiftKey) {
      e.preventDefault()
      goToPrevMatch()
      return
    }
    if (e.key === 'Enter') {
      e.preventDefault()
      goToNextMatch()
      return
    }
    if (e.key === 'Escape') {
      e.preventDefault()
      closeSearch()
      return
    }
  }

  // Escape: close search first, then modal
  if (e.key === 'Escape' && isSearchOpen.value) {
    e.preventDefault()
    closeSearch()
    return
  }

  // Arrow navigation (only in preview mode, not when search input focused)
  if (props.isEditMode) return
  if (document.activeElement === searchInputRef.value) return

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
  if (searchDebounceTimer) clearTimeout(searchDebounceTimer)
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
          <!-- Search toggle button -->
          <Button
            variant="outline"
            size="sm"
            class="h-7 w-7 p-0"
            :class="{ 'bg-accent': isSearchOpen }"
            @click="toggleSearch"
          >
            <Search class="w-3.5 h-3.5" />
          </Button>
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

      <!-- Collapsible search bar -->
      <Transition name="search-slide">
        <div v-if="isSearchOpen" class="flex items-center gap-2 mb-2 px-1">
          <div class="relative flex-1 max-w-md">
            <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground" />
            <input
              ref="searchInputRef"
              v-model="searchQuery"
              type="text"
              placeholder="Search in document..."
              class="w-full h-8 pl-8 pr-3 text-sm rounded-md border border-input bg-background text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
            >
          </div>
          <Badge
            v-if="searchQuery.trim()"
            variant="secondary"
            class="text-xs tabular-nums whitespace-nowrap"
          >
            {{ matchCount > 0 ? `${currentMatchIndex} / ${matchCount}` : '0 / 0' }}
          </Badge>
          <Button
            variant="outline"
            size="sm"
            class="h-7 w-7 p-0"
            :disabled="matchCount === 0"
            @click="goToPrevMatch"
          >
            <ChevronUp class="w-3.5 h-3.5" />
          </Button>
          <Button
            variant="outline"
            size="sm"
            class="h-7 w-7 p-0"
            :disabled="matchCount === 0"
            @click="goToNextMatch"
          >
            <ChevronDown class="w-3.5 h-3.5" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            class="h-7 w-7 p-0"
            @click="closeSearch"
          >
            <X class="w-3.5 h-3.5" />
          </Button>
        </div>
      </Transition>

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
            ref="previewContentRef"
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
