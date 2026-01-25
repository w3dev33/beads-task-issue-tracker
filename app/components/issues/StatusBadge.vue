<script setup lang="ts">
import type { IssueStatus } from '~/types/issue'
import { Badge } from '~/components/ui/badge'

const props = defineProps<{
  status: IssueStatus
  size?: 'default' | 'sm'
}>()

const statusConfig: Record<IssueStatus, { label: string; class: string }> = {
  open: { label: 'OPEN', class: 'bg-status-open text-white' },
  in_progress: { label: 'IN PROGRESS', class: 'bg-status-in-progress text-white' },
  blocked: { label: 'BLOCKED', class: 'bg-status-blocked text-white' },
  closed: { label: 'CLOSED', class: 'bg-status-closed text-white' },
}

const config = computed(() => statusConfig[props.status] || statusConfig.open)
</script>

<template>
  <Badge :class="[config.class, size === 'sm' ? 'text-[10px] px-1.5 py-0' : '']" variant="secondary">
    {{ config.label }}
  </Badge>
</template>
