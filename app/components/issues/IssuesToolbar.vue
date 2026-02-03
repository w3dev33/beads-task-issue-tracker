<script setup lang="ts">
import type { IssueStatus, IssueType, IssuePriority, ColumnConfig } from '~/types/issue'
import { onClickOutside } from '@vueuse/core'
import { Input } from '~/components/ui/input'
import { Button } from '~/components/ui/button'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '~/components/ui/tooltip'
import ColumnConfigPopover from '~/components/issues/ColumnConfig.vue'
import StatusFilterDropdown from '~/components/issues/StatusFilterDropdown.vue'
import TypeFilterDropdown from '~/components/issues/TypeFilterDropdown.vue'
import PriorityFilterDropdown from '~/components/issues/PriorityFilterDropdown.vue'
import LabelFilterDropdown from '~/components/issues/LabelFilterDropdown.vue'
import AssigneeFilterDropdown from '~/components/issues/AssigneeFilterDropdown.vue'
import ExclusionFilterDropdown from '~/components/issues/ExclusionFilterDropdown.vue'

const search = defineModel<string>('search', { default: '' })

defineProps<{
  selectedStatuses: IssueStatus[]
  selectedTypes: IssueType[]
  selectedPriorities: IssuePriority[]
  availableLabels: string[]
  selectedLabels: string[]
  availableAssignees: string[]
  selectedAssignees: string[]
  hasSelection?: boolean
  multiSelectMode?: boolean
  selectedCount?: number
  columns: ColumnConfig[]
}>()

defineEmits<{
  add: []
  delete: []
  toggleMultiSelect: []
  toggleStatus: [status: IssueStatus]
  toggleType: [type: IssueType]
  togglePriority: [priority: IssuePriority]
  toggleLabel: [label: string]
  toggleAssignee: [assignee: string]
  'update:columns': [columns: ColumnConfig[]]
  resetColumns: []
}>()

// Track which filter dropdown is currently open (exclusive group)
type FilterType = 'type' | 'label' | 'status' | 'priority' | 'assignee' | 'exclusion' | null
const activeFilter = ref<FilterType>(null)

// Ref for the filter buttons container
const filterButtonsRef = ref<HTMLElement | null>(null)

// Close dropdown when clicking outside filter buttons (ignore dropdown content)
onClickOutside(filterButtonsRef, () => {
  activeFilter.value = null
}, {
  ignore: ['[data-slot="dropdown-menu-content"]']
})

// Handle click on filter button - toggle the filter
const handleFilterClick = (filter: FilterType) => {
  if (activeFilter.value === filter) {
    activeFilter.value = null
  } else {
    activeFilter.value = filter
  }
}
</script>

<template>
  <div class="space-y-2">
    <div class="flex items-center gap-2">
        <!-- Multi-select toggle -->
        <Tooltip>
          <TooltipTrigger as-child>
            <Button
              :variant="multiSelectMode ? 'default' : 'outline'"
              size="icon"
              class="h-8 w-8 shrink-0"
              @click="$emit('toggleMultiSelect')"
            >
              <svg
                class="w-3.5 h-3.5"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <rect x="3" y="5" width="6" height="6" rx="1" />
                <rect x="3" y="13" width="6" height="6" rx="1" />
                <line x1="12" y1="8" x2="21" y2="8" />
                <line x1="12" y1="16" x2="21" y2="16" />
              </svg>
              <span class="sr-only">Multi-select</span>
            </Button>
          </TooltipTrigger>
          <TooltipContent>Toggle multi-select</TooltipContent>
        </Tooltip>

      <div class="relative flex-1">
        <svg
          class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <circle cx="11" cy="11" r="8" />
          <line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <Input
          v-model="search"
          type="search"
          placeholder="Search issues..."
          class="pl-10 h-8 text-xs"
        />
      </div>

      <!-- Filter dropdowns (order matches table columns) -->
      <div ref="filterButtonsRef" class="flex items-center gap-2">
        <div @pointerdown.capture="handleFilterClick('type')">
          <TypeFilterDropdown
            :selected-types="selectedTypes"
            :open="activeFilter === 'type'"
            @toggle="$emit('toggleType', $event)"
          />
        </div>

        <div @pointerdown.capture="handleFilterClick('label')">
          <LabelFilterDropdown
            :available-labels="availableLabels"
            :selected-labels="selectedLabels"
            :open="activeFilter === 'label'"
            @toggle="$emit('toggleLabel', $event)"
          />
        </div>

        <div @pointerdown.capture="handleFilterClick('status')">
          <StatusFilterDropdown
            :selected-statuses="selectedStatuses"
            :open="activeFilter === 'status'"
            @toggle="$emit('toggleStatus', $event)"
          />
        </div>

        <div @pointerdown.capture="handleFilterClick('priority')">
          <PriorityFilterDropdown
            :selected-priorities="selectedPriorities"
            :open="activeFilter === 'priority'"
            @toggle="$emit('togglePriority', $event)"
          />
        </div>

        <div @pointerdown.capture="handleFilterClick('assignee')">
          <AssigneeFilterDropdown
            :available-assignees="availableAssignees"
            :selected-assignees="selectedAssignees"
            :open="activeFilter === 'assignee'"
            @toggle="$emit('toggleAssignee', $event)"
          />
        </div>

        <div @pointerdown.capture="handleFilterClick('exclusion')">
          <ExclusionFilterDropdown
            :available-labels="availableLabels"
            :available-assignees="availableAssignees"
            :open="activeFilter === 'exclusion'"
          />
        </div>
      </div>

    <Button
      v-if="hasSelection"
      variant="outline"
      size="icon"
      class="h-8 w-8 text-destructive hover:bg-destructive hover:text-destructive-foreground"
      @click="$emit('delete')"
    >
      <svg
        class="w-3.5 h-3.5"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <polyline points="3 6 5 6 21 6" />
        <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
        <line x1="10" y1="11" x2="10" y2="17" />
        <line x1="14" y1="11" x2="14" y2="17" />
      </svg>
      <span class="sr-only">Delete</span>
    </Button>

    <ColumnConfigPopover
      :columns="columns"
      @update:columns="$emit('update:columns', $event)"
      @reset="$emit('resetColumns')"
    />

    <Button size="sm" class="h-8 text-xs" @click="$emit('add')">
      <svg
        class="w-3.5 h-3.5 mr-1"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <line x1="12" y1="5" x2="12" y2="19" />
        <line x1="5" y1="12" x2="19" y2="12" />
      </svg>
      New
    </Button>
    </div>

    <!-- Selected count -->
    <div v-if="multiSelectMode && selectedCount" class="text-xs text-primary font-medium">
      {{ selectedCount }} issue{{ selectedCount > 1 ? 's' : '' }} selected
    </div>
  </div>
</template>
