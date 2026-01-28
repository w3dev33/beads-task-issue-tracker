<script setup lang="ts">
import type { IssueStatus, IssueType, IssuePriority } from '~/types/issue'
import { Button } from '~/components/ui/button'
import StatusBadge from '~/components/issues/StatusBadge.vue'
import TypeBadge from '~/components/issues/TypeBadge.vue'
import PriorityBadge from '~/components/issues/PriorityBadge.vue'
import LabelBadge from '~/components/issues/LabelBadge.vue'

const props = defineProps<{
  statusFilters: IssueStatus[]
  typeFilters: IssueType[]
  priorityFilters: IssuePriority[]
  labelFilters: string[]
}>()

defineEmits<{
  removeStatus: [status: IssueStatus]
  removeType: [type: IssueType]
  removePriority: [priority: IssuePriority]
  removeLabel: [label: string]
  clearAll: []
}>()

const hasFilters = computed(
  () =>
    props.statusFilters.length > 0 ||
    props.typeFilters.length > 0 ||
    props.priorityFilters.length > 0 ||
    props.labelFilters.length > 0
)
</script>

<template>
  <div v-if="hasFilters" class="flex flex-wrap items-center gap-1.5">
    <span class="text-[10px] text-muted-foreground uppercase tracking-wide">Filters:</span>

    <div
      v-for="status in statusFilters"
      :key="`status-${status}`"
      class="inline-flex items-center gap-1 cursor-pointer group"
      @click="$emit('removeStatus', status)"
    >
      <StatusBadge :status="status" size="sm" />
      <svg class="w-2.5 h-2.5 text-muted-foreground group-hover:text-destructive" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18" />
        <line x1="6" y1="6" x2="18" y2="18" />
      </svg>
    </div>

    <div
      v-for="type in typeFilters"
      :key="`type-${type}`"
      class="inline-flex items-center gap-1 cursor-pointer group"
      @click="$emit('removeType', type)"
    >
      <TypeBadge :type="type" size="sm" />
      <svg class="w-2.5 h-2.5 text-muted-foreground group-hover:text-destructive" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18" />
        <line x1="6" y1="6" x2="18" y2="18" />
      </svg>
    </div>

    <div
      v-for="priority in priorityFilters"
      :key="`priority-${priority}`"
      class="inline-flex items-center gap-1 cursor-pointer group"
      @click="$emit('removePriority', priority)"
    >
      <PriorityBadge :priority="priority" size="sm" />
      <svg class="w-2.5 h-2.5 text-muted-foreground group-hover:text-destructive" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18" />
        <line x1="6" y1="6" x2="18" y2="18" />
      </svg>
    </div>

    <div
      v-for="label in labelFilters"
      :key="`label-${label}`"
      class="inline-flex items-center gap-1 cursor-pointer group"
      @click="$emit('removeLabel', label)"
    >
      <LabelBadge :label="label" size="sm" />
      <svg class="w-2.5 h-2.5 text-muted-foreground group-hover:text-destructive" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18" />
        <line x1="6" y1="6" x2="18" y2="18" />
      </svg>
    </div>

    <Button variant="ghost" size="sm" class="h-5 px-1.5 text-[10px]" @click="$emit('clearAll')">
      Clear
    </Button>
  </div>
</template>
