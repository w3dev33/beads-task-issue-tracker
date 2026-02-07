<script setup lang="ts">
import { nextTick } from 'vue'
import { ImageIcon } from 'lucide-vue-next'
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
import { LabelMultiSelect } from '~/components/ui/label-multiselect'

interface EpicOption {
  id: string
  title: string
}

const props = defineProps<{
  issue?: Issue | null
  isNew?: boolean
  isSaving?: boolean
  availableEpics?: EpicOption[]
  availableLabels?: string[]
  defaultParent?: string
}>()

const { beadsPath } = useBeadsPath()
const { notify } = useNotification()

const emit = defineEmits<{
  save: [payload: UpdateIssuePayload]
  cancel: []
}>()

// Filter out "cleared:" placeholder from externalRef for display
const cleanExternalRef = (ref: string | undefined): string => {
  if (!ref) return ''
  return ref
    .split('\n')
    .filter(line => !line.trim().startsWith('cleared:'))
    .join('\n')
}

const form = reactive({
  title: props.issue?.title || '',
  description: props.issue?.description || '',
  type: props.issue?.type || 'task',
  status: props.issue?.status || 'open',
  priority: props.issue?.priority || 'p3',
  assignee: props.issue?.assignee || '',
  labels: props.issue?.labels || [] as string[],
  externalRef: cleanExternalRef(props.issue?.externalRef),
  estimateMinutes: props.issue?.estimateMinutes as number | undefined,
  designNotes: props.issue?.designNotes || '',
  acceptanceCriteria: props.issue?.acceptanceCriteria || '',
  workingNotes: props.issue?.workingNotes || '',
  parent: props.issue?.parent?.id || props.defaultParent || '',
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
      form.labels = newIssue.labels || []
      form.externalRef = cleanExternalRef(newIssue.externalRef)
      form.estimateMinutes = newIssue.estimateMinutes
      form.designNotes = newIssue.designNotes || ''
      form.acceptanceCriteria = newIssue.acceptanceCriteria || ''
      form.workingNotes = newIssue.workingNotes || ''
      form.parent = newIssue.parent?.id || ''
    }
  }
)

// When defaultParent changes (e.g., when creating child from epic), update form
watch(
  () => props.defaultParent,
  (newParent) => {
    if (props.isNew && newParent) {
      form.parent = newParent
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

// Special value for "no parent" since SelectItem doesn't allow empty string
const NO_PARENT_VALUE = '__none__'

// Filter out current issue from available epics (can't be parent of itself)
const filteredEpics = computed(() => {
  if (!props.availableEpics) return []
  if (!props.issue?.id) return props.availableEpics
  return props.availableEpics.filter(epic => epic.id !== props.issue?.id)
})

// Convert form.parent for display (empty string -> NO_PARENT_VALUE)
const parentSelectValue = computed({
  get: () => form.parent || NO_PARENT_VALUE,
  set: (val: string) => {
    form.parent = val === NO_PARENT_VALUE ? '' : val
  }
})

const handleSubmit = () => {
  // Determine parent value:
  // - Empty string "" means detach (pass empty to bd CLI)
  // - Non-empty string means attach to that epic
  const parentValue = form.parent.trim()

  const payload: UpdateIssuePayload = {
    title: form.title,
    description: form.description,
    type: form.type as IssueType,
    status: form.status as IssueStatus,
    priority: form.priority as IssuePriority,
    assignee: form.assignee,
    labels: form.labels,
    externalRef: form.externalRef,
    estimateMinutes: form.estimateMinutes || undefined,
    designNotes: form.designNotes,
    acceptanceCriteria: form.acceptanceCriteria,
    workingNotes: form.workingNotes,
    parent: parentValue, // Empty string to detach, epic ID to attach
  }
  emit('save', payload)
}

const attachFile = async () => {
  const { open } = await import('@tauri-apps/plugin-dialog')
  const { invoke } = await import('@tauri-apps/api/core')

  const selected = await open({
    multiple: false,
    filters: [
      { name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp'] },
      { name: 'Markdown', extensions: ['md', 'markdown'] },
    ],
  })

  if (selected) {
    // Check for duplicates: extract filename and compare with existing refs
    const selectedFilename = selected.split('/').pop() || selected
    const existingRefs = form.externalRef ? form.externalRef.split('\n').filter(Boolean) : []
    const isDuplicate = existingRefs.some((ref) => {
      const refFilename = ref.split('/').pop() || ref
      return refFilename === selectedFilename
    })

    if (isDuplicate) {
      notify('File already attached', selectedFilename)
      return
    }

    try {
      // Copy the file to .beads/attachments/{issue-id}/
      const issueId = props.issue?.id || `new-${Date.now()}`
      const copiedPath = await invoke<string>('copy_file_to_attachments', {
        projectPath: beadsPath.value,
        sourcePath: selected,
        issueId,
      })

      // Add the absolute path of the copy to externalRef
      if (form.externalRef) {
        form.externalRef += `\n${copiedPath}`
      } else {
        form.externalRef = copiedPath
      }
    } catch (error) {
      console.error('Failed to copy file:', error)
      // Fallback: use original path if copy fails
      if (form.externalRef) {
        form.externalRef += `\n${selected}`
      } else {
        form.externalRef = selected
      }
    }
  }
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

        <div v-if="filteredEpics.length > 0 && form.type !== 'epic'" class="flex items-center gap-1.5">
          <Label for="parent" class="text-[10px] uppercase tracking-wide text-sky-400 whitespace-nowrap">Parent</Label>
          <Select v-model="parentSelectValue">
            <SelectTrigger class="h-7 text-xs w-40">
              <SelectValue placeholder="No parent" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="__none__" class="text-xs text-muted-foreground">
                No parent
              </SelectItem>
              <SelectItem
                v-for="epic in filteredEpics"
                :key="epic.id"
                :value="epic.id"
                class="text-xs"
              >
                {{ epic.id }} - {{ epic.title.slice(0, 20) }}{{ epic.title.length > 20 ? '...' : '' }}
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
          <LabelMultiSelect
            v-model="form.labels"
            :available-labels="availableLabels || []"
          />
        </div>

        <div class="space-y-1">
          <div class="flex items-center gap-2">
            <Label for="externalRef" class="text-[10px] uppercase tracking-wide text-sky-400">External Reference</Label>
            <Button
              type="button"
              variant="outline"
              size="sm"
              class="h-5 px-1.5 text-[10px] hover:bg-sky-500/20 hover:border-sky-500 hover:text-sky-400 active:scale-95 active:bg-sky-500/30 transition-all"
              @click="attachFile"
            >
              <ImageIcon class="w-3 h-3 mr-1" />
              Attach
            </Button>
          </div>
          <Textarea
            id="externalRef"
            v-model="form.externalRef"
            placeholder="URLs, IDs, or image paths (one per line)"
            rows="2"
            class="text-xs"
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
            class="h-8 text-xs w-32"
          />
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
