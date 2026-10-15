import { bdBlocked, bdShow, unwrapBrEnvelope } from '../../../utils/bd-executor'
import { transformIssues } from '../../../utils/bd-transformers'

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

  const blockedResult = await bdBlocked(cwd)
  if (!blockedResult.success) {
    throw createError({
      statusCode: 500,
      message: blockedResult.error || 'Failed to get blocked issues',
    })
  }

  // Transform bd CLI response to match Issue type interface
  // bd show --json returns an array with one element
  const rawIssue = Array.isArray(result.data) ? result.data[0] : result.data
  if (!rawIssue) return null

  return transformIssues(
    [rawIssue as Parameters<typeof transformIssues>[0][number]],
    unwrapBrEnvelope(blockedResult.data),
  )[0] || null
})
