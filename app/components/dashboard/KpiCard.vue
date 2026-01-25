<script setup lang="ts">
defineProps<{
  title: string
  value: number
  color?: string
  active?: boolean
}>()

defineEmits<{
  click: []
}>()
</script>

<template>
  <button
    class="p-1.5 rounded-md bg-secondary/30 border border-border/50 text-left w-full transition-colors hover:bg-secondary/50"
    :class="{ 'ring-2 ring-primary': active }"
    @click="$emit('click')"
  >
    <div class="text-[9px] text-muted-foreground uppercase tracking-wide mb-0.5 truncate">
      {{ title }}
    </div>
    <div class="text-lg font-bold" :style="color ? { color } : {}">
      {{ value }}
    </div>
  </button>
</template>
