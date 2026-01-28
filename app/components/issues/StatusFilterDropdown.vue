<script setup lang="ts">
import type { IssueStatus } from '~/types/issue'
import { Button } from '~/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuCheckboxItem,
  DropdownMenuTrigger,
} from '~/components/ui/dropdown-menu'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '~/components/ui/tooltip'
import StatusBadge from '~/components/issues/StatusBadge.vue'

const props = defineProps<{
  selectedStatuses: IssueStatus[]
}>()

defineEmits<{
  toggle: [status: IssueStatus]
}>()

const statusOptions: { value: IssueStatus; label: string }[] = [
  { value: 'open', label: 'Open' },
  { value: 'in_progress', label: 'In Progress' },
  { value: 'blocked', label: 'Blocked' },
  { value: 'closed', label: 'Closed' },
]

const isSelected = (status: IssueStatus) => props.selectedStatuses.includes(status)
</script>

<template>
  <DropdownMenu>
    <Tooltip>
      <TooltipTrigger as-child>
        <DropdownMenuTrigger as-child>
          <Button variant="outline" size="sm" class="h-8 text-xs gap-1">
            <svg
              class="w-3.5 h-3.5"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <circle cx="12" cy="12" r="10" />
              <polyline points="12 6 12 12 16 14" />
            </svg>
            Status
            <span
              v-if="selectedStatuses.length > 0"
              class="ml-0.5 rounded-full bg-primary text-primary-foreground text-[10px] px-1.5 min-w-[18px] text-center"
            >
              {{ selectedStatuses.length }}
            </span>
          </Button>
        </DropdownMenuTrigger>
      </TooltipTrigger>
      <TooltipContent>Filter by status</TooltipContent>
    </Tooltip>
    <DropdownMenuContent align="start" class="w-40">
      <DropdownMenuCheckboxItem
        v-for="opt in statusOptions"
        :key="opt.value"
        :checked="isSelected(opt.value)"
        class="text-xs cursor-pointer"
        @select.prevent="$emit('toggle', opt.value)"
      >
        <StatusBadge :status="opt.value" size="sm" />
      </DropdownMenuCheckboxItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
