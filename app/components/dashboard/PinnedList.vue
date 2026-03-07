<script setup lang="ts">
import type { Issue } from '~/types/issue'
import { ScrollArea } from '~/components/ui/scroll-area'
import TypeBadge from '~/components/issues/TypeBadge.vue'
import PriorityBadge from '~/components/issues/PriorityBadge.vue'
import Sortable from 'sortablejs'
import { useKeyboardNavigation } from '~/composables/useKeyboardNavigation'

const props = withDefaults(defineProps<{
  issues: Issue[]
  dragEnabled?: boolean
}>(), {
  dragEnabled: true,
})

const emit = defineEmits<{
  select: [issue: Issue]
  reorder: [newOrder: string[]]
  unpin: [issueId: string]
}>()

const pinnedItemIds = computed(() => props.issues.map(i => i.id))
const issueMap = computed(() => new Map(props.issues.map(i => [i.id, i])))

const { focusedId, setFocused, handleKeydown, isFocused } = useKeyboardNavigation({
  itemIds: pinnedItemIds,
  onSelect: (id) => {
    const issue = issueMap.value.get(id)
    if (issue) emit('select', issue)
  },
})

const listRef = ref<HTMLElement | null>(null)
let sortableInstance: Sortable | null = null

const getShortId = (id: string) => {
  const dotIndex = id.indexOf('.')
  const baseId = dotIndex > 0 ? id.slice(0, dotIndex) : id
  const indexSuffix = dotIndex > 0 ? id.slice(dotIndex) : ''
  const lastHyphen = baseId.lastIndexOf('-')
  if (lastHyphen > 0) {
    return baseId.slice(lastHyphen + 1) + indexSuffix
  }
  return id
}

const initSortable = () => {
  if (sortableInstance) {
    sortableInstance.destroy()
    sortableInstance = null
  }
  if (!listRef.value || !props.dragEnabled) return

  sortableInstance = Sortable.create(listRef.value, {
    handle: '.drag-handle',
    animation: 200,
    ghostClass: 'opacity-30',
    forceFallback: true,
    fallbackClass: 'sortable-fallback',
    fallbackOnBody: true,
    onEnd: (evt) => {
      if (evt.oldIndex == null || evt.newIndex == null || evt.oldIndex === evt.newIndex) return
      const container = evt.from
      const newOrder: string[] = []
      for (const child of container.children) {
        const id = (child as HTMLElement).dataset.issueId
        if (id) newOrder.push(id)
      }
      // Revert DOM change so Vue handles re-rendering
      container.removeChild(evt.item)
      const refNode = container.children[evt.oldIndex] || null
      container.insertBefore(evt.item, refNode)
      emit('reorder', newOrder)
    },
  })
}

onMounted(() => {
  nextTick(() => initSortable())
})

watch(() => props.issues.length, () => {
  nextTick(() => initSortable())
})

watch(() => props.dragEnabled, () => {
  nextTick(() => initSortable())
})

onBeforeUnmount(() => {
  if (sortableInstance) {
    sortableInstance.destroy()
    sortableInstance = null
  }
})
</script>

<template>
  <div class="flex-1 min-h-0">
    <ScrollArea class="h-full">
      <div v-if="issues.length === 0" class="text-center text-muted-foreground py-4">
        No pinned issues
      </div>

      <div v-else ref="listRef" class="space-y-1 pr-4 outline-none" tabindex="0" @keydown="handleKeydown">
        <div
          v-for="issue in issues"
          :key="issue.id"
          :data-issue-id="issue.id"
          class="group relative w-full text-left p-1.5 rounded hover:bg-secondary/50 transition-colors flex items-start gap-1.5"
          :class="isFocused(issue.id) ? 'bg-primary/10 ring-1 ring-inset ring-primary/40' : ''"
          @click="setFocused(issue.id)"
        >
          <!-- Drag handle (only in manual mode) -->
          <span v-if="dragEnabled" class="drag-handle cursor-grab active:cursor-grabbing opacity-0 group-hover:opacity-60 transition-opacity mt-0.5 shrink-0">
            <svg class="w-2.5 h-3" viewBox="0 0 24 24" fill="currentColor">
              <circle cx="8" cy="4" r="2" /><circle cx="16" cy="4" r="2" />
              <circle cx="8" cy="12" r="2" /><circle cx="16" cy="12" r="2" />
              <circle cx="8" cy="20" r="2" /><circle cx="16" cy="20" r="2" />
            </svg>
          </span>

          <!-- Issue content (clickable) -->
          <button class="flex-1 min-w-0 text-left" @click="$emit('select', issue)">
            <div class="flex items-center gap-1.5 mb-0.5">
              <TypeBadge :type="issue.type" size="sm" />
              <PriorityBadge :priority="issue.priority" size="sm" />
              <span class="text-[10px] text-muted-foreground font-mono">{{ getShortId(issue.id) }}</span>
            </div>
            <p class="text-[11px] leading-tight line-clamp-2">{{ issue.title }}</p>
          </button>

          <!-- Unpin button -->
          <button
            class="opacity-0 group-hover:opacity-60 hover:!opacity-100 transition-opacity mt-0.5 shrink-0 text-muted-foreground hover:text-foreground"
            @click.stop="$emit('unpin', issue.id)"
          >
            <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
      </div>
    </ScrollArea>
  </div>
</template>
