<script setup lang="ts">
import type { IssuePriority } from '~/types/issue'

const props = defineProps<{
  byPriority: Record<IssuePriority, number>
}>()

const priorities: { key: IssuePriority; label: string; color: string }[] = [
  { key: 'p0', label: 'P0', color: 'var(--color-priority-p0)' },
  { key: 'p1', label: 'P1', color: 'var(--color-priority-p1)' },
  { key: 'p2', label: 'P2', color: 'var(--color-priority-p2)' },
  { key: 'p3', label: 'P3', color: 'var(--color-priority-p3)' },
  { key: 'p4', label: 'P4', color: 'var(--color-priority-p4)' },
]

const total = computed(() =>
  Object.values(props.byPriority).reduce((sum, val) => sum + val, 0)
)

const getPercent = (value: number) =>
  total.value > 0 ? (value / total.value) * 100 : 0
</script>

<template>
  <div class="space-y-3">
    <h4 class="text-sm font-medium text-muted-foreground">Priority Distribution</h4>

    <div class="space-y-2">
      <div
        v-for="priority in priorities"
        :key="priority.key"
        class="flex items-center gap-2"
      >
        <span class="w-8 text-xs text-muted-foreground">{{ priority.label }}</span>
        <div class="flex-1 h-4 bg-secondary rounded overflow-hidden">
          <div
            class="h-full transition-all"
            :style="{
              width: `${getPercent(byPriority[priority.key])}%`,
              backgroundColor: priority.color,
            }"
          />
        </div>
        <span class="w-8 text-xs text-right">{{ byPriority[priority.key] }}</span>
      </div>
    </div>
  </div>
</template>
