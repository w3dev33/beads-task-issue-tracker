<script setup lang="ts">
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
import { ScrollArea } from '~/components/ui/scroll-area'

const props = defineProps<{
  availableAssignees: string[]
  selectedAssignees: string[]
  open?: boolean
}>()

defineEmits<{
  toggle: [assignee: string]
  'update:open': [value: boolean]
}>()

const isSelected = (assignee: string) => props.selectedAssignees.includes(assignee)
</script>

<template>
  <Tooltip>
    <DropdownMenu :open="open" :modal="false" @update:open="$emit('update:open', $event)">
      <TooltipTrigger as-child>
        <DropdownMenuTrigger as-child>
          <Button variant="outline" size="sm" class="h-8 text-xs gap-1">
            <!-- User icon -->
            <svg
              class="w-3.5 h-3.5"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
              <circle cx="12" cy="7" r="4" />
            </svg>
            Assignee
            <span
              v-if="selectedAssignees.length > 0"
              class="ml-0.5 rounded-full bg-primary text-primary-foreground text-[10px] px-1.5 min-w-[18px] text-center"
            >
              {{ selectedAssignees.length }}
            </span>
          </Button>
        </DropdownMenuTrigger>
      </TooltipTrigger>
      <TooltipContent>Filter by assignee</TooltipContent>
      <DropdownMenuContent align="start" class="w-48">
        <div v-if="availableAssignees.length === 0" class="px-2 py-3 text-xs text-muted-foreground text-center">
          No assignees found
        </div>
        <ScrollArea v-else class="max-h-64">
          <DropdownMenuCheckboxItem
            v-for="assignee in availableAssignees"
            :key="assignee"
            :checked="isSelected(assignee)"
            class="text-xs cursor-pointer"
            @select.prevent="$emit('toggle', assignee)"
          >
            <span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded bg-slate-600 text-white text-xs font-medium">
              <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                <circle cx="12" cy="7" r="4" />
              </svg>
              {{ assignee }}
            </span>
          </DropdownMenuCheckboxItem>
        </ScrollArea>
      </DropdownMenuContent>
    </DropdownMenu>
  </Tooltip>
</template>
