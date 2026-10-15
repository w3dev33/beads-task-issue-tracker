import { bdBlocked, bdList, unwrapBrEnvelope } from '../../utils/bd-executor'
import { transformIssues, priorityToNumber } from '../../utils/bd-transformers'
import type { BdRawIssue } from '../../utils/bd-transformers'

export default defineEventHandler(async (event) => {
  const query = getQuery(event)

  // Convert priority from "p3" format to "3" format for bd CLI
  const priorityFilters = query.priority
    ? String(query.priority).split(',').map(priorityToNumber)
    : undefined

  const filters = {
    status: query.status ? String(query.status).split(',') : undefined,
    type: query.type ? String(query.type).split(',') : undefined,
    priority: priorityFilters,
    assignee: query.assignee ? String(query.assignee) : undefined,
    includeAll: query.all === 'true',
  }

  const cwd = query.path ? String(query.path) : undefined

  const result = await bdList(filters, cwd)

  if (!result.success) {
    throw createError({
      statusCode: 500,
      message: result.error || 'Failed to list issues',
    })
  }

  const blockedResult = await bdBlocked(cwd)
  if (!blockedResult.success) {
    throw createError({
      statusCode: 500,
      message: blockedResult.error || 'Failed to get blocked issues',
    })
  }

  const issues = transformIssues(
    unwrapBrEnvelope<BdRawIssue>(result.data),
    unwrapBrEnvelope<BdRawIssue>(blockedResult.data),
  )

  return issues
})
