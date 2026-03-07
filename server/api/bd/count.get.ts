import { bdList } from '../../utils/bd-executor'

interface Issue {
  id: string
  priority: number
  issue_type: string
  status: string
  updated_at?: string
}

export default defineEventHandler(async (event) => {
  const query = getQuery(event)
  const cwd = query.path ? String(query.path) : undefined

  const result = await bdList({}, cwd)

  if (!result.success) {
    throw createError({
      statusCode: 500,
      message: result.error || 'Failed to get counts',
    })
  }

  const issues = (result.data as Issue[]) || []

  // Calculate counts by type
  const byType: Record<string, number> = {
    bug: 0,
    task: 0,
    feature: 0,
    epic: 0,
    chore: 0,
  }

  // Calculate counts by priority (p0-p4)
  const byPriority: Record<string, number> = {
    p0: 0,
    p1: 0,
    p2: 0,
    p3: 0,
    p4: 0,
  }

  let lastUpdated: string | null = null

  for (const issue of issues) {
    // Count by type
    const type = issue.issue_type?.toLowerCase() || 'task'
    if (type in byType && byType[type] !== undefined) {
      byType[type] = (byType[type] ?? 0) + 1
    }

    // Count by priority (priority is a number 0-4)
    const priorityNum = typeof issue.priority === 'number' ? issue.priority : 3
    const priorityKey = `p${priorityNum}`
    if (priorityKey in byPriority && byPriority[priorityKey] !== undefined) {
      byPriority[priorityKey] = (byPriority[priorityKey] ?? 0) + 1
    }

    // Track most recent update
    if (issue.updated_at && (!lastUpdated || issue.updated_at > lastUpdated)) {
      lastUpdated = issue.updated_at
    }
  }

  return {
    count: issues.length,
    byType,
    byPriority,
    lastUpdated,
  }
})
