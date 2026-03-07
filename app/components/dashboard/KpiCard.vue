<script setup lang="ts">
const props = defineProps<{
  title: string
  value: number
  color?: string
  active?: boolean
}>()

defineEmits<{
  click: []
}>()

const { currentTheme } = useTheme()
const isNeon = computed(() => currentTheme.value.id === 'neon')

// Parse a CSS color string to extract RGB for rgba()
const parseColorToRgb = (color: string): string | null => {
  // Handle var(--xxx) by reading computed style
  if (color.startsWith('var(')) {
    if (!import.meta.client) return null
    const varName = color.slice(4, -1).trim()
    const resolved = getComputedStyle(document.documentElement).getPropertyValue(varName).trim()
    if (resolved) return parseColorToRgb(resolved)
    return null
  }
  // Handle #hex
  if (color.startsWith('#')) {
    const hex = color.slice(1)
    const r = parseInt(hex.slice(0, 2), 16)
    const g = parseInt(hex.slice(2, 4), 16)
    const b = parseInt(hex.slice(4, 6), 16)
    return `${r}, ${g}, ${b}`
  }
  return null
}

const neonStyle = computed(() => {
  if (!isNeon.value) return {}
  const rgb = props.color ? parseColorToRgb(props.color) : '224, 224, 232' // foreground neutral
  if (!rgb) return {}
  return {
    background: `rgba(${rgb}, 0.08)`,
    borderColor: `rgba(${rgb}, 0.25)`,
    boxShadow: `inset 0 0 14px rgba(${rgb}, 0.06)`,
  }
})

const neonTitleStyle = computed(() => {
  if (!isNeon.value) return {}
  if (props.color) return { color: props.color, opacity: 0.6 }
  return { opacity: 0.5 }
})
</script>

<template>
  <button
    class="p-1.5 rounded-md text-left w-full transition-colors"
    :class="[
      active ? 'ring-2 ring-primary' : '',
      isNeon
        ? 'border hover:brightness-125'
        : 'bg-secondary/30 border border-border/50 hover:bg-secondary/50'
    ]"
    :style="isNeon ? neonStyle : {}"
    @click="$emit('click')"
  >
    <div
      class="text-[9px] uppercase tracking-wide mb-0.5 truncate"
      :class="isNeon && color ? '' : 'text-muted-foreground'"
      :style="neonTitleStyle"
    >
      {{ title }}
    </div>
    <div class="text-lg font-bold" :style="color ? { color } : {}">
      {{ value }}
    </div>
  </button>
</template>
