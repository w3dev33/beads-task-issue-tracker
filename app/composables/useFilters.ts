import type { FilterState, IssueStatus, IssueType, IssuePriority } from '~/types/issue'

const defaultFilters: FilterState = {
  status: [],
  type: [],
  priority: [],
  assignee: null,
  search: '',
}

export function useFilters() {
  const filters = useLocalStorage<FilterState>('beads:filters', defaultFilters)

  // Clear search on init (search should not persist across page loads)
  if (import.meta.client) {
    filters.value.search = ''
  }

  const toggleStatus = (status: IssueStatus) => {
    const index = filters.value.status.indexOf(status)
    if (index === -1) {
      filters.value.status.push(status)
    } else {
      filters.value.status.splice(index, 1)
    }
  }

  const toggleType = (type: IssueType) => {
    const index = filters.value.type.indexOf(type)
    if (index === -1) {
      filters.value.type.push(type)
    } else {
      filters.value.type.splice(index, 1)
    }
  }

  const togglePriority = (priority: IssuePriority) => {
    const index = filters.value.priority.indexOf(priority)
    if (index === -1) {
      filters.value.priority.push(priority)
    } else {
      filters.value.priority.splice(index, 1)
    }
  }

  const setAssignee = (assignee: string | null) => {
    filters.value.assignee = assignee
  }

  const setSearch = (search: string) => {
    filters.value.search = search
  }

  const clearFilters = () => {
    filters.value.status = []
    filters.value.type = []
    filters.value.priority = []
    filters.value.assignee = null
    filters.value.search = ''
  }

  const setStatusFilter = (statuses: IssueStatus[]) => {
    filters.value.status = [...statuses]
  }

  const hasActiveFilters = computed(() => {
    return (
      filters.value.status.length > 0 ||
      filters.value.type.length > 0 ||
      filters.value.priority.length > 0 ||
      filters.value.assignee !== null ||
      filters.value.search !== ''
    )
  })

  return {
    filters,
    toggleStatus,
    toggleType,
    togglePriority,
    setAssignee,
    setSearch,
    clearFilters,
    setStatusFilter,
    hasActiveFilters,
  }
}
