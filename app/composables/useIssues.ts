import type { Issue, CreateIssuePayload, UpdateIssuePayload } from '~/types/issue'
import { bdList, bdCount, bdShow, bdCreate, bdUpdate, bdClose, bdDelete, bdAddComment, bdPurgeOrphanAttachments, type BdListOptions } from '~/utils/bd-api'

// Shared state across all components (singleton pattern)
const issues = ref<Issue[]>([])
const selectedIssue = ref<Issue | null>(null)
const isLoading = ref(false)
const isUpdating = ref(false)
const error = ref<string | null>(null)

// Polling state for change detection
const lastKnownCount = ref<number>(0)
const lastKnownUpdated = ref<string | null>(null)

// Pagination state
const pageSize = ref(50)
const currentPage = ref(1)
const sortField = ref<string | null>('updatedAt')
const sortDirection = ref<'asc' | 'desc'>('desc')

export function useIssues() {
  const { filters } = useFilters()
  const { beadsPath } = useBeadsPath()

  // Helper to get the current path (for IPC or web)
  const getPath = () => beadsPath.value && beadsPath.value !== '.' ? beadsPath.value : undefined

  const fetchIssues = async (ignoreFilters = false, silent = false) => {
    if (!silent) {
      isLoading.value = true
    }
    error.value = null

    try {
      // Workaround for bd CLI bug: --all flag returns incorrect results
      // Instead, we make two parallel calls and merge the results:
      // 1. bd list (returns non-closed issues correctly)
      // 2. bd list --status=closed (returns closed issues)
      const path = getPath()

      const [openIssues, closedIssues] = await Promise.all([
        bdList({ path }),
        bdList({ path, status: ['closed'] }),
      ])

      const newIssues = [...(openIssues || []), ...(closedIssues || [])]

      // Only update if data actually changed (compare by serialization)
      const currentSignature = JSON.stringify(issues.value.map(i => i.id + i.updatedAt))
      const newSignature = JSON.stringify(newIssues.map(i => i.id + i.updatedAt))

      if (currentSignature !== newSignature) {
        issues.value = newIssues

        // Sync selectedIssue with updated data (e.g., if closed externally via CLI)
        if (selectedIssue.value) {
          const updatedSelected = newIssues.find(i => i.id === selectedIssue.value!.id)
          if (updatedSelected && updatedSelected.status !== selectedIssue.value.status) {
            // Status changed - update selectedIssue to trigger watchers
            selectedIssue.value = updatedSelected
          }
        }
      }

      // Update polling state
      lastKnownCount.value = newIssues.length
      const maxUpdated = newIssues.reduce((max, issue) => {
        return issue.updatedAt > max ? issue.updatedAt : max
      }, '')
      lastKnownUpdated.value = maxUpdated || null
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch issues'
    } finally {
      if (!silent) {
        isLoading.value = false
      }
    }
  }

  // Check if there are changes without fetching full list
  const checkForChanges = async (): Promise<boolean> => {
    try {
      const data = await bdCount(getPath())

      const hasChanges =
        data.count !== lastKnownCount.value ||
        data.lastUpdated !== lastKnownUpdated.value

      return hasChanges
    } catch {
      return false
    }
  }

  const filteredIssues = computed(() => {
    let result = issues.value

    // Check if search is active - if so, bypass all other filters
    // Search takes priority and searches ALL issues including closed
    const searchTerm = filters.value.search?.trim()
    if (searchTerm) {
      const search = searchTerm.toLowerCase()
      return result.filter(
        (issue) =>
          issue.title.toLowerCase().includes(search) ||
          issue.id.toLowerCase().includes(search) ||
          issue.description?.toLowerCase().includes(search)
      )
    }

    // No search active - apply additive filters (AND logic)

    // Apply client-side status filter
    // When no status filter is selected, exclude closed issues by default
    if (filters.value.status.length > 0) {
      result = result.filter((issue) =>
        filters.value.status.includes(issue.status)
      )
    } else {
      // Default: show all except closed
      result = result.filter((issue) => issue.status !== 'closed')
    }

    // Apply client-side type filter
    if (filters.value.type.length > 0) {
      result = result.filter((issue) =>
        filters.value.type.includes(issue.type)
      )
    }

    // Apply client-side priority filter
    if (filters.value.priority.length > 0) {
      result = result.filter((issue) =>
        filters.value.priority.includes(issue.priority)
      )
    }

    // Apply client-side assignee filter
    if (filters.value.assignee) {
      result = result.filter((issue) =>
        issue.assignee === filters.value.assignee
      )
    }

    // Apply client-side label filter (AND logic - issue must have ALL selected labels)
    if (filters.value.labels.length > 0) {
      result = result.filter((issue) =>
        filters.value.labels.every(filterLabel =>
          issue.labels?.some(l => l.toLowerCase() === filterLabel.toLowerCase())
        )
      )
    }

    return result
  })

  // Sort order for status and priority (matching IssueTable logic)
  const statusOrder: Record<string, number> = {
    in_progress: 0,
    open: 1,
    blocked: 2,
    closed: 3,
  }

  const priorityOrder: Record<string, number> = {
    p0: 0,
    p1: 1,
    p2: 2,
    p3: 3,
    p4: 4,
  }

  const typeOrder: Record<string, number> = {
    bug: 0,
    feature: 1,
    task: 2,
    epic: 3,
    chore: 4,
  }

  // Computed for sorted issues (default: updatedAt DESC, null = no sort)
  const sortedIssues = computed(() => {
    // If no sort field, return unsorted
    if (!sortField.value) {
      return filteredIssues.value
    }

    const sorted = [...filteredIssues.value]
    const dir = sortDirection.value === 'asc' ? 1 : -1
    const field = sortField.value

    sorted.sort((a, b) => {
      let aVal: string | number | null = null
      let bVal: string | number | null = null

      switch (field) {
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
          aVal = a.labels?.length ? a.labels[0]!.toLowerCase() : '\uffff'
          bVal = b.labels?.length ? b.labels[0]!.toLowerCase() : '\uffff'
          break
        case 'createdAt':
        case 'updatedAt':
          aVal = a[field] ? new Date(a[field]).getTime() : 0
          bVal = b[field] ? new Date(b[field]).getTime() : 0
          break
        default:
          aVal = String(a[field as keyof Issue] ?? '').toLowerCase()
          bVal = String(b[field as keyof Issue] ?? '').toLowerCase()
      }

      if (aVal < bVal) return -1 * dir
      if (aVal > bVal) return 1 * dir
      return 0
    })

    return sorted
  })

  // Computed for paginated issues
  const paginatedIssues = computed(() => {
    const end = currentPage.value * pageSize.value
    return sortedIssues.value.slice(0, end)
  })

  const totalPages = computed(() =>
    Math.ceil(filteredIssues.value.length / pageSize.value)
  )

  const hasMore = computed(() =>
    currentPage.value < totalPages.value
  )

  // Pagination functions
  function loadMore() {
    if (hasMore.value) currentPage.value++
  }

  function resetPagination() {
    currentPage.value = 1
  }

  function setSort(field: string | null, direction: 'asc' | 'desc') {
    sortField.value = field
    sortDirection.value = direction
    resetPagination()
  }

  // Watch filters to reset pagination when they change
  watch(
    () => filters.value,
    () => {
      resetPagination()
    },
    { deep: true }
  )

  const fetchIssue = async (id: string) => {
    isLoading.value = true
    error.value = null

    try {
      const data = await bdShow(id, getPath())
      selectedIssue.value = data
      return data
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch issue'
      return null
    } finally {
      isLoading.value = false
    }
  }

  const createIssue = async (payload: CreateIssuePayload) => {
    isLoading.value = true
    error.value = null

    try {
      const data = await bdCreate(payload, getPath())
      await fetchIssues()
      return data
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create issue'
      return null
    } finally {
      isLoading.value = false
    }
  }

  const updateIssue = async (id: string, payload: UpdateIssuePayload) => {
    isUpdating.value = true
    error.value = null

    try {
      const data = await bdUpdate(id, payload, getPath())

      // Update local list directly with API response (no need to refetch)
      if (data) {
        const index = issues.value.findIndex(i => i.id === id)
        if (index !== -1) {
          issues.value[index] = data
        }
        if (selectedIssue.value?.id === id) {
          selectedIssue.value = data
        }
      }

      return data
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update issue'
      return null
    } finally {
      isUpdating.value = false
    }
  }

  const closeIssue = async (id: string) => {
    isLoading.value = true
    error.value = null

    try {
      await bdClose(id, getPath())
      await fetchIssues()
      if (selectedIssue.value?.id === id) {
        await fetchIssue(id)
      }
      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to close issue'
      return false
    } finally {
      isLoading.value = false
    }
  }

  const deleteIssue = async (id: string) => {
    isUpdating.value = true
    error.value = null

    try {
      await bdDelete(id, getPath())

      // Remove from local list
      const index = issues.value.findIndex(i => i.id === id)
      if (index !== -1) {
        issues.value.splice(index, 1)
        lastKnownCount.value = issues.value.length
      }

      // Clear selection if deleted issue was selected
      if (selectedIssue.value?.id === id) {
        selectedIssue.value = null
      }

      // Purge orphan attachments (silently, no need to wait or handle errors)
      bdPurgeOrphanAttachments(getPath()).catch(() => {
        // Silently ignore purge errors - it's a cleanup operation
      })

      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to delete issue'
      return false
    } finally {
      isUpdating.value = false
    }
  }

  const selectIssue = (issue: Issue | null) => {
    selectedIssue.value = issue
  }

  const addComment = async (id: string, content: string) => {
    isUpdating.value = true
    error.value = null

    try {
      await bdAddComment(id, content, getPath())

      // Refetch the issue to get updated comments
      if (selectedIssue.value?.id === id) {
        await fetchIssue(id)
      }

      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to add comment'
      return false
    } finally {
      isUpdating.value = false
    }
  }

  // Clear all issues data (used when removing last favorite)
  const clearIssues = () => {
    issues.value = []
    selectedIssue.value = null
    lastKnownCount.value = 0
    lastKnownUpdated.value = null
    error.value = null
  }

  return {
    issues,
    filteredIssues,
    sortedIssues,
    paginatedIssues,
    selectedIssue,
    isLoading,
    isUpdating,
    error,
    // Pagination
    currentPage,
    pageSize,
    totalPages,
    hasMore,
    loadMore,
    resetPagination,
    sortField,
    sortDirection,
    setSort,
    // Actions
    fetchIssues,
    fetchIssue,
    createIssue,
    updateIssue,
    closeIssue,
    deleteIssue,
    selectIssue,
    addComment,
    checkForChanges,
    clearIssues,
  }
}
