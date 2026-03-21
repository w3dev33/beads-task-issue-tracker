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

const copiedIssueId = ref<string | null>(null)
let copiedResetTimer: ReturnType<typeof setTimeout> | null = null

const copyIssueId = async (issueId: string, event: Event) => {
  event.stopPropagation()

  try {
    await navigator.clipboard.writeText(issueId)
    copiedIssueId.value = issueId

    if (copiedResetTimer) {
      clearTimeout(copiedResetTimer)
    }

    copiedResetTimer = setTimeout(() => {
      copiedIssueId.value = null
    }, 2000)
  } catch (err) {
    console.error('Failed to copy issue ID:', err)
  }
}

onBeforeUnmount(() => {
  if (copiedResetTimer) {
    clearTimeout(copiedResetTimer)
  }
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
        <div
          v-for="issue in issues"
          :key="issue.id"
          :data-issue-id="issue.id"
          class="w-full p-1.5 rounded hover:bg-secondary/50 transition-colors flex items-start gap-1.5"
          :class="isFocused(issue.id) ? 'bg-primary/10 ring-1 ring-inset ring-primary/40' : ''"
        >
          <button
            class="flex-1 min-w-0 text-left"
            @click="setFocused(issue.id); $emit('select', issue)"
          >
            <div class="flex items-center gap-1.5 mb-0.5">
              <TypeBadge :type="issue.type" size="sm" />
              <PriorityBadge :priority="issue.priority" size="sm" />
              <span class="text-[10px] text-muted-foreground font-mono">{{ getShortId(issue.id) }}</span>
            </div>
            <p class="text-[11px] leading-tight line-clamp-2">{{ issue.title }}</p>
          </button>

          <button
            class="shrink-0 p-0.5 mt-0.5 rounded text-muted-foreground hover:text-foreground hover:bg-secondary transition-colors"
            :title="`Copy issue ID ${issue.id}`"
            :aria-label="`Copy issue ID ${issue.id}`"
            @click="copyIssueId(issue.id, $event)"
          >
            <svg
              v-if="copiedIssueId !== issue.id"
              class="w-3 h-3"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
            </svg>
            <svg
              v-else
              class="w-3 h-3 text-green-500"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </button>
        </div>
      </div>
    </ScrollArea>
  </div>
</template>
