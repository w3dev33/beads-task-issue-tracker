import { describe, it, expect } from 'vitest'
import type { Issue } from '~/types/issue'
import {
  deduplicateIssues,
  naturalCompare,
  getParentIdFromIssue,
  compareChildIssues,
  sortIssues,
  filterIssues,
  groupIssues,
  computeReadyIssues,
  statusOrder,
  priorityOrder,
  typeOrder,
} from '~/utils/issue-helpers'

// ---------------------------------------------------------------------------
// Test data factory
// ---------------------------------------------------------------------------
function makeIssue(overrides: Partial<Issue> = {}): Issue {
  return {
    id: 'test-1',
    title: 'Test issue',
    description: '',
    type: 'task',
    status: 'open',
    priority: 'p2',
    assignee: '',
    labels: [],
    createdAt: '2025-01-01T00:00:00Z',
    updatedAt: '2025-01-01T00:00:00Z',
    comments: [],
    ...overrides,
  } as Issue
}

// ---------------------------------------------------------------------------
// deduplicateIssues
// ---------------------------------------------------------------------------
describe('deduplicateIssues', () => {
  it('returns empty array for empty input', () => {
    expect(deduplicateIssues([])).toEqual([])
  })

  it('keeps unique issues as-is', () => {
    const issues = [makeIssue({ id: 'a' }), makeIssue({ id: 'b' })]
    expect(deduplicateIssues(issues)).toHaveLength(2)
  })

  it('keeps most recently updated when duplicates exist', () => {
    const old = makeIssue({ id: 'a', updatedAt: '2025-01-01T00:00:00Z', title: 'old' })
    const recent = makeIssue({ id: 'a', updatedAt: '2025-06-01T00:00:00Z', title: 'recent' })
    const result = deduplicateIssues([old, recent])
    expect(result).toHaveLength(1)
    expect(result[0]!.title).toBe('recent')
  })

  it('keeps first when dates are equal', () => {
    const first = makeIssue({ id: 'a', updatedAt: '2025-01-01T00:00:00Z', title: 'first' })
    const second = makeIssue({ id: 'a', updatedAt: '2025-01-01T00:00:00Z', title: 'second' })
    const result = deduplicateIssues([first, second])
    expect(result).toHaveLength(1)
    expect(result[0]!.title).toBe('first')
  })
})

// ---------------------------------------------------------------------------
// naturalCompare
// ---------------------------------------------------------------------------
describe('naturalCompare', () => {
  it('sorts simple strings alphabetically', () => {
    expect(naturalCompare('abc', 'def')).toBeLessThan(0)
    expect(naturalCompare('def', 'abc')).toBeGreaterThan(0)
    expect(naturalCompare('abc', 'abc')).toBe(0)
  })

  it('sorts numbers numerically, not lexicographically', () => {
    expect(naturalCompare('2', '10')).toBeLessThan(0)
    expect(naturalCompare('10', '2')).toBeGreaterThan(0)
  })

  it('handles mixed alpha-numeric strings', () => {
    const ids = ['item10', 'item2', 'item1', 'item20']
    ids.sort(naturalCompare)
    expect(ids).toEqual(['item1', 'item2', 'item10', 'item20'])
  })

  it('handles dot-notation IDs', () => {
    const ids = ['proj-40b.2', 'proj-40b.10', 'proj-40b.1']
    ids.sort(naturalCompare)
    expect(ids).toEqual(['proj-40b.1', 'proj-40b.2', 'proj-40b.10'])
  })

  it('handles empty strings', () => {
    expect(naturalCompare('', '')).toBe(0)
    expect(naturalCompare('', 'a')).toBeLessThan(0)
  })
})

// ---------------------------------------------------------------------------
// getParentIdFromIssue
// ---------------------------------------------------------------------------
describe('getParentIdFromIssue', () => {
  it('returns explicit parent ID when available', () => {
    const issue = makeIssue({ id: 'child-1', parent: { id: 'parent-1', title: 'P', status: 'open', priority: 'p2' } })
    expect(getParentIdFromIssue(issue)).toBe('parent-1')
  })

  it('derives parent from dot notation', () => {
    const issue = makeIssue({ id: 'proj-abc.3' })
    expect(getParentIdFromIssue(issue)).toBe('proj-abc')
  })

  it('returns null for top-level issue (no dot)', () => {
    const issue = makeIssue({ id: 'proj-abc' })
    expect(getParentIdFromIssue(issue)).toBeNull()
  })

  it('returns null when dot suffix is not numeric', () => {
    const issue = makeIssue({ id: 'proj-v1.beta' })
    expect(getParentIdFromIssue(issue)).toBeNull()
  })

  it('handles multi-level dots (uses last dot)', () => {
    const issue = makeIssue({ id: 'proj-abc.1.2' })
    expect(getParentIdFromIssue(issue)).toBe('proj-abc.1')
  })
})

// ---------------------------------------------------------------------------
// compareChildIssues
// ---------------------------------------------------------------------------
describe('compareChildIssues', () => {
  it('sorts by numeric suffix ascending', () => {
    const a = makeIssue({ id: 'epic.1' })
    const b = makeIssue({ id: 'epic.3' })
    const c = makeIssue({ id: 'epic.2' })
    const sorted = [a, b, c].sort(compareChildIssues)
    expect(sorted.map(i => i.id)).toEqual(['epic.1', 'epic.2', 'epic.3'])
  })

  it('puts suffixed IDs before non-suffixed', () => {
    const a = makeIssue({ id: 'epic.1' })
    const b = makeIssue({ id: 'no-suffix' })
    expect(compareChildIssues(a, b)).toBeLessThan(0)
  })

  it('falls back to createdAt for non-suffixed IDs', () => {
    const older = makeIssue({ id: 'a', createdAt: '2025-01-01T00:00:00Z' })
    const newer = makeIssue({ id: 'b', createdAt: '2025-06-01T00:00:00Z' })
    expect(compareChildIssues(older, newer)).toBeLessThan(0)
  })
})

// ---------------------------------------------------------------------------
// Sort orders
// ---------------------------------------------------------------------------
describe('sort orders', () => {
  it('status: in_progress < open < blocked < closed', () => {
    expect(statusOrder['in_progress']).toBeLessThan(statusOrder['open']!)
    expect(statusOrder['open']).toBeLessThan(statusOrder['blocked']!)
    expect(statusOrder['blocked']).toBeLessThan(statusOrder['closed']!)
  })

  it('priority: p0 < p1 < p2 < p3 < p4', () => {
    expect(priorityOrder['p0']).toBeLessThan(priorityOrder['p4']!)
  })

  it('type: bug < feature < task < epic < chore', () => {
    expect(typeOrder['bug']).toBeLessThan(typeOrder['chore']!)
  })
})

// ---------------------------------------------------------------------------
// sortIssues
// ---------------------------------------------------------------------------
describe('sortIssues', () => {
  const issues = [
    makeIssue({ id: 'z-3', status: 'open', priority: 'p2', updatedAt: '2025-03-01T00:00:00Z' }),
    makeIssue({ id: 'a-1', status: 'in_progress', priority: 'p0', updatedAt: '2025-01-01T00:00:00Z' }),
    makeIssue({ id: 'm-2', status: 'closed', priority: 'p4', updatedAt: '2025-02-01T00:00:00Z' }),
  ]

  it('returns input unchanged when field is null', () => {
    const result = sortIssues(issues, null, 'asc')
    expect(result).toBe(issues) // same reference
  })

  it('sorts by ID with natural sort', () => {
    const result = sortIssues(issues, 'id', 'asc')
    expect(result.map(i => i.id)).toEqual(['a-1', 'm-2', 'z-3'])
  })

  it('sorts by status using custom order', () => {
    const result = sortIssues(issues, 'status', 'asc')
    expect(result.map(i => i.status)).toEqual(['in_progress', 'open', 'closed'])
  })

  it('sorts by priority', () => {
    const result = sortIssues(issues, 'priority', 'asc')
    expect(result.map(i => i.priority)).toEqual(['p0', 'p2', 'p4'])
  })

  it('sorts by updatedAt', () => {
    const result = sortIssues(issues, 'updatedAt', 'desc')
    expect(result.map(i => i.id)).toEqual(['z-3', 'm-2', 'a-1'])
  })

  it('respects direction', () => {
    const asc = sortIssues(issues, 'id', 'asc')
    const desc = sortIssues(issues, 'id', 'desc')
    expect(asc.map(i => i.id)).toEqual(['a-1', 'm-2', 'z-3'])
    expect(desc.map(i => i.id)).toEqual(['z-3', 'm-2', 'a-1'])
  })

  it('does not mutate input array', () => {
    const copy = [...issues]
    sortIssues(issues, 'id', 'asc')
    expect(issues.map(i => i.id)).toEqual(copy.map(i => i.id))
  })

  it('sorts pinned issues before unpinned (asc)', () => {
    const a = makeIssue({ id: 'a' })
    const b = makeIssue({ id: 'b' })
    const c = makeIssue({ id: 'c' })
    const result = sortIssues([a, b, c], 'pinned', 'asc', ['b', 'c'])
    expect(result.map(i => i.id)).toEqual(['b', 'c', 'a'])
  })

  it('keeps pinned issues on top even in desc mode', () => {
    const a = makeIssue({ id: 'a', updatedAt: '2025-01-01T00:00:00Z' })
    const b = makeIssue({ id: 'b', updatedAt: '2025-01-02T00:00:00Z' })
    const c = makeIssue({ id: 'c', updatedAt: '2025-01-03T00:00:00Z' })
    const result = sortIssues([a, b, c], 'pinned', 'desc', ['b'])
    // Pinned (b) always on top; non-pinned sorted desc by updatedAt: c (Jan 3) then a (Jan 1)
    expect(result.map(i => i.id)).toEqual(['b', 'c', 'a'])
  })

  it('sorts pinned without pinnedIds as no-op (all equal)', () => {
    const a = makeIssue({ id: 'a' })
    const b = makeIssue({ id: 'b' })
    const result = sortIssues([a, b], 'pinned', 'asc')
    // All unpinned, falls back to natural ID sort
    expect(result.map(i => i.id)).toEqual(['a', 'b'])
  })

  it('pinned issues float to top regardless of sort field', () => {
    const a = makeIssue({ id: 'a', priority: 'p0' })
    const b = makeIssue({ id: 'b', priority: 'p3' })
    const c = makeIssue({ id: 'c', priority: 'p1' })
    // b is pinned but has lowest priority — should still be first
    const result = sortIssues([a, b, c], 'priority', 'asc', ['b'])
    expect(result[0]!.id).toBe('b')
    // Non-pinned sorted by priority asc: p0 (a) then p1 (c)
    expect(result[1]!.id).toBe('a')
    expect(result[2]!.id).toBe('c')
  })

  it('sorts issues without labels last when sorting by labels', () => {
    const withLabel = makeIssue({ id: 'b', labels: ['frontend'] })
    const noLabel = makeIssue({ id: 'a', labels: [] })
    const result = sortIssues([noLabel, withLabel], 'labels', 'asc')
    expect(result[0]!.id).toBe('b')
  })
})

// ---------------------------------------------------------------------------
// filterIssues
// ---------------------------------------------------------------------------
describe('filterIssues', () => {
  const noFilters = { status: [] as string[], type: [] as string[], priority: [] as string[], assignee: [] as string[], search: '', labels: [] as string[] }
  const noExclusions = { status: [] as string[], priority: [] as string[], type: [] as string[], labels: [] as string[], assignee: [] as string[] }

  const issues = [
    makeIssue({ id: '1', title: 'Login bug', status: 'open', type: 'bug', priority: 'p0', labels: ['frontend'], assignee: 'alice' }),
    makeIssue({ id: '2', title: 'Add tests', status: 'in_progress', type: 'task', priority: 'p2', labels: ['backend'], assignee: 'bob' }),
    makeIssue({ id: '3', title: 'Old feature', status: 'closed', type: 'feature', priority: 'p3' }),
    makeIssue({ id: '4', title: 'Tombstone', status: 'tombstone' as any, type: 'task', priority: 'p2' }),
  ]

  it('excludes closed and tombstone by default (no status filter)', () => {
    const result = filterIssues(issues, noFilters, noExclusions)
    expect(result.map(i => i.id)).toEqual(['1', '2'])
  })

  it('shows only selected statuses when status filter active', () => {
    const result = filterIssues(issues, { ...noFilters, status: ['closed'] }, noExclusions)
    expect(result.map(i => i.id)).toEqual(['3'])
  })

  it('filters by type', () => {
    const result = filterIssues(issues, { ...noFilters, type: ['bug'] }, noExclusions)
    expect(result.map(i => i.id)).toEqual(['1'])
  })

  it('filters by priority', () => {
    const result = filterIssues(issues, { ...noFilters, priority: ['p0'] }, noExclusions)
    expect(result.map(i => i.id)).toEqual(['1'])
  })

  it('filters by assignee', () => {
    const result = filterIssues(issues, { ...noFilters, assignee: ['bob'] }, noExclusions)
    expect(result.map(i => i.id)).toEqual(['2'])
  })

  it('filters by labels (OR logic)', () => {
    const result = filterIssues(issues, { ...noFilters, labels: ['backend'] }, noExclusions)
    expect(result.map(i => i.id)).toEqual(['2'])
  })

  it('label matching is case-insensitive', () => {
    const result = filterIssues(issues, { ...noFilters, labels: ['FRONTEND'] }, noExclusions)
    expect(result.map(i => i.id)).toEqual(['1'])
  })

  it('search bypasses all other filters (includes closed)', () => {
    const result = filterIssues(issues, { ...noFilters, search: 'old feature' }, noExclusions)
    expect(result.map(i => i.id)).toEqual(['3'])
  })

  it('search matches title, id, and description', () => {
    const withDesc = [makeIssue({ id: 'x', title: 'Nothing', description: 'hidden keyword' })]
    const result = filterIssues(withDesc, { ...noFilters, search: 'keyword' }, noExclusions)
    expect(result).toHaveLength(1)
  })

  it('applies exclusion filters', () => {
    const result = filterIssues(issues, noFilters, { ...noExclusions, priority: ['p0'] })
    expect(result.map(i => i.id)).toEqual(['2'])
  })

  it('excludes by label', () => {
    const result = filterIssues(issues, noFilters, { ...noExclusions, labels: ['frontend'] })
    expect(result.map(i => i.id)).toEqual(['2'])
  })

  it('excludes by assignee', () => {
    const result = filterIssues(issues, noFilters, { ...noExclusions, assignee: ['alice'] })
    expect(result.map(i => i.id)).toEqual(['2'])
  })

  it('returns all non-closed when no filters and no exclusions', () => {
    const result = filterIssues(issues, noFilters, noExclusions)
    expect(result).toHaveLength(2)
  })

  it('returns empty when no issues match', () => {
    const result = filterIssues(issues, { ...noFilters, search: 'nonexistent' }, noExclusions)
    expect(result).toEqual([])
  })
})

// ---------------------------------------------------------------------------
// groupIssues
// ---------------------------------------------------------------------------
describe('groupIssues', () => {
  it('returns empty array for empty input', () => {
    expect(groupIssues([], [])).toEqual([])
  })

  it('groups epic with its children', () => {
    const epic = makeIssue({ id: 'epic-1', type: 'epic' })
    const child1 = makeIssue({ id: 'epic-1.1', type: 'task', parent: { id: 'epic-1', title: 'E', status: 'open', priority: 'p2' } })
    const child2 = makeIssue({ id: 'epic-1.2', type: 'task', parent: { id: 'epic-1', title: 'E', status: 'open', priority: 'p2' } })
    const all = [epic, child1, child2]

    const result = groupIssues(all, all)
    expect(result).toHaveLength(1)
    expect(result[0]!.epic!.id).toBe('epic-1')
    expect(result[0]!.children).toHaveLength(2)
    expect(result[0]!.childCount).toBe(2)
  })

  it('puts non-epic, non-child issues as orphans', () => {
    const standalone = makeIssue({ id: 'standalone', type: 'task' })
    const result = groupIssues([standalone], [standalone])
    expect(result).toHaveLength(1)
    expect(result[0]!.epic).toBeNull()
    expect(result[0]!.children).toEqual([standalone])
  })

  it('counts closed children from allIssues', () => {
    const epic = makeIssue({ id: 'e', type: 'epic' })
    const openChild = makeIssue({ id: 'e.1', status: 'open', parent: { id: 'e', title: '', status: 'open', priority: 'p2' } })
    const closedChild = makeIssue({ id: 'e.2', status: 'closed', parent: { id: 'e', title: '', status: 'open', priority: 'p2' } })
    const all = [epic, openChild, closedChild]

    // Only epic and openChild are visible (paginated)
    const result = groupIssues([epic, openChild], all)
    expect(result[0]!.childCount).toBe(2)
    expect(result[0]!.closedChildCount).toBe(1)
    expect(result[0]!.children).toHaveLength(1) // only visible children
  })

  it('detects in-progress child', () => {
    const epic = makeIssue({ id: 'e', type: 'epic' })
    const child = makeIssue({ id: 'e.1', status: 'in_progress', priority: 'p1', parent: { id: 'e', title: '', status: 'open', priority: 'p2' } })
    const all = [epic, child]

    const result = groupIssues(all, all)
    expect(result[0]!.inProgressChild).toEqual({ id: 'e.1', title: 'Test issue', priority: 'p1' })
  })

  it('sorts children by numeric suffix', () => {
    const epic = makeIssue({ id: 'e', type: 'epic' })
    const c3 = makeIssue({ id: 'e.3', parent: { id: 'e', title: '', status: 'open', priority: 'p2' } })
    const c1 = makeIssue({ id: 'e.1', parent: { id: 'e', title: '', status: 'open', priority: 'p2' } })
    const c2 = makeIssue({ id: 'e.2', parent: { id: 'e', title: '', status: 'open', priority: 'p2' } })
    const all = [epic, c3, c1, c2]

    const result = groupIssues(all, all)
    expect(result[0]!.children.map(c => c.id)).toEqual(['e.1', 'e.2', 'e.3'])
  })
})

// ---------------------------------------------------------------------------
// computeReadyIssues
// ---------------------------------------------------------------------------
describe('computeReadyIssues', () => {
  it('returns open issues without blockers', () => {
    const issues = [
      makeIssue({ id: '1', status: 'open' }),
      makeIssue({ id: '2', status: 'open' }),
    ]
    expect(computeReadyIssues(issues).map(i => i.id)).toEqual(['1', '2'])
  })

  it('excludes issues with blockedBy', () => {
    const issues = [
      makeIssue({ id: '1', status: 'open' }),
      makeIssue({ id: '2', status: 'open', blockedBy: ['1'] }),
    ]
    expect(computeReadyIssues(issues).map(i => i.id)).toEqual(['1'])
  })

  it('excludes non-open statuses', () => {
    const issues = [
      makeIssue({ id: '1', status: 'open' }),
      makeIssue({ id: '2', status: 'in_progress' }),
      makeIssue({ id: '3', status: 'closed' }),
      makeIssue({ id: '4', status: 'blocked' }),
    ]
    expect(computeReadyIssues(issues).map(i => i.id)).toEqual(['1'])
  })

  it('treats empty blockedBy as not blocked', () => {
    const issues = [
      makeIssue({ id: '1', status: 'open', blockedBy: [] }),
    ]
    expect(computeReadyIssues(issues)).toHaveLength(1)
  })

  it('returns empty array for empty input', () => {
    expect(computeReadyIssues([])).toEqual([])
  })
})
