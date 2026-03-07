<script setup lang="ts">
import type { ColumnConfig } from '~/types/issue'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  DropdownMenuCheckboxItem,
} from '~/components/ui/dropdown-menu'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '~/components/ui/tooltip'
import { Button } from '~/components/ui/button'

const props = defineProps<{
  columns: ColumnConfig[]
}>()

const emit = defineEmits<{
  'update:columns': [columns: ColumnConfig[]]
  reset: []
}>()

const handleToggle = (columnId: string, checked: boolean) => {
  const updatedColumns = props.columns.map((col) =>
    col.id === columnId ? { ...col, visible: checked } : col
  )
  emit('update:columns', updatedColumns)
}
</script>

<template>
  <Tooltip>
    <DropdownMenu>
      <TooltipTrigger as-child>
        <DropdownMenuTrigger as-child>
          <Button variant="outline" size="icon" class="h-8 w-8">
            <svg
              class="w-3.5 h-3.5"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <rect x="3" y="3" width="7" height="7" />
              <rect x="14" y="3" width="7" height="7" />
              <rect x="14" y="14" width="7" height="7" />
              <rect x="3" y="14" width="7" height="7" />
            </svg>
            <span class="sr-only">Column settings</span>
          </Button>
        </DropdownMenuTrigger>
      </TooltipTrigger>
      <TooltipContent>Column settings</TooltipContent>
    <DropdownMenuContent align="end" class="w-48">
      <DropdownMenuLabel class="text-xs">Visible Columns</DropdownMenuLabel>
      <DropdownMenuSeparator />
      <DropdownMenuCheckboxItem
        v-for="column in columns"
        :key="column.id"
        :model-value="column.visible"
        class="text-xs [&_svg]:text-green-500"
        @update:model-value="handleToggle(column.id, $event)"
      >
        {{ column.label }}
      </DropdownMenuCheckboxItem>
      <DropdownMenuSeparator />
      <div class="p-1">
        <Button variant="ghost" size="sm" class="w-full text-xs h-7 justify-start" @click="emit('reset')">
          Reset to defaults
        </Button>
      </div>
    </DropdownMenuContent>
    </DropdownMenu>
  </Tooltip>
</template>
