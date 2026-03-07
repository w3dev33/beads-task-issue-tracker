<script setup lang="ts">
import type { Issue, IssueStatus, IssueType, IssuePriority, ColumnConfig } from '~/types/issue'
import type { IssueGroup } from '~/composables/useIssues'
import IssuesToolbar from '~/components/issues/IssuesToolbar.vue'
import FilterChips from '~/components/issues/FilterChips.vue'
import IssueTable from '~/components/issues/IssueTable.vue'

const searchValue = defineModel<string>('search', { required: true })
const selectedIds = defineModel<string[]>('selectedIds', { required: true })

defineProps<{
  filters: { status: IssueStatus[]; type: IssueType[]; priority: IssuePriority[]; labels: string[]; assignee: string[] }
  availableLabels: string[]
  availableAssignees: string[]
  hasSelection: boolean
  multiSelectMode: boolean
  selectedCount: number
  columns: ColumnConfig[]
  isSearchActive: boolean
  issues: Issue[]
  groupedIssues: IssueGroup[]
  selectedId?: string
  hasMore: boolean
  totalCount: number
  sortField: string | null
  sortDirection: 'asc' | 'desc'
  newlyAddedIds: Set<string>
  pinnedIds?: string[]
}>()

const emit = defineEmits<{
  add: []
  delete: []
  'toggle-multi-select': []
  'update:columns': [columns: ColumnConfig[]]
  'reset-columns': []
  'toggle-status': [status: IssueStatus]
  'toggle-type': [type: IssueType]
  'toggle-priority': [priority: IssuePriority]
  'toggle-label': [label: string]
  'toggle-assignee': [assignee: string]
  'remove-label': [label: string]
  'clear-filters': []
  select: [issue: Issue]
  edit: [issue: Issue]
  deselect: []
  'load-more': []
  sort: [field: string | null, direction: 'asc' | 'desc']
  'toggle-pin': [issueId: string]
}>()

const handleSort = (field: string | null, direction: 'asc' | 'desc') => {
  emit('sort', field, direction)
}
</script>

<template>
  <div class="p-4 border-b border-border space-y-3">
    <IssuesToolbar
      v-model:search="searchValue"
      :selected-statuses="filters.status"
      :selected-types="filters.type"
      :selected-priorities="filters.priority"
      :available-labels="availableLabels"
      :selected-labels="filters.labels"
      :available-assignees="availableAssignees"
      :selected-assignees="filters.assignee"
      :has-selection="hasSelection"
      :multi-select-mode="multiSelectMode"
      :selected-count="selectedCount"
      :columns="columns"
      @add="emit('add')"
      @delete="emit('delete')"
      @toggle-multi-select="emit('toggle-multi-select')"
      @update:columns="emit('update:columns', $event)"
      @reset-columns="emit('reset-columns')"
      @toggle-status="emit('toggle-status', $event)"
      @toggle-type="emit('toggle-type', $event)"
      @toggle-priority="emit('toggle-priority', $event)"
      @toggle-label="emit('toggle-label', $event)"
      @toggle-assignee="emit('toggle-assignee', $event)"
    />

    <FilterChips
      v-if="!isSearchActive"
      :status-filters="filters.status"
      :type-filters="filters.type"
      :priority-filters="filters.priority"
      :label-filters="filters.labels"
      :assignee-filters="filters.assignee"
      @remove-status="emit('toggle-status', $event)"
      @remove-type="emit('toggle-type', $event)"
      @remove-priority="emit('toggle-priority', $event)"
      @remove-label="emit('remove-label', $event)"
      @remove-assignee="emit('toggle-assignee', $event)"
      @clear-all="emit('clear-filters')"
    />
  </div>

  <div class="flex-1 overflow-auto p-4">
    <IssueTable
      v-model:selected-ids="selectedIds"
      :issues="issues"
      :grouped-issues="groupedIssues"
      :columns="columns"
      :selected-id="selectedId"
      :multi-select-mode="multiSelectMode"
      :has-more="hasMore"
      :total-count="totalCount"
      :external-sort-column="sortField"
      :external-sort-direction="sortDirection"
      :newly-added-ids="newlyAddedIds"
      :pinned-ids="pinnedIds"
      @select="emit('select', $event)"
      @edit="emit('edit', $event)"
      @deselect="emit('deselect')"
      @load-more="emit('load-more')"
      @sort="handleSort"
      @toggle-pin="emit('toggle-pin', $event)"
    />
  </div>
</template>
