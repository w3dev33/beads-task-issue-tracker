<script setup lang="ts">
import type { IssueType } from '~/types/issue'
import { Badge } from '~/components/ui/badge'

const props = defineProps<{
  type: IssueType
  size?: 'default' | 'sm'
}>()

const typeConfig: Record<IssueType, { label: string; class: string }> = {
  bug: { label: 'BUG', class: 'bg-type-bug text-white' },
  task: { label: 'TASK', class: 'bg-type-task text-white' },
  feature: { label: 'FEATURE', class: 'bg-type-feature text-white' },
  epic: { label: 'EPIC', class: 'bg-type-epic text-white' },
  chore: { label: 'CHORE', class: 'bg-type-chore text-white' },
}

const config = computed(() => typeConfig[props.type] || typeConfig.task)
</script>

<template>
  <Badge :class="[config.class, size === 'sm' ? 'text-[10px] px-1.5 py-0' : '']" variant="secondary">
    {{ config.label }}
  </Badge>
</template>
