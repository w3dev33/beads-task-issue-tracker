<script setup lang="ts">
const props = defineProps<{
  label: string
  size?: 'sm' | 'md'
}>()

// Generate a consistent color based on the label text using djb2 hash
const getColorFromLabel = (label: string) => {
  // djb2 hash algorithm - produces better distribution
  let hash = 5381
  for (let i = 0; i < label.length; i++) {
    hash = ((hash << 5) + hash) ^ label.charCodeAt(i)
  }

  // Solid color palette - vibrant fuchsia/purple tones with good contrast
  // Avoiding colors used by Type/Status/Priority badges
  const colors = [
    '#be185d', // pink-700
    '#a21caf', // fuchsia-700
    '#7e22ce', // purple-700
    '#6d28d9', // violet-700
    '#4f46e5', // indigo-600
    '#0284c7', // sky-600
    '#0d9488', // teal-600
    '#c026d3', // fuchsia-600
    '#db2777', // pink-600
    '#7c3aed', // violet-600
    '#0891b2', // cyan-600
    '#9333ea', // purple-600
  ]

  const index = Math.abs(hash) % colors.length
  return colors[index]
}

const bgColor = computed(() => getColorFromLabel(props.label))

const sizeClasses = computed(() => {
  return props.size === 'sm'
    ? 'text-[10px] px-1.5 py-0.5'
    : 'text-xs px-2 py-0.5'
})
</script>

<template>
  <span
    class="inline-flex items-center rounded font-medium whitespace-nowrap text-white"
    :class="sizeClasses"
    :style="{ backgroundColor: bgColor }"
  >
    {{ label }}
  </span>
</template>
