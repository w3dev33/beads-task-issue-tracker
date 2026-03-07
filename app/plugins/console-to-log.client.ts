import { logFrontend } from '~/utils/bd-api'

export default defineNuxtPlugin(() => {
  if (!import.meta.client) return

  const originalError = console.error
  const originalWarn = console.warn

  // Serialize arguments to a single string for the log file
  const serialize = (args: unknown[]): string => {
    return args
      .map((arg) => {
        if (arg instanceof Error) return `${arg.message}\n${arg.stack}`
        if (typeof arg === 'object') {
          try {
            return JSON.stringify(arg)
          } catch {
            return String(arg)
          }
        }
        return String(arg)
      })
      .join(' ')
  }

  console.error = (...args: unknown[]) => {
    originalError.apply(console, args)
    logFrontend('error', serialize(args)).catch(() => {
      // Avoid infinite loop - silently ignore
    })
  }

  console.warn = (...args: unknown[]) => {
    originalWarn.apply(console, args)
    logFrontend('warn', serialize(args)).catch(() => {
      // Avoid infinite loop - silently ignore
    })
  }

  // Capture unhandled errors
  window.addEventListener('error', (event) => {
    const msg = event.error
      ? `${event.error.message}\n${event.error.stack}`
      : `${event.message} (${event.filename}:${event.lineno}:${event.colno})`
    logFrontend('error', `[unhandled] ${msg}`).catch(() => {})
  })

  // Capture unhandled promise rejections
  window.addEventListener('unhandledrejection', (event) => {
    const reason = event.reason instanceof Error
      ? `${event.reason.message}\n${event.reason.stack}`
      : String(event.reason)
    logFrontend('error', `[unhandled-rejection] ${reason}`).catch(() => {})
  })
})
