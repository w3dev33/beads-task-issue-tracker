import { describe, it, expect } from 'vitest'
import type { Issue } from '~/types/issue'
import { computeStatsFromIssues } from '~/utils/issue-helpers'

function makeIssue(overrides: Partial<Issue> = {}): Issue {
  return {
    id: 'test-1',
    title: 'Test',
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

describe('computeStatsFromIssues', () => {
  it('returns zeroed stats for empty array', () => {
    const stats = computeStatsFromIssues([])
    expect(stats.total).toBe(0)
    expect(stats.open).toBe(0)
    expect(stats.inProgress).toBe(0)
    expect(stats.blocked).toBe(0)
    expect(stats.closed).toBe(0)
    expect(stats.workflow).toBe(0)
    expect(stats.byType.bug).toBe(0)
    expect(stats.byPriority.p0).toBe(0)
  })

  it('counts total across all issues', () => {
    const issues = [
      makeIssue({ id: '1', status: 'open' }),
      makeIssue({ id: '2', status: 'closed' }),
      makeIssue({ id: '3', status: 'deferred' }),
    ]
    const stats = computeStatsFromIssues(issues)
    expect(stats.total).toBe(3)
  })

  it('groups open, deferred, pinned, hooked as "open"', () => {
    const issues = [
      makeIssue({ id: '1', status: 'open' }),
      makeIssue({ id: '2', status: 'deferred' }),
      makeIssue({ id: '3', status: 'pinned' }),
      makeIssue({ id: '4', status: 'hooked' }),
    ]
    const stats = computeStatsFromIssues(issues)
    expect(stats.open).toBe(4)
    expect(stats.inProgress).toBe(0)
  })

  it('counts in_progress separately', () => {
    const issues = [
      makeIssue({ id: '1', status: 'in_progress' }),
      makeIssue({ id: '2', status: 'in_progress' }),
    ]
    const stats = computeStatsFromIssues(issues)
    expect(stats.inProgress).toBe(2)
  })

  it('counts blocked separately', () => {
    const issues = [makeIssue({ status: 'blocked' })]
    const stats = computeStatsFromIssues(issues)
    expect(stats.blocked).toBe(1)
  })

  it('counts open issues with blockers as blocked', () => {
    const issues = [
      makeIssue({ id: '1', status: 'open', blockedBy: ['2'] }),
      makeIssue({ id: '2', status: 'open' }),
    ]
    const stats = computeStatsFromIssues(issues)
    expect(stats.blocked).toBe(1)
    expect(stats.open).toBe(1)
    expect(stats.workflow).toBe(1)
  })

  it('counts workflow as non-blocked, non-closed, non-deleted issues', () => {
    const issues = [
      makeIssue({ id: '1', status: 'open' }),
      makeIssue({ id: '2', status: 'in_progress' }),
      makeIssue({ id: '3', status: 'deferred' }),
      makeIssue({ id: '4', status: 'blocked' }),
      makeIssue({ id: '5', status: 'closed' }),
      makeIssue({ id: '6', status: 'deleted' as any }),
      makeIssue({ id: '7', status: 'tombstone' as any }),
      makeIssue({ id: '8', status: 'open', blockedBy: ['1'] }),
    ]
    const stats = computeStatsFromIssues(issues)
    expect(stats.workflow).toBe(3)
  })

  it('counts closed separately', () => {
    const issues = [makeIssue({ status: 'closed' })]
    const stats = computeStatsFromIssues(issues)
    expect(stats.closed).toBe(1)
  })

  it('counts by type', () => {
    const issues = [
      makeIssue({ id: '1', type: 'bug' }),
      makeIssue({ id: '2', type: 'bug' }),
      makeIssue({ id: '3', type: 'feature' }),
      makeIssue({ id: '4', type: 'epic' }),
      makeIssue({ id: '5', type: 'chore' }),
    ]
    const stats = computeStatsFromIssues(issues)
    expect(stats.byType.bug).toBe(2)
    expect(stats.byType.feature).toBe(1)
    expect(stats.byType.task).toBe(0)
    expect(stats.byType.epic).toBe(1)
    expect(stats.byType.chore).toBe(1)
  })

  it('counts by priority', () => {
    const issues = [
      makeIssue({ id: '1', priority: 'p0' }),
      makeIssue({ id: '2', priority: 'p0' }),
      makeIssue({ id: '3', priority: 'p4' }),
    ]
    const stats = computeStatsFromIssues(issues)
    expect(stats.byPriority.p0).toBe(2)
    expect(stats.byPriority.p1).toBe(0)
    expect(stats.byPriority.p4).toBe(1)
  })

  it('includes closed issues in type and priority counts', () => {
    const issues = [
      makeIssue({ id: '1', status: 'closed', type: 'bug', priority: 'p0' }),
      makeIssue({ id: '2', status: 'open', type: 'bug', priority: 'p0' }),
    ]
    const stats = computeStatsFromIssues(issues)
    expect(stats.byType.bug).toBe(2)
    expect(stats.byPriority.p0).toBe(2)
  })

  it('handles mixed realistic data', () => {
    const issues = [
      makeIssue({ id: '1', status: 'open', type: 'bug', priority: 'p0' }),
      makeIssue({ id: '2', status: 'in_progress', type: 'task', priority: 'p2' }),
      makeIssue({ id: '3', status: 'blocked', type: 'feature', priority: 'p1' }),
      makeIssue({ id: '4', status: 'closed', type: 'task', priority: 'p3' }),
      makeIssue({ id: '5', status: 'deferred', type: 'chore', priority: 'p4' }),
    ]
    const stats = computeStatsFromIssues(issues)
    expect(stats.total).toBe(5)
    expect(stats.open).toBe(2) // open + deferred
    expect(stats.inProgress).toBe(1)
    expect(stats.blocked).toBe(1)
    expect(stats.closed).toBe(1)
    expect(stats.workflow).toBe(3)
  })

  it('initializes ready to 0', () => {
    const stats = computeStatsFromIssues([makeIssue()])
    expect(stats.ready).toBe(0)
  })
})
