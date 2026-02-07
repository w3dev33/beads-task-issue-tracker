<script setup lang="ts">
import type { IssuePriority } from '~/types/issue'
import { Badge } from '~/components/ui/badge'

const props = defineProps<{
  priority: IssuePriority
  size?: 'default' | 'sm'
}>()

const priorityConfig: Record<IssuePriority, { label: string; class: string }> = {
  p0: { label: 'P0', class: 'badge-gradient bg-priority-p0-gradient text-white' },
  p1: { label: 'P1', class: 'badge-gradient bg-priority-p1-gradient text-white' },
  p2: { label: 'P2', class: 'badge-gradient bg-priority-p2-gradient text-white' },
  p3: { label: 'P3', class: 'badge-gradient bg-priority-p3-gradient text-white' },
  p4: { label: 'P4', class: 'badge-gradient bg-priority-p4-gradient text-white' },
}

const config = computed(() => priorityConfig[props.priority] || priorityConfig.p3)
</script>

<template>
  <Badge :class="[config.class, size === 'sm' ? 'text-[10px] px-1.5 py-0' : '']" variant="secondary">
    {{ config.label }}
  </Badge>
</template>
