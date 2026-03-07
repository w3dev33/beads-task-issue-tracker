import { readImageFile } from '~/utils/open-url'

const isOpen = ref(false)
const imageSrc = ref<string | null>(null)
const imageAlt = ref('Image')
const isLoading = ref(false)

// Gallery state
const imageList = ref<{ path: string; alt: string }[]>([])
const currentIndex = ref(0)

export function useImagePreview() {
  // Computed properties for gallery navigation
  const hasMultipleImages = computed(() => imageList.value.length > 1)
  const canGoNext = computed(() => currentIndex.value < imageList.value.length - 1)
  const canGoPrev = computed(() => currentIndex.value > 0)
  const imageCounter = computed(() => `${currentIndex.value + 1} / ${imageList.value.length}`)

  // Load image at current index
  const loadCurrentImage = async () => {
    const current = imageList.value[currentIndex.value]
    if (!current) return

    isLoading.value = true
    imageSrc.value = null
    imageAlt.value = current.alt

    const data = await readImageFile(current.path)
    if (data) {
      imageSrc.value = `data:${data.mimeType};base64,${data.base64}`
    }
    isLoading.value = false
  }

  // Open single image (legacy support)
  const openImage = async (filePath: string, alt = 'Image') => {
    imageList.value = [{ path: filePath, alt }]
    currentIndex.value = 0
    isOpen.value = true
    await loadCurrentImage()
  }

  // Open gallery with multiple images
  const openGallery = async (images: { path: string; alt: string }[], startIndex = 0) => {
    if (images.length === 0) return
    imageList.value = images
    currentIndex.value = Math.max(0, Math.min(startIndex, images.length - 1))
    isOpen.value = true
    await loadCurrentImage()
  }

  // Navigate to next image
  const goNext = async () => {
    if (!canGoNext.value) return
    currentIndex.value++
    await loadCurrentImage()
  }

  // Navigate to previous image
  const goPrev = async () => {
    if (!canGoPrev.value) return
    currentIndex.value--
    await loadCurrentImage()
  }

  const closeImage = () => {
    isOpen.value = false
    imageSrc.value = null
    imageList.value = []
    currentIndex.value = 0
  }

  return {
    isOpen,
    imageSrc,
    imageAlt,
    isLoading,
    hasMultipleImages,
    canGoNext,
    canGoPrev,
    imageCounter,
    openImage,
    openGallery,
    goNext,
    goPrev,
    closeImage,
  }
}
