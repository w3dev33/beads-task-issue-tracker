<script setup lang="ts">
import { GithubIcon } from 'lucide-vue-next'

const { updateInfo } = useUpdateChecker()
const { showUpdateDialog } = useAppMenu()

const currentVersion = computed(() => {
  return updateInfo.value?.currentVersion || useRuntimeConfig().public.appVersion
})

const handleClick = () => {
  showUpdateDialog.value = true
}
</script>

<template>
  <button
    class="inline-flex items-center gap-1.5 text-muted-foreground/70 hover:text-muted-foreground transition-colors cursor-pointer whitespace-nowrap"
    @click="handleClick"
  >
    <GithubIcon class="w-3.5 h-3.5" />
    <span>v{{ currentVersion }}</span>

    <!-- Update available badge -->
    <span
      v-if="updateInfo?.hasUpdate"
      class="w-2 h-2 rounded-full bg-green-500 animate-pulse"
    />
  </button>
</template>
