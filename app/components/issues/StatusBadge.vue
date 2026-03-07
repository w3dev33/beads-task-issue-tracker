<script setup lang="ts">
import type { IssueStatus } from '~/types/issue'
import { Badge } from '~/components/ui/badge'

const props = defineProps<{
  status: IssueStatus
  size?: 'default' | 'sm'
}>()

const { showBadgeIcons } = useTheme()

const statusConfig: Record<IssueStatus, { label: string; class: string }> = {
  open: { label: 'OPEN', class: 'badge-gradient bg-status-open-gradient text-white' },
  in_progress: { label: 'IN PROGRESS', class: 'badge-gradient bg-status-in-progress-gradient text-white' },
  blocked: { label: 'BLOCKED', class: 'badge-gradient bg-status-blocked-gradient text-white' },
  closed: { label: 'CLOSED', class: 'badge-gradient bg-status-closed-gradient text-white' },
  deferred: { label: 'DEFERRED', class: 'badge-gradient bg-status-deferred-gradient text-white' },
  tombstone: { label: 'DELETED', class: 'badge-gradient bg-status-tombstone-gradient text-white' },
  pinned: { label: 'PINNED', class: 'badge-gradient bg-status-pinned-gradient text-white' },
  hooked: { label: 'HOOKED', class: 'badge-gradient bg-status-hooked-gradient text-white' },
}

// SVG icon paths for each status (12x12 viewBox)
const statusIcons: Partial<Record<IssueStatus, string>> = {
  open: 'M6 1a5 5 0 1 0 0 10A5 5 0 0 0 6 1zM2 6a4 4 0 1 1 8 0 4 4 0 0 1-8 0z',
  in_progress: 'M6 1a5 5 0 1 0 0 10A5 5 0 0 0 6 1zM2 6a4 4 0 1 1 8 0 4 4 0 0 1-8 0zM5 4v3l2.5 1.5',
  blocked: 'M6 1a5 5 0 1 0 0 10A5 5 0 0 0 6 1zM2 6a4 4 0 1 1 8 0 4 4 0 0 1-8 0zM4 4l4 4M8 4l-4 4',
  closed: 'M6 1a5 5 0 1 0 0 10A5 5 0 0 0 6 1zM2 6a4 4 0 1 1 8 0 4 4 0 0 1-8 0zM4 6l1.5 1.5L8 4.5',
}

const config = computed(() => statusConfig[props.status] || statusConfig.open)
</script>

<template>
  <Badge :class="[config.class, size === 'sm' ? 'text-[10px] px-1.5 py-0' : '']" variant="secondary">
    <span v-if="showBadgeIcons && statusIcons[status]" class="inline-flex items-center mr-1">
      <svg class="w-3 h-3" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
        <path :d="statusIcons[status]" />
      </svg>
    </span>
    {{ config.label }}
  </Badge>
</template>
