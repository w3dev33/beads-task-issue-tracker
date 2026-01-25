<script setup lang="ts">
import { nextTick } from 'vue'
import type { Issue, IssueType, IssueStatus, IssuePriority, UpdateIssuePayload } from '~/types/issue'
import { Button } from '~/components/ui/button'
import { Input } from '~/components/ui/input'
import { Textarea } from '~/components/ui/textarea'
import { Label } from '~/components/ui/label'
import { ScrollArea } from '~/components/ui/scroll-area'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '~/components/ui/select'

const props = defineProps<{
  issue?: Issue | null
  isNew?: boolean
  isSaving?: boolean
}>()

const emit = defineEmits<{
  save: [payload: UpdateIssuePayload]
  cancel: []
}>()

const form = reactive({
  title: props.issue?.title || '',
  description: props.issue?.description || '',
  type: props.issue?.type || 'task',
  status: props.issue?.status || 'open',
  priority: props.issue?.priority || 'p3',
  assignee: props.issue?.assignee || '',
  labels: props.issue?.labels?.join(', ') || '',
  externalRef: props.issue?.externalRef || '',
  estimateMinutes: props.issue?.estimateMinutes as number | undefined,
  designNotes: props.issue?.designNotes || '',
  acceptanceCriteria: props.issue?.acceptanceCriteria || '',
  workingNotes: props.issue?.workingNotes || '',
})

watch(
  () => props.issue,
  (newIssue) => {
    if (newIssue) {
      form.title = newIssue.title
      form.description = newIssue.description || ''
      form.type = newIssue.type
      form.status = newIssue.status
      form.priority = newIssue.priority
      form.assignee = newIssue.assignee || ''
      form.labels = newIssue.labels?.join(', ') || ''
      form.externalRef = newIssue.externalRef || ''
      form.estimateMinutes = newIssue.estimateMinutes
      form.designNotes = newIssue.designNotes || ''
      form.acceptanceCriteria = newIssue.acceptanceCriteria || ''
      form.workingNotes = newIssue.workingNotes || ''
    }
  }
)

watch(
  () => props.isNew,
  (isNew) => {
    if (isNew) {
      nextTick(() => {
        const titleInput = document.getElementById('title') as HTMLInputElement | null
        titleInput?.focus()
      })
    }
  },
  { immediate: true }
)

const typeOptions: { value: IssueType; label: string }[] = [
  { value: 'bug', label: 'Bug' },
  { value: 'task', label: 'Task' },
  { value: 'feature', label: 'Feature' },
  { value: 'epic', label: 'Epic' },
  { value: 'chore', label: 'Chore' },
]

const statusOptions: { value: IssueStatus; label: string }[] = [
  { value: 'open', label: 'Open' },
  { value: 'in_progress', label: 'In Progress' },
  { value: 'blocked', label: 'Blocked' },
  { value: 'closed', label: 'Closed' },
]

const priorityOptions: { value: IssuePriority; label: string }[] = [
  { value: 'p0', label: 'P0 - Critical' },
  { value: 'p1', label: 'P1 - High' },
  { value: 'p2', label: 'P2 - Medium' },
  { value: 'p3', label: 'P3 - Low' },
  { value: 'p4', label: 'P4 - Minimal' },
]

const handleSubmit = () => {
  const payload: UpdateIssuePayload = {
    title: form.title,
    description: form.description || undefined,
    type: form.type as IssueType,
    status: form.status as IssueStatus,
    priority: form.priority as IssuePriority,
    assignee: form.assignee || undefined,
    labels: form.labels ? form.labels.split(',').map((l) => l.trim()).filter(Boolean) : undefined,
    externalRef: form.externalRef || undefined,
    estimateMinutes: form.estimateMinutes || undefined,
    designNotes: form.designNotes || undefined,
    acceptanceCriteria: form.acceptanceCriteria || undefined,
    workingNotes: form.workingNotes || undefined,
  }
  emit('save', payload)
}
</script>

<template>
  <form class="h-full flex flex-col" @submit.prevent="handleSubmit">
    <!-- Header fixe: Type, Status, Priority + Title -->
    <div class="space-y-3 pb-3 border-b border-border">
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-1.5">
          <Label for="type" class="text-[10px] uppercase tracking-wide text-sky-400 whitespace-nowrap">Type</Label>
          <Select v-model="form.type">
            <SelectTrigger class="h-7 text-xs w-28">
              <SelectValue placeholder="Type" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="opt in typeOptions"
                :key="opt.value"
                :value="opt.value"
                class="text-xs"
              >
                {{ opt.label }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div class="flex items-center gap-1.5">
          <Label for="status" class="text-[10px] uppercase tracking-wide text-sky-400 whitespace-nowrap">Status</Label>
          <Select v-model="form.status" :disabled="isNew">
            <SelectTrigger class="h-7 text-xs w-32">
              <SelectValue placeholder="Status" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="opt in statusOptions"
                :key="opt.value"
                :value="opt.value"
                class="text-xs"
              >
                {{ opt.label }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div class="flex items-center gap-1.5">
          <Label for="priority" class="text-[10px] uppercase tracking-wide text-sky-400 whitespace-nowrap">Priority</Label>
          <Select v-model="form.priority">
            <SelectTrigger class="h-7 text-xs w-36">
              <SelectValue placeholder="Priority" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="opt in priorityOptions"
                :key="opt.value"
                :value="opt.value"
                class="text-xs"
              >
                {{ opt.label }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      <div class="space-y-1">
        <Label for="title" class="text-[10px] uppercase tracking-wide text-sky-400">Title</Label>
        <Input
          id="title"
          v-model="form.title"
          placeholder="Issue title"
          required
          class="h-8 text-xs"
        />
      </div>
    </div>

    <!-- Body scrollable: tous les autres champs -->
    <ScrollArea class="flex-1 min-h-0">
      <div class="space-y-3 py-3">
        <div class="space-y-1">
          <Label for="description" class="text-[10px] uppercase tracking-wide text-sky-400">Description</Label>
          <Textarea
            id="description"
            v-model="form.description"
            placeholder="Describe the issue..."
            rows="3"
            class="text-xs"
          />
        </div>

        <div class="space-y-1">
          <Label for="assignee" class="text-[10px] uppercase tracking-wide text-sky-400">Assignee</Label>
          <Input
            id="assignee"
            v-model="form.assignee"
            placeholder="Username"
            class="h-8 text-xs"
          />
        </div>

        <div class="space-y-1">
          <Label for="labels" class="text-[10px] uppercase tracking-wide text-sky-400">Labels</Label>
          <Input
            id="labels"
            v-model="form.labels"
            placeholder="Comma-separated labels"
            class="h-8 text-xs"
          />
        </div>

        <div class="grid grid-cols-2 gap-2">
          <div class="space-y-1">
            <Label for="externalRef" class="text-[10px] uppercase tracking-wide text-sky-400">External Reference</Label>
            <Input
              id="externalRef"
              v-model="form.externalRef"
              placeholder="URL or ID"
              class="h-8 text-xs"
            />
          </div>

          <div class="space-y-1">
            <Label for="estimateMinutes" class="text-[10px] uppercase tracking-wide text-sky-400">Estimate (minutes)</Label>
            <Input
              id="estimateMinutes"
              v-model.number="form.estimateMinutes"
              type="number"
              min="0"
              placeholder="30"
              class="h-8 text-xs"
            />
          </div>
        </div>

        <div class="space-y-1">
          <Label for="designNotes" class="text-[10px] uppercase tracking-wide text-sky-400">Design Notes</Label>
          <Textarea
            id="designNotes"
            v-model="form.designNotes"
            placeholder="Design decisions and architectural notes..."
            rows="3"
            class="text-xs"
          />
        </div>

        <div class="space-y-1">
          <Label for="acceptanceCriteria" class="text-[10px] uppercase tracking-wide text-sky-400">Acceptance Criteria</Label>
          <Textarea
            id="acceptanceCriteria"
            v-model="form.acceptanceCriteria"
            placeholder="What must be true for this to be done..."
            rows="3"
            class="text-xs"
          />
        </div>

        <div class="space-y-1">
          <Label for="workingNotes" class="text-[10px] uppercase tracking-wide text-sky-400">Working Notes</Label>
          <Textarea
            id="workingNotes"
            v-model="form.workingNotes"
            placeholder="Progress notes and observations..."
            rows="3"
            class="text-xs"
          />
        </div>
      </div>
    </ScrollArea>

    <!-- Footer fixe: boutons -->
    <div class="flex justify-end gap-2 pt-3 border-t border-border">
      <Button type="button" variant="outline" size="sm" class="h-7 text-xs" :disabled="isSaving" @click="$emit('cancel')">
        Cancel
      </Button>
      <Button type="submit" size="sm" class="h-7 text-xs" :disabled="isSaving">
        <svg v-if="isSaving" class="animate-spin -ml-1 mr-1.5 h-3 w-3" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        {{ isSaving ? 'Saving...' : (isNew ? 'Create' : 'Save') }}
      </Button>
    </div>
  </form>
</template>
