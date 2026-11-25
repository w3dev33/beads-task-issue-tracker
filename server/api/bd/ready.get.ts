import { bdReady, unwrapBrEnvelope } from '../../utils/bd-executor'
import { transformIssue } from '../../utils/bd-transformers'
import type { BdRawIssue } from '../../utils/bd-transformers'

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

  const issues = unwrapBrEnvelope<BdRawIssue>(result.data).map(transformIssue)

  return issues
})
