import type { ColumnConfig } from '~/types/issue'
import { useProjectStorage } from '~/composables/useProjectStorage'

const defaultColumns: ColumnConfig[] = [
  { id: 'id', label: 'ID', visible: true, sortable: true },
  { id: 'type', label: 'Type', visible: true, sortable: true },
  { id: 'labels', label: 'Labels', visible: true, sortable: true },
  { id: 'title', label: 'Title', visible: true, sortable: true },
  { id: 'status', label: 'Status', visible: true, sortable: true },
  { id: 'priority', label: 'Priority', visible: true, sortable: true },
  { id: 'assignee', label: 'Assignee', visible: true, sortable: true },
  { id: 'createdAt', label: 'Created', visible: true, sortable: true },
  { id: 'updatedAt', label: 'Updated', visible: true, sortable: true },
]

export function useColumnConfig() {
  const columns = useProjectStorage<ColumnConfig[]>('columns', defaultColumns)

  // Sync sortable property from defaults (in case it changed)
  const defaultSortableMap = new Map(defaultColumns.map(c => [c.id, c.sortable]))
  columns.value = columns.value.map(col => ({
    ...col,
    sortable: defaultSortableMap.get(col.id) ?? col.sortable,
  }))

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
