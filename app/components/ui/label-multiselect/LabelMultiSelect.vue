<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import { X, Plus, ChevronDown } from 'lucide-vue-next'
import LabelBadge from '~/components/issues/LabelBadge.vue'

const props = withDefaults(defineProps<{
  modelValue: string[]
  availableLabels: string[]
  placeholder?: string
}>(), {
  placeholder: 'Add labels...',
})

const emit = defineEmits<{
  'update:modelValue': [value: string[]]
}>()

const searchQuery = ref('')
const isOpen = ref(false)
const inputRef = ref<HTMLInputElement | null>(null)
const containerRef = ref<HTMLDivElement | null>(null)

// Filter available labels based on search, excluding already selected
const filteredLabels = computed(() => {
  const query = searchQuery.value.toLowerCase().trim()
  return props.availableLabels
    .filter(label => !props.modelValue.includes(label))
    .filter(label => !query || label.toLowerCase().includes(query))
})

// Check if current query matches no existing label (for "create new" option)
const showCreateOption = computed(() => {
  const query = searchQuery.value.trim()
  if (!query) return false
  const normalizedQuery = query.toLowerCase()
  const exactMatch = props.availableLabels.some(label => label.toLowerCase() === normalizedQuery)
  const alreadySelected = props.modelValue.some(label => label.toLowerCase() === normalizedQuery)
  return !exactMatch && !alreadySelected
})

const handleSelect = (label: string) => {
  if (!props.modelValue.includes(label)) {
    emit('update:modelValue', [...props.modelValue, label])
  }
  searchQuery.value = ''
  nextTick(() => inputRef.value?.focus())
}

const handleCreateNew = () => {
  const newLabel = searchQuery.value.trim()
  if (newLabel && !props.modelValue.includes(newLabel)) {
    emit('update:modelValue', [...props.modelValue, newLabel])
  }
  searchQuery.value = ''
  nextTick(() => inputRef.value?.focus())
}

const handleRemove = (label: string) => {
  emit('update:modelValue', props.modelValue.filter(l => l !== label))
}

const handleInputFocus = () => {
  isOpen.value = true
}

const handleInputBlur = (e: FocusEvent) => {
  // Check if focus is moving to an element inside the container
  const relatedTarget = e.relatedTarget as HTMLElement | null
  if (containerRef.value?.contains(relatedTarget)) {
    return
  }
  // Delay closing to allow click events on dropdown items
  setTimeout(() => {
    isOpen.value = false
    searchQuery.value = ''
  }, 150)
}

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter' && showCreateOption.value) {
    e.preventDefault()
    handleCreateNew()
  } else if (e.key === 'Escape') {
    isOpen.value = false
    searchQuery.value = ''
  }
}
</script>

<template>
  <div ref="containerRef" class="space-y-2">
    <!-- Selected labels as badges -->
    <div v-if="modelValue.length > 0" class="flex flex-wrap gap-1.5">
      <span
        v-for="label in modelValue"
        :key="label"
        class="inline-flex items-center gap-1"
      >
        <LabelBadge :label="label" size="sm" />
        <button
          type="button"
          class="inline-flex items-center justify-center rounded-full w-4 h-4 hover:bg-destructive/20 hover:text-destructive text-muted-foreground transition-colors"
          @click="handleRemove(label)"
        >
          <X class="w-3 h-3" />
        </button>
      </span>
    </div>

    <!-- Input with dropdown -->
    <div class="relative">
      <input
        ref="inputRef"
        v-model="searchQuery"
        type="text"
        :placeholder="placeholder"
        class="flex h-8 w-full rounded-md border border-input bg-background px-3 py-2 text-xs ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
        @focus="handleInputFocus"
        @blur="handleInputBlur"
        @keydown="handleKeydown"
      />

      <!-- Dropdown -->
      <div
        v-if="isOpen && (filteredLabels.length > 0 || showCreateOption)"
        class="absolute z-50 mt-1 w-full min-w-[8rem] overflow-hidden rounded-md border bg-popover text-popover-foreground shadow-md"
      >
        <div class="p-1 max-h-48 overflow-y-auto">
          <!-- Create new option -->
          <button
            v-if="showCreateOption"
            type="button"
            class="relative flex w-full cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-xs outline-none hover:bg-accent hover:text-accent-foreground"
            @mousedown.prevent
            @click="handleCreateNew"
          >
            <Plus class="w-3.5 h-3.5 mr-2 text-sky-500" />
            <span class="text-sky-500">Create "{{ searchQuery.trim() }}"</span>
          </button>

          <!-- Existing labels -->
          <button
            v-for="label in filteredLabels"
            :key="label"
            type="button"
            class="relative flex w-full cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-xs outline-none hover:bg-accent hover:text-accent-foreground"
            @mousedown.prevent
            @click="handleSelect(label)"
          >
            <LabelBadge :label="label" size="sm" />
          </button>
        </div>
      </div>

      <!-- Empty state when open but no results -->
      <div
        v-if="isOpen && filteredLabels.length === 0 && !showCreateOption && searchQuery"
        class="absolute z-50 mt-1 w-full min-w-[8rem] overflow-hidden rounded-md border bg-popover text-popover-foreground shadow-md"
      >
        <div class="px-2 py-3 text-xs text-muted-foreground text-center">
          No labels found
        </div>
      </div>
    </div>
  </div>
</template>
