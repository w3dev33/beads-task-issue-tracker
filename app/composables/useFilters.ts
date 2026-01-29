import type { FilterState, IssueStatus, IssueType, IssuePriority } from '~/types/issue'

const defaultFilters: FilterState = {
  status: [],
  type: [],
  priority: [],
  assignee: [],
  search: '',
  labels: [],
}

export function useFilters() {
  const filters = useLocalStorage<FilterState>('beads:filters', defaultFilters)

  // Clear search, labels and assignee on init (should not persist across page loads)
  if (import.meta.client) {
    filters.value.search = ''
    filters.value.labels = []
    filters.value.assignee = []
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

  const toggleAssignee = (assignee: string) => {
    const index = filters.value.assignee.indexOf(assignee)
    if (index === -1) {
      filters.value.assignee.push(assignee)
    } else {
      filters.value.assignee.splice(index, 1)
    }
  }

  const setSearch = (search: string) => {
    filters.value.search = search
  }

  const toggleLabelFilter = (label: string) => {
    const index = filters.value.labels.indexOf(label)
    if (index === -1) {
      filters.value.labels.push(label)
    } else {
      filters.value.labels.splice(index, 1)
    }
  }

  const clearFilters = () => {
    filters.value.status = []
    filters.value.type = []
    filters.value.priority = []
    filters.value.assignee = []
    filters.value.search = ''
    filters.value.labels = []
  }

  const setStatusFilter = (statuses: IssueStatus[]) => {
    filters.value.status = [...statuses]
  }

  const hasActiveFilters = computed(() => {
    return (
      filters.value.status.length > 0 ||
      filters.value.type.length > 0 ||
      filters.value.priority.length > 0 ||
      filters.value.assignee.length > 0 ||
      filters.value.search !== '' ||
      filters.value.labels.length > 0
    )
  })

  return {
    filters,
    toggleStatus,
    toggleType,
    togglePriority,
    toggleAssignee,
    setSearch,
    toggleLabelFilter,
    clearFilters,
    setStatusFilter,
    hasActiveFilters,
  }
}
