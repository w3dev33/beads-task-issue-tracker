<script setup lang="ts">
import { ImageIcon } from 'lucide-vue-next'
import type { Issue } from '~/types/issue'
import { Badge } from '~/components/ui/badge'
import { Button } from '~/components/ui/button'
import { LinkifiedText } from '~/components/ui/linkified-text'
import LabelBadge from '~/components/issues/LabelBadge.vue'
import StatusBadge from '~/components/issues/StatusBadge.vue'
import PriorityBadge from '~/components/issues/PriorityBadge.vue'
import ImageThumbnail from '~/components/ui/image-preview/ImageThumbnail.vue'
import { extractImagesFromExternalRef, extractNonImageRefs, isUrl } from '~/utils/markdown'

const props = defineProps<{
  issue: Issue
  readonly?: boolean
}>()

const { beadsPath } = useBeadsPath()
const { openImage } = useImagePreview()

// Extract images from externalRef
const attachedImages = computed(() => extractImagesFromExternalRef(props.issue.externalRef))

// Extract non-image external references (URLs, IDs)
const nonImageRefs = computed(() => extractNonImageRefs(props.issue.externalRef))

const handleImageClick = async (src: string, alt: string) => {
  // For URLs, open in browser
  if (isUrl(src)) {
    const { open } = await import('@tauri-apps/plugin-shell')
    await open(src)
    return
  }
  // For local paths, open in preview modal
  const fullPath = src.startsWith('/') ? src : `${beadsPath.value}/.beads/${src}`
  openImage(fullPath, alt)
}

const attachImage = async () => {
  const { open } = await import('@tauri-apps/plugin-dialog')
  const selected = await open({
    multiple: false,
    filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp'] }],
  })
  if (selected) {
    emit('attach-image', selected)
  }
}

const emit = defineEmits<{
  'navigate-to-issue': [id: string]
  'attach-image': [path: string]
  'detach-image': [path: string]
}>()

// Collapsible section states (all open by default)
const isAttachmentsOpen = ref(true)
const isDescriptionOpen = ref(true)
const isParentOpen = ref(true)
const isChildrenOpen = ref(true)
const isDetailsOpen = ref(true)
const isDependenciesOpen = ref(true)
const isExternalRefOpen = ref(true)
const isEstimateOpen = ref(true)
const isDesignNotesOpen = ref(true)
const isAcceptanceCriteriaOpen = ref(true)
const isWorkingNotesOpen = ref(true)

const formatDate = (dateStr: string) => {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  return date.toLocaleDateString(undefined, {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

const formatEstimate = (minutes: number) => {
  if (minutes < 60) return `${minutes}m`
  const hours = Math.floor(minutes / 60)
  const mins = minutes % 60
  return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`
}
</script>

<template>
  <div class="space-y-3">
    <!-- Attachments Section (images from externalRef) -->
    <div>
      <div class="flex items-center justify-between">
        <button
          class="flex items-center gap-1.5 text-left group"
          @click="isAttachmentsOpen = !isAttachmentsOpen"
        >
          <svg
            class="w-3 h-3 text-muted-foreground transition-transform"
            :class="{ '-rotate-90': !isAttachmentsOpen }"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <polyline points="6 9 12 15 18 9" />
          </svg>
          <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide group-hover:text-foreground transition-colors">
            Attachments
            <span v-if="attachedImages.length > 0" class="text-muted-foreground">({{ attachedImages.length }})</span>
          </h4>
        </button>
        <Button
          v-if="!readonly"
          type="button"
          variant="outline"
          size="sm"
          class="h-5 px-1.5 text-[10px] hover:bg-sky-500/20 hover:border-sky-500 hover:text-sky-400 active:scale-95 active:bg-sky-500/30 transition-all"
          @click="attachImage"
        >
          <ImageIcon class="w-3 h-3 mr-1" />
          Attach
        </Button>
      </div>
      <div v-show="isAttachmentsOpen" class="mt-2 pl-4.5">
        <div v-if="attachedImages.length > 0" class="flex flex-wrap gap-4">
          <ImageThumbnail
            v-for="img in attachedImages"
            :key="img.src"
            :src="img.src"
            :alt="img.alt"
            :show-remove="!readonly"
            @click="handleImageClick(img.src, img.alt)"
            @remove="emit('detach-image', img.src)"
          />
        </div>
        <p v-else class="text-xs text-muted-foreground">No attachments</p>
      </div>
    </div>

    <!-- Description Section -->
    <div>
      <button
        class="flex items-center gap-1.5 w-full text-left group"
        @click="isDescriptionOpen = !isDescriptionOpen"
      >
        <svg
          class="w-3 h-3 text-muted-foreground transition-transform"
          :class="{ '-rotate-90': !isDescriptionOpen }"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
        <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide group-hover:text-foreground transition-colors">Description</h4>
      </button>
      <div v-show="isDescriptionOpen" class="mt-1 pl-4.5">
        <p class="text-xs whitespace-pre-wrap"><LinkifiedText :text="issue.description" fallback="No description provided." /></p>
      </div>
    </div>

    <!-- Parent Section (only if exists) -->
    <div v-if="issue.parent">
      <button
        class="flex items-center gap-1.5 w-full text-left group"
        @click="isParentOpen = !isParentOpen"
      >
        <svg
          class="w-3 h-3 text-muted-foreground transition-transform"
          :class="{ '-rotate-90': !isParentOpen }"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
        <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide group-hover:text-foreground transition-colors">Parent</h4>
      </button>
      <div v-show="isParentOpen" class="mt-1 pl-4.5">
        <div
          class="flex items-center justify-between gap-2 py-1 cursor-pointer hover:bg-muted/50 rounded px-1 -mx-1"
          @click="emit('navigate-to-issue', issue.parent.id)"
        >
          <div class="flex items-center gap-2 min-w-0">
            <span class="text-xs text-sky-400 hover:underline font-mono shrink-0">{{ issue.parent.id }}</span>
            <span class="text-xs truncate">{{ issue.parent.title }}</span>
          </div>
          <div class="flex items-center gap-1 shrink-0">
            <StatusBadge :status="issue.parent.status" size="sm" />
            <PriorityBadge :priority="issue.parent.priority" size="sm" />
          </div>
        </div>
      </div>
    </div>

    <!-- Children Section (only if exists) -->
    <div v-if="issue.children?.length">
      <button
        class="flex items-center gap-1.5 w-full text-left group"
        @click="isChildrenOpen = !isChildrenOpen"
      >
        <svg
          class="w-3 h-3 text-muted-foreground transition-transform"
          :class="{ '-rotate-90': !isChildrenOpen }"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
        <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide group-hover:text-foreground transition-colors">Children</h4>
        <span class="text-[10px] text-muted-foreground">({{ issue.children.length }})</span>
      </button>
      <div v-show="isChildrenOpen" class="mt-1 pl-4.5 space-y-0.5">
        <div
          v-for="child in issue.children"
          :key="child.id"
          class="flex items-center justify-between gap-2 py-1 cursor-pointer hover:bg-muted/50 rounded px-1 -mx-1"
          @click="emit('navigate-to-issue', child.id)"
        >
          <div class="flex items-center gap-2 min-w-0">
            <span class="text-xs text-sky-400 hover:underline font-mono shrink-0">{{ child.id }}</span>
            <span class="text-xs truncate">{{ child.title }}</span>
          </div>
          <div class="flex items-center gap-1 shrink-0">
            <StatusBadge :status="child.status" size="sm" />
            <PriorityBadge :priority="child.priority" size="sm" />
          </div>
        </div>
      </div>
    </div>

    <!-- External Reference Section (only if exists) -->
    <div v-if="nonImageRefs.length > 0">
      <button
        class="flex items-center gap-1.5 w-full text-left group"
        @click="isExternalRefOpen = !isExternalRefOpen"
      >
        <svg
          class="w-3 h-3 text-muted-foreground transition-transform"
          :class="{ '-rotate-90': !isExternalRefOpen }"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
        <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide group-hover:text-foreground transition-colors">
          External Reference
          <span class="text-muted-foreground">({{ nonImageRefs.length }})</span>
        </h4>
      </button>
      <div v-show="isExternalRefOpen" class="mt-1 pl-4.5 space-y-1">
        <p v-for="(ref, index) in nonImageRefs" :key="index" class="text-xs break-all">
          <LinkifiedText :text="ref" />
        </p>
      </div>
    </div>

    <!-- Details Section (Assignee, Labels, Dates) -->
    <div>
      <button
        class="flex items-center gap-1.5 w-full text-left group"
        @click="isDetailsOpen = !isDetailsOpen"
      >
        <svg
          class="w-3 h-3 text-muted-foreground transition-transform"
          :class="{ '-rotate-90': !isDetailsOpen }"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
        <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide group-hover:text-foreground transition-colors">Details</h4>
      </button>
      <div v-show="isDetailsOpen" class="mt-1 pl-4.5">
        <div class="grid grid-cols-2 gap-3">
          <div>
            <h5 class="text-[10px] font-medium text-sky-400 uppercase tracking-wide mb-0.5">Assignee</h5>
            <p class="text-xs">{{ issue.assignee || 'Unassigned' }}</p>
          </div>

          <div>
            <h5 class="text-[10px] font-medium text-sky-400 uppercase tracking-wide mb-0.5">Labels</h5>
            <div v-if="issue.labels?.length" class="flex flex-wrap gap-1">
              <LabelBadge v-for="label in issue.labels" :key="label" :label="label" size="sm" />
            </div>
            <p v-else class="text-xs text-muted-foreground">No labels</p>
          </div>

          <div>
            <h5 class="text-[10px] font-medium text-sky-400 uppercase tracking-wide mb-0.5">Created</h5>
            <p class="text-xs">{{ formatDate(issue.createdAt) }}</p>
          </div>

          <div>
            <h5 class="text-[10px] font-medium text-sky-400 uppercase tracking-wide mb-0.5">Updated</h5>
            <p class="text-xs">{{ formatDate(issue.updatedAt) }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- Dependencies Section (only if exists) -->
    <div v-if="issue.blockedBy?.length || issue.blocks?.length">
      <button
        class="flex items-center gap-1.5 w-full text-left group"
        @click="isDependenciesOpen = !isDependenciesOpen"
      >
        <svg
          class="w-3 h-3 text-muted-foreground transition-transform"
          :class="{ '-rotate-90': !isDependenciesOpen }"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
        <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide group-hover:text-foreground transition-colors">Dependencies</h4>
      </button>
      <div v-show="isDependenciesOpen" class="mt-1 pl-4.5 space-y-2">
        <div v-if="issue.blockedBy?.length">
          <h5 class="text-[10px] font-medium text-sky-400 uppercase tracking-wide mb-0.5">Blocked By</h5>
          <div class="flex flex-wrap gap-1">
            <Badge v-for="id in issue.blockedBy" :key="id" variant="secondary" class="text-[10px] px-1.5 py-0">
              {{ id }}
            </Badge>
          </div>
        </div>

        <div v-if="issue.blocks?.length">
          <h5 class="text-[10px] font-medium text-sky-400 uppercase tracking-wide mb-0.5">Blocks</h5>
          <div class="flex flex-wrap gap-1">
            <Badge v-for="id in issue.blocks" :key="id" variant="secondary" class="text-[10px] px-1.5 py-0">
              {{ id }}
            </Badge>
          </div>
        </div>
      </div>
    </div>

    <!-- Estimate Section (only if exists) -->
    <div v-if="issue.estimateMinutes">
      <button
        class="flex items-center gap-1.5 w-full text-left group"
        @click="isEstimateOpen = !isEstimateOpen"
      >
        <svg
          class="w-3 h-3 text-muted-foreground transition-transform"
          :class="{ '-rotate-90': !isEstimateOpen }"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
        <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide group-hover:text-foreground transition-colors">Estimate</h4>
      </button>
      <div v-show="isEstimateOpen" class="mt-1 pl-4.5">
        <p class="text-xs">{{ formatEstimate(issue.estimateMinutes) }}</p>
      </div>
    </div>

    <!-- Design Notes Section (only if exists) -->
    <div v-if="issue.designNotes">
      <button
        class="flex items-center gap-1.5 w-full text-left group"
        @click="isDesignNotesOpen = !isDesignNotesOpen"
      >
        <svg
          class="w-3 h-3 text-muted-foreground transition-transform"
          :class="{ '-rotate-90': !isDesignNotesOpen }"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
        <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide group-hover:text-foreground transition-colors">Design Notes</h4>
      </button>
      <div v-show="isDesignNotesOpen" class="mt-1 pl-4.5">
        <p class="text-xs whitespace-pre-wrap"><LinkifiedText :text="issue.designNotes" /></p>
      </div>
    </div>

    <!-- Acceptance Criteria Section (only if exists) -->
    <div v-if="issue.acceptanceCriteria">
      <button
        class="flex items-center gap-1.5 w-full text-left group"
        @click="isAcceptanceCriteriaOpen = !isAcceptanceCriteriaOpen"
      >
        <svg
          class="w-3 h-3 text-muted-foreground transition-transform"
          :class="{ '-rotate-90': !isAcceptanceCriteriaOpen }"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
        <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide group-hover:text-foreground transition-colors">Acceptance Criteria</h4>
      </button>
      <div v-show="isAcceptanceCriteriaOpen" class="mt-1 pl-4.5">
        <p class="text-xs whitespace-pre-wrap"><LinkifiedText :text="issue.acceptanceCriteria" /></p>
      </div>
    </div>

    <!-- Working Notes Section (only if exists) -->
    <div v-if="issue.workingNotes">
      <button
        class="flex items-center gap-1.5 w-full text-left group"
        @click="isWorkingNotesOpen = !isWorkingNotesOpen"
      >
        <svg
          class="w-3 h-3 text-muted-foreground transition-transform"
          :class="{ '-rotate-90': !isWorkingNotesOpen }"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
        <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide group-hover:text-foreground transition-colors">Working Notes</h4>
      </button>
      <div v-show="isWorkingNotesOpen" class="mt-1 pl-4.5">
        <p class="text-xs whitespace-pre-wrap"><LinkifiedText :text="issue.workingNotes" /></p>
      </div>
    </div>
  </div>
</template>
