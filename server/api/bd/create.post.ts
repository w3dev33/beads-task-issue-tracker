import { bdCreate } from '../../utils/bd-executor'
import { transformIssue, priorityToNumber } from '../../utils/bd-transformers'

export default defineEventHandler(async (event) => {
  const body = await readBody(event)
  const query = getQuery(event)

  if (!body.title) {
    throw createError({
      statusCode: 400,
      message: 'Title is required',
    })
  }

  const cwd = query.path ? String(query.path) : undefined

  // Convert priority from "p3" format to "3" for bd CLI
  const priority = body.priority ? priorityToNumber(body.priority) : undefined

  const result = await bdCreate(
    body.title,
    {
      description: body.description,
      type: body.type,
      priority,
      assignee: body.assignee,
      labels: body.labels,
      externalRef: body.externalRef,
      estimate: body.estimateMinutes,
      design: body.designNotes,
      acceptance: body.acceptanceCriteria,
      notes: body.workingNotes,
    },
    cwd
  )

  if (!result.success) {
    throw createError({
      statusCode: 500,
      message: result.error || 'Failed to create issue',
    })
  }

  // Transform bd CLI response to match Issue type interface
  return result.data ? transformIssue(result.data as Parameters<typeof transformIssue>[0]) : null
})
