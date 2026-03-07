<script setup lang="ts">
import { ChevronLeft, ChevronRight } from 'lucide-vue-next'
import {
  Dialog,
  DialogContent,
  DialogTitle,
} from '~/components/ui/dialog'
import { Button } from '~/components/ui/button'

const props = defineProps<{
  open: boolean
  imageSrc: string | null
  imageAlt: string
  hasMultipleImages?: boolean
  canGoNext?: boolean
  canGoPrev?: boolean
  imageCounter?: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  next: []
  prev: []
}>()

// Keyboard navigation
const handleKeydown = (e: KeyboardEvent) => {
  if (!props.open) return

  if (e.key === 'ArrowRight' && props.canGoNext) {
    e.preventDefault()
    emit('next')
  } else if (e.key === 'ArrowLeft' && props.canGoPrev) {
    e.preventDefault()
    emit('prev')
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="!w-[calc(100vw-5rem)] !h-[calc(100vh-5rem)] !max-w-none p-4 flex flex-col">
      <div class="flex items-center gap-2 mb-2">
        <DialogTitle class="text-sm font-medium">{{ imageAlt }}</DialogTitle>
        <span v-if="hasMultipleImages" class="text-xs text-muted-foreground">
          {{ imageCounter }}
        </span>
      </div>
      <div class="relative flex-1 flex items-center justify-center min-h-0 overflow-auto">
        <!-- Previous button -->
        <Button
          v-if="hasMultipleImages"
          variant="outline"
          size="icon"
          class="absolute left-2 top-1/2 -translate-y-1/2 z-10 w-10 h-10 rounded-full bg-background/80 border border-border shadow-md hover:bg-background disabled:opacity-30 disabled:cursor-not-allowed"
          :disabled="!canGoPrev"
          @click="emit('prev')"
        >
          <ChevronLeft class="w-5 h-5" />
        </Button>

        <img
          v-if="imageSrc"
          :src="imageSrc"
          :alt="imageAlt"
          class="max-w-full max-h-full object-contain rounded"
        />
        <div v-else class="flex items-center justify-center h-32 text-muted-foreground">
          Loading...
        </div>

        <!-- Next button -->
        <Button
          v-if="hasMultipleImages"
          variant="outline"
          size="icon"
          class="absolute right-2 top-1/2 -translate-y-1/2 z-10 w-10 h-10 rounded-full bg-background/80 border border-border shadow-md hover:bg-background disabled:opacity-30 disabled:cursor-not-allowed"
          :disabled="!canGoNext"
          @click="emit('next')"
        >
          <ChevronRight class="w-5 h-5" />
        </Button>
      </div>
    </DialogContent>
  </Dialog>
</template>
