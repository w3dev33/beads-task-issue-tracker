<script setup lang="ts">
import type { Issue, IssueStatus, IssuePriority, IssueType } from '~/types/issue'
import type { ColumnConfig } from '~/types/issue'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '~/components/ui/table'
import TypeBadge from '~/components/issues/TypeBadge.vue'
import StatusBadge from '~/components/issues/StatusBadge.vue'
import PriorityBadge from '~/components/issues/PriorityBadge.vue'
import LabelBadge from '~/components/issues/LabelBadge.vue'

const props = defineProps<{
  issues: Issue[]
  columns: ColumnConfig[]
  selectedId?: string | null
  multiSelectMode?: boolean
  selectedIds?: string[]
}>()

const emit = defineEmits<{
  select: [issue: Issue]
  edit: [issue: Issue]
  deselect: []
  'update:selectedIds': [ids: string[]]
}>()

// Sorting state
type SortDirection = 'asc' | 'desc'
const sortColumn = ref<string | null>('createdAt')
const sortDirection = ref<SortDirection>('desc')

const toggleSort = (columnId: string) => {
  if (sortColumn.value === columnId) {
    // Toggle direction or clear sort
    if (sortDirection.value === 'asc') {
      sortDirection.value = 'desc'
    } else {
      sortColumn.value = null
      sortDirection.value = 'asc'
    }
  } else {
    sortColumn.value = columnId
    sortDirection.value = 'asc'
  }
}

// Sort order for status and priority
// in_progress first (active work), then open (ready to start), then blocked, then closed
const statusOrder: Record<IssueStatus, number> = {
  in_progress: 0,
  open: 1,
  blocked: 2,
  closed: 3,
}

const priorityOrder: Record<IssuePriority, number> = {
  p0: 0,
  p1: 1,
  p2: 2,
  p3: 3,
  p4: 4,
}

const typeOrder: Record<IssueType, number> = {
  bug: 0,
  feature: 1,
  task: 2,
  epic: 3,
  chore: 4,
}

const sortedIssues = computed(() => {
  if (!sortColumn.value) return props.issues

  const col = sortColumn.value
  const dir = sortDirection.value === 'asc' ? 1 : -1

  return [...props.issues].sort((a, b) => {
    let aVal: string | number | null = null
    let bVal: string | number | null = null

    switch (col) {
      case 'status':
        aVal = statusOrder[a.status] ?? 99
        bVal = statusOrder[b.status] ?? 99
        break
      case 'priority':
        aVal = priorityOrder[a.priority] ?? 99
        bVal = priorityOrder[b.priority] ?? 99
        break
      case 'type':
        aVal = typeOrder[a.type] ?? 99
        bVal = typeOrder[b.type] ?? 99
        break
      case 'labels':
        // Sort by first label alphabetically, issues without labels at the end
        aVal = a.labels?.length ? a.labels[0].toLowerCase() : '\uffff'
        bVal = b.labels?.length ? b.labels[0].toLowerCase() : '\uffff'
        break
      case 'createdAt':
      case 'updatedAt':
        aVal = a[col] ? new Date(a[col]).getTime() : 0
        bVal = b[col] ? new Date(b[col]).getTime() : 0
        break
      default:
        aVal = String(a[col as keyof Issue] ?? '').toLowerCase()
        bVal = String(b[col as keyof Issue] ?? '').toLowerCase()
    }

    if (aVal < bVal) return -1 * dir
    if (aVal > bVal) return 1 * dir
    return 0
  })
})

const isSelected = (id: string) => props.selectedIds?.includes(id) ?? false

const toggleSelect = (id: string) => {
  const current = props.selectedIds ?? []
  if (isSelected(id)) {
    emit('update:selectedIds', current.filter(i => i !== id))
  } else {
    emit('update:selectedIds', [...current, id])
  }
}

const toggleSelectAll = () => {
  const current = props.selectedIds ?? []
  if (current.length === props.issues.length) {
    emit('update:selectedIds', [])
  } else {
    emit('update:selectedIds', props.issues.map(i => i.id))
  }
}

const isAllSelected = computed(() => {
  return props.issues.length > 0 && (props.selectedIds?.length ?? 0) === props.issues.length
})

const isSomeSelected = computed(() => {
  const len = props.selectedIds?.length ?? 0
  return len > 0 && len < props.issues.length
})

const visibleColumns = computed(() =>
  props.columns.filter((col) => col.visible)
)

const formatDate = (dateStr: string) => {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  return date.toLocaleDateString('fr-FR', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
  })
}

const formatTime = (dateStr: string) => {
  if (!dateStr) return ''
  const date = new Date(dateStr)
  return date.toLocaleTimeString('fr-FR', {
    hour: '2-digit',
    minute: '2-digit',
  })
}
</script>

<template>
  <div class="h-full rounded border border-border overflow-auto" @click.self="$emit('deselect')">
    <Table @click.self="$emit('deselect')">
      <TableHeader>
        <TableRow class="bg-secondary/30 hover:bg-secondary/30">
          <TableHead v-if="multiSelectMode" class="w-10 px-2">
            <button
              class="flex items-center justify-center w-4 h-4 rounded border transition-colors"
              :class="isAllSelected
                ? 'bg-primary border-primary text-primary-foreground'
                : isSomeSelected
                  ? 'bg-primary/50 border-primary text-primary-foreground'
                  : 'border-muted-foreground/30 hover:border-muted-foreground'"
              @click="toggleSelectAll"
            >
              <svg v-if="isAllSelected || isSomeSelected" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                <polyline v-if="isAllSelected" points="20 6 9 17 4 12" />
                <line v-else x1="5" y1="12" x2="19" y2="12" />
              </svg>
            </button>
          </TableHead>
          <TableHead
            v-for="col in visibleColumns"
            :key="col.id"
            class="font-medium"
            :class="{ 'cursor-pointer select-none hover:bg-secondary/50': col.sortable }"
            @click="col.sortable && toggleSort(col.id)"
          >
            <div class="flex items-center gap-1">
              <span>{{ col.label }}</span>
              <template v-if="col.sortable">
                <svg
                  v-if="sortColumn === col.id"
                  class="w-3 h-3 text-primary"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <path v-if="sortDirection === 'asc'" d="M12 19V5M5 12l7-7 7 7" />
                  <path v-else d="M12 5v14M5 12l7 7 7-7" />
                </svg>
                <svg
                  v-else
                  class="w-3 h-3 text-muted-foreground/40"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <path d="M7 15l5 5 5-5M7 9l5-5 5 5" />
                </svg>
              </template>
            </div>
          </TableHead>
        </TableRow>
      </TableHeader>

      <TableBody>
        <TableRow
          v-if="sortedIssues.length === 0"
          class="hover:bg-transparent"
        >
          <TableCell
            :colspan="visibleColumns.length + (multiSelectMode ? 1 : 0)"
            class="h-24 text-center text-muted-foreground"
          >
            No tasks / issues found
          </TableCell>
        </TableRow>

        <TableRow
          v-for="issue in sortedIssues"
          :key="issue.id"
          class="cursor-pointer"
          :class="multiSelectMode
            ? (isSelected(issue.id) ? 'bg-accent/50 hover:bg-accent/70' : 'hover:bg-muted/50')
            : (selectedId === issue.id ? 'bg-accent/50 hover:bg-accent/70' : 'hover:bg-muted/50')"
          @click="multiSelectMode ? toggleSelect(issue.id) : $emit('select', issue)"
          @dblclick="!multiSelectMode && $emit('edit', issue)"
        >
          <TableCell v-if="multiSelectMode" class="w-10 px-2">
            <button
              class="flex items-center justify-center w-4 h-4 rounded border transition-colors"
              :class="isSelected(issue.id)
                ? 'bg-primary border-primary text-primary-foreground'
                : 'border-muted-foreground/30 hover:border-muted-foreground'"
              @click.stop="toggleSelect(issue.id)"
            >
              <svg v-if="isSelected(issue.id)" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                <polyline points="20 6 9 17 4 12" />
              </svg>
            </button>
          </TableCell>
          <TableCell v-for="col in visibleColumns" :key="col.id" :class="col.id === 'title' ? 'whitespace-normal max-w-md' : ''">
            <template v-if="col.id === 'id'">
              <CopyableId :value="issue.id" />
            </template>

            <template v-else-if="col.id === 'type'">
              <TypeBadge :type="issue.type" size="sm" />
            </template>

            <template v-else-if="col.id === 'title'">
              <span class="text-xs font-medium line-clamp-2 break-words">{{ issue.title }}</span>
            </template>

            <template v-else-if="col.id === 'status'">
              <StatusBadge :status="issue.status" size="sm" />
            </template>

            <template v-else-if="col.id === 'priority'">
              <PriorityBadge :priority="issue.priority" size="sm" />
            </template>

            <template v-else-if="col.id === 'labels'">
              <div v-if="issue.labels?.length" class="flex flex-wrap gap-1">
                <LabelBadge
                  v-for="label in issue.labels"
                  :key="label"
                  :label="label"
                  size="sm"
                />
              </div>
              <span v-else class="text-xs text-muted-foreground">-</span>
            </template>

            <template v-else-if="col.id === 'assignee'">
              <span class="text-xs">{{ issue.assignee || '-' }}</span>
            </template>

            <template v-else-if="col.id === 'createdAt'">
              <div class="flex flex-col">
                <span class="text-xs text-muted-foreground">{{ formatDate(issue.createdAt) }}</span>
                <span class="text-[10px] text-muted-foreground/70">{{ formatTime(issue.createdAt) }}</span>
              </div>
            </template>

            <template v-else-if="col.id === 'updatedAt'">
              <div class="flex flex-col">
                <span class="text-xs text-muted-foreground">{{ formatDate(issue.updatedAt) }}</span>
                <span class="text-[10px] text-muted-foreground/70">{{ formatTime(issue.updatedAt) }}</span>
              </div>
            </template>

            <template v-else>
              <span class="text-xs">{{ (issue as Record<string, unknown>)[col.id] || '-' }}</span>
            </template>
          </TableCell>
        </TableRow>
      </TableBody>
    </Table>
  </div>
</template>
