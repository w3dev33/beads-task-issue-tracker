import { describe, expect, it } from 'vitest'
import { transformIssues, type BdRawIssue } from '../../server/utils/bd-transformers'

function makeRawIssue(overrides: Partial<BdRawIssue> = {}): BdRawIssue {
  return {
    id: 'child-open',
    title: 'Child issue',
    status: 'open',
    priority: 2,
    issue_type: 'task',
    created_at: '2025-01-01T00:00:00Z',
    updated_at: '2025-01-01T00:00:00Z',
    ...overrides,
  }
}

describe('bd issue transformation', () => {
  it('applies canonical blocked state from bd blocked output', () => {
    const [issue] = transformIssues(
      [makeRawIssue()],
      [makeRawIssue({ blocked_by: ['open-blocker'] })],
    )

    expect(issue?.isBlocked).toBe(true)
    expect(issue?.blockedBy).toEqual(['open-blocker'])
  })

  it('does not infer current blocked state from a historical relationship', () => {
    const [issue] = transformIssues(
      [makeRawIssue({ blocked_by: ['closed-blocker'] })],
      [],
    )

    expect(issue?.isBlocked).toBe(false)
    expect(issue?.blockedBy).toEqual(['closed-blocker'])
  })

  it('retains blockers from the current dependencies list shape', () => {
    const [issue] = transformIssues(
      [makeRawIssue({
        dependencies: [{
          issue_id: 'child-open',
          depends_on_id: 'open-blocker',
          type: 'blocks',
        }],
      })],
      [makeRawIssue({ blocked_by: ['open-blocker'] })],
    )

    expect(issue?.blockedBy).toEqual(['open-blocker'])
    expect(issue?.isBlocked).toBe(true)
  })
})
