<script setup lang="ts">
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

// Temporary message state
const showUpToDate = ref(false)
let messageTimeout: ReturnType<typeof setTimeout> | null = null

const handleClick = async () => {
  if (updateInfo.value?.hasUpdate) {
    openReleasesPage()
  } else {
    await check()

    // Show "up to date" message temporarily
    if (updateInfo.value && !updateInfo.value.hasUpdate) {
      showUpToDate.value = true
      if (messageTimeout) clearTimeout(messageTimeout)
      messageTimeout = setTimeout(() => {
        showUpToDate.value = false
      }, 2000)
    }
  }
}

onUnmounted(() => {
  if (messageTimeout) clearTimeout(messageTimeout)
})
</script>

<template>
  <Tooltip>
    <TooltipTrigger as-child>
      <button
        class="inline-flex items-center gap-1.5 text-muted-foreground/70 hover:text-muted-foreground transition-colors cursor-pointer whitespace-nowrap"
        :class="{ 'opacity-50 pointer-events-none': isChecking }"
        @click="handleClick"
      >
        <!-- Fade between version and "Up to date" message -->
        <span class="relative inline-flex justify-end" style="width: 115px;">
          <span
            class="transition-opacity duration-300"
            :class="showUpToDate ? 'opacity-0' : 'opacity-100'"
          >
            v{{ currentVersion }}
          </span>
          <span
            class="absolute right-0 text-green-500 transition-opacity duration-300"
            :class="showUpToDate ? 'opacity-100' : 'opacity-0'"
          >
            Already up to date
          </span>
        </span>

        <!-- Fixed width container for badge/spinner to prevent layout shift -->
        <span class="w-3 h-3 inline-flex items-center justify-center">
          <!-- Update available badge -->
          <span
            v-if="updateInfo?.hasUpdate && !showUpToDate && !isChecking"
            class="w-2 h-2 rounded-full bg-green-500 animate-pulse"
          />
          <!-- Checking spinner -->
          <svg
            v-else-if="isChecking"
            class="w-3 h-3 animate-spin"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" />
          </svg>
        </span>
      </button>
    </TooltipTrigger>
    <TooltipContent>
      <span :class="{ 'text-green-500': updateInfo?.hasUpdate }">
        {{ tooltipText }}
      </span>
    </TooltipContent>
  </Tooltip>
</template>
