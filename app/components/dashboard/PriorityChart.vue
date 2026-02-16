<script setup lang="ts">
import type { IssuePriority } from '~/types/issue'

const props = defineProps<{
  byPriority: Record<IssuePriority, number>
}>()

const { currentTheme } = useTheme()
const isNeon = computed(() => currentTheme.value.id === 'neon')

const priorities: { key: IssuePriority; label: string; color: string; rgb: string }[] = [
  { key: 'p0', label: 'P0', color: 'var(--color-priority-p0)', rgb: '255, 51, 102' },
  { key: 'p1', label: 'P1', color: 'var(--color-priority-p1)', rgb: '255, 51, 102' },
  { key: 'p2', label: 'P2', color: 'var(--color-priority-p2)', rgb: '255, 170, 0' },
  { key: 'p3', label: 'P3', color: 'var(--color-priority-p3)', rgb: '0, 255, 135' },
  { key: 'p4', label: 'P4', color: 'var(--color-priority-p4)', rgb: '136, 146, 160' },
]

const total = computed(() =>
  Object.values(props.byPriority).reduce((sum, val) => sum + val, 0)
)

const getPercent = (value: number) =>
  total.value > 0 ? (value / total.value) * 100 : 0

const neonBarStyle = (rgb: string, value: number) => {
  if (!isNeon.value || value === 0) return {}
  return {
    boxShadow: `0 0 8px rgba(${rgb}, 0.4), inset 0 0 6px rgba(${rgb}, 0.2)`,
  }
}
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
        <span class="w-8 text-xs" :style="isNeon ? { color: priority.color } : {}" :class="!isNeon ? 'text-muted-foreground' : ''">{{ priority.label }}</span>
        <div class="flex-1 h-2 rounded overflow-hidden" :class="isNeon ? 'bg-white/5' : 'bg-secondary'">
          <div
            class="h-full transition-all rounded"
            :style="{
              width: `${getPercent(byPriority[priority.key])}%`,
              backgroundColor: priority.color,
              ...neonBarStyle(priority.rgb, byPriority[priority.key]),
            }"
          />
        </div>
        <span class="w-8 text-xs text-right" :style="isNeon ? { color: priority.color } : {}">{{ byPriority[priority.key] }}</span>
      </div>
    </div>
  </div>
</template>
