<script setup lang="ts">
const props = defineProps<{
  open: number
  closed: number
}>()

const total = computed(() => props.open + props.closed)
const openPercent = computed(() =>
  total.value > 0 ? (props.open / total.value) * 100 : 0
)
const closedPercent = computed(() =>
  total.value > 0 ? (props.closed / total.value) * 100 : 0
)
</script>

<template>
  <div class="space-y-3">
    <h4 class="text-sm font-medium text-muted-foreground">Status Distribution</h4>

    <div class="space-y-2">
      <div class="flex items-center gap-2">
        <span class="w-16 text-xs text-muted-foreground">Open</span>
        <div class="flex-1 h-4 bg-secondary rounded overflow-hidden">
          <div
            class="h-full bg-status-open transition-all"
            :style="{ width: `${openPercent}%` }"
          />
        </div>
        <span class="w-8 text-xs text-right">{{ open }}</span>
      </div>

      <div class="flex items-center gap-2">
        <span class="w-16 text-xs text-muted-foreground">Closed</span>
        <div class="flex-1 h-4 bg-secondary rounded overflow-hidden">
          <div
            class="h-full bg-status-closed transition-all"
            :style="{ width: `${closedPercent}%` }"
          />
        </div>
        <span class="w-8 text-xs text-right">{{ closed }}</span>
      </div>
    </div>
  </div>
</template>
