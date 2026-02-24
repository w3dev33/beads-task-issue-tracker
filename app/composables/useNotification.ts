export type NotificationType = 'info' | 'success' | 'error' | 'warning'

interface Notification {
  id: number
  message: string
  description?: string
  type: NotificationType
}

const notifications = ref<Notification[]>([])
let nextId = 0

export function useNotification() {
  const addNotification = (message: string, description?: string, type: NotificationType = 'info', durationMs = 3000) => {
    const id = nextId++
    notifications.value.push({ id, message, description, type })

    setTimeout(() => {
      notifications.value = notifications.value.filter(n => n.id !== id)
    }, durationMs)
  }

  const notify = (message: string, description?: string, durationMs?: number) => addNotification(message, description, 'info', durationMs)
  const success = (message: string, description?: string, durationMs?: number) => addNotification(message, description, 'success', durationMs)
  const error = (message: string, description?: string, durationMs?: number) => addNotification(message, description, 'error', durationMs)
  const warning = (message: string, description?: string, durationMs?: number) => addNotification(message, description, 'warning', durationMs)

  const dismiss = (id: number) => {
    notifications.value = notifications.value.filter(n => n.id !== id)
  }

  return {
    notifications: readonly(notifications),
    notify,
    success,
    error,
    warning,
    dismiss,
  }
}
