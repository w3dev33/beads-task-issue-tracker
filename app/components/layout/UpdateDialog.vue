<script setup lang="ts">
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog'
import { Button } from '~/components/ui/button'
import { renderMarkdown } from '~/utils/markdown'

const isDev = import.meta.dev

const open = defineModel<boolean>('open', { default: false })

const { updateInfo, isChecking, isDownloading, downloadError, error, demoMode, check, toggleDemo, openReleasesPage, downloadAndQuit } = useUpdateChecker()

const renderedChangelog = computed(() => {
  if (!updateInfo.value?.releaseNotes) return ''
  return renderMarkdown(updateInfo.value.releaseNotes)
})

// Trigger check when dialog opens
watch(open, async (isOpen) => {
  if (isOpen) {
    await check()
  }
})

const xattrCommand = 'xattr -cr /Applications/Beads\\ Task-Issue\\ Tracker.app'
const xattrCopied = ref(false)

const copyXattrCommand = async () => {
  try {
    await navigator.clipboard.writeText(xattrCommand)
    xattrCopied.value = true
    setTimeout(() => {
      xattrCopied.value = false
    }, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}

const handleDownloadAndQuit = () => {
  downloadAndQuit()
}

const handleViewOnGitHub = () => {
  openReleasesPage()
  open.value = false
}

</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="sm:max-w-xl">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          Check for Updates
          <button
            v-if="isDev"
            tabindex="-1"
            @click="toggleDemo"
            class="text-[10px] font-normal cursor-pointer transition-colors px-1.5 py-0.5 rounded border"
            :class="demoMode ? 'text-amber-500 border-amber-500/50 bg-amber-500/10' : 'text-muted-foreground/40 border-border hover:text-muted-foreground hover:border-muted-foreground/50'"
          >
            Demo
          </button>
        </DialogTitle>
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
            <!-- Downloading state -->
            <div v-if="isDownloading" class="flex items-center gap-3">
              <svg
                class="w-5 h-5 animate-spin text-muted-foreground"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" />
              </svg>
              <span>Downloading update...</span>
            </div>

            <template v-else>
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

              <!-- Changelog -->
              <div v-if="renderedChangelog" class="mt-4">
                <p class="text-xs font-medium text-muted-foreground mb-2">What's new</p>
                <div
                  class="max-h-48 overflow-y-auto rounded border border-border p-3 text-sm markdown-base compact"
                  v-html="renderedChangelog"
                />
              </div>

              <!-- Download error -->
              <div v-if="downloadError" class="mt-3 p-2 rounded bg-destructive/10 text-destructive text-xs">
                {{ downloadError }}
              </div>

              <!-- macOS xattr instructions -->
              <div v-if="updateInfo.platform === 'macos'" class="mt-4 space-y-2">
                <p class="text-xs text-muted-foreground">
                  After installing, macOS may block the app. Run this command once to fix it:
                </p>
                <button
                  class="flex items-center gap-2 w-full text-left px-3 py-2 rounded bg-muted/50 font-mono text-xs text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
                  :title="xattrCopied ? 'Copied!' : 'Click to copy'"
                  @click="copyXattrCommand"
                >
                  <span class="flex-1 truncate">{{ xattrCommand }}</span>
                  <svg
                    v-if="!xattrCopied"
                    class="w-3.5 h-3.5 flex-shrink-0"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                  >
                    <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                  </svg>
                  <svg
                    v-else
                    class="w-3.5 h-3.5 flex-shrink-0 text-green-500"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                  >
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                </button>
              </div>
            </template>
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
      <DialogFooter class="flex items-center justify-end">
        <div class="flex items-center gap-2">
          <template v-if="updateInfo?.hasUpdate && !isChecking">
            <Button
              @click="handleDownloadAndQuit"
              :disabled="isDownloading"
            >
              Download &amp; Quit
            </Button>
            <Button
              variant="outline"
              @click="handleViewOnGitHub"
              :disabled="isDownloading"
            >
              View on GitHub
            </Button>
          </template>
          <Button
            v-if="!isDownloading"
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
