<script setup lang="ts">
import type { IssueStatus } from '~/types/issue'
import { Badge } from '~/components/ui/badge'

const props = defineProps<{
  status: IssueStatus
  size?: 'default' | 'sm'
}>()

const statusConfig: Record<IssueStatus, { label: string; class: string }> = {
  open: { label: 'OPEN', class: 'badge-gradient bg-status-open-gradient text-white' },
  in_progress: { label: 'IN PROGRESS', class: 'badge-gradient bg-status-in-progress-gradient text-white' },
  blocked: { label: 'BLOCKED', class: 'badge-gradient bg-status-blocked-gradient text-white' },
  closed: { label: 'CLOSED', class: 'badge-gradient bg-status-closed-gradient text-white' },
}

const config = computed(() => statusConfig[props.status] || statusConfig.open)
</script>

<template>
  <Badge :class="[config.class, size === 'sm' ? 'text-[10px] px-1.5 py-0' : '']" variant="secondary">
    {{ config.label }}
  </Badge>
</template>
