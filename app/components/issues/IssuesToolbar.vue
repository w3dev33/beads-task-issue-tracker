<script setup lang="ts">
import type { IssueStatus, IssueType, IssuePriority, ColumnConfig } from '~/types/issue'
import { Input } from '~/components/ui/input'
import { Button } from '~/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '~/components/ui/dropdown-menu'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '~/components/ui/tooltip'
import ColumnConfigPopover from '~/components/issues/ColumnConfig.vue'

const search = defineModel<string>('search', { default: '' })

const props = defineProps<{
  activeStatusFilters?: IssueStatus[]
  activeTypeFilters?: IssueType[]
  activePriorityFilters?: IssuePriority[]
  hasSelection?: boolean
  multiSelectMode?: boolean
  selectedCount?: number
  columns: ColumnConfig[]
}>()

defineEmits<{
  add: []
  delete: []
  toggleMultiSelect: []
  addStatusFilter: [status: IssueStatus]
  addTypeFilter: [type: IssueType]
  addPriorityFilter: [priority: IssuePriority]
  'update:columns': [columns: ColumnConfig[]]
  resetColumns: []
}>()

const isStatusActive = (status: IssueStatus) => props.activeStatusFilters?.includes(status)
const isTypeActive = (type: IssueType) => props.activeTypeFilters?.includes(type)
const isPriorityActive = (priority: IssuePriority) => props.activePriorityFilters?.includes(priority)

const statusOptions: { value: IssueStatus; label: string }[] = [
  { value: 'open', label: 'Open' },
  { value: 'in_progress', label: 'In Progress' },
  { value: 'blocked', label: 'Blocked' },
  { value: 'closed', label: 'Closed' },
]

const typeOptions: { value: IssueType; label: string }[] = [
  { value: 'bug', label: 'Bug' },
  { value: 'task', label: 'Task' },
  { value: 'feature', label: 'Feature' },
  { value: 'epic', label: 'Epic' },
  { value: 'chore', label: 'Chore' },
]

const priorityOptions: { value: IssuePriority; label: string }[] = [
  { value: 'p0', label: 'P0 - Critical' },
  { value: 'p1', label: 'P1 - High' },
  { value: 'p2', label: 'P2 - Medium' },
  { value: 'p3', label: 'P3 - Low' },
  { value: 'p4', label: 'P4 - Minimal' },
]
</script>

<template>
  <div class="space-y-2">
    <div class="flex items-center gap-2">
        <!-- Multi-select toggle -->
        <Tooltip>
          <TooltipTrigger as-child>
            <Button
              :variant="multiSelectMode ? 'default' : 'outline'"
              size="icon"
              class="h-8 w-8 shrink-0"
              @click="$emit('toggleMultiSelect')"
            >
              <svg
                class="w-3.5 h-3.5"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <rect x="3" y="5" width="6" height="6" rx="1" />
                <rect x="3" y="13" width="6" height="6" rx="1" />
                <line x1="12" y1="8" x2="21" y2="8" />
                <line x1="12" y1="16" x2="21" y2="16" />
              </svg>
              <span class="sr-only">Multi-select</span>
            </Button>
          </TooltipTrigger>
          <TooltipContent>Toggle multi-select</TooltipContent>
        </Tooltip>

      <div class="relative flex-1">
        <svg
          class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <circle cx="11" cy="11" r="8" />
          <line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <Input
          v-model="search"
          type="search"
          placeholder="Search issues..."
          class="pl-10 h-8 text-xs"
        />
      </div>

      <!-- Filter dropdown -->
    <DropdownMenu>
      <DropdownMenuTrigger as-child>
        <Button variant="outline" size="sm" class="h-8 text-xs gap-1">
          <svg
            class="w-3.5 h-3.5"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" />
          </svg>
          Filter
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" class="w-48">
        <DropdownMenuLabel class="text-xs">Status</DropdownMenuLabel>
        <DropdownMenuGroup>
          <DropdownMenuItem
            v-for="opt in statusOptions"
            :key="opt.value"
            class="text-xs"
            :class="isStatusActive(opt.value) ? 'opacity-40 cursor-default' : ''"
            :disabled="isStatusActive(opt.value)"
            @click="!isStatusActive(opt.value) && $emit('addStatusFilter', opt.value)"
          >
            {{ opt.label }}
            <svg v-if="isStatusActive(opt.value)" class="w-3 h-3 ml-auto text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </DropdownMenuItem>
        </DropdownMenuGroup>
        <DropdownMenuSeparator />
        <DropdownMenuLabel class="text-xs">Type</DropdownMenuLabel>
        <DropdownMenuGroup>
          <DropdownMenuItem
            v-for="opt in typeOptions"
            :key="opt.value"
            class="text-xs"
            :class="isTypeActive(opt.value) ? 'opacity-40 cursor-default' : ''"
            :disabled="isTypeActive(opt.value)"
            @click="!isTypeActive(opt.value) && $emit('addTypeFilter', opt.value)"
          >
            {{ opt.label }}
            <svg v-if="isTypeActive(opt.value)" class="w-3 h-3 ml-auto text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </DropdownMenuItem>
        </DropdownMenuGroup>
        <DropdownMenuSeparator />
        <DropdownMenuLabel class="text-xs">Priority</DropdownMenuLabel>
        <DropdownMenuGroup>
          <DropdownMenuItem
            v-for="opt in priorityOptions"
            :key="opt.value"
            class="text-xs"
            :class="isPriorityActive(opt.value) ? 'opacity-40 cursor-default' : ''"
            :disabled="isPriorityActive(opt.value)"
            @click="!isPriorityActive(opt.value) && $emit('addPriorityFilter', opt.value)"
          >
            {{ opt.label }}
            <svg v-if="isPriorityActive(opt.value)" class="w-3 h-3 ml-auto text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </DropdownMenuItem>
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>

    <Button
      v-if="hasSelection"
      variant="outline"
      size="icon"
      class="h-8 w-8 text-destructive hover:bg-destructive hover:text-destructive-foreground"
      @click="$emit('delete')"
    >
      <svg
        class="w-3.5 h-3.5"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <polyline points="3 6 5 6 21 6" />
        <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
        <line x1="10" y1="11" x2="10" y2="17" />
        <line x1="14" y1="11" x2="14" y2="17" />
      </svg>
      <span class="sr-only">Delete</span>
    </Button>

    <ColumnConfigPopover
      :columns="columns"
      @update:columns="$emit('update:columns', $event)"
      @reset="$emit('resetColumns')"
    />

    <Button size="sm" class="h-8 text-xs" @click="$emit('add')">
      <svg
        class="w-3.5 h-3.5 mr-1"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <line x1="12" y1="5" x2="12" y2="19" />
        <line x1="5" y1="12" x2="19" y2="12" />
      </svg>
      New
    </Button>
    </div>

    <!-- Selected count -->
    <div v-if="multiSelectMode && selectedCount" class="text-xs text-primary font-medium">
      {{ selectedCount }} issue{{ selectedCount > 1 ? 's' : '' }} sélectionnée{{ selectedCount > 1 ? 's' : '' }}
    </div>
  </div>
</template>
