import type { Issue, IssueType, IssueStatus, IssuePriority } from '~/types/issue'

// Raw issue format from bd CLI
interface BdRawIssue {
  id: string
  title: string
  description?: string
  status: string
  priority: number
  issue_type: string
  owner?: string
  labels?: string[]
  created_at: string
  created_by?: string
  updated_at: string
  closed_at?: string
  blocked_by?: string[]
  blocks?: string[]
  comments?: Array<{
    id: string | number
    author: string
    text?: string
    content?: string
    created_at: string
  }>
  dependency_count?: number
  dependent_count?: number
  external_ref?: string
  estimate?: number
  design?: string
  acceptance_criteria?: string
  notes?: string
}

/**
 * Convert priority number (0-4) to string format ("p0"-"p4")
 */
export function priorityToString(priority: number): IssuePriority {
  const validPriorities = [0, 1, 2, 3, 4]
  const p = validPriorities.includes(priority) ? priority : 3
  return `p${p}` as IssuePriority
}

/**
 * Convert priority string ("p0"-"p4") to number (0-4)
 */
export function priorityToNumber(priority: string): string {
  const match = priority.match(/^p(\d)$/)
  return match && match[1] ? match[1] : '3'
}

/**
 * Validate and normalize issue type
 */
export function normalizeIssueType(type: string): IssueType {
  const validTypes: IssueType[] = ['bug', 'task', 'feature', 'epic', 'chore']
  return validTypes.includes(type as IssueType) ? (type as IssueType) : 'task'
}

/**
 * Validate and normalize issue status
 */
export function normalizeIssueStatus(status: string): IssueStatus {
  const validStatuses: IssueStatus[] = ['open', 'in_progress', 'blocked', 'closed']
  return validStatuses.includes(status as IssueStatus) ? (status as IssueStatus) : 'open'
}

/**
 * Transform raw bd CLI issue to Issue type interface
 */
export function transformIssue(raw: BdRawIssue): Issue {
  return {
    id: raw.id,
    title: raw.title,
    description: raw.description || '',
    type: normalizeIssueType(raw.issue_type),
    status: normalizeIssueStatus(raw.status),
    priority: priorityToString(raw.priority),
    assignee: raw.owner,
    labels: raw.labels || [],
    createdAt: raw.created_at,
    updatedAt: raw.updated_at,
    closedAt: raw.closed_at,
    comments: (raw.comments || []).map((c) => ({
      id: String(c.id),
      author: c.author,
      content: c.text || c.content || '',
      createdAt: c.created_at,
    })),
    blockedBy: raw.blocked_by,
    blocks: raw.blocks,
    externalRef: raw.external_ref,
    estimateMinutes: raw.estimate,
    designNotes: raw.design,
    acceptanceCriteria: raw.acceptance_criteria,
    workingNotes: raw.notes,
  }
}

/**
 * Transform Issue type back to bd CLI format for updates
 */
export function transformToRaw(issue: Partial<Issue>): Record<string, unknown> {
  const raw: Record<string, unknown> = {}

  if (issue.title !== undefined) raw.title = issue.title
  if (issue.description !== undefined) raw.description = issue.description
  if (issue.type !== undefined) raw.issue_type = issue.type
  if (issue.status !== undefined) raw.status = issue.status
  if (issue.priority !== undefined) raw.priority = parseInt(issue.priority.replace('p', ''), 10)
  if (issue.assignee !== undefined) raw.owner = issue.assignee
  if (issue.labels !== undefined) raw.labels = issue.labels
  if (issue.externalRef !== undefined) raw.external_ref = issue.externalRef
  if (issue.estimateMinutes !== undefined) raw.estimate = issue.estimateMinutes
  if (issue.designNotes !== undefined) raw.design = issue.designNotes
  if (issue.acceptanceCriteria !== undefined) raw.acceptance_criteria = issue.acceptanceCriteria
  if (issue.workingNotes !== undefined) raw.notes = issue.workingNotes

  return raw
}
