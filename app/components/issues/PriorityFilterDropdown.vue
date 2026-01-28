<script setup lang="ts">
import type { IssuePriority } from '~/types/issue'
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
import PriorityBadge from '~/components/issues/PriorityBadge.vue'

const props = defineProps<{
  selectedPriorities: IssuePriority[]
}>()

defineEmits<{
  toggle: [priority: IssuePriority]
}>()

const priorityOptions: { value: IssuePriority; label: string }[] = [
  { value: 'p0', label: 'P0 - Critical' },
  { value: 'p1', label: 'P1 - High' },
  { value: 'p2', label: 'P2 - Medium' },
  { value: 'p3', label: 'P3 - Low' },
  { value: 'p4', label: 'P4 - Minimal' },
]

const isSelected = (priority: IssuePriority) => props.selectedPriorities.includes(priority)
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
              <path d="M22 12h-4l-3 9L9 3l-3 9H2" />
            </svg>
            Priority
            <span
              v-if="selectedPriorities.length > 0"
              class="ml-0.5 rounded-full bg-primary text-primary-foreground text-[10px] px-1.5 min-w-[18px] text-center"
            >
              {{ selectedPriorities.length }}
            </span>
          </Button>
        </DropdownMenuTrigger>
      </TooltipTrigger>
      <TooltipContent>Filter by priority</TooltipContent>
    </Tooltip>
    <DropdownMenuContent align="start" class="w-36">
      <DropdownMenuCheckboxItem
        v-for="opt in priorityOptions"
        :key="opt.value"
        :checked="isSelected(opt.value)"
        class="text-xs cursor-pointer"
        @select.prevent="$emit('toggle', opt.value)"
      >
        <PriorityBadge :priority="opt.value" size="sm" />
      </DropdownMenuCheckboxItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
