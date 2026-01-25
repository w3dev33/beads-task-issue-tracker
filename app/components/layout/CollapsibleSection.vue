<script setup lang="ts">
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '~/components/ui/collapsible'
import { Button } from '~/components/ui/button'

defineProps<{
  title: string
  isOpen: boolean
  count?: number
}>()

defineEmits<{
  toggle: []
}>()
</script>

<template>
  <Collapsible :open="isOpen" class="border border-border rounded-md bg-card">
    <CollapsibleTrigger as-child>
      <Button
        variant="ghost"
        class="w-full justify-between px-4 py-3 h-auto font-medium hover:bg-secondary/50"
        @click="$emit('toggle')"
      >
        <div class="flex items-center gap-2">
          <svg
            class="w-4 h-4 transition-transform"
            :class="{ 'rotate-90': isOpen }"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <polyline points="9 18 15 12 9 6" />
          </svg>
          <span>{{ title }}</span>
          <span v-if="count !== undefined" class="text-muted-foreground text-sm">
            ({{ count }})
          </span>
        </div>
      </Button>
    </CollapsibleTrigger>

    <CollapsibleContent class="border-t border-border">
      <div class="p-4">
        <slot />
      </div>
    </CollapsibleContent>
  </Collapsible>
</template>
