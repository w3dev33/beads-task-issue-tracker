<script setup lang="ts">
import type { IssueStatus, IssueType, IssuePriority } from '~/types/issue'
import { Check } from 'lucide-vue-next'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  DropdownMenuCheckboxItem,
} from '~/components/ui/dropdown-menu'
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '~/components/ui/collapsible'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '~/components/ui/tooltip'
import { Button } from '~/components/ui/button'

defineProps<{
  availableLabels: string[]
  availableAssignees: string[]
}>()

const { exclusions, toggleStatus, togglePriority, toggleType, toggleLabel, toggleAssignee, clearAll, activeCount } = useExclusionFilters()

// Collapsible section states - open by default only if they have exclusions
const typeOpen = ref(false)
const labelsOpen = ref(false)
const statusOpen = ref(false)
const priorityOpen = ref(false)
const assigneeOpen = ref(false)

// Watch exclusions and open sections that have active exclusions
watchEffect(() => {
  if (exclusions.value.type.length > 0) typeOpen.value = true
  if (exclusions.value.labels.length > 0) labelsOpen.value = true
  if (exclusions.value.status.length > 0) statusOpen.value = true
  if (exclusions.value.priority.length > 0) priorityOpen.value = true
  if (exclusions.value.assignee.length > 0) assigneeOpen.value = true
})

// Status options
const statusOptions: { value: IssueStatus; label: string }[] = [
  { value: 'open', label: 'Open' },
  { value: 'in_progress', label: 'In Progress' },
  { value: 'blocked', label: 'Blocked' },
  { value: 'closed', label: 'Closed' },
]

// Priority options
const priorityOptions: { value: IssuePriority; label: string }[] = [
  { value: 'p0', label: 'P0 - Critical' },
  { value: 'p1', label: 'P1 - High' },
  { value: 'p2', label: 'P2 - Medium' },
  { value: 'p3', label: 'P3 - Low' },
  { value: 'p4', label: 'P4 - Minimal' },
]

// Type options
const typeOptions: { value: IssueType; label: string }[] = [
  { value: 'bug', label: 'Bug' },
  { value: 'feature', label: 'Feature' },
  { value: 'task', label: 'Task' },
  { value: 'epic', label: 'Epic' },
  { value: 'chore', label: 'Chore' },
]

const isStatusExcluded = (status: IssueStatus) => exclusions.value.status.includes(status)
const isPriorityExcluded = (priority: IssuePriority) => exclusions.value.priority.includes(priority)
const isTypeExcluded = (type: IssueType) => exclusions.value.type.includes(type)
const isLabelExcluded = (label: string) => exclusions.value.labels.includes(label.toLowerCase())
const isAssigneeExcluded = (assignee: string) => exclusions.value.assignee.includes(assignee)
</script>

<template>
  <Tooltip>
    <DropdownMenu>
      <TooltipTrigger as-child>
        <DropdownMenuTrigger as-child>
          <Button variant="outline" size="icon" class="h-8 w-8 relative">
            <!-- Filter/funnel icon -->
            <svg
              class="w-3.5 h-3.5"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" />
            </svg>
            <!-- Badge showing active exclusion count -->
            <span
              v-if="activeCount > 0"
              class="absolute -top-1 -right-1 flex h-4 min-w-4 items-center justify-center rounded-full bg-primary text-[10px] font-medium text-primary-foreground px-1"
            >
              {{ activeCount }}
            </span>
            <span class="sr-only">Exclusion filters</span>
          </Button>
        </DropdownMenuTrigger>
      </TooltipTrigger>
      <TooltipContent>Hide issues by criteria</TooltipContent>
      <DropdownMenuContent align="end" class="w-52">
        <DropdownMenuLabel class="text-xs">Hide Issues</DropdownMenuLabel>
        <DropdownMenuSeparator />

        <!-- Type Section -->
        <Collapsible v-model:open="typeOpen" class="px-1">
          <CollapsibleTrigger class="flex items-center gap-2 w-full py-1.5 px-2 text-xs font-medium hover:bg-accent rounded-sm">
            <svg
              class="w-3 h-3 transition-transform"
              :class="{ '-rotate-90': !typeOpen }"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <polyline points="6 9 12 15 18 9" />
            </svg>
            Type
            <span v-if="exclusions.type.length > 0" class="ml-auto text-[10px] text-muted-foreground">
              ({{ exclusions.type.length }})
            </span>
          </CollapsibleTrigger>
          <CollapsibleContent class="pl-3">
            <DropdownMenuCheckboxItem
              v-for="option in typeOptions"
              :key="option.value"
              :model-value="isTypeExcluded(option.value)"
              class="text-xs"
              @update:model-value="toggleType(option.value)"
            >
              <template #indicator-icon>
                <Check class="size-4" style="color: #ff3333" />
              </template>
              <span :class="{ 'opacity-50': isTypeExcluded(option.value) }">{{ option.label }}</span>
            </DropdownMenuCheckboxItem>
          </CollapsibleContent>
        </Collapsible>

        <!-- Labels Section (dynamic) -->
        <Collapsible v-if="availableLabels.length > 0" v-model:open="labelsOpen" class="px-1">
          <CollapsibleTrigger class="flex items-center gap-2 w-full py-1.5 px-2 text-xs font-medium hover:bg-accent rounded-sm">
            <svg
              class="w-3 h-3 transition-transform"
              :class="{ '-rotate-90': !labelsOpen }"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <polyline points="6 9 12 15 18 9" />
            </svg>
            Labels
            <span v-if="exclusions.labels.length > 0" class="ml-auto text-[10px] text-muted-foreground">
              ({{ exclusions.labels.length }})
            </span>
          </CollapsibleTrigger>
          <CollapsibleContent class="pl-3 max-h-32 overflow-y-auto">
            <DropdownMenuCheckboxItem
              v-for="label in availableLabels"
              :key="label"
              :model-value="isLabelExcluded(label)"
              class="text-xs"
              @update:model-value="toggleLabel(label)"
            >
              <template #indicator-icon>
                <Check class="size-4" style="color: #ff3333" />
              </template>
              <span :class="{ 'opacity-50': isLabelExcluded(label) }">{{ label }}</span>
            </DropdownMenuCheckboxItem>
          </CollapsibleContent>
        </Collapsible>

        <!-- Status Section -->
        <Collapsible v-model:open="statusOpen" class="px-1">
          <CollapsibleTrigger class="flex items-center gap-2 w-full py-1.5 px-2 text-xs font-medium hover:bg-accent rounded-sm">
            <svg
              class="w-3 h-3 transition-transform"
              :class="{ '-rotate-90': !statusOpen }"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <polyline points="6 9 12 15 18 9" />
            </svg>
            Status
            <span v-if="exclusions.status.length > 0" class="ml-auto text-[10px] text-muted-foreground">
              ({{ exclusions.status.length }})
            </span>
          </CollapsibleTrigger>
          <CollapsibleContent class="pl-3">
            <DropdownMenuCheckboxItem
              v-for="option in statusOptions"
              :key="option.value"
              :model-value="isStatusExcluded(option.value)"
              class="text-xs"
              @update:model-value="toggleStatus(option.value)"
            >
              <template #indicator-icon>
                <Check class="size-4" style="color: #ff3333" />
              </template>
              <span :class="{ 'opacity-50': isStatusExcluded(option.value) }">{{ option.label }}</span>
            </DropdownMenuCheckboxItem>
          </CollapsibleContent>
        </Collapsible>

        <!-- Priority Section -->
        <Collapsible v-model:open="priorityOpen" class="px-1">
          <CollapsibleTrigger class="flex items-center gap-2 w-full py-1.5 px-2 text-xs font-medium hover:bg-accent rounded-sm">
            <svg
              class="w-3 h-3 transition-transform"
              :class="{ '-rotate-90': !priorityOpen }"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <polyline points="6 9 12 15 18 9" />
            </svg>
            Priority
            <span v-if="exclusions.priority.length > 0" class="ml-auto text-[10px] text-muted-foreground">
              ({{ exclusions.priority.length }})
            </span>
          </CollapsibleTrigger>
          <CollapsibleContent class="pl-3">
            <DropdownMenuCheckboxItem
              v-for="option in priorityOptions"
              :key="option.value"
              :model-value="isPriorityExcluded(option.value)"
              class="text-xs"
              @update:model-value="togglePriority(option.value)"
            >
              <template #indicator-icon>
                <Check class="size-4" style="color: #ff3333" />
              </template>
              <span :class="{ 'opacity-50': isPriorityExcluded(option.value) }">{{ option.label }}</span>
            </DropdownMenuCheckboxItem>
          </CollapsibleContent>
        </Collapsible>

        <!-- Assignee Section (dynamic) -->
        <Collapsible v-if="availableAssignees.length > 0" v-model:open="assigneeOpen" class="px-1">
          <CollapsibleTrigger class="flex items-center gap-2 w-full py-1.5 px-2 text-xs font-medium hover:bg-accent rounded-sm">
            <svg
              class="w-3 h-3 transition-transform"
              :class="{ '-rotate-90': !assigneeOpen }"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <polyline points="6 9 12 15 18 9" />
            </svg>
            Assignee
            <span v-if="exclusions.assignee.length > 0" class="ml-auto text-[10px] text-muted-foreground">
              ({{ exclusions.assignee.length }})
            </span>
          </CollapsibleTrigger>
          <CollapsibleContent class="pl-3 max-h-32 overflow-y-auto">
            <DropdownMenuCheckboxItem
              v-for="assignee in availableAssignees"
              :key="assignee"
              :model-value="isAssigneeExcluded(assignee)"
              class="text-xs"
              @update:model-value="toggleAssignee(assignee)"
            >
              <template #indicator-icon>
                <Check class="size-4" style="color: #ff3333" />
              </template>
              <span :class="{ 'opacity-50': isAssigneeExcluded(assignee) }">{{ assignee }}</span>
            </DropdownMenuCheckboxItem>
          </CollapsibleContent>
        </Collapsible>

        <!-- Clear All Button -->
        <template v-if="activeCount > 0">
          <DropdownMenuSeparator />
          <div class="p-1">
            <Button variant="ghost" size="sm" class="w-full text-xs h-7 justify-start" @click="clearAll">
              Clear all exclusions
            </Button>
          </div>
        </template>
      </DropdownMenuContent>
    </DropdownMenu>
  </Tooltip>
</template>
