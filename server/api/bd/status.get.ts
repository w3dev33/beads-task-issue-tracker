import { bdStatus } from '../../utils/bd-executor'

export default defineEventHandler(async (event) => {
  const query = getQuery(event)
  const cwd = query.path ? String(query.path) : undefined

  const result = await bdStatus(cwd)

  if (!result.success) {
    throw createError({
      statusCode: 500,
      message: result.error || 'Failed to get status',
    })
  }

  return result.data
})
