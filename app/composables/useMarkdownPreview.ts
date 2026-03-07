import { readTextFile, writeTextFile } from '~/utils/open-url'

const isOpen = ref(false)
const markdownContent = ref('')
const markdownTitle = ref('Markdown')
const markdownPath = ref('')
const isLoading = ref(false)

// Gallery state
const fileList = ref<{ path: string; alt: string }[]>([])
const currentIndex = ref(0)

// Edit mode state
const isEditMode = ref(false)
const editedContent = ref('')
const isSaving = ref(false)
const showSaveConfirm = ref(false)

export function useMarkdownPreview() {
  const hasMultipleFiles = computed(() => fileList.value.length > 1)
  const canGoNext = computed(() => currentIndex.value < fileList.value.length - 1)
  const canGoPrev = computed(() => currentIndex.value > 0)
  const fileCounter = computed(() => `${currentIndex.value + 1} / ${fileList.value.length}`)

  const loadCurrentFile = async () => {
    const current = fileList.value[currentIndex.value]
    if (!current) return

    isLoading.value = true
    markdownContent.value = ''
    markdownTitle.value = current.alt
    markdownPath.value = current.path

    const data = await readTextFile(current.path)
    if (data) {
      markdownContent.value = data.content
    }
    isLoading.value = false
  }

  // Open single markdown file (legacy)
  const openMarkdown = async (filePath: string, title = 'Markdown') => {
    fileList.value = [{ path: filePath, alt: title }]
    currentIndex.value = 0
    isEditMode.value = false
    isOpen.value = true
    await loadCurrentFile()
  }

  // Open gallery with multiple markdown files
  const openMarkdownGallery = async (files: { path: string; alt: string }[], startIndex = 0) => {
    if (files.length === 0) return
    fileList.value = files
    currentIndex.value = Math.max(0, Math.min(startIndex, files.length - 1))
    isEditMode.value = false
    isOpen.value = true
    await loadCurrentFile()
  }

  const goNext = async () => {
    if (!canGoNext.value) return
    currentIndex.value++
    await loadCurrentFile()
  }

  const goPrev = async () => {
    if (!canGoPrev.value) return
    currentIndex.value--
    await loadCurrentFile()
  }

  const toggleEdit = () => {
    if (isEditMode.value) {
      // Switch back to read mode (cancel)
      cancelEdit()
    } else {
      // Enter edit mode
      editedContent.value = markdownContent.value
      isEditMode.value = true
    }
  }

  const requestSave = () => {
    showSaveConfirm.value = true
  }

  const confirmSave = async () => {
    if (!markdownPath.value || isSaving.value) return

    isSaving.value = true
    const success = await writeTextFile(markdownPath.value, editedContent.value)

    if (success) {
      showSaveConfirm.value = false
      await loadCurrentFile()
      isEditMode.value = false
    }
    isSaving.value = false
  }

  const cancelSave = () => {
    showSaveConfirm.value = false
  }

  const cancelEdit = () => {
    editedContent.value = ''
    isEditMode.value = false
  }

  const closeMarkdown = () => {
    isOpen.value = false
    markdownContent.value = ''
    markdownPath.value = ''
    fileList.value = []
    currentIndex.value = 0
    isLoading.value = false
    isEditMode.value = false
    editedContent.value = ''
    isSaving.value = false
    showSaveConfirm.value = false
  }

  return {
    isOpen,
    markdownContent,
    markdownTitle,
    markdownPath,
    isLoading,
    hasMultipleFiles,
    canGoNext,
    canGoPrev,
    fileCounter,
    isEditMode,
    editedContent,
    isSaving,
    showSaveConfirm,
    openMarkdown,
    openMarkdownGallery,
    goNext,
    goPrev,
    toggleEdit,
    requestSave,
    confirmSave,
    cancelSave,
    cancelEdit,
    closeMarkdown,
  }
}
