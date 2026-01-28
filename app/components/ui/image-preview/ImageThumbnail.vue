<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { X } from 'lucide-vue-next'
import { readImageFile } from '~/utils/open-url'
import { isUrl } from '~/utils/markdown'

const props = defineProps<{
  src: string
  alt: string
}>()

const emit = defineEmits<{
  click: []
  remove: []
}>()

const imageDataUrl = ref<string | null>(null)
const isLoading = ref(true)
const hasError = ref(false)

// Check if src is a URL (http/https)
const isRemoteUrl = computed(() => isUrl(props.src))

const { beadsPath } = useBeadsPath()

onMounted(async () => {
  // For URLs, use directly without loading via backend
  if (isRemoteUrl.value) {
    imageDataUrl.value = props.src
    isLoading.value = false
    return
  }

  // For local paths, load via backend
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
  // Absolute paths used directly, relative paths get .beads prefix
  const fullPath = props.src.startsWith('/')
    ? props.src
    : `${beadsPath.value}/.beads/${props.src}`
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

const handleRemove = (event: Event) => {
  event.stopPropagation()
  emit('remove')
}

// Handle image load error for remote URLs
const handleImageError = () => {
  if (isRemoteUrl.value) {
    hasError.value = true
  }
}
</script>

<template>
  <div
    class="relative inline-block cursor-pointer group"
    @click="emit('click')"
  >
    <!-- Remove button (appears on hover) -->
    <button
      type="button"
      class="absolute -top-2 -right-2 z-10 w-6 h-6 rounded-full bg-destructive text-destructive-foreground opacity-0 group-hover:opacity-100 hover:bg-destructive/80 active:scale-90 transition-all flex items-center justify-center shadow-md"
      @click="handleRemove"
    >
      <X class="w-4 h-4" />
    </button>

    <!-- Loading state -->
    <div v-if="isLoading" class="w-[250px] h-[150px] bg-muted rounded-lg flex items-center justify-center">
      <span class="text-sm text-muted-foreground">Loading...</span>
    </div>

    <!-- Error state -->
    <div v-else-if="hasError" class="w-[250px] h-[150px] bg-destructive/10 rounded-lg flex items-center justify-center">
      <span class="text-sm text-destructive">Error loading image</span>
    </div>

    <!-- Image -->
    <img
      v-else
      :src="imageDataUrl!"
      :alt="alt"
      class="w-[250px] h-auto rounded-lg border-2 border-border hover:border-primary transition-colors"
      @error="handleImageError"
    />
  </div>
</template>
