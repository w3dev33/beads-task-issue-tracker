import type { Issue, IssueType, IssueStatus, IssuePriority } from '~/types/issue'
import type { PollData } from '~/utils/bd-api'

/**
 * Probe response format — the shape returned by GET /metrics/:name
 *
 * Field mapping vs app types:
 *   probe.issue_type  → app.type
 *   probe.owner       → app.assignee
 *   probe.priority    → app.priority  (P0→p0, P2→p2, etc.)
 */
export interface ProbeIssue {
  id: string
  title: string
  priority?: string
  issue_type?: string
  owner?: string
}

export interface ProbeEpic {
  id: string
  title: string
  progress: string
}

export interface ProbeMetricsResponse {
  project: string
  path: string
  last_refresh: string
  issues: {
    open: ProbeIssue[]
    in_progress: ProbeIssue[]
    closed: ProbeIssue[]
  }
  epics: ProbeEpic[]
  counts: {
    open: number
    in_progress: number
    closed: number
    total: number
  }
}

export interface ProbeProject {
  name: string
  path: string
  expose?: boolean
}

/**
 * Find the matching probe project for a given beads path.
 * Handles both `/path/to/project` and `/path/to/project/.beads` forms.
 * Pure function — no side effects, fully testable.
 */
export function matchProbeProject(projects: ProbeProject[], beadsPath: string): ProbeProject | undefined {
  const normalized = beadsPath.endsWith('.beads') ? beadsPath : `${beadsPath}/.beads`
  return projects.find(p => p.path === normalized || p.path === beadsPath)
}

function normalizePriority(p?: string): IssuePriority {
  if (!p) return 'p2'
  // Probe sends uppercase "P0", "P2" etc. — app uses lowercase "p0", "p2"
  return p.toLowerCase() as IssuePriority
}

function normalizeType(t?: string): IssueType {
  if (!t) return 'task'
  return t.toLowerCase() as IssueType
}

function normalizeStatus(s: string): IssueStatus {
  return s.toLowerCase().replace(/ /g, '_') as IssueStatus
}

function normalizeIssue(p: ProbeIssue, status: IssueStatus): Issue {
  return {
    id: p.id,
    title: p.title,
    description: '',
    type: normalizeType(p.issue_type),
    status,
    priority: normalizePriority(p.priority),
    assignee: p.owner,
    labels: [],
    createdAt: '',
    updatedAt: '',
    comments: [],
  }
}

export function probeMetricsToPollData(metrics: ProbeMetricsResponse): PollData {
  const openIssues = [
    ...(metrics.issues.open || []).map(i => normalizeIssue(i, 'open')),
    ...(metrics.issues.in_progress || []).map(i => normalizeIssue(i, 'in_progress')),
  ]
  const closedIssues = (metrics.issues.closed || []).map(i => normalizeIssue(i, 'closed'))

  // Probe doesn't have a separate "ready" concept — return empty
  return { openIssues, closedIssues, readyIssues: [] }
}

export function probeMetricsToIssues(metrics: ProbeMetricsResponse): Issue[] {
  return [
    ...(metrics.issues.open || []).map(i => normalizeIssue(i, 'open')),
    ...(metrics.issues.in_progress || []).map(i => normalizeIssue(i, 'in_progress')),
    ...(metrics.issues.closed || []).map(i => normalizeIssue(i, 'closed')),
  ]
}
