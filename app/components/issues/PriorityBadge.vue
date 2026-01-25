<script setup lang="ts">
import type { IssuePriority } from '~/types/issue'
import { Badge } from '~/components/ui/badge'

const props = defineProps<{
  priority: IssuePriority
  size?: 'default' | 'sm'
}>()

const priorityConfig: Record<IssuePriority, { label: string; class: string }> = {
  p0: { label: 'P0', class: 'bg-priority-p0 text-white' },
  p1: { label: 'P1', class: 'bg-priority-p1 text-white' },
  p2: { label: 'P2', class: 'bg-priority-p2 text-black' },
  p3: { label: 'P3', class: 'bg-priority-p3 text-white' },
  p4: { label: 'P4', class: 'bg-priority-p4 text-white' },
}

const config = computed(() => priorityConfig[props.priority] || priorityConfig.p3)
</script>

<template>
  <Badge :class="[config.class, size === 'sm' ? 'text-[10px] px-1.5 py-0' : '']" variant="secondary">
    {{ config.label }}
  </Badge>
</template>
