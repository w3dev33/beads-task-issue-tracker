import { bdReady } from '../../utils/bd-executor'
import { transformIssue } from '../../utils/bd-transformers'

export default defineEventHandler(async (event) => {
  const query = getQuery(event)
  const cwd = query.path ? String(query.path) : undefined

  const result = await bdReady(cwd)

  if (!result.success) {
    throw createError({
      statusCode: 500,
      message: result.error || 'Failed to get ready issues',
    })
  }

  // Transform bd CLI response to match Issue type interface
  const issues = Array.isArray(result.data)
    ? result.data.map(transformIssue)
    : []

  return issues
})
