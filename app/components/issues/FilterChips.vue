<script setup lang="ts">
import type { IssueStatus, IssueType, IssuePriority } from '~/types/issue'
import { Badge } from '~/components/ui/badge'
import { Button } from '~/components/ui/button'

const props = defineProps<{
  statusFilters: IssueStatus[]
  typeFilters: IssueType[]
  priorityFilters: IssuePriority[]
}>()

defineEmits<{
  removeStatus: [status: IssueStatus]
  removeType: [type: IssueType]
  removePriority: [priority: IssuePriority]
  clearAll: []
}>()

const hasFilters = computed(
  () =>
    props.statusFilters.length > 0 ||
    props.typeFilters.length > 0 ||
    props.priorityFilters.length > 0
)

const statusLabels: Record<IssueStatus, string> = {
  open: 'Open',
  in_progress: 'In Progress',
  blocked: 'Blocked',
  closed: 'Closed',
}

const typeLabels: Record<IssueType, string> = {
  bug: 'Bug',
  task: 'Task',
  feature: 'Feature',
  epic: 'Epic',
  chore: 'Chore',
}

const priorityLabels: Record<IssuePriority, string> = {
  p0: 'P0',
  p1: 'P1',
  p2: 'P2',
  p3: 'P3',
  p4: 'P4',
}
</script>

<template>
  <div v-if="hasFilters" class="flex flex-wrap items-center gap-1.5">
    <span class="text-[10px] text-muted-foreground uppercase tracking-wide">Filters:</span>

    <Badge
      v-for="status in statusFilters"
      :key="`status-${status}`"
      variant="secondary"
      class="gap-1 cursor-pointer hover:bg-destructive/20 text-[10px] px-1.5 py-0"
      @click="$emit('removeStatus', status)"
    >
      {{ statusLabels[status] }}
      <svg class="w-2.5 h-2.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18" />
        <line x1="6" y1="6" x2="18" y2="18" />
      </svg>
    </Badge>

    <Badge
      v-for="type in typeFilters"
      :key="`type-${type}`"
      variant="secondary"
      class="gap-1 cursor-pointer hover:bg-destructive/20 text-[10px] px-1.5 py-0"
      @click="$emit('removeType', type)"
    >
      {{ typeLabels[type] }}
      <svg class="w-2.5 h-2.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18" />
        <line x1="6" y1="6" x2="18" y2="18" />
      </svg>
    </Badge>

    <Badge
      v-for="priority in priorityFilters"
      :key="`priority-${priority}`"
      variant="secondary"
      class="gap-1 cursor-pointer hover:bg-destructive/20 text-[10px] px-1.5 py-0"
      @click="$emit('removePriority', priority)"
    >
      {{ priorityLabels[priority] }}
      <svg class="w-2.5 h-2.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18" />
        <line x1="6" y1="6" x2="18" y2="18" />
      </svg>
    </Badge>

    <Button variant="ghost" size="sm" class="h-5 px-1.5 text-[10px]" @click="$emit('clearAll')">
      Clear
    </Button>
  </div>
</template>
