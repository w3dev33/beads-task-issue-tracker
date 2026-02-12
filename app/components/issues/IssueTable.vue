<script setup lang="ts">
import type { Issue, IssueStatus, IssuePriority, IssueType } from '~/types/issue'
import type { ColumnConfig } from '~/types/issue'
import type { IssueGroup } from '~/composables/useIssues'
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
import { Button } from '~/components/ui/button'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '~/components/ui/tooltip'
import { Ban } from 'lucide-vue-next'

const props = defineProps<{
  issues: Issue[]
  groupedIssues?: IssueGroup[]
  columns: ColumnConfig[]
  selectedId?: string | null
  multiSelectMode?: boolean
  selectedIds?: string[]
  hasMore?: boolean
  totalCount?: number
  externalSortColumn?: string | null
  externalSortDirection?: 'asc' | 'desc'
}>()

const emit = defineEmits<{
  select: [issue: Issue]
  edit: [issue: Issue]
  deselect: []
  'update:selectedIds': [ids: string[]]
  loadMore: []
  sort: [field: string | null, direction: 'asc' | 'desc']
}>()

// Sorting state - sync with external (composable) state if provided
type SortDirection = 'asc' | 'desc'
const internalSortColumn = ref<string | null>('updatedAt')
const internalSortDirection = ref<SortDirection>('desc')

// Use external sort state if provided, otherwise use internal
const sortColumn = computed(() => props.externalSortColumn !== undefined ? props.externalSortColumn : internalSortColumn.value)
const sortDirection = computed(() => props.externalSortDirection !== undefined ? props.externalSortDirection : internalSortDirection.value)


const toggleSort = (columnId: string) => {
  let newDirection: SortDirection = 'asc'
  let newColumn: string | null = columnId

  if (sortColumn.value === columnId) {
    // Cycle: asc -> desc -> null -> asc -> ...
    if (sortDirection.value === 'asc') {
      newDirection = 'desc'
    } else {
      // Was desc, clear sort
      newColumn = null
      newDirection = 'asc'
    }
  }

  // Always emit sort event (including null to clear)
  emit('sort', newColumn, newDirection)

  // Update internal state as fallback
  internalSortColumn.value = newColumn
  internalSortDirection.value = newDirection
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

// Natural sort comparison for IDs (handles multi-digit numbers correctly)
// e.g., "40b.2" < "40b.10" instead of "40b.10" < "40b.2"
const naturalCompare = (a: string, b: string): number => {
  const aParts = a.split(/(\d+)/)
  const bParts = b.split(/(\d+)/)

  for (let i = 0; i < Math.max(aParts.length, bParts.length); i++) {
    const aPart = aParts[i] || ''
    const bPart = bParts[i] || ''

    // Check if both parts are numeric
    const aNum = parseInt(aPart, 10)
    const bNum = parseInt(bPart, 10)

    if (!isNaN(aNum) && !isNaN(bNum)) {
      if (aNum !== bNum) return aNum - bNum
    } else {
      // String comparison
      if (aPart < bPart) return -1
      if (aPart > bPart) return 1
    }
  }
  return 0
}

const sortedIssues = computed(() => {
  // When external sort is provided, data is already sorted by the composable
  if (props.externalSortColumn !== undefined) {
    return props.issues
  }

  if (!sortColumn.value) return props.issues

  const col = sortColumn.value
  const dir = sortDirection.value === 'asc' ? 1 : -1

  return [...props.issues].sort((a, b) => {
    let aVal: string | number | null = null
    let bVal: string | number | null = null

    switch (col) {
      case 'id':
        // Use natural sort for IDs
        return naturalCompare(a.id.toLowerCase(), b.id.toLowerCase()) * dir
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
        aVal = a.labels?.length ? a.labels[0]!.toLowerCase() : '\uffff'
        bVal = b.labels?.length ? b.labels[0]!.toLowerCase() : '\uffff'
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

// Epic expand/collapse state (from dedicated composable, avoids creating
// duplicate computed properties and watchers from full useIssues())
const { isEpicExpanded, toggleEpicExpand } = useEpicExpand()

const isExpanded = (epicId: string) => isEpicExpanded(epicId)

const toggleExpand = (epicId: string, event: Event) => {
  event.stopPropagation()
  toggleEpicExpand(epicId)
}

// Check if we should use hierarchical display
const useHierarchicalDisplay = computed(() => {
  return props.groupedIssues && props.groupedIssues.length > 0
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

// Extract common prefix from all issue IDs to show short IDs
const commonPrefix = computed(() => {
  const ids = props.issues.map(i => i.id)
  if (ids.length === 0) return ''

  // Find common prefix by comparing all IDs
  let prefix = ids[0] || ''
  for (const id of ids) {
    while (prefix && !id.startsWith(prefix)) {
      // Trim prefix to the previous hyphen boundary
      const lastHyphen = prefix.lastIndexOf('-')
      if (lastHyphen > 0) {
        const newPrefix = prefix.slice(0, lastHyphen + 1)
        if (newPrefix === prefix) {
          // prefix ends with '-': trimming keeps the same string → would loop forever.
          // Remove the trailing hyphen to make progress.
          prefix = prefix.slice(0, lastHyphen)
          if (!prefix) break
        } else {
          prefix = newPrefix
        }
      } else {
        prefix = ''
        break
      }
    }
  }
  return prefix
})

// Get short display ID (without common prefix)
const getShortId = (id: string) => {
  const prefix = commonPrefix.value
  if (prefix && id.startsWith(prefix)) {
    const short = id.slice(prefix.length)
    if (!short) {
      // ID equals prefix (e.g., EPIC ID "beads-demo-5tg" when prefix is "beads-demo-5tg")
      // Return last segment after hyphen
      const lastHyphen = id.lastIndexOf('-')
      return lastHyphen > 0 ? id.slice(lastHyphen + 1) : id
    }
    return short
  }
  return id
}

const formatDate = (dateStr: string) => {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  return date.toLocaleDateString(undefined, {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
  })
}

const formatTime = (dateStr: string) => {
  if (!dateStr) return ''
  const date = new Date(dateStr)
  return date.toLocaleTimeString(undefined, {
    hour: '2-digit',
    minute: '2-digit',
  })
}

// Epic border colors for visual grouping (left and right)
const epicBorderColors = [
  { left: 'border-l-blue-500', right: 'border-r-blue-500' },
  { left: 'border-l-green-500', right: 'border-r-green-500' },
  { left: 'border-l-purple-500', right: 'border-r-purple-500' },
  { left: 'border-l-orange-500', right: 'border-r-orange-500' },
  { left: 'border-l-pink-500', right: 'border-r-pink-500' },
  { left: 'border-l-cyan-500', right: 'border-r-cyan-500' },
  { left: 'border-l-yellow-500', right: 'border-r-yellow-500' },
  { left: 'border-l-red-500', right: 'border-r-red-500' },
]

const getEpicBorderColors = (epicId: string): { left: string; right: string } => {
  let hash = 0
  for (let i = 0; i < epicId.length; i++) {
    hash = ((hash << 5) - hash) + epicId.charCodeAt(i)
    hash = hash & hash
  }
  return epicBorderColors[Math.abs(hash) % epicBorderColors.length]!
}

function getIssueField(issue: Issue, field: string): string {
  return String((issue as unknown as Record<string, unknown>)[field] ?? '-')
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
            :class="[
              { 'cursor-pointer select-none hover:bg-secondary/50': col.sortable },
              col.id === 'id' ? 'pl-7' : ''
            ]"
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
        <!-- Empty state -->
        <TableRow
          v-if="(useHierarchicalDisplay ? groupedIssues?.length === 0 : sortedIssues.length === 0)"
          class="hover:bg-transparent"
        >
          <TableCell
            :colspan="visibleColumns.length + (multiSelectMode ? 1 : 0)"
            class="h-24 text-center text-muted-foreground"
          >
            No tasks / issues found
          </TableCell>
        </TableRow>

        <!-- Hierarchical display with grouped issues -->
        <template v-if="useHierarchicalDisplay">
          <template v-for="group in groupedIssues" :key="group.epic?.id || group.children[0]?.id">
            <!-- Epic row with expand/collapse -->
            <template v-if="group.epic">
              <TableRow
                class="cursor-pointer bg-black/30"
                :class="[
                  multiSelectMode
                    ? (isSelected(group.epic.id) ? 'bg-accent/50 hover:bg-accent/70' : 'hover:bg-muted/50')
                    : (selectedId === group.epic.id ? 'bg-accent/50 hover:bg-accent/70' : 'hover:bg-muted/50'),
                  isExpanded(group.epic.id) && group.childCount > 0 ? ['border-l-4', 'border-r-4', getEpicBorderColors(group.epic.id).left, getEpicBorderColors(group.epic.id).right] : ''
                ]"
                @click="multiSelectMode ? toggleSelect(group.epic.id) : $emit('select', group.epic)"
                @dblclick="!multiSelectMode && $emit('edit', group.epic)"
              >
                <TableCell v-if="multiSelectMode" class="w-10 px-2 !py-0.5">
                  <button
                    class="flex items-center justify-center w-4 h-4 rounded border transition-colors"
                    :class="isSelected(group.epic.id)
                      ? 'bg-primary border-primary text-primary-foreground'
                      : 'border-muted-foreground/30 hover:border-muted-foreground'"
                    @click.stop="toggleSelect(group.epic.id)"
                  >
                    <svg v-if="isSelected(group.epic.id)" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                      <polyline points="20 6 9 17 4 12" />
                    </svg>
                  </button>
                </TableCell>
                <TableCell v-for="col in visibleColumns" :key="col.id" class="!py-0.5" :class="col.id === 'title' ? 'whitespace-normal max-w-md' : ''">
                  <template v-if="col.id === 'id'">
                    <div class="flex items-center gap-2">
                      <!-- Expand/collapse chevron button -->
                      <button
                        v-if="group.childCount > 0"
                        class="flex items-center justify-center w-5 h-5 rounded border border-border bg-background hover:bg-muted hover:border-muted-foreground/50 transition-colors shrink-0"
                        @click="toggleExpand(group.epic!.id, $event)"
                      >
                        <svg
                          class="w-3 h-3 text-muted-foreground transition-transform"
                          :class="{ 'rotate-90': isExpanded(group.epic!.id) }"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2"
                        >
                          <polyline points="9 18 15 12 9 6" />
                        </svg>
                      </button>
                      <!-- Spacer when no chevron to align IDs -->
                      <div v-else class="w-5 shrink-0" />
                      <CopyableId :value="group.epic.id" :display-value="getShortId(group.epic.id)" />
                    </div>
                  </template>

                  <template v-else-if="col.id === 'type'">
                    <div class="flex items-center gap-1.5">
                      <TypeBadge :type="group.epic.type" size="sm" />
                      <!-- Child count badge with tooltip -->
                      <Tooltip v-if="group.childCount > 0">
                        <TooltipTrigger as-child>
                          <span class="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded-full cursor-help">
                            {{ group.closedChildCount }}/{{ group.childCount }}
                          </span>
                        </TooltipTrigger>
                        <TooltipContent>
                          <p>{{ group.closedChildCount }} closed / {{ group.childCount }} {{ group.childCount === 1 ? 'child' : 'children' }}</p>
                        </TooltipContent>
                      </Tooltip>
                    </div>
                  </template>

                  <template v-else-if="col.id === 'title'">
                    <span class="text-xs font-medium line-clamp-2 break-words">{{ group.epic.title }}</span>
                  </template>

                  <template v-else-if="col.id === 'status'">
                    <div class="flex items-center gap-1">
                      <StatusBadge :status="group.epic.status" size="sm" />
                      <Tooltip v-if="group.epic.blockedBy?.length">
                        <TooltipTrigger as-child>
                          <Ban class="w-3 h-3 text-red-400" />
                        </TooltipTrigger>
                        <TooltipContent side="top">
                          <p class="text-xs">Blocked by {{ group.epic.blockedBy.join(', ') }}</p>
                        </TooltipContent>
                      </Tooltip>
                    </div>
                  </template>

                  <template v-else-if="col.id === 'priority'">
                    <PriorityBadge :priority="group.epic.priority" size="sm" />
                  </template>

                  <template v-else-if="col.id === 'labels'">
                    <div v-if="group.epic.labels?.length" class="flex flex-wrap gap-1">
                      <LabelBadge
                        v-for="label in group.epic.labels"
                        :key="label"
                        :label="label"
                        size="sm"
                      />
                    </div>
                    <span v-else class="text-xs text-muted-foreground">-</span>
                  </template>

                  <template v-else-if="col.id === 'assignee'">
                    <span class="text-xs">{{ group.epic.assignee || '-' }}</span>
                  </template>

                  <template v-else-if="col.id === 'createdAt'">
                    <div class="flex flex-col">
                      <span class="text-xs text-muted-foreground">{{ formatDate(group.epic.createdAt) }}</span>
                      <span class="text-[10px] text-muted-foreground/70">{{ formatTime(group.epic.createdAt) }}</span>
                    </div>
                  </template>

                  <template v-else-if="col.id === 'updatedAt'">
                    <div class="flex flex-col">
                      <span class="text-xs text-muted-foreground">{{ formatDate(group.epic.updatedAt) }}</span>
                      <span class="text-[10px] text-muted-foreground/70">{{ formatTime(group.epic.updatedAt) }}</span>
                    </div>
                  </template>

                  <template v-else-if="col.id === 'commentCount'">
                    <span v-if="group.epic.commentCount" class="text-xs text-muted-foreground">{{ group.epic.commentCount }}</span>
                    <span v-else class="text-xs text-muted-foreground">-</span>
                  </template>

                  <template v-else>
                    <span class="text-xs">{{ getIssueField(group.epic, col.id) }}</span>
                  </template>
                </TableCell>
              </TableRow>

              <!-- Child rows (when expanded) -->
              <template v-if="isExpanded(group.epic.id)">
                <TableRow
                  v-for="child in group.children"
                  :key="child.id"
                  class="cursor-pointer border-l-4 border-r-4"
                  :class="[
                    getEpicBorderColors(group.epic.id).left,
                    getEpicBorderColors(group.epic.id).right,
                    multiSelectMode
                      ? (isSelected(child.id) ? 'bg-accent/50 hover:bg-accent/70' : 'hover:bg-muted/50')
                      : (selectedId === child.id ? 'bg-accent/50 hover:bg-accent/70' : 'hover:bg-muted/50')
                  ]"
                  @click="multiSelectMode ? toggleSelect(child.id) : $emit('select', child)"
                  @dblclick="!multiSelectMode && $emit('edit', child)"
                >
                  <TableCell v-if="multiSelectMode" class="w-10 px-2 !py-0.5">
                    <button
                      class="flex items-center justify-center w-4 h-4 rounded border transition-colors"
                      :class="isSelected(child.id)
                        ? 'bg-primary border-primary text-primary-foreground'
                        : 'border-muted-foreground/30 hover:border-muted-foreground'"
                      @click.stop="toggleSelect(child.id)"
                    >
                      <svg v-if="isSelected(child.id)" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                        <polyline points="20 6 9 17 4 12" />
                      </svg>
                    </button>
                  </TableCell>
                  <TableCell v-for="col in visibleColumns" :key="col.id" class="!py-0.5" :class="col.id === 'title' ? 'whitespace-normal max-w-md' : ''">
                    <template v-if="col.id === 'id'">
                      <div class="pl-10">
                        <CopyableId :value="child.id" :display-value="getShortId(child.id)" />
                      </div>
                    </template>

                    <template v-else-if="col.id === 'type'">
                      <TypeBadge :type="child.type" size="sm" />
                    </template>

                    <template v-else-if="col.id === 'title'">
                      <span class="text-xs font-medium line-clamp-2 break-words">{{ child.title }}</span>
                    </template>

                    <template v-else-if="col.id === 'status'">
                      <StatusBadge :status="child.status" size="sm" />
                    </template>

                    <template v-else-if="col.id === 'priority'">
                      <PriorityBadge :priority="child.priority" size="sm" />
                    </template>

                    <template v-else-if="col.id === 'labels'">
                      <div v-if="child.labels?.length" class="flex flex-wrap gap-1">
                        <LabelBadge
                          v-for="label in child.labels"
                          :key="label"
                          :label="label"
                          size="sm"
                        />
                      </div>
                      <span v-else class="text-xs text-muted-foreground">-</span>
                    </template>

                    <template v-else-if="col.id === 'assignee'">
                      <span class="text-xs">{{ child.assignee || '-' }}</span>
                    </template>

                    <template v-else-if="col.id === 'createdAt'">
                      <div class="flex flex-col">
                        <span class="text-xs text-muted-foreground">{{ formatDate(child.createdAt) }}</span>
                        <span class="text-[10px] text-muted-foreground/70">{{ formatTime(child.createdAt) }}</span>
                      </div>
                    </template>

                    <template v-else-if="col.id === 'updatedAt'">
                      <div class="flex flex-col">
                        <span class="text-xs text-muted-foreground">{{ formatDate(child.updatedAt) }}</span>
                        <span class="text-[10px] text-muted-foreground/70">{{ formatTime(child.updatedAt) }}</span>
                      </div>
                    </template>

                    <template v-else-if="col.id === 'commentCount'">
                      <span v-if="child.commentCount" class="text-xs text-muted-foreground">{{ child.commentCount }}</span>
                      <span v-else class="text-xs text-muted-foreground">-</span>
                    </template>

                    <template v-else>
                      <span class="text-xs">{{ getIssueField(child, col.id) }}</span>
                    </template>
                  </TableCell>
                </TableRow>
              </template>
            </template>

            <!-- Non-epic issues (orphans or regular issues) -->
            <template v-else>
              <TableRow
                v-for="issue in group.children"
                :key="issue.id"
                class="cursor-pointer"
                :class="multiSelectMode
                  ? (isSelected(issue.id) ? 'bg-accent/50 hover:bg-accent/70' : 'hover:bg-muted/50')
                  : (selectedId === issue.id ? 'bg-accent/50 hover:bg-accent/70' : 'hover:bg-muted/50')"
                @click="multiSelectMode ? toggleSelect(issue.id) : $emit('select', issue)"
                @dblclick="!multiSelectMode && $emit('edit', issue)"
              >
                <TableCell v-if="multiSelectMode" class="w-10 px-2 !py-0.5">
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
                <TableCell v-for="col in visibleColumns" :key="col.id" class="!py-0.5" :class="col.id === 'title' ? 'whitespace-normal max-w-md' : ''">
                  <template v-if="col.id === 'id'">
                    <div class="pl-7">
                      <CopyableId :value="issue.id" :display-value="getShortId(issue.id)" />
                    </div>
                  </template>

                  <template v-else-if="col.id === 'type'">
                    <TypeBadge :type="issue.type" size="sm" />
                  </template>

                  <template v-else-if="col.id === 'title'">
                    <span class="text-xs font-medium line-clamp-2 break-words">{{ issue.title }}</span>
                  </template>

                  <template v-else-if="col.id === 'status'">
                    <div class="flex items-center gap-1">
                      <StatusBadge :status="issue.status" size="sm" />
                      <Tooltip v-if="issue.blockedBy?.length">
                        <TooltipTrigger as-child>
                          <Ban class="w-3 h-3 text-red-400" />
                        </TooltipTrigger>
                        <TooltipContent side="top">
                          <p class="text-xs">Blocked by {{ issue.blockedBy.join(', ') }}</p>
                        </TooltipContent>
                      </Tooltip>
                    </div>
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

                  <template v-else-if="col.id === 'commentCount'">
                    <span v-if="issue.commentCount" class="text-xs text-muted-foreground">{{ issue.commentCount }}</span>
                    <span v-else class="text-xs text-muted-foreground">-</span>
                  </template>

                  <template v-else>
                    <span class="text-xs">{{ getIssueField(issue, col.id) }}</span>
                  </template>
                </TableCell>
              </TableRow>
            </template>
          </template>
        </template>

        <!-- Flat display (fallback when no grouped issues) -->
        <template v-else>
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
            <TableCell v-if="multiSelectMode" class="w-10 px-2 !py-0.5">
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
            <TableCell v-for="col in visibleColumns" :key="col.id" class="!py-0.5" :class="col.id === 'title' ? 'whitespace-normal max-w-md' : ''">
              <template v-if="col.id === 'id'">
                <div class="pl-7">
                  <CopyableId :value="issue.id" :display-value="getShortId(issue.id)" />
                </div>
              </template>

              <template v-else-if="col.id === 'type'">
                <TypeBadge :type="issue.type" size="sm" />
              </template>

              <template v-else-if="col.id === 'title'">
                <span class="text-xs font-medium line-clamp-2 break-words">{{ issue.title }}</span>
              </template>

              <template v-else-if="col.id === 'status'">
                <div class="flex items-center gap-1">
                  <StatusBadge :status="issue.status" size="sm" />
                  <Tooltip v-if="issue.blockedBy?.length">
                    <TooltipTrigger as-child>
                      <Ban class="w-3 h-3 text-red-400" />
                    </TooltipTrigger>
                    <TooltipContent side="top">
                      <p class="text-xs">Blocked by {{ issue.blockedBy.join(', ') }}</p>
                    </TooltipContent>
                  </Tooltip>
                </div>
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

              <template v-else-if="col.id === 'commentCount'">
                <span v-if="issue.commentCount" class="text-xs text-muted-foreground">{{ issue.commentCount }}</span>
                <span v-else class="text-xs text-muted-foreground">-</span>
              </template>

              <template v-else>
                <span class="text-xs">{{ getIssueField(issue, col.id) }}</span>
              </template>
            </TableCell>
          </TableRow>
        </template>
      </TableBody>
    </Table>

    <!-- Load More Button -->
    <div v-if="hasMore" class="flex justify-center py-4 border-t border-border">
      <Button variant="outline" size="sm" @click="emit('loadMore')">
        Load more ({{ (totalCount ?? 0) - issues.length }} remaining)
      </Button>
    </div>
  </div>
</template>
