import type { Issue, CreateIssuePayload, UpdateIssuePayload } from '~/types/issue'
import { bdList, bdCount, bdShow, bdCreate, bdUpdate, bdClose, bdDelete, bdAddComment, bdAddDependency, bdRemoveDependency, bdAddRelation, bdRemoveRelation, bdPurgeOrphanAttachments, bdPollData, type BdListOptions, type PollData } from '~/utils/bd-api'
import { useProjectStorage } from '~/composables/useProjectStorage'

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

// Pagination state
const pageSize = ref(50)
const currentPage = ref(1)
const sortField = ref<string | null>('updatedAt')
const sortDirection = ref<'asc' | 'desc'>('desc')

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
 * Deduplicate issues by ID, keeping the most recently updated version.
 * Handles edge cases where an issue appears in both open and closed lists
 * (e.g., when reopened externally and sync/caches are out of sync).
 */
function deduplicateIssues(issues: Issue[]): Issue[] {
  const issueMap = new Map<string, Issue>()

  for (const issue of issues) {
    const existing = issueMap.get(issue.id)

    if (!existing) {
      issueMap.set(issue.id, issue)
    } else {
      // Keep the one with the most recent updatedAt
      const existingDate = new Date(existing.updatedAt).getTime()
      const currentDate = new Date(issue.updatedAt).getTime()

      if (currentDate > existingDate) {
        issueMap.set(issue.id, issue)
      }
    }
  }

  return Array.from(issueMap.values())
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
  const { exclusions } = useExclusionFilters()

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

      const mergedIssues = [...(openIssues || []), ...(closedIssues || [])]
      const newIssues = deduplicateIssues(mergedIssues)

      // Build parent-child relationships from the data we already have (no bdShow needed)
      const issueMap = new Map(newIssues.map(i => [i.id, i]))

      // 1. Fill in parent details: Rust sets parent.id from bd list, but title/status/priority are defaults
      //    We have all issues in the map, so we can look up the real values
      for (const issue of newIssues) {
        if (issue.parent?.id) {
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
        if (epic && !epic.children?.length) {
          epic.children = children
        }
      }

      // Only update if data actually changed (compare by serialization)
      const currentSignature = JSON.stringify(issues.value.map(i => i.id + i.updatedAt))
      const newSignature = JSON.stringify(newIssues.map(i => i.id + i.updatedAt))

      if (currentSignature !== newSignature) {
        issues.value = newIssues
      }

      // Sync selectedIssue with updated data (e.g., if modified externally via CLI)
      // Check even if list signature unchanged — selectedIssue may be stale from a
      // previous bdShow while the list was already updated by an earlier poll
      if (selectedIssue.value) {
        const updatedSelected = newIssues.find(i => i.id === selectedIssue.value!.id)
        if (updatedSelected && updatedSelected.updatedAt !== selectedIssue.value.updatedAt) {
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
      // Check if this is a schema migration error that needs repair
      if (checkRepairError(e)) {
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

      for (const issue of newIssues) {
        if (issue.parent?.id) {
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
        if (epic && !epic.children?.length) {
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
      const currentSignature = JSON.stringify(issues.value.map(i => i.id + i.updatedAt))
      const newSignature = JSON.stringify(newIssues.map(i => i.id + i.updatedAt))

      if (currentSignature !== newSignature) {
        issues.value = newIssues
      }

      // Sync selectedIssue with updated data (check even if list signature unchanged,
      // because selectedIssue may be stale from a previous bdShow while the list was
      // already updated by an earlier poll before the issue was selected)
      if (selectedIssue.value) {
        const updatedSelected = newIssues.find(i => i.id === selectedIssue.value!.id)
        if (updatedSelected && updatedSelected.updatedAt !== selectedIssue.value.updatedAt) {
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
      if (checkRepairError(e)) {
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
      result = result.filter((issue) => issue.status !== 'closed' && issue.status !== 'tombstone')
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
    if (filters.value.assignee.length > 0) {
      result = result.filter((issue) =>
        issue.assignee && filters.value.assignee.includes(issue.assignee)
      )
    }

    // Apply client-side label filter (OR logic - issue must have AT LEAST ONE selected label)
    if (filters.value.labels.length > 0) {
      result = result.filter((issue) =>
        filters.value.labels.some(filterLabel =>
          issue.labels?.some(l => l.toLowerCase() === filterLabel.toLowerCase())
        )
      )
    }

    // Apply exclusion filters (checked = hidden)
    if (exclusions.value.status.length > 0) {
      result = result.filter(issue => !exclusions.value.status.includes(issue.status))
    }
    if (exclusions.value.priority.length > 0) {
      result = result.filter(issue => !exclusions.value.priority.includes(issue.priority))
    }
    if (exclusions.value.type.length > 0) {
      result = result.filter(issue => !exclusions.value.type.includes(issue.type))
    }
    if (exclusions.value.labels.length > 0) {
      result = result.filter(issue =>
        !issue.labels?.some(l => exclusions.value.labels.includes(l.toLowerCase()))
      )
    }
    if (exclusions.value.assignee.length > 0) {
      result = result.filter(issue =>
        !exclusions.value.assignee.includes(issue.assignee || '')
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
      // Stable sort: use ID as tiebreaker to prevent DOM thrashing
      return naturalCompare(a.id.toLowerCase(), b.id.toLowerCase())
    })

    return sorted
  })

  // Computed for paginated issues
  const paginatedIssues = computed(() => {
    const end = currentPage.value * pageSize.value
    return sortedIssues.value.slice(0, end)
  })

  // Helper to get parent ID - uses issue.parent.id if available, falls back to ID pattern
  const getParentIdFromIssue = (issue: Issue): string | null => {
    // Priority 1: Use explicit parent field if available (from bdShow)
    if (issue.parent?.id) {
      return issue.parent.id
    }

    // Priority 2: Fallback to ID pattern (e.g., "beads-manager-40b.1" -> "beads-manager-40b")
    // This works for newly created children but not for re-parented issues
    const lastDotIndex = issue.id.lastIndexOf('.')
    if (lastDotIndex === -1) return null

    const suffix = issue.id.slice(lastDotIndex + 1)
    // Check if suffix is a number (child indicator)
    if (/^\d+$/.test(suffix)) {
      return issue.id.slice(0, lastDotIndex)
    }
    return null
  }

  // Compare child issues by ID suffix (ascending), falling back to createdAt
  const compareChildIssues = (a: Issue, b: Issue): number => {
    const getSuffix = (id: string): number | null => {
      const lastDotIndex = id.lastIndexOf('.')
      if (lastDotIndex === -1) return null
      const suffix = id.slice(lastDotIndex + 1)
      return /^\d+$/.test(suffix) ? parseInt(suffix, 10) : null
    }

    const suffixA = getSuffix(a.id)
    const suffixB = getSuffix(b.id)

    if (suffixA !== null && suffixB !== null) return suffixA - suffixB
    if (suffixA !== null) return -1
    if (suffixB !== null) return 1

    // Fall back to createdAt (oldest first)
    return new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime()
  }

  // Computed for hierarchical grouping of epics and their children
  const groupedIssues = computed((): IssueGroup[] => {
    const issueList = paginatedIssues.value
    const allIssues = issues.value
    const groups: IssueGroup[] = []
    const processedIds = new Set<string>()

    // Pass 1: Identify all epic IDs (from full list) and visible epic IDs (from paginated list)
    const allEpicIds = new Set<string>()
    for (const issue of allIssues) {
      if (issue.type === 'epic') allEpicIds.add(issue.id)
    }

    const visibleEpicIds = new Set<string>()
    for (const issue of issueList) {
      if (issue.type === 'epic') visibleEpicIds.add(issue.id)
    }

    // Pass 2: Build children maps (needs epic IDs from pass 1)
    const allEpicChildrenMap = new Map<string, Issue[]>()
    for (const issue of allIssues) {
      const parentId = getParentIdFromIssue(issue)
      if (parentId && allEpicIds.has(parentId)) {
        let children = allEpicChildrenMap.get(parentId)
        if (!children) { children = []; allEpicChildrenMap.set(parentId, children) }
        children.push(issue)
      }
    }

    const filteredEpicChildrenMap = new Map<string, Issue[]>()
    for (const issue of issueList) {
      const parentId = getParentIdFromIssue(issue)
      if (parentId && visibleEpicIds.has(parentId)) {
        let children = filteredEpicChildrenMap.get(parentId)
        if (!children) { children = []; filteredEpicChildrenMap.set(parentId, children) }
        children.push(issue)
      }
    }

    // Create groups - process EPICs first to ensure correct grouping
    // First pass: Add all EPICs with their children
    for (const issue of issueList) {
      if (issue.type === 'epic' && !processedIds.has(issue.id)) {
        const filteredChildren = (filteredEpicChildrenMap.get(issue.id) || []).sort(compareChildIssues)
        const allChildren = (allEpicChildrenMap.get(issue.id) || []).sort(compareChildIssues)
        const closedCount = allChildren.filter(c => c.status === 'closed').length
        const inProgressChild = allChildren.find(c => c.status === 'in_progress')

        groups.push({
          epic: issue,
          children: filteredChildren,
          childCount: allChildren.length,
          closedChildCount: closedCount,
          inProgressChild: inProgressChild ? { id: inProgressChild.id, title: inProgressChild.title, priority: inProgressChild.priority } : undefined,
        })
        processedIds.add(issue.id)
        filteredChildren.forEach(c => processedIds.add(c.id))
      }
    }

    // Second pass: Add orphan issues (not EPICs, not children of visible EPICs)
    for (const issue of issueList) {
      if (processedIds.has(issue.id)) continue

      const parentId = getParentIdFromIssue(issue)
      // Skip if this is a child of a visible EPIC (should have been processed above)
      if (parentId && visibleEpicIds.has(parentId)) continue

      // Add as orphan issue
      groups.push({
        epic: null,
        children: [issue],
        childCount: 0,
        closedChildCount: 0,
      })
      processedIds.add(issue.id)
    }

    return groups
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
    error.value = null

    try {
      const data = await bdShow(id, getPath())

      if (data === null) {
        selectedIssue.value = null
        error.value = `Issue ${id} not found`
        return null
      }

      selectedIssue.value = data

      // Also update the issue in the issues array to preserve parent/children info
      // (bd list doesn't return parent field, but bd show does)
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
  }
}
