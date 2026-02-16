<script setup lang="ts">
const props = defineProps<{
  label: string
  size?: 'sm' | 'md'
}>()

const { currentTheme } = useTheme()

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
  return colors[index] ?? '#7e22ce'
}

// Neon-brightened palette for neon theme
const getNeonColorFromLabel = (label: string) => {
  let hash = 5381
  for (let i = 0; i < label.length; i++) {
    hash = ((hash << 5) + hash) ^ label.charCodeAt(i)
  }

  const colors = [
    '#ff69b4', // hot pink
    '#e040fb', // bright fuchsia
    '#b388ff', // bright purple
    '#7c4dff', // bright violet
    '#536dfe', // bright indigo
    '#40c4ff', // bright sky
    '#64ffda', // bright teal
    '#ea80fc', // light fuchsia
    '#ff4081', // pink accent
    '#b47cff', // light violet
    '#18ffff', // bright cyan
    '#d500f9', // vivid purple
  ]

  const index = Math.abs(hash) % colors.length
  return colors[index] ?? '#b388ff'
}

const bgColor = computed(() => getColorFromLabel(props.label))
const neonColor = computed(() => getNeonColorFromLabel(props.label))

// Darken a hex color by mixing with black
const darkenColor = (hex: string, factor = 0.45) => {
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  const dr = Math.round(r * (1 - factor))
  const dg = Math.round(g * (1 - factor))
  const db = Math.round(b * (1 - factor))
  return `#${dr.toString(16).padStart(2, '0')}${dg.toString(16).padStart(2, '0')}${db.toString(16).padStart(2, '0')}`
}

// Convert hex to rgba
const hexToRgba = (hex: string, alpha: number) => {
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  return `rgba(${r}, ${g}, ${b}, ${alpha})`
}

const badgeStyle = computed(() => {
  const isNeon = currentTheme.value.id === 'neon'
  const isFlat = currentTheme.value.id === 'flat' || currentTheme.value.id === 'light'
  const color = isNeon ? neonColor.value : bgColor.value

  if (isNeon) {
    return {
      background: hexToRgba(color, 0.12),
      color: color,
      border: `1px solid ${hexToRgba(color, 0.5)}`,
      boxShadow: `inset 0 0 12px ${hexToRgba(color, 0.1)}, 0 0 8px ${hexToRgba(color, 0.15)}`,
    }
  }

  if (isFlat) {
    return {
      background: color,
    }
  }

  // Classic gradient (light + dark)
  return {
    background: `linear-gradient(135deg, ${color}, ${darkenColor(color)})`,
  }
})

const sizeClasses = computed(() => {
  return props.size === 'sm'
    ? 'text-[10px] px-1.5 py-0.5'
    : 'text-xs'
})
</script>

<template>
  <span
    class="badge-gradient inline-flex items-center rounded font-medium whitespace-nowrap"
    :class="[sizeClasses, currentTheme.id === 'neon' ? '' : 'text-white']"
    :style="badgeStyle"
  >
    {{ label }}
  </span>
</template>
