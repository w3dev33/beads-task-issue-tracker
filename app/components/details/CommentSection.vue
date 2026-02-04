<script setup lang="ts">
import type { Comment } from '~/types/issue'
import { Button } from '~/components/ui/button'
import { Textarea } from '~/components/ui/textarea'
import { Avatar, AvatarFallback } from '~/components/ui/avatar'
import { ScrollArea } from '~/components/ui/scroll-area'
import { LinkifiedText } from '~/components/ui/linkified-text'

const props = defineProps<{
  comments: Comment[]
  readonly?: boolean
}>()

// Collapsible state (persisted per project, open by default)
const commentsSectionState = useProjectStorage<{ open: boolean }>('commentsSection', { open: true })
const isCommentsOpen = computed(() => commentsSectionState.value.open)
const toggleComments = () => {
  const newValue = { open: !commentsSectionState.value.open }
  commentsSectionState.value = newValue
  // Explicitly save since watcher doesn't trigger reliably
  saveProjectValue('commentsSection', newValue)
}

// Sort comments by date descending (most recent first)
const sortedComments = computed(() => {
  return [...props.comments].sort((a, b) => {
    return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
  })
})

const newComment = ref('')

const formatDate = (dateStr: string) => {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  return date.toLocaleDateString(undefined, {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

const getInitials = (name: string) => {
  return name
    .split(' ')
    .map((n) => n[0])
    .join('')
    .toUpperCase()
    .slice(0, 2)
}

const emit = defineEmits<{
  addComment: [content: string]
}>()

const handleSubmit = () => {
  if (newComment.value.trim()) {
    emit('addComment', newComment.value.trim())
    newComment.value = ''
  }
}
</script>

<template>
  <div class="space-y-3">
    <button
      class="flex items-center gap-1.5 w-full text-left group"
      @click="toggleComments"
    >
      <svg
        class="w-3 h-3 text-muted-foreground transition-transform"
        :class="{ '-rotate-90': !isCommentsOpen }"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <polyline points="6 9 12 15 18 9" />
      </svg>
      <h4 class="text-[10px] font-medium uppercase tracking-wide text-muted-foreground group-hover:text-foreground transition-colors">
        Comments ({{ comments.length }})
      </h4>
    </button>

    <div v-show="isCommentsOpen" class="pl-4.5 space-y-3">
      <ScrollArea v-if="sortedComments.length > 0" class="h-40">
        <div class="space-y-3 pr-4">
          <div
            v-for="comment in sortedComments"
            :key="comment.id"
            class="flex gap-2"
          >
            <Avatar class="h-6 w-6">
              <AvatarFallback class="text-[10px]">
                {{ getInitials(comment.author) }}
              </AvatarFallback>
            </Avatar>

            <div class="flex-1 space-y-0.5">
              <div class="flex items-center gap-2">
                <span class="text-xs font-medium">{{ comment.author }}</span>
                <span class="text-[10px] text-muted-foreground">
                  {{ formatDate(comment.createdAt) }}
                </span>
              </div>
              <p class="text-xs whitespace-pre-wrap"><LinkifiedText :text="comment.content" /></p>
            </div>
          </div>
        </div>
      </ScrollArea>

      <div v-else class="text-center text-muted-foreground text-xs py-3">
        No comments yet
      </div>

      <form v-if="!readonly" class="space-y-2" @submit.prevent="handleSubmit">
        <Textarea
          v-model="newComment"
          placeholder="Add a comment..."
          rows="2"
          class="text-xs"
        />
        <div class="flex justify-end">
          <Button type="submit" size="sm" class="h-7 text-xs" :disabled="!newComment.trim()">
            Add Comment
          </Button>
        </div>
      </form>
    </div>
  </div>
</template>
