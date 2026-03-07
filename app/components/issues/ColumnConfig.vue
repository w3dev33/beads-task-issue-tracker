<script setup lang="ts">
import type { ColumnConfig } from '~/types/issue'
import { onClickOutside } from '@vueuse/core'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '~/components/ui/tooltip'
import { Button } from '~/components/ui/button'
import { Checkbox } from '~/components/ui/checkbox'
import Sortable from 'sortablejs'

const props = defineProps<{
  columns: ColumnConfig[]
}>()

const emit = defineEmits<{
  'update:columns': [columns: ColumnConfig[]]
  reset: []
}>()

const isOpen = ref(false)
const panelRef = ref<HTMLElement | null>(null)
const triggerRef = ref<HTMLElement | null>(null)
const listRef = ref<HTMLElement | null>(null)
let sortableInstance: Sortable | null = null

const handleToggle = (columnId: string, checked: boolean) => {
  const updatedColumns = props.columns.map((col) =>
    col.id === columnId ? { ...col, visible: checked } : col
  )
  emit('update:columns', updatedColumns)
}

const initSortable = () => {
  if (sortableInstance) {
    sortableInstance.destroy()
    sortableInstance = null
  }
  if (!listRef.value) return

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
        const id = (child as HTMLElement).dataset.columnId
        if (id) newOrder.push(id)
      }
      // Revert DOM change so Vue handles re-rendering
      container.removeChild(evt.item)
      const refNode = container.children[evt.oldIndex] || null
      container.insertBefore(evt.item, refNode)
      // Build reordered columns array
      const reordered = newOrder
        .map(id => props.columns.find(c => c.id === id))
        .filter((c): c is ColumnConfig => !!c)
      if (reordered.length === props.columns.length) {
        emit('update:columns', reordered)
      }
    },
  })
}

onClickOutside(panelRef, (event) => {
  // Don't close if clicking the trigger button
  if (triggerRef.value && (triggerRef.value === event.target || triggerRef.value.contains(event.target as Node))) return
  isOpen.value = false
}, { ignore: [triggerRef] })

watch(isOpen, (open) => {
  if (open) {
    nextTick(() => initSortable())
  } else if (sortableInstance) {
    sortableInstance.destroy()
    sortableInstance = null
  }
})

onBeforeUnmount(() => {
  if (sortableInstance) {
    sortableInstance.destroy()
    sortableInstance = null
  }
})
</script>

<template>
  <div class="relative">
    <Tooltip>
      <TooltipTrigger as-child>
        <Button ref="triggerRef" variant="outline" size="icon" class="h-8 w-8" @click="isOpen = !isOpen">
          <svg
            class="w-3.5 h-3.5"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <rect x="3" y="3" width="7" height="7" />
            <rect x="14" y="3" width="7" height="7" />
            <rect x="14" y="14" width="7" height="7" />
            <rect x="3" y="14" width="7" height="7" />
          </svg>
          <span class="sr-only">Column settings</span>
        </Button>
      </TooltipTrigger>
      <TooltipContent>Column settings</TooltipContent>
    </Tooltip>

    <div
      v-if="isOpen"
      ref="panelRef"
      class="absolute right-0 top-full mt-1 z-50 w-52 rounded-md border bg-popover text-popover-foreground shadow-md"
    >
      <div class="px-2 py-1.5">
        <span class="text-xs font-semibold">Visible Columns</span>
      </div>
      <div class="h-px bg-border" />
      <div ref="listRef" class="py-1">
        <div
          v-for="column in columns"
          :key="column.id"
          :data-column-id="column.id"
          class="flex items-center gap-2 px-2 py-1 text-xs hover:bg-accent/50 rounded-sm mx-1"
        >
          <span class="drag-handle cursor-grab active:cursor-grabbing text-muted-foreground/40 hover:text-muted-foreground flex-shrink-0">
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
              <circle cx="9" cy="5" r="1.5" /><circle cx="15" cy="5" r="1.5" />
              <circle cx="9" cy="12" r="1.5" /><circle cx="15" cy="12" r="1.5" />
              <circle cx="9" cy="19" r="1.5" /><circle cx="15" cy="19" r="1.5" />
            </svg>
          </span>
          <Checkbox
            :id="`col-${column.id}`"
            :model-value="column.visible"
            class="h-3.5 w-3.5"
            @update:model-value="handleToggle(column.id, !!$event)"
          />
          <label :for="`col-${column.id}`" class="flex-1 cursor-pointer select-none">{{ column.label }}</label>
        </div>
      </div>
      <div class="h-px bg-border" />
      <div class="p-1">
        <Button variant="ghost" size="sm" class="w-full text-xs h-7 justify-start" @click="emit('reset')">
          Reset to defaults
        </Button>
      </div>
    </div>
  </div>
</template>
