import { bdCommentsAdd } from '../../../utils/bd-executor'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody(event)
  const query = getQuery(event)

  if (!id) {
    throw createError({
      statusCode: 400,
      message: 'Issue ID is required',
    })
  }

  if (!body.content || !body.content.trim()) {
    throw createError({
      statusCode: 400,
      message: 'Comment content is required',
    })
  }

  const cwd = query.path ? String(query.path) : undefined

  const result = await bdCommentsAdd(id, body.content.trim(), cwd)

  if (!result.success) {
    throw createError({
      statusCode: 500,
      message: result.error || 'Failed to add comment',
    })
  }

  return { success: true }
})
