export type IssueType = 'bug' | 'task' | 'feature' | 'epic' | 'chore'
export type IssueStatus = 'open' | 'in_progress' | 'blocked' | 'closed'
export type IssuePriority = 'p0' | 'p1' | 'p2' | 'p3' | 'p4'

export interface Comment {
  id: string
  author: string
  content: string
  createdAt: string
}

export interface Issue {
  id: string
  title: string
  description: string
  type: IssueType
  status: IssueStatus
  priority: IssuePriority
  assignee?: string
  labels: string[]
  createdAt: string
  updatedAt: string
  closedAt?: string
  comments: Comment[]
  blockedBy?: string[]
  blocks?: string[]
  externalRef?: string
  estimateMinutes?: number
  designNotes?: string
  acceptanceCriteria?: string
  workingNotes?: string
}

export interface FilterState {
  status: IssueStatus[]
  type: IssueType[]
  priority: IssuePriority[]
  assignee: string | null
  search: string
}

export interface ColumnConfig {
  id: string
  label: string
  visible: boolean
  sortable: boolean
}

export interface DashboardStats {
  total: number
  open: number
  inProgress: number
  blocked: number
  closed: number
  ready: number
  byType: Record<IssueType, number>
  byPriority: Record<IssuePriority, number>
}

export interface CreateIssuePayload {
  title: string
  description?: string
  type?: IssueType
  priority?: IssuePriority
  assignee?: string
  labels?: string[]
  externalRef?: string
  estimateMinutes?: number
  designNotes?: string
  acceptanceCriteria?: string
  workingNotes?: string
}

export interface UpdateIssuePayload {
  title?: string
  description?: string
  type?: IssueType
  status?: IssueStatus
  priority?: IssuePriority
  assignee?: string
  labels?: string[]
  externalRef?: string
  estimateMinutes?: number
  designNotes?: string
  acceptanceCriteria?: string
  workingNotes?: string
}

export interface CollapsibleState {
  dashboard: boolean
  issues: boolean
  details: boolean
}
