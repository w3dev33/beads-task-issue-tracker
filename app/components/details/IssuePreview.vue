<script setup lang="ts">
import type { Issue } from '~/types/issue'
import { Badge } from '~/components/ui/badge'
import LabelBadge from '~/components/issues/LabelBadge.vue'

defineProps<{
  issue: Issue
}>()

const formatDate = (dateStr: string) => {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  return date.toLocaleDateString('fr-FR', {
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
    <div>
      <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide mb-1">Description</h4>
      <p class="text-xs whitespace-pre-wrap">
        {{ issue.description || 'No description provided.' }}
      </p>
    </div>

    <div class="grid grid-cols-2 gap-3 pb-3">
      <div>
        <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide mb-1">Assignee</h4>
        <p class="text-xs">{{ issue.assignee || 'Unassigned' }}</p>
      </div>

      <div>
        <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide mb-1">Labels</h4>
        <div v-if="issue.labels?.length" class="flex flex-wrap gap-1">
          <LabelBadge v-for="label in issue.labels" :key="label" :label="label" size="sm" />
        </div>
        <p v-else class="text-xs text-muted-foreground">No labels</p>
      </div>

      <div>
        <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide mb-1">Created</h4>
        <p class="text-xs">{{ formatDate(issue.createdAt) }}</p>
      </div>

      <div>
        <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide mb-1">Updated</h4>
        <p class="text-xs">{{ formatDate(issue.updatedAt) }}</p>
      </div>
    </div>

    <div v-if="issue.blockedBy?.length">
      <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide mb-1">Blocked By</h4>
      <div class="flex flex-wrap gap-1">
        <Badge v-for="id in issue.blockedBy" :key="id" variant="secondary" class="text-[10px] px-1.5 py-0">
          {{ id }}
        </Badge>
      </div>
    </div>

    <div v-if="issue.blocks?.length">
      <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide mb-1">Blocks</h4>
      <div class="flex flex-wrap gap-1">
        <Badge v-for="id in issue.blocks" :key="id" variant="secondary" class="text-[10px] px-1.5 py-0">
          {{ id }}
        </Badge>
      </div>
    </div>

    <div v-if="issue.externalRef || issue.estimateMinutes" class="grid grid-cols-2 gap-3">
      <div v-if="issue.externalRef">
        <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide mb-1">External Reference</h4>
        <p class="text-xs break-all">{{ issue.externalRef }}</p>
      </div>

      <div v-if="issue.estimateMinutes">
        <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide mb-1">Estimate</h4>
        <p class="text-xs">{{ formatEstimate(issue.estimateMinutes) }}</p>
      </div>
    </div>

    <div v-if="issue.designNotes">
      <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide mb-1">Design Notes</h4>
      <p class="text-xs whitespace-pre-wrap">{{ issue.designNotes }}</p>
    </div>

    <div v-if="issue.acceptanceCriteria">
      <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide mb-1">Acceptance Criteria</h4>
      <p class="text-xs whitespace-pre-wrap">{{ issue.acceptanceCriteria }}</p>
    </div>

    <div v-if="issue.workingNotes">
      <h4 class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide mb-1">Working Notes</h4>
      <p class="text-xs whitespace-pre-wrap">{{ issue.workingNotes }}</p>
    </div>
  </div>
</template>
