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
import LabelBadge from '~/components/issues/LabelBadge.vue'

const props = defineProps<{
  availableLabels: string[]
  selectedLabels: string[]
}>()

defineEmits<{
  toggle: [label: string]
}>()

const isSelected = (label: string) => props.selectedLabels.includes(label)
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
              <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z" />
              <line x1="7" y1="7" x2="7.01" y2="7" />
            </svg>
            Labels
            <span
              v-if="selectedLabels.length > 0"
              class="ml-0.5 rounded-full bg-primary text-primary-foreground text-[10px] px-1.5 min-w-[18px] text-center"
            >
              {{ selectedLabels.length }}
            </span>
          </Button>
        </DropdownMenuTrigger>
      </TooltipTrigger>
      <TooltipContent>Filter by label</TooltipContent>
    </Tooltip>
    <DropdownMenuContent align="start" class="w-48">
      <div v-if="availableLabels.length === 0" class="px-2 py-3 text-xs text-muted-foreground text-center">
        No labels found
      </div>
      <ScrollArea v-else class="max-h-64">
        <DropdownMenuCheckboxItem
          v-for="label in availableLabels"
          :key="label"
          :checked="isSelected(label)"
          class="text-xs cursor-pointer"
          @select.prevent="$emit('toggle', label)"
        >
          <LabelBadge :label="label" size="sm" />
        </DropdownMenuCheckboxItem>
      </ScrollArea>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
