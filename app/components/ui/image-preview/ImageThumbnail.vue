<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { readImageFile } from '~/utils/open-url'

const props = defineProps<{
  src: string
  alt: string
}>()

const emit = defineEmits<{
  click: []
}>()

const imageDataUrl = ref<string | null>(null)
const isLoading = ref(true)
const hasError = ref(false)

const { beadsPath } = useBeadsPath()

onMounted(async () => {
  if (!beadsPath.value) {
    // Wait for beadsPath to be available
    const checkPath = setInterval(async () => {
      if (beadsPath.value) {
        clearInterval(checkPath)
        await loadImage()
      }
    }, 100)
    return
  }
  await loadImage()
})

const loadImage = async () => {
  const fullPath = `${beadsPath.value}/.beads/${props.src}`
  try {
    const imageData = await readImageFile(fullPath)
    if (imageData) {
      imageDataUrl.value = `data:${imageData.mimeType};base64,${imageData.base64}`
    } else {
      hasError.value = true
    }
  } catch {
    hasError.value = true
  }
  isLoading.value = false
}
</script>

<template>
  <div
    class="inline-block cursor-pointer"
    @click="emit('click')"
  >
    <div v-if="isLoading" class="w-[100px] h-[60px] bg-muted rounded flex items-center justify-center">
      <span class="text-xs text-muted-foreground">Loading...</span>
    </div>
    <div v-else-if="hasError" class="w-[100px] h-[60px] bg-destructive/10 rounded flex items-center justify-center">
      <span class="text-xs text-destructive">Error</span>
    </div>
    <img
      v-else
      :src="imageDataUrl!"
      :alt="alt"
      class="w-[100px] h-auto rounded border border-border hover:border-primary transition-colors"
    />
  </div>
</template>
