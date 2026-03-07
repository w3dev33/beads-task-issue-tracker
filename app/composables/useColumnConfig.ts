import type { ColumnConfig } from '~/types/issue'
import { useProjectStorage } from '~/composables/useProjectStorage'

const defaultColumns: ColumnConfig[] = [
  { id: 'id', label: 'ID', visible: true, sortable: true },
  { id: 'type', label: 'Type', visible: true, sortable: true },
  { id: 'labels', label: 'Labels', visible: true, sortable: true },
  { id: 'pinned', label: 'Pin', visible: true, sortable: true },
  { id: 'title', label: 'Title', visible: true, sortable: true },
  { id: 'status', label: 'Status', visible: true, sortable: true },
  { id: 'priority', label: 'Priority', visible: true, sortable: true },
  { id: 'assignee', label: 'Assignee', visible: true, sortable: true },
  { id: 'createdAt', label: 'Created', visible: true, sortable: true },
  { id: 'updatedAt', label: 'Updated', visible: true, sortable: true },
  { id: 'commentCount', label: 'Comments', visible: false, sortable: true },
]

export function useColumnConfig() {
  const columns = useProjectStorage<ColumnConfig[]>('columns', defaultColumns)

  // Sync with defaults: update sortable property and add any new columns
  const defaultSortableMap = new Map(defaultColumns.map(c => [c.id, c.sortable]))
  const existingIds = new Set(columns.value.map(c => c.id))
  const newColumns = defaultColumns.filter(c => !existingIds.has(c.id))
  let synced = [
    ...columns.value.map(col => ({
      ...col,
      sortable: defaultSortableMap.get(col.id) ?? col.sortable,
    })),
    ...newColumns,
  ]

  // Migration: ensure 'pinned' column is positioned right before 'title'
  const pinnedIdx = synced.findIndex(c => c.id === 'pinned')
  const titleIdx = synced.findIndex(c => c.id === 'title')
  if (pinnedIdx >= 0 && titleIdx >= 0 && pinnedIdx !== titleIdx - 1) {
    const [pinnedCol] = synced.splice(pinnedIdx, 1)
    const newTitleIdx = synced.findIndex(c => c.id === 'title')
    synced.splice(newTitleIdx, 0, pinnedCol!)
  }

  columns.value = synced

  const visibleColumns = computed(() =>
    columns.value.filter((col) => col.visible)
  )

  const toggleColumn = (columnId: string) => {
    columns.value = columns.value.map((col) =>
      col.id === columnId ? { ...col, visible: !col.visible } : col
    )
  }

  const setColumnVisibility = (columnId: string, visible: boolean) => {
    columns.value = columns.value.map((col) =>
      col.id === columnId ? { ...col, visible } : col
    )
  }

  const setColumns = (newColumns: ColumnConfig[]) => {
    columns.value = newColumns
  }

  const resetColumns = () => {
    columns.value = [...defaultColumns]
  }

  return {
    columns,
    visibleColumns,
    toggleColumn,
    setColumnVisibility,
    setColumns,
    resetColumns,
  }
}
