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
  assigneeFilters: string[]
}>()

defineEmits<{
  removeStatus: [status: IssueStatus]
  removeType: [type: IssueType]
  removePriority: [priority: IssuePriority]
  removeLabel: [label: string]
  removeAssignee: [assignee: string]
  clearAll: []
}>()

// Get exclusion filters
const { exclusions, toggleStatus: toggleExclusionStatus, togglePriority: toggleExclusionPriority, toggleType: toggleExclusionType, toggleLabel: toggleExclusionLabel, toggleAssignee: toggleExclusionAssignee, clearAll: clearAllExclusions, hasActiveExclusions } = useExclusionFilters()

const hasInclusionFilters = computed(
  () =>
    props.statusFilters.length > 0 ||
    props.typeFilters.length > 0 ||
    props.priorityFilters.length > 0 ||
    props.labelFilters.length > 0 ||
    props.assigneeFilters.length > 0
)

const hasFilters = computed(
  () => hasInclusionFilters.value || hasActiveExclusions.value
)

// Labels for status, priority, type exclusions
const statusLabels: Record<IssueStatus, string> = {
  open: 'Open',
  in_progress: 'In Progress',
  blocked: 'Blocked',
  closed: 'Closed',
  deferred: 'Deferred',
  tombstone: 'Tombstone',
  pinned: 'Pinned',
  hooked: 'Hooked',
}

const priorityLabels: Record<IssuePriority, string> = {
  p0: 'P0',
  p1: 'P1',
  p2: 'P2',
  p3: 'P3',
  p4: 'P4',
}

const typeLabels: Record<IssueType, string> = {
  bug: 'Bug',
  feature: 'Feature',
  task: 'Task',
  epic: 'Epic',
  chore: 'Chore',
}
</script>

<template>
  <div v-if="hasFilters" class="flex flex-col gap-1.5">
    <!-- Inclusion filters row -->
    <div v-if="hasInclusionFilters" class="flex flex-wrap items-center gap-1.5">
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

      <div
        v-for="assignee in assigneeFilters"
        :key="`assignee-${assignee}`"
        class="inline-flex items-center gap-1 cursor-pointer group"
        @click="$emit('removeAssignee', assignee)"
      >
        <span class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-slate-600 text-white text-[10px] font-medium">
          <svg class="w-2.5 h-2.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
            <circle cx="12" cy="7" r="4" />
          </svg>
          {{ assignee }}
        </span>
        <svg class="w-2.5 h-2.5 text-muted-foreground group-hover:text-destructive" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </div>

      <Button variant="ghost" size="sm" class="h-5 px-1.5 text-[10px]" @click="$emit('clearAll')">
        Clear
      </Button>
    </div>

    <!-- Exclusion filters row -->
    <div v-if="hasActiveExclusions" class="flex flex-wrap items-center gap-1.5">
      <span class="text-[10px] text-muted-foreground uppercase tracking-wide">Hidden:</span>

      <div
        v-for="status in exclusions.status"
        :key="`exc-status-${status}`"
        class="inline-flex items-center gap-1 cursor-pointer group opacity-50 hover:opacity-100"
        @click="toggleExclusionStatus(status)"
      >
        <StatusBadge :status="status" size="sm" />
        <svg class="w-2.5 h-2.5 text-muted-foreground group-hover:text-destructive" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </div>

      <div
        v-for="type in exclusions.type"
        :key="`exc-type-${type}`"
        class="inline-flex items-center gap-1 cursor-pointer group opacity-50 hover:opacity-100"
        @click="toggleExclusionType(type)"
      >
        <TypeBadge :type="type" size="sm" />
        <svg class="w-2.5 h-2.5 text-muted-foreground group-hover:text-destructive" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </div>

      <div
        v-for="priority in exclusions.priority"
        :key="`exc-priority-${priority}`"
        class="inline-flex items-center gap-1 cursor-pointer group opacity-50 hover:opacity-100"
        @click="toggleExclusionPriority(priority)"
      >
        <PriorityBadge :priority="priority" size="sm" />
        <svg class="w-2.5 h-2.5 text-muted-foreground group-hover:text-destructive" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </div>

      <div
        v-for="label in exclusions.labels"
        :key="`exc-label-${label}`"
        class="inline-flex items-center gap-1 cursor-pointer group opacity-50 hover:opacity-100"
        @click="toggleExclusionLabel(label)"
      >
        <LabelBadge :label="label" size="sm" />
        <svg class="w-2.5 h-2.5 text-muted-foreground group-hover:text-destructive" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </div>

      <div
        v-for="assignee in exclusions.assignee"
        :key="`exc-assignee-${assignee}`"
        class="inline-flex items-center gap-1 cursor-pointer group opacity-50 hover:opacity-100"
        @click="toggleExclusionAssignee(assignee)"
      >
        <span class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-slate-600 text-white text-[10px] font-medium">
          <svg class="w-2.5 h-2.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
            <circle cx="12" cy="7" r="4" />
          </svg>
          {{ assignee }}
        </span>
        <svg class="w-2.5 h-2.5 text-muted-foreground group-hover:text-destructive" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </div>

      <Button variant="ghost" size="sm" class="h-5 px-1.5 text-[10px]" @click="clearAllExclusions">
        Clear
      </Button>
    </div>
  </div>
</template>
