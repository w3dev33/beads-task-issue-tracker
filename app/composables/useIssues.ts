import type { Issue, CreateIssuePayload, UpdateIssuePayload } from '~/types/issue'
import { bdList, bdCount, bdShow, bdCreate, bdUpdate, bdClose, bdDelete, bdAddComment, type BdListOptions } from '~/utils/bd-api'

// Shared state across all components (singleton pattern)
const issues = ref<Issue[]>([])
const selectedIssue = ref<Issue | null>(null)
const isLoading = ref(false)
const isUpdating = ref(false)
const error = ref<string | null>(null)

// Polling state for change detection
const lastKnownCount = ref<number>(0)
const lastKnownUpdated = ref<string | null>(null)

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

    // Apply search filter
    const searchTerm = filters.value.search?.trim()
    if (searchTerm) {
      const search = searchTerm.toLowerCase()
      result = result.filter(
        (issue) =>
          issue.title.toLowerCase().includes(search) ||
          issue.id.toLowerCase().includes(search) ||
          issue.description?.toLowerCase().includes(search)
      )
    }

    return result
  })

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
    selectedIssue,
    isLoading,
    isUpdating,
    error,
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
