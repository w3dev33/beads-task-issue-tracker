<script setup lang="ts">
  import { TooltipProvider } from '~/components/ui/tooltip'

  const { isDark } = useTheme()

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
</script>

<template>
  <TooltipProvider>
    <NuxtPage />
  </TooltipProvider>
</template>
