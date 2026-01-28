<script setup lang="ts">
  import { TooltipProvider } from '~/components/ui/tooltip'
  import { NotificationToast } from '~/components/ui/notification-toast'

  const { isDark } = useTheme()
  const { showUpdateDialog, showAboutDialog, initializeMenu } = useAppMenu()

  useHead({
    title: 'Beads Task-Issue Tracker',
    meta: [
      { name: 'description', content: 'Beads Task / Issue Tracking Manager' },
      { name: 'theme-color', content: () => isDark.value ? '#1e1e1e' : '#ffffff' },
    ],
    htmlAttrs: {
      lang: 'en',
      class: () => isDark.value ? 'dark' : '',
    },
  })

  onMounted(() => {
    initializeMenu()
  })
</script>

<template>
  <TooltipProvider>
    <NuxtPage />
    <LayoutUpdateDialog v-model:open="showUpdateDialog" />
    <LayoutAboutDialog v-model:open="showAboutDialog" />
    <NotificationToast />
  </TooltipProvider>
</template>
