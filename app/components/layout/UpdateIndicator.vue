<script setup lang="ts">
import { toast } from 'vue-sonner'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '~/components/ui/tooltip'

const { updateInfo, isChecking, check, openReleasesPage } = useUpdateChecker()

const currentVersion = computed(() => {
  return updateInfo.value?.currentVersion || useRuntimeConfig().public.appVersion
})

const tooltipText = computed(() => {
  if (isChecking.value) {
    return 'Checking for updates...'
  }
  if (updateInfo.value?.hasUpdate) {
    return `v${updateInfo.value.latestVersion} available - Click to download`
  }
  return 'Click to check for updates'
})

const handleClick = async () => {
  if (updateInfo.value?.hasUpdate) {
    openReleasesPage()
  } else {
    await check()

    // Show toast message based on result
    if (updateInfo.value) {
      if (updateInfo.value.hasUpdate) {
        toast.success(`v${updateInfo.value.latestVersion} available!`, {
          description: 'Click the version to download',
        })
      } else {
        toast.success('You\'re up to date!')
      }
    }
  }
}
</script>

<template>
  <Tooltip>
    <TooltipTrigger as-child>
      <button
        class="inline-flex items-center gap-1.5 text-muted-foreground/70 hover:text-muted-foreground transition-colors cursor-pointer"
        :class="{ 'opacity-50 pointer-events-none': isChecking }"
        @click="handleClick"
      >
        <span>v{{ currentVersion }}</span>
        <!-- Update available badge -->
        <span
          v-if="updateInfo?.hasUpdate"
          class="w-2 h-2 rounded-full bg-green-500 animate-pulse"
        />
        <!-- Checking spinner -->
        <svg
          v-if="isChecking"
          class="w-3 h-3 animate-spin"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" />
        </svg>
      </button>
    </TooltipTrigger>
    <TooltipContent>
      <span :class="{ 'text-green-500': updateInfo?.hasUpdate }">
        {{ tooltipText }}
      </span>
    </TooltipContent>
  </Tooltip>
</template>
