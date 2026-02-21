/**
 * Pure helper functions for issue data manipulation.
 * Extracted from useIssues composable for testability.
 */
import type { Issue, DashboardStats, IssueType, IssuePriority } from '~/types/issue'
import type { IssueGroup } from '~/composables/useIssues'

/**
 * Deduplicate issues by ID, keeping the most recently updated version.
 */
export function deduplicateIssues(issues: Issue[]): Issue[] {
  const issueMap = new Map<string, Issue>()

  for (const issue of issues) {
    const existing = issueMap.get(issue.id)

    if (!existing) {
      issueMap.set(issue.id, issue)
    } else {
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
 * Natural sort comparison for IDs (handles multi-digit numbers correctly).
 * e.g., "40b.2" < "40b.10" instead of "40b.10" < "40b.2"
 */
export function naturalCompare(a: string, b: string): number {
  const aParts = a.split(/(\d+)/)
  const bParts = b.split(/(\d+)/)

  for (let i = 0; i < Math.max(aParts.length, bParts.length); i++) {
    const aPart = aParts[i] || ''
    const bPart = bParts[i] || ''

    const aNum = parseInt(aPart, 10)
    const bNum = parseInt(bPart, 10)

    if (!isNaN(aNum) && !isNaN(bNum)) {
      if (aNum !== bNum) return aNum - bNum
    } else {
      if (aPart < bPart) return -1
      if (aPart > bPart) return 1
    }
  }
  return 0
}

/**
 * Get parent ID from an issue — uses explicit parent.id if available,
 * falls back to dot notation pattern (e.g., "abc.1" → "abc").
 */
export function getParentIdFromIssue(issue: Issue): string | null {
  if (issue.parent?.id) {
    return issue.parent.id
  }

  const lastDotIndex = issue.id.lastIndexOf('.')
  if (lastDotIndex === -1) return null

  const suffix = issue.id.slice(lastDotIndex + 1)
  if (/^\d+$/.test(suffix)) {
    return issue.id.slice(0, lastDotIndex)
  }
  return null
}

/**
 * Compare child issues by ID suffix (ascending), falling back to createdAt.
 */
export function compareChildIssues(a: Issue, b: Issue): number {
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

  return new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime()
}

/** Sort orders for status, priority, and type fields */
export const statusOrder: Record<string, number> = {
  in_progress: 0,
  open: 1,
  blocked: 2,
  closed: 3,
}

export const priorityOrder: Record<string, number> = {
  p0: 0,
  p1: 1,
  p2: 2,
  p3: 3,
  p4: 4,
}

export const typeOrder: Record<string, number> = {
  bug: 0,
  feature: 1,
  task: 2,
  epic: 3,
  chore: 4,
}

/**
 * Sort issues by a given field and direction.
 * Returns a new sorted array (does not mutate input).
 */
export function sortIssues(issues: Issue[], field: string | null, direction: 'asc' | 'desc'): Issue[] {
  if (!field) return issues

  const sorted = [...issues]
  const dir = direction === 'asc' ? 1 : -1

  sorted.sort((a, b) => {
    let aVal: string | number | null = null
    let bVal: string | number | null = null

    switch (field) {
      case 'id':
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
    return naturalCompare(a.id.toLowerCase(), b.id.toLowerCase())
  })

  return sorted
}

/**
 * Filter issues based on inclusion filters, exclusion filters, and search.
 */
export function filterIssues(
  issues: Issue[],
  filters: { status: string[]; type: string[]; priority: string[]; assignee: string[]; search: string; labels: string[] },
  exclusions: { status: string[]; priority: string[]; type: string[]; labels: string[]; assignee: string[] },
): Issue[] {
  let result = issues

  // Search bypasses all other filters
  const searchTerm = filters.search?.trim()
  if (searchTerm) {
    const search = searchTerm.toLowerCase()
    return result.filter(
      (issue) =>
        issue.title.toLowerCase().includes(search) ||
        issue.id.toLowerCase().includes(search) ||
        issue.description?.toLowerCase().includes(search),
    )
  }

  // Status filter (default: exclude closed + tombstone)
  if (filters.status.length > 0) {
    result = result.filter((issue) => filters.status.includes(issue.status))
  } else {
    result = result.filter((issue) => issue.status !== 'closed' && issue.status !== 'tombstone')
  }

  if (filters.type.length > 0) {
    result = result.filter((issue) => filters.type.includes(issue.type))
  }

  if (filters.priority.length > 0) {
    result = result.filter((issue) => filters.priority.includes(issue.priority))
  }

  if (filters.assignee.length > 0) {
    result = result.filter((issue) => issue.assignee && filters.assignee.includes(issue.assignee))
  }

  // Labels: OR logic (issue must have AT LEAST ONE selected label)
  if (filters.labels.length > 0) {
    result = result.filter((issue) =>
      filters.labels.some(filterLabel =>
        issue.labels?.some(l => l.toLowerCase() === filterLabel.toLowerCase()),
      ),
    )
  }

  // Exclusion filters
  if (exclusions.status.length > 0) {
    result = result.filter(issue => !exclusions.status.includes(issue.status))
  }
  if (exclusions.priority.length > 0) {
    result = result.filter(issue => !exclusions.priority.includes(issue.priority))
  }
  if (exclusions.type.length > 0) {
    result = result.filter(issue => !exclusions.type.includes(issue.type))
  }
  if (exclusions.labels.length > 0) {
    result = result.filter(issue =>
      !issue.labels?.some(l => exclusions.labels.includes(l.toLowerCase())),
    )
  }
  if (exclusions.assignee.length > 0) {
    result = result.filter(issue =>
      !exclusions.assignee.includes(issue.assignee || ''),
    )
  }

  return result
}

/**
 * Group issues into epic/children hierarchy.
 */
export function groupIssues(
  paginatedIssues: Issue[],
  allIssues: Issue[],
): IssueGroup[] {
  const groups: IssueGroup[] = []
  const processedIds = new Set<string>()

  // Pass 1: Identify epic IDs
  const allEpicIds = new Set<string>()
  for (const issue of allIssues) {
    if (issue.type === 'epic') allEpicIds.add(issue.id)
  }

  const visibleEpicIds = new Set<string>()
  for (const issue of paginatedIssues) {
    if (issue.type === 'epic') visibleEpicIds.add(issue.id)
  }

  // Pass 2: Build children maps
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
  for (const issue of paginatedIssues) {
    const parentId = getParentIdFromIssue(issue)
    if (parentId && visibleEpicIds.has(parentId)) {
      let children = filteredEpicChildrenMap.get(parentId)
      if (!children) { children = []; filteredEpicChildrenMap.set(parentId, children) }
      children.push(issue)
    }
  }

  // EPICs with their children
  for (const issue of paginatedIssues) {
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

  // Orphan issues
  for (const issue of paginatedIssues) {
    if (processedIds.has(issue.id)) continue

    const parentId = getParentIdFromIssue(issue)
    if (parentId && visibleEpicIds.has(parentId)) continue

    groups.push({
      epic: null,
      children: [issue],
      childCount: 0,
      closedChildCount: 0,
    })
    processedIds.add(issue.id)
  }

  return groups
}

/**
 * Compute dashboard stats from an issues array.
 * Excludes tombstone issues. Groups open/deferred/pinned/hooked as "open".
 */
export function computeStatsFromIssues(issues: Issue[]): DashboardStats {
  const stats: DashboardStats = {
    total: issues.length,
    open: 0,
    inProgress: 0,
    blocked: 0,
    closed: 0,
    ready: 0,
    byType: { bug: 0, task: 0, feature: 0, epic: 0, chore: 0 },
    byPriority: { p0: 0, p1: 0, p2: 0, p3: 0, p4: 0 },
  }

  const activeIssues = issues.filter((issue) => issue.status !== 'tombstone')
  stats.total = activeIssues.length

  for (const issue of activeIssues) {
    switch (issue.status) {
      case 'open':
      case 'deferred':
      case 'pinned':
      case 'hooked':
        stats.open++
        break
      case 'in_progress':
        stats.inProgress++
        break
      case 'blocked':
        stats.blocked++
        break
      case 'closed':
        stats.closed++
        break
    }

    if (issue.type in stats.byType) {
      stats.byType[issue.type]++
    }

    if (issue.priority in stats.byPriority) {
      stats.byPriority[issue.priority]++
    }
  }

  return stats
}
