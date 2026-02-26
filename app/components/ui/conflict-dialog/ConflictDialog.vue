<script setup lang="ts">
import { Button } from '~/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog'

const {
  isDialogOpen,
  isResolving,
  activeConflict,
  activeConflictIndex,
  conflictCount,
  parsedLocal,
  parsedRemote,
  diffFields,
  resolve,
  dismiss,
} = useConflicts()

/** Human-readable field label. */
function fieldLabel(field: string): string {
  const labels: Record<string, string> = {
    title: 'Title',
    description: 'Description',
    status: 'Status',
    priority: 'Priority',
    issue_type: 'Type',
    owner: 'Assignee',
    assignee: 'Assignee',
    closed_at: 'Closed at',
    notes: 'Notes',
    design: 'Design',
    acceptance_criteria: 'Acceptance Criteria',
    labels: 'Labels',
    parent: 'Parent',
    estimate_minutes: 'Estimate',
    external_ref: 'External Ref',
  }
  return labels[field] || field
}

/** Format a field value for display. */
function formatValue(value: unknown): string {
  if (value === null || value === undefined || value === '') return '(empty)'
  if (Array.isArray(value)) return value.length ? value.join(', ') : '(none)'
  return String(value)
}

function goNext() {
  if (activeConflictIndex.value < conflictCount.value - 1) {
    activeConflictIndex.value++
  }
}

function goPrev() {
  if (activeConflictIndex.value > 0) {
    activeConflictIndex.value--
  }
}
</script>

<template>
  <Dialog v-model:open="isDialogOpen">
    <DialogContent class="sm:max-w-2xl max-h-[80vh] flex flex-col">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <svg class="w-5 h-5 text-amber-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
            <line x1="12" y1="9" x2="12" y2="13" />
            <line x1="12" y1="17" x2="12.01" y2="17" />
          </svg>
          Sync Conflict
        </DialogTitle>
        <DialogDescription>
          <template v-if="activeConflict">
            Issue <span class="font-mono text-sky-400">{{ activeConflict.issue_id }}</span>
            was modified both locally and remotely.
            <span v-if="conflictCount > 1" class="text-muted-foreground">
              ({{ activeConflictIndex + 1 }} of {{ conflictCount }})
            </span>
          </template>
        </DialogDescription>
      </DialogHeader>

      <!-- Diff table -->
      <div v-if="parsedLocal && parsedRemote && diffFields.length" class="flex-1 overflow-y-auto -mx-6 px-6">
        <table class="w-full text-sm">
          <thead class="sticky top-0 bg-background z-10">
            <tr class="border-b">
              <th class="text-left py-2 pr-3 text-muted-foreground font-medium w-[120px]">Field</th>
              <th class="text-left py-2 px-3 text-blue-400 font-medium">Local</th>
              <th class="text-left py-2 pl-3 text-amber-400 font-medium">Remote</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="field in diffFields" :key="field" class="border-b border-border/50">
              <td class="py-2 pr-3 text-muted-foreground font-medium align-top">{{ fieldLabel(field) }}</td>
              <td class="py-2 px-3 align-top">
                <span class="text-blue-300 break-words whitespace-pre-wrap">{{ formatValue(parsedLocal[field]) }}</span>
              </td>
              <td class="py-2 pl-3 align-top">
                <span class="text-amber-300 break-words whitespace-pre-wrap">{{ formatValue(parsedRemote[field]) }}</span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- No diff (e.g., metadata-only difference) -->
      <div v-else-if="parsedLocal && parsedRemote && !diffFields.length" class="py-4 text-center text-muted-foreground text-sm">
        No visible field differences (may be metadata or timestamp only).
      </div>

      <!-- Navigation + actions -->
      <DialogFooter class="flex-col gap-3 sm:flex-row sm:justify-between">
        <!-- Navigation -->
        <div v-if="conflictCount > 1" class="flex items-center gap-2">
          <Button variant="ghost" size="sm" :disabled="activeConflictIndex === 0" @click="goPrev">
            Prev
          </Button>
          <Button variant="ghost" size="sm" :disabled="activeConflictIndex >= conflictCount - 1" @click="goNext">
            Next
          </Button>
        </div>
        <div v-else />

        <!-- Actions -->
        <div class="flex items-center gap-2">
          <Button variant="ghost" size="sm" :disabled="isResolving" @click="dismiss">
            Dismiss
          </Button>
          <Button variant="outline" :disabled="isResolving" @click="resolve('remote')">
            <svg v-if="isResolving" class="animate-spin -ml-1 mr-2 h-4 w-4" viewBox="0 0 24 24" fill="none">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
            </svg>
            Keep Remote
          </Button>
          <Button :disabled="isResolving" @click="resolve('local')">
            Keep Local
          </Button>
        </div>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
