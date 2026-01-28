<script setup lang="ts">
import {
  Dialog,
  DialogContent,
  DialogTitle,
} from '~/components/ui/dialog'

defineProps<{
  open: boolean
  imageSrc: string | null
  imageAlt: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="!w-[calc(100vw-5rem)] !h-[calc(100vh-5rem)] !max-w-none p-4 flex flex-col">
      <DialogTitle class="text-sm font-medium mb-2">{{ imageAlt }}</DialogTitle>
      <div class="flex-1 flex items-center justify-center min-h-0 overflow-auto">
        <img
          v-if="imageSrc"
          :src="imageSrc"
          :alt="imageAlt"
          class="max-w-full max-h-full object-contain rounded"
        />
        <div v-else class="flex items-center justify-center h-32 text-muted-foreground">
          Loading...
        </div>
      </div>
    </DialogContent>
  </Dialog>
</template>
