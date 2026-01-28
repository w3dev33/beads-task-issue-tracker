<script setup lang="ts">
import type { IssueType } from '~/types/issue'
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
import TypeBadge from '~/components/issues/TypeBadge.vue'

const props = defineProps<{
  selectedTypes: IssueType[]
}>()

defineEmits<{
  toggle: [type: IssueType]
}>()

const typeOptions: { value: IssueType; label: string }[] = [
  { value: 'bug', label: 'Bug' },
  { value: 'task', label: 'Task' },
  { value: 'feature', label: 'Feature' },
  { value: 'epic', label: 'Epic' },
  { value: 'chore', label: 'Chore' },
]

const isSelected = (type: IssueType) => props.selectedTypes.includes(type)
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
              <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z" />
              <polyline points="14 2 14 8 20 8" />
            </svg>
            Type
            <span
              v-if="selectedTypes.length > 0"
              class="ml-0.5 rounded-full bg-primary text-primary-foreground text-[10px] px-1.5 min-w-[18px] text-center"
            >
              {{ selectedTypes.length }}
            </span>
          </Button>
        </DropdownMenuTrigger>
      </TooltipTrigger>
      <TooltipContent>Filter by type</TooltipContent>
    </Tooltip>
    <DropdownMenuContent align="start" class="w-36">
      <DropdownMenuCheckboxItem
        v-for="opt in typeOptions"
        :key="opt.value"
        :checked="isSelected(opt.value)"
        class="text-xs cursor-pointer"
        @select.prevent="$emit('toggle', opt.value)"
      >
        <TypeBadge :type="opt.value" size="sm" />
      </DropdownMenuCheckboxItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
