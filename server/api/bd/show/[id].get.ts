import { bdShow } from '../../../utils/bd-executor'
import { transformIssue } from '../../../utils/bd-transformers'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const query = getQuery(event)

  if (!id) {
    throw createError({
      statusCode: 400,
      message: 'Issue ID is required',
    })
  }

  const cwd = query.path ? String(query.path) : undefined
  const result = await bdShow(id, cwd)

  if (!result.success) {
    throw createError({
      statusCode: 404,
      message: result.error || 'Issue not found',
    })
  }

  // Transform bd CLI response to match Issue type interface
  // bd show --json returns an array with one element
  const rawIssue = Array.isArray(result.data) ? result.data[0] : result.data
  return rawIssue ? transformIssue(rawIssue as Parameters<typeof transformIssue>[0]) : null
})
