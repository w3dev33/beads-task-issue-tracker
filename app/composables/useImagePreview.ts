import { readImageFile } from '~/utils/open-url'

const isOpen = ref(false)
const imageSrc = ref<string | null>(null)
const imageAlt = ref('Image')
const isLoading = ref(false)

export function useImagePreview() {
  const openImage = async (filePath: string, alt = 'Image') => {
    isLoading.value = true
    isOpen.value = true
    imageAlt.value = alt
    imageSrc.value = null

    const data = await readImageFile(filePath)
    if (data) {
      imageSrc.value = `data:${data.mimeType};base64,${data.base64}`
    }
    isLoading.value = false
  }

  const closeImage = () => {
    isOpen.value = false
    imageSrc.value = null
  }

  return {
    isOpen,
    imageSrc,
    imageAlt,
    isLoading,
    openImage,
    closeImage,
  }
}
