<script setup lang="ts">
import type { Issue } from '~/types/issue'
import { ScrollArea } from '~/components/ui/scroll-area'
import TypeBadge from '~/components/issues/TypeBadge.vue'
import PriorityBadge from '~/components/issues/PriorityBadge.vue'

import { useKeyboardNavigation } from '~/composables/useKeyboardNavigation'

const props = defineProps<{
  issues: Issue[]
}>()

const emit = defineEmits<{
  select: [issue: Issue]
}>()

const quickItemIds = computed(() => props.issues.map(i => i.id))
const issueMap = computed(() => new Map(props.issues.map(i => [i.id, i])))

const { focusedId, setFocused, handleKeydown, isFocused } = useKeyboardNavigation({
  itemIds: quickItemIds,
  onSelect: (id) => {
    const issue = issueMap.value.get(id)
    if (issue) emit('select', issue)
  },
})

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
</script>

<template>
  <div class="flex-1 min-h-0">
    <ScrollArea class="h-full">
      <div v-if="issues.length === 0" class="text-center text-muted-foreground py-4">
        No issues ready to work on
      </div>

      <div v-else class="space-y-1 pr-4 outline-none" tabindex="0" @keydown="handleKeydown">
        <button
          v-for="issue in issues"
          :key="issue.id"
          class="w-full text-left p-1.5 rounded hover:bg-secondary/50 transition-colors"
          :class="isFocused(issue.id) ? 'bg-primary/10 ring-1 ring-inset ring-primary/40' : ''"
          @click="setFocused(issue.id); $emit('select', issue)"
        >
          <div class="flex items-center gap-1.5 mb-0.5">
            <TypeBadge :type="issue.type" size="sm" />
            <PriorityBadge :priority="issue.priority" size="sm" />
            <span class="text-[10px] text-muted-foreground font-mono">{{ getShortId(issue.id) }}</span>
          </div>
          <p class="text-[11px] leading-tight line-clamp-2">{{ issue.title }}</p>
        </button>
      </div>
    </ScrollArea>
  </div>
</template>
