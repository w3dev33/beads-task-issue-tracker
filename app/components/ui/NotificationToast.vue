<script setup lang="ts">
import { XIcon, InfoIcon, CheckCircleIcon, AlertCircleIcon, AlertTriangleIcon } from 'lucide-vue-next'
import type { NotificationType } from '~/composables/useNotification'

const { notifications, dismiss } = useNotification()

const typeStyles: Record<NotificationType, { bg: string; border: string; icon: string }> = {
  info: {
    bg: 'bg-[#0a1929]',
    border: 'border-sky-500',
    icon: 'text-sky-400',
  },
  success: {
    bg: 'bg-[#071a12]',
    border: 'border-emerald-500',
    icon: 'text-emerald-400',
  },
  error: {
    bg: 'bg-[#1f0a0a]',
    border: 'border-red-500',
    icon: 'text-red-400',
  },
  warning: {
    bg: 'bg-[#1a1408]',
    border: 'border-amber-500',
    icon: 'text-amber-400',
  },
}

const getIcon = (type: NotificationType) => {
  switch (type) {
    case 'success': return CheckCircleIcon
    case 'error': return AlertCircleIcon
    case 'warning': return AlertTriangleIcon
    default: return InfoIcon
  }
}
</script>

<template>
  <Teleport to="body">
    <div class="fixed top-4 right-4 z-[9999] flex flex-col gap-2 pointer-events-none">
      <TransitionGroup name="notification">
        <div
          v-for="notification in notifications"
          :key="notification.id"
          class="pointer-events-auto rounded-lg shadow-xl p-3 min-w-[280px] max-w-[380px] flex items-start gap-3 border-l-4"
          :class="[typeStyles[notification.type].bg, typeStyles[notification.type].border]"
        >
          <component
            :is="getIcon(notification.type)"
            class="w-5 h-5 shrink-0 mt-0.5"
            :class="typeStyles[notification.type].icon"
          />
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium text-foreground">{{ notification.message }}</p>
            <p v-if="notification.description" class="text-xs text-muted-foreground mt-0.5 truncate">
              {{ notification.description }}
            </p>
          </div>
          <button
            class="shrink-0 text-muted-foreground hover:text-foreground transition-colors"
            @click="dismiss(notification.id)"
          >
            <XIcon class="w-4 h-4" />
          </button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<style scoped>
.notification-enter-active,
.notification-leave-active {
  transition: all 0.3s ease;
}

.notification-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.notification-leave-to {
  opacity: 0;
  transform: translateX(100%);
}
</style>
