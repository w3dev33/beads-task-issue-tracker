import type { Issue, CreateIssuePayload, UpdateIssuePayload } from '~/types/issue'
import { bdList, bdCount, bdShow, bdCreate, bdUpdate, bdClose, bdDelete, bdAddComment, bdAddDependency, bdRemoveDependency, bdAddRelation, bdRemoveRelation, bdPurgeOrphanAttachments, bdPollData, bdSearch, bdLabelAdd, bdLabelRemove, type BdListOptions, type PollData } from '~/utils/bd-api'
import { useProjectStorage } from '~/composables/useProjectStorage'
import {
  deduplicateIssues,
  naturalCompare,
  getParentIdFromIssue,
  compareChildIssues,
  statusOrder,
  priorityOrder,
  typeOrder,
  filterIssues as filterIssuesPure,
  sortIssues as sortIssuesPure,
  groupIssues as groupIssuesPure,
} from '~/utils/issue-helpers'

// Interface for hierarchical grouping of epics and their children
export interface IssueGroup {
  epic: Issue | null  // null for orphan issues (no parent, not an epic)
  children: Issue[]   // child issues of the epic
  childCount: number
  closedChildCount: number
  inProgressChild?: { id: string; title: string; priority: string }
}

// Shared state across all components (singleton pattern)
const issues = ref<Issue[]>([])
const selectedIssue = ref<Issue | null>(null)
const isLoading = ref(false)
const isUpdating = ref(false)
const error = ref<string | null>(null)

// Polling state for change detection
const lastKnownCount = ref<number>(0)
const lastKnownUpdated = ref<string | null>(null)

// Track newly added issue IDs for flash animation
const newlyAddedIds = ref<Set<string>>(new Set())

const markAsNewlyAdded = (id: string) => {
  newlyAddedIds.value = new Set(newlyAddedIds.value).add(id)
  setTimeout(() => {
    const next = new Set(newlyAddedIds.value)
    next.delete(id)
    newlyAddedIds.value = next
  }, 3000)
}

// Pagination state
const pageSize = ref(50)
const currentPage = ref(1)
const sortField = useProjectStorage<string | null>('sortField', 'updatedAt')
const sortDirection = useProjectStorage<'asc' | 'desc'>('sortDirection', 'desc')

// Epic expand/collapse state (persisted per project) - default to expanded (true)
const expandedEpics = useProjectStorage<Record<string, boolean>>('expandedEpics', {})

const isEpicExpanded = (epicId: string) => {
  // Default to expanded (true) if not explicitly set to false
  return expandedEpics.value[epicId] !== false
}

const toggleEpicExpand = (epicId: string) => {
  expandedEpics.value[epicId] = !isEpicExpanded(epicId)
}

const expandEpic = (epicId: string) => {
  expandedEpics.value[epicId] = true
}

/**
 * Separate composable for epic expand/collapse state.
 * Use this instead of useIssues() in components that only need
 * epic expand/collapse functionality (e.g., IssueTable.vue).
 * This avoids creating duplicate computed properties and watchers.
 */
export function useEpicExpand() {
  return { isEpicExpanded, toggleEpicExpand, expandEpic }
}

export function useIssues() {
  const { filters } = useFilters()
  const { beadsPath } = useBeadsPath()
  const { checkError: checkRepairError } = useRepairDatabase()
  const { checkError: checkMigrateError } = useMigrateToDolt()
  const { exclusions } = useExclusionFilters()

  // Helper to get the current path (for IPC or web)
  const getPath = () => beadsPath.value && beadsPath.value !== '.' ? beadsPath.value : undefined

  const fetchIssues = async (ignoreFilters = false, silent = false) => {
    if (!silent) {
      isLoading.value = true
    }
    error.value = null

    try {
      // Single call with --all to get all issues (bd >= 0.55 fixed the --all flag)
      const path = getPath()

      const allIssues = await bdList({ path, includeAll: true })
      const newIssues = deduplicateIssues(allIssues || [])

      // Build parent-child relationships from the data we already have (no bdShow needed)
      const issueMap = new Map(newIssues.map(i => [i.id, i]))

      // 1. Fill in parent details
      //    - Use explicit parent.id from bd list if available (bd < 0.50)
      //    - Derive from dot notation in ID for bd >= 0.50 (e.g., "abc.1" → parent "abc")
      for (const issue of newIssues) {
        // Derive parent from ID pattern if not already set
        if (!issue.parent?.id) {
          const lastDot = issue.id.lastIndexOf('.')
          if (lastDot !== -1) {
            const suffix = issue.id.slice(lastDot + 1)
            if (/^\d+$/.test(suffix)) {
              const derivedParentId = issue.id.slice(0, lastDot)
              const parentIssue = issueMap.get(derivedParentId)
              if (parentIssue) {
                issue.parent = {
                  id: parentIssue.id,
                  title: parentIssue.title,
                  status: parentIssue.status,
                  priority: parentIssue.priority,
                }
              }
            }
          }
        } else {
          // Enrich explicit parent with full details from loaded data
          const parentIssue = issueMap.get(issue.parent.id)
          if (parentIssue) {
            issue.parent = {
              id: parentIssue.id,
              title: parentIssue.title,
              status: parentIssue.status,
              priority: parentIssue.priority,
            }
          }
        }
      }

      // 2. Build children lists for epics from child issues that reference them
      const childrenByParent = new Map<string, Array<{ id: string; title: string; status: typeof newIssues[0]['status']; priority: typeof newIssues[0]['priority'] }>>()
      for (const issue of newIssues) {
        if (issue.parent?.id) {
          const list = childrenByParent.get(issue.parent.id) || []
          list.push({ id: issue.id, title: issue.title, status: issue.status, priority: issue.priority })
          childrenByParent.set(issue.parent.id, list)
        }
      }
      for (const [epicId, children] of childrenByParent) {
        const epic = issueMap.get(epicId)
        if (epic) {
          epic.children = children
        }
      }

      // Only update if data actually changed (compare by serialization)
      const currentSignature = JSON.stringify(issues.value.map(i => i.id + i.updatedAt))
      const newSignature = JSON.stringify(newIssues.map(i => i.id + i.updatedAt))

      if (currentSignature !== newSignature) {
        // Detect newly added issues (skip initial load)
        if (issues.value.length > 0) {
          const existingIds = new Set(issues.value.map(i => i.id))
          const addedIds = newIssues.filter(i => !existingIds.has(i.id))

          // If most IDs changed, this is a project switch — don't flash
          const isProjectSwitch = addedIds.length > issues.value.length * 0.5
          if (!isProjectSwitch) {
            for (const issue of addedIds) {
              markAsNewlyAdded(issue.id)
            }
          }
        }
        issues.value = newIssues
      }

      // Sync selectedIssue with updated data (e.g., if modified externally via CLI)
      // Check even if list signature unchanged — selectedIssue may be stale from a
      // previous bdShow while the list was already updated by an earlier poll
      if (selectedIssue.value) {
        const updatedSelected = newIssues.find(i => i.id === selectedIssue.value!.id)
        if (!updatedSelected) {
          selectedIssue.value = null
        } else if (updatedSelected.updatedAt !== selectedIssue.value.updatedAt) {
          fetchIssue(updatedSelected.id)
        }
      }

      // Update polling state
      lastKnownCount.value = newIssues.length
      const maxUpdated = newIssues.reduce((max, issue) => {
        return issue.updatedAt > max ? issue.updatedAt : max
      }, '')
      lastKnownUpdated.value = maxUpdated || null

    } catch (e) {
      // Check for Dolt migration error first (more specific)
      if (checkMigrateError(e, beadsPath.value)) {
        error.value = 'Database needs migration to Dolt backend.'
      // Check if this is a schema migration error that needs repair
      } else if (checkRepairError(e)) {
        error.value = 'Database needs repair due to a schema migration issue.'
      } else {
        error.value = e instanceof Error ? e.message : 'Failed to fetch issues'
      }
    } finally {
      if (!silent) {
        isLoading.value = false
      }
    }
  }

  /**
   * Fetch all poll data in a single batched IPC call.
   * Used by the polling system for lower overhead (1 IPC instead of 3).
   * Returns the ready issues for dashboard use.
   */
  const fetchPollData = async (): Promise<Issue[] | null> => {
    error.value = null
    try {
      const path = getPath()
      const data = await bdPollData(path)

      const mergedIssues = [...(data.openIssues || []), ...(data.closedIssues || [])]
      const newIssues = deduplicateIssues(mergedIssues)

      // Build parent-child relationships from the data we already have (no bdShow needed)
      const issueMap = new Map(newIssues.map(i => [i.id, i]))

      // Derive parent from dot notation if not set (bd >= 0.50)
      for (const issue of newIssues) {
        if (!issue.parent?.id) {
          const lastDot = issue.id.lastIndexOf('.')
          if (lastDot !== -1) {
            const suffix = issue.id.slice(lastDot + 1)
            if (/^\d+$/.test(suffix)) {
              const derivedParentId = issue.id.slice(0, lastDot)
              const parentIssue = issueMap.get(derivedParentId)
              if (parentIssue) {
                issue.parent = { id: parentIssue.id, title: parentIssue.title, status: parentIssue.status, priority: parentIssue.priority }
              }
            }
          }
        } else {
          const parentIssue = issueMap.get(issue.parent.id)
          if (parentIssue) {
            issue.parent = { id: parentIssue.id, title: parentIssue.title, status: parentIssue.status, priority: parentIssue.priority }
          }
        }
      }

      const childrenByParent = new Map<string, Array<{ id: string; title: string; status: typeof newIssues[0]['status']; priority: typeof newIssues[0]['priority'] }>>()
      for (const issue of newIssues) {
        if (issue.parent?.id) {
          const list = childrenByParent.get(issue.parent.id) || []
          list.push({ id: issue.id, title: issue.title, status: issue.status, priority: issue.priority })
          childrenByParent.set(issue.parent.id, list)
        }
      }
      for (const [epicId, children] of childrenByParent) {
        const epic = issueMap.get(epicId)
        if (epic) {
          epic.children = children
        }
      }

      // Preserve blockedBy/blocks from previous enrichments (fetchIssues or fetchIssue)
      // Poll data doesn't return these, but they were populated by earlier bdShow calls
      const existingMap = new Map(issues.value.map(i => [i.id, i]))
      for (const issue of newIssues) {
        const existing = existingMap.get(issue.id)
        if (existing) {
          if (!issue.blockedBy && existing.blockedBy) issue.blockedBy = existing.blockedBy
          if (!issue.blocks && existing.blocks) issue.blocks = existing.blocks
        }
      }

      // Only update if data actually changed
      // Signature includes status+priority+title so probe mode (no updatedAt) still detects changes
      const sig = (i: Issue) => `${i.id}|${i.updatedAt}|${i.status}|${i.priority}|${i.title}`
      const currentSignature = JSON.stringify(issues.value.map(sig))
      const newSignature = JSON.stringify(newIssues.map(sig))

      if (currentSignature !== newSignature) {
        // Detect newly added + externally modified issues (skip initial load)
        if (issues.value.length > 0) {
          const existingSigs = new Map(issues.value.map(i => [i.id, sig(i)]))
          const addedIds: string[] = []
          const modifiedIds: string[] = []

          for (const issue of newIssues) {
            const oldSig = existingSigs.get(issue.id)
            if (!oldSig) {
              addedIds.push(issue.id)
            } else if (oldSig !== sig(issue)) {
              modifiedIds.push(issue.id)
            }
          }

          // If most IDs changed, this is a project switch — don't flash
          const isProjectSwitch = addedIds.length > issues.value.length * 0.5
          if (!isProjectSwitch) {
            for (const id of [...addedIds, ...modifiedIds]) {
              markAsNewlyAdded(id)
            }
          }
        }
        issues.value = newIssues
      }

      // Sync selectedIssue with updated data (check even if list signature unchanged,
      // because selectedIssue may be stale from a previous bdShow while the list was
      // already updated by an earlier poll before the issue was selected)
      if (selectedIssue.value) {
        const updatedSelected = newIssues.find(i => i.id === selectedIssue.value!.id)
        if (!updatedSelected) {
          selectedIssue.value = null
        } else if (updatedSelected.updatedAt !== selectedIssue.value.updatedAt) {
          fetchIssue(updatedSelected.id)
        }
      }

      // Update polling state
      lastKnownCount.value = newIssues.length
      const maxUpdated = newIssues.reduce((max, issue) => {
        return issue.updatedAt > max ? issue.updatedAt : max
      }, '')
      lastKnownUpdated.value = maxUpdated || null

      return data.readyIssues || []
    } catch (e) {
      if (checkMigrateError(e, beadsPath.value)) {
        error.value = 'Database needs migration to Dolt backend.'
      } else if (checkRepairError(e)) {
        error.value = 'Database needs repair due to a schema migration issue.'
      } else {
        error.value = e instanceof Error ? e.message : 'Failed to fetch poll data'
      }
      return null
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

  const filteredIssues = computed(() =>
    filterIssuesPure(issues.value, filters.value, exclusions.value)
  )

  // Computed for sorted issues (default: updatedAt DESC, null = no sort)
  const sortedIssues = computed(() =>
    sortIssuesPure(filteredIssues.value, sortField.value, sortDirection.value)
  )

  // Computed for paginated issues
  const paginatedIssues = computed(() => {
    const end = currentPage.value * pageSize.value
    return sortedIssues.value.slice(0, end)
  })

  // Computed for hierarchical grouping of epics and their children
  const groupedIssues = computed((): IssueGroup[] =>
    groupIssuesPure(paginatedIssues.value, issues.value)
  )

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
    error.value = null

    try {
      const data = await bdShow(id, getPath())

      if (data === null) {
        selectedIssue.value = null
        error.value = `Issue ${id} not found`
        return null
      }

      // Enrich parent/children from loaded issues list (bd >= 0.50 doesn't return these)
      if (!data.parent?.id) {
        const lastDot = data.id.lastIndexOf('.')
        if (lastDot !== -1 && /^\d+$/.test(data.id.slice(lastDot + 1))) {
          const parentIssue = issues.value.find(i => i.id === data.id.slice(0, lastDot))
          if (parentIssue) {
            data.parent = { id: parentIssue.id, title: parentIssue.title, status: parentIssue.status, priority: parentIssue.priority }
          }
        }
      }
      if (!data.children?.length) {
        const prefix = data.id + '.'
        const children = issues.value
          .filter(i => i.id.startsWith(prefix) && !i.id.slice(prefix.length).includes('.'))
          .map(i => ({ id: i.id, title: i.title, status: i.status, priority: i.priority }))
        if (children.length) {
          data.children = children
        }
      }

      selectedIssue.value = data

      // Also update the issue in the issues array to keep in sync
      const index = issues.value.findIndex(i => i.id === id)
      if (index !== -1) {
        issues.value[index] = data
      }

      return data
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch issue'
      return null
    }
  }

  const createIssue = async (payload: CreateIssuePayload) => {
    isLoading.value = true
    error.value = null

    try {
      const data = await bdCreate(payload, getPath())
      if (data?.id) markAsNewlyAdded(data.id)
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
      const result = await bdClose(id, getPath())
      await fetchIssues()
      if (selectedIssue.value?.id === id) {
        await fetchIssue(id)
      }
      return result
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

  const addDependency = async (issueId: string, blockerId: string) => {
    if (isUpdating.value) return false

    isUpdating.value = true
    error.value = null

    try {
      await bdAddDependency(issueId, blockerId, getPath())

      // Refetch the issue to get updated dependencies
      if (selectedIssue.value?.id === issueId) {
        await fetchIssue(issueId)
      }

      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to add dependency'
      return false
    } finally {
      isUpdating.value = false

    }
  }

  const removeDependency = async (issueId: string, blockerId: string) => {
    if (isUpdating.value) return false

    isUpdating.value = true
    error.value = null

    try {
      await bdRemoveDependency(issueId, blockerId, getPath())

      // Refetch the issue to get updated dependencies
      if (selectedIssue.value?.id === issueId) {
        await fetchIssue(issueId)
      }

      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to remove dependency'
      return false
    } finally {
      isUpdating.value = false

    }
  }

  const addRelation = async (issueId: string, targetId: string, relationType: string) => {
    if (isUpdating.value) return false

    isUpdating.value = true
    error.value = null

    try {
      await bdAddRelation(issueId, targetId, relationType, getPath())

      // Refetch the issue to get updated relations
      if (selectedIssue.value?.id === issueId) {
        await fetchIssue(issueId)
      }

      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to add relation'
      return false
    } finally {
      isUpdating.value = false

    }
  }

  const removeRelation = async (sourceId: string, targetId: string) => {
    if (isUpdating.value) return false

    isUpdating.value = true
    error.value = null

    try {
      await bdRemoveRelation(sourceId, targetId, getPath())

      // Refetch the selected issue to get updated relations
      // (sourceId may be the selected issue or the target, depending on direction)
      const selected = selectedIssue.value?.id
      if (selected && (selected === sourceId || selected === targetId)) {
        await fetchIssue(selected)
      }

      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to remove relation'
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
    newlyAddedIds.value = new Set()
  }

  // br-only: Full-text search via CLI (replaces client-side filtering when br is active)
  const searchIssues = async (query: string) => {
    error.value = null
    try {
      const results = await bdSearch(query, getPath())
      issues.value = deduplicateIssues(results || [])
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Search failed'
    }
  }

  // br-only: Granular label add (uses `br label add` instead of full update)
  const addLabel = async (id: string, label: string) => {
    isUpdating.value = true
    error.value = null
    try {
      await bdLabelAdd(id, label, getPath())
      // Refresh the issue to reflect the change
      await fetchIssue(id)
      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to add label'
      return false
    } finally {
      isUpdating.value = false
    }
  }

  // br-only: Granular label remove (uses `br label remove` instead of full update)
  const removeLabel = async (id: string, label: string) => {
    isUpdating.value = true
    error.value = null
    try {
      await bdLabelRemove(id, label, getPath())
      // Refresh the issue to reflect the change
      await fetchIssue(id)
      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to remove label'
      return false
    } finally {
      isUpdating.value = false
    }
  }

  return {
    issues,
    filteredIssues,
    sortedIssues,
    paginatedIssues,
    groupedIssues,
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
    // Epic expand/collapse
    isEpicExpanded,
    toggleEpicExpand,
    expandEpic,
    // Actions
    fetchIssues,
    fetchPollData,
    fetchIssue,
    createIssue,
    updateIssue,
    closeIssue,
    deleteIssue,
    selectIssue,
    addComment,
    addDependency,
    removeDependency,
    addRelation,
    removeRelation,
    checkForChanges,
    clearIssues,
    newlyAddedIds,
    // br-only features
    searchIssues,
    addLabel,
    removeLabel,
  }
}
