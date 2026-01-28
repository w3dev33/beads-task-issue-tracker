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
  const addNotification = (message: string, description?: string, type: NotificationType = 'info') => {
    const id = nextId++
    notifications.value.push({ id, message, description, type })

    // Auto-remove after 3 seconds
    setTimeout(() => {
      notifications.value = notifications.value.filter(n => n.id !== id)
    }, 3000)
  }

  const notify = (message: string, description?: string) => addNotification(message, description, 'info')
  const success = (message: string, description?: string) => addNotification(message, description, 'success')
  const error = (message: string, description?: string) => addNotification(message, description, 'error')
  const warning = (message: string, description?: string) => addNotification(message, description, 'warning')

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
