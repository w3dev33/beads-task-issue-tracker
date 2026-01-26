<script setup lang="ts">
import type { Issue } from '~/types/issue'
import { Badge } from '~/components/ui/badge'
import { LinkifiedText } from '~/components/ui/linkified-text'
import LabelBadge from '~/components/issues/LabelBadge.vue'
import StatusBadge from '~/components/issues/StatusBadge.vue'
import PriorityBadge from '~/components/issues/PriorityBadge.vue'

defineProps<{
  issue: Issue
}>()

const emit = defineEmits<{
  'navigate-to-issue': [id: string]
}>()

// Collapsible section states (all open by default)
const isDescriptionOpen = ref(true)
const isParentOpen = ref(true)
const isChildrenOpen = ref(true)
const isDetailsOpen = ref(true)
const isDependenciesOpen = ref(true)
const isExtendedOpen = ref(true)
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

    <!-- Extended Info Section (only if exists) -->
    <div v-if="issue.externalRef || issue.estimateMinutes" class="pb-2">
      <button
        class="flex items-center gap-1.5 w-full text-left group"
        @click="isExtendedOpen = !isExtendedOpen"
      >
        <svg
          class="w-3 h-3 text-muted-foreground transition-transform"
          :class="{ '-rotate-90': !isExtendedOpen }"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
        <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide group-hover:text-foreground transition-colors">Extended Info</h4>
      </button>
      <div v-show="isExtendedOpen" class="mt-1 pl-4.5">
        <div class="grid grid-cols-2 gap-3">
          <div v-if="issue.externalRef">
            <h5 class="text-[10px] font-medium text-sky-400 uppercase tracking-wide mb-0.5">External Reference</h5>
            <p class="text-xs break-all"><LinkifiedText :text="issue.externalRef" /></p>
          </div>

          <div v-if="issue.estimateMinutes">
            <h5 class="text-[10px] font-medium text-sky-400 uppercase tracking-wide mb-0.5">Estimate</h5>
            <p class="text-xs">{{ formatEstimate(issue.estimateMinutes) }}</p>
          </div>
        </div>
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
