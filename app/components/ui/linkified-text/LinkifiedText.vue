<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { renderMarkdown } from '~/utils/markdown'
import { openUrl, isValidUrl, normalizeUrl } from '~/utils/open-url'

const props = defineProps<{
  text: string | undefined | null
  fallback?: string
}>()

const renderedHtml = computed(() => {
  if (!props.text) return ''
  return renderMarkdown(props.text)
})

const displayFallback = computed(() => props.fallback || '')

const containerRef = ref<HTMLElement | null>(null)

// Debounce flag to prevent multiple opens
let isOpening = false

// Handle clicks on links to open them with our openUrl utility
const handleClick = (event: MouseEvent) => {
  const target = event.target as HTMLElement
  const link = target.closest('a[data-external-link]') as HTMLAnchorElement | null

  if (link) {
    event.preventDefault()
    event.stopPropagation()

    // Prevent multiple rapid clicks
    if (isOpening) return
    isOpening = true
    setTimeout(() => { isOpening = false }, 500)

    const href = link.getAttribute('href')
    if (href) {
      const normalizedUrl = normalizeUrl(href)
      if (isValidUrl(normalizedUrl)) {
        openUrl(href)
      }
    }
  }
}

onMounted(() => {
  containerRef.value?.addEventListener('click', handleClick)
})

onUnmounted(() => {
  containerRef.value?.removeEventListener('click', handleClick)
})
</script>

<template>
  <span v-if="text" ref="containerRef" class="markdown-content" v-html="renderedHtml" />
  <span v-else>{{ displayFallback }}</span>
</template>
