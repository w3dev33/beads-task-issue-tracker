<script setup lang="ts">
import type { IssueType } from '~/types/issue'
import { Badge } from '~/components/ui/badge'

const props = defineProps<{
  type: IssueType
  size?: 'default' | 'sm'
}>()

const typeConfig: Record<IssueType, { label: string; class: string }> = {
  bug: { label: 'BUG', class: 'badge-gradient bg-type-bug-gradient text-white' },
  task: { label: 'TASK', class: 'badge-gradient bg-type-task-gradient text-white' },
  feature: { label: 'FEATURE', class: 'badge-gradient bg-type-feature-gradient text-white' },
  epic: { label: 'EPIC', class: 'badge-gradient bg-type-epic-gradient text-white' },
  chore: { label: 'CHORE', class: 'badge-gradient bg-type-chore-gradient text-white' },
}

const config = computed(() => typeConfig[props.type] || typeConfig.task)
</script>

<template>
  <Badge :class="[config.class, size === 'sm' ? 'text-[10px] px-1.5 py-0' : '']" variant="secondary">
    {{ config.label }}
  </Badge>
</template>
