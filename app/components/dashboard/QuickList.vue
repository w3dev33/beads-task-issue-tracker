<script setup lang="ts">
import type { Issue } from '~/types/issue'
import { ScrollArea } from '~/components/ui/scroll-area'
import TypeBadge from '~/components/issues/TypeBadge.vue'
import PriorityBadge from '~/components/issues/PriorityBadge.vue'

defineProps<{
  issues: Issue[]
}>()

defineEmits<{
  select: [issue: Issue]
}>()
</script>

<template>
  <div class="flex-1 min-h-0">
    <ScrollArea class="h-full">
      <div v-if="issues.length === 0" class="text-center text-muted-foreground py-4">
        No issues ready to work on
      </div>

      <div v-else class="space-y-2 pr-4">
        <button
          v-for="issue in issues"
          :key="issue.id"
          class="w-full text-left p-2 rounded hover:bg-secondary/50 transition-colors"
          @click="$emit('select', issue)"
        >
          <div class="flex items-center gap-1.5 mb-1">
            <TypeBadge :type="issue.type" size="sm" />
            <PriorityBadge :priority="issue.priority" size="sm" />
            <span class="text-[10px] text-muted-foreground font-mono">{{ issue.id }}</span>
          </div>
          <p class="text-[11px] leading-tight line-clamp-2">{{ issue.title }}</p>
        </button>
      </div>
    </ScrollArea>
  </div>
</template>
