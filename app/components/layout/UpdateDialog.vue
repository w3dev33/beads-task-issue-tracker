<script setup lang="ts">
import { GithubIcon } from 'lucide-vue-next'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog'
import { Button } from '~/components/ui/button'
import { openUrl } from '~/utils/open-url'

const open = defineModel<boolean>('open', { default: false })

const { updateInfo, isChecking, error, check, openReleasesPage } = useUpdateChecker()

// Trigger check when dialog opens
watch(open, async (isOpen) => {
  if (isOpen) {
    await check()
  }
})

const handleDownload = () => {
  openReleasesPage()
  open.value = false
}

const openGitHubRepo = () => {
  openUrl('https://github.com/w3dev33/beads-task-issue-tracker')
}
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>Check for Updates</DialogTitle>
        <DialogDescription as="div">
          <!-- Loading state -->
          <div v-if="isChecking" class="flex items-center gap-3 py-4">
            <svg
              class="w-5 h-5 animate-spin text-muted-foreground"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" />
            </svg>
            <span>Checking for updates...</span>
          </div>

          <!-- Error state -->
          <div v-else-if="error" class="py-4">
            <div class="flex items-center gap-2 text-destructive">
              <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10" />
                <line x1="12" y1="8" x2="12" y2="12" />
                <line x1="12" y1="16" x2="12.01" y2="16" />
              </svg>
              <span>Failed to check for updates</span>
            </div>
            <p class="mt-2 text-sm text-muted-foreground">{{ error }}</p>
          </div>

          <!-- Update available -->
          <div v-else-if="updateInfo?.hasUpdate" class="py-4">
            <div class="flex items-center gap-2 text-green-500">
              <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="7 10 12 15 17 10" />
                <line x1="12" y1="15" x2="12" y2="3" />
              </svg>
              <span>Update available</span>
            </div>
            <div class="mt-3 space-y-1 text-sm">
              <p><span class="text-muted-foreground">Current version:</span> v{{ updateInfo.currentVersion }}</p>
              <p><span class="text-muted-foreground">Latest version:</span> <span class="text-green-500 font-medium">v{{ updateInfo.latestVersion }}</span></p>
            </div>
          </div>

          <!-- Up to date -->
          <div v-else-if="updateInfo" class="py-4">
            <div class="flex items-center gap-2 text-green-500">
              <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
                <polyline points="22 4 12 14.01 9 11.01" />
              </svg>
              <span>You're up to date</span>
            </div>
            <p class="mt-2 text-sm text-muted-foreground">
              Version v{{ updateInfo.currentVersion }} is the latest version.
            </p>
          </div>
        </DialogDescription>
      </DialogHeader>
      <DialogFooter class="flex items-center justify-between sm:justify-between">
        <button
          @click="openGitHubRepo"
          class="inline-flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
        >
          <GithubIcon class="w-3.5 h-3.5" />
          <span>GitHub</span>
        </button>
        <div class="flex items-center gap-2">
          <Button
            v-if="updateInfo?.hasUpdate && !isChecking"
            @click="handleDownload"
          >
            Download
          </Button>
          <Button
            variant="outline"
            @click="open = false"
          >
            {{ updateInfo?.hasUpdate ? 'Later' : 'Close' }}
          </Button>
        </div>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
