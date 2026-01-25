<script setup lang="ts">
import { Button } from '~/components/ui/button'

defineEmits<{
  browse: []
}>()
</script>

<template>
  <div class="flex flex-col items-center text-center space-y-3 p-4 max-w-xs">
    <div class="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center">
      <svg class="w-6 h-6 text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
      </svg>
    </div>
    <div class="space-y-1">
      <h3 class="text-sm font-semibold">No project selected</h3>
      <p class="text-xs text-muted-foreground">
        Browse to select a Beads project folder to get started
      </p>
    </div>
    <Button variant="default" size="sm" class="h-8" @click="$emit('browse')">
      <svg class="w-3.5 h-3.5 mr-1.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
      </svg>
      Browse
    </Button>
  </div>
</template>
