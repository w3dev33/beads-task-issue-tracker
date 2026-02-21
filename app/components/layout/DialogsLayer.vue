<script setup lang="ts">
import StatusBadge from '~/components/issues/StatusBadge.vue'
import { Button } from '~/components/ui/button'
import { ConfirmDialog } from '~/components/ui/confirm-dialog'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog'
import ImagePreviewDialog from '~/components/ui/image-preview/ImagePreviewDialog.vue'
import MarkdownPreviewDialog from '~/components/ui/markdown-preview/MarkdownPreviewDialog.vue'

const {
  isDeleteDialogOpen, deleteTargetTitles, isDeleting, confirmDelete,
  isEpicDeleteDialogOpen, epicToDelete, epicChildren, isDeletingEpic, confirmEpicDelete,
  isCloseDialogOpen, isClosing, confirmClose,
  isDetachDialogOpen, detachImagePath, isDetaching, handleDetachImage,
  isRemoveDepDialogOpen, pendingDepRemoval, isRemovingDep, handleRemoveDependency,
  availableRelationTypes, availableIssuesForDeps, priorityTextColor,
  isAddBlockerDialogOpen, addBlockerIssueId, addBlockerSearchQuery, addBlockerSelectedTarget, addBlockerFilteredOptions, isAddingBlocker, handleAddBlocker,
  isAddRelDialogOpen, addRelIssueId, addRelSelectedType, addRelSearchQuery, addRelSelectedTarget, addRelFilterClosed, addRelFilteredOptions, isAddingRel, handleAddRelation,
  isRemoveRelDialogOpen, pendingRelRemoval, isRemovingRel, handleRemoveRelation,
} = useIssueDialogs()

const { selectedIssue } = useIssues()
const imagePreview = useImagePreview()
const markdownPreview = useMarkdownPreview()
</script>

<template>
  <!-- Delete Confirmation Dialog -->
  <ConfirmDialog
    v-model:open="isDeleteDialogOpen"
    title="Delete"
    confirm-text="Delete"
    cancel-text="Cancel"
    variant="destructive"
    :is-loading="isDeleting"
    @confirm="confirmDelete"
  >
    <template #description>
      <p class="text-sm text-muted-foreground">
        You are about to permanently delete
        {{ deleteTargetTitles.length > 1 ? 'the following issues' : 'the issue' }}:
      </p>
      <div class="mt-2 space-y-1">
        <p
          v-for="title in deleteTargetTitles.slice(0, 5)"
          :key="title"
          class="text-sm font-medium text-sky-400"
        >
          {{ title }}
        </p>
        <p v-if="deleteTargetTitles.length > 5" class="text-sm text-muted-foreground">
          ... and {{ deleteTargetTitles.length - 5 }} more ({{ deleteTargetTitles.length }} total)
        </p>
      </div>
      <p class="mt-3 text-sm text-muted-foreground">
        This action cannot be undone.
      </p>
    </template>
  </ConfirmDialog>

  <!-- Epic Delete Confirmation Dialog (for issues with children) -->
  <Dialog v-model:open="isEpicDeleteDialogOpen">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <svg class="w-5 h-5 text-destructive" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <line x1="12" y1="16" x2="12.01" y2="16" />
          </svg>
          Delete Issue with Children
        </DialogTitle>
        <DialogDescription as="div">
          <p class="text-sm text-muted-foreground">
            The issue "<span class="font-medium text-sky-400">{{ epicToDelete?.title }}</span>" has {{ epicChildren.length }} child issue{{ epicChildren.length > 1 ? 's' : '' }}:
          </p>
          <div class="mt-2 space-y-1 max-h-32 overflow-y-auto">
            <p v-for="child in epicChildren.slice(0, 5)" :key="child.id" class="text-sm text-muted-foreground">
              <span class="font-medium">{{ child.title }}</span>
            </p>
            <p v-if="epicChildren.length > 5" class="text-sm text-muted-foreground">
              ... and {{ epicChildren.length - 5 }} more
            </p>
          </div>
          <p class="mt-3 text-sm text-muted-foreground">
            What would you like to do?
          </p>
        </DialogDescription>
      </DialogHeader>
      <DialogFooter class="flex-col gap-2 sm:flex-col">
        <Button
          variant="destructive"
          class="w-full"
          :disabled="isDeletingEpic"
          @click="confirmEpicDelete('delete-all')"
        >
          <svg v-if="isDeletingEpic" class="animate-spin -ml-1 mr-2 h-4 w-4" viewBox="0 0 24 24" fill="none">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
          </svg>
          Delete Issue and All Children
        </Button>
        <Button
          variant="outline"
          class="w-full"
          :disabled="isDeletingEpic"
          @click="confirmEpicDelete('detach')"
        >
          Delete Issue Only (Detach Children)
        </Button>
        <Button
          variant="ghost"
          class="w-full"
          :disabled="isDeletingEpic"
          @click="isEpicDeleteDialogOpen = false"
        >
          Cancel
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>

  <!-- Close Confirmation Dialog -->
  <ConfirmDialog
    v-model:open="isCloseDialogOpen"
    title="Close issue"
    confirm-text="Close"
    cancel-text="Cancel"
    :is-loading="isClosing"
    @confirm="confirmClose"
  >
    <template #description>
      <p class="text-sm text-muted-foreground">
        You are about to close the issue:
      </p>
      <div class="mt-2">
        <p class="text-sm text-sky-400 font-mono">{{ selectedIssue?.id }}</p>
        <p class="text-sm font-medium">{{ selectedIssue?.title }}</p>
      </div>
      <p class="mt-3 text-sm text-muted-foreground">
        The issue will be marked as completed.
      </p>
    </template>
  </ConfirmDialog>

  <!-- Detach Attachment Confirmation Dialog -->
  <ConfirmDialog
    v-model:open="isDetachDialogOpen"
    title="Detach attachment"
    confirm-text="Detach"
    cancel-text="Cancel"
    variant="destructive"
    :is-loading="isDetaching"
    @confirm="handleDetachImage"
  >
    <template #description>
      <p class="text-sm text-muted-foreground">
        Are you sure you want to detach this attachment?
      </p>
      <p class="mt-2 text-xs text-muted-foreground font-mono break-all">
        {{ detachImagePath }}
      </p>
      <p v-if="detachImagePath?.includes('.beads/attachments/')" class="mt-3 text-sm text-destructive">
        The attachment file will be permanently deleted.
      </p>
      <p v-else class="mt-3 text-sm text-muted-foreground">
        Only the reference will be removed. The original file will not be deleted.
      </p>
    </template>
  </ConfirmDialog>

  <!-- Remove Dependency Confirmation Dialog -->
  <ConfirmDialog
    v-model:open="isRemoveDepDialogOpen"
    title="Remove dependency"
    confirm-text="Remove"
    cancel-text="Cancel"
    variant="destructive"
    :is-loading="isRemovingDep"
    @confirm="handleRemoveDependency"
  >
    <template #description>
      <p class="text-sm text-muted-foreground">
        Are you sure you want to remove this dependency?
      </p>
      <div v-if="pendingDepRemoval" class="mt-2 space-y-2">
        <div>
          <p class="text-xs text-muted-foreground uppercase tracking-wide">Issue</p>
          <p class="text-sm font-mono text-sky-400">{{ pendingDepRemoval.issueId }}</p>
          <p class="text-sm text-muted-foreground">{{ availableIssuesForDeps.find(i => i.id === pendingDepRemoval!.issueId)?.title }}</p>
        </div>
        <div>
          <p class="text-xs text-muted-foreground uppercase tracking-wide">Blocker</p>
          <p class="text-sm font-mono text-sky-400">{{ pendingDepRemoval.blockerId }}</p>
          <p class="text-sm text-muted-foreground">{{ availableIssuesForDeps.find(i => i.id === pendingDepRemoval!.blockerId)?.title }}</p>
        </div>
      </div>
    </template>
  </ConfirmDialog>

  <!-- Add Blocker Dialog -->
  <Dialog v-model:open="isAddBlockerDialogOpen">
    <DialogContent class="sm:max-w-lg overflow-hidden">
      <DialogHeader>
        <DialogTitle>Add a blocker</DialogTitle>
        <DialogDescription>
          From <span class="font-mono text-sky-400">{{ addBlockerIssueId }}</span>
          <br />
          <span class="text-muted-foreground">{{ availableIssuesForDeps.find(i => i.id === addBlockerIssueId)?.title }}</span>
        </DialogDescription>
      </DialogHeader>
      <div class="space-y-4 py-2 overflow-hidden">
        <div class="space-y-1.5 overflow-hidden">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wide">Blocked by</label>
          <input
            v-model="addBlockerSearchQuery"
            type="text"
            class="h-9 w-full text-sm px-3 rounded-md border border-border bg-background text-foreground focus:outline-none focus:ring-2 focus:ring-ring"
            placeholder="Search by ID or title..."
          />
          <div class="max-h-64 overflow-y-auto rounded-md border border-border">
            <button
              v-for="opt in addBlockerFilteredOptions"
              :key="opt.id"
              class="flex items-center gap-2 w-full text-left px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground transition-colors min-w-0"
              :class="{ 'bg-accent': addBlockerSelectedTarget === opt.id }"
              @click="addBlockerSelectedTarget = opt.id"
            >
              <span :class="['font-medium shrink-0', priorityTextColor(opt.priority)]">{{ opt.id }}</span>
              <span class="truncate text-muted-foreground flex-1 min-w-0">{{ opt.title }}</span>
              <StatusBadge :status="opt.status" class="shrink-0 scale-75 origin-right" />
            </button>
            <p v-if="!addBlockerFilteredOptions.length && addBlockerSearchQuery" class="px-3 py-2 text-sm text-muted-foreground">
              No matching issues found
            </p>
            <p v-if="!addBlockerFilteredOptions.length && !addBlockerSearchQuery" class="px-3 py-2 text-sm text-muted-foreground">
              Type to search for issues...
            </p>
          </div>
        </div>
      </div>
      <DialogFooter class="gap-3">
        <Button variant="outline" @click="isAddBlockerDialogOpen = false">Cancel</Button>
        <Button
          :disabled="!addBlockerSelectedTarget || isAddingBlocker"
          @click="handleAddBlocker"
        >
          <svg
            v-if="isAddingBlocker"
            class="animate-spin -ml-1 mr-2 h-4 w-4"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
          >
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          Add blocker
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>

  <!-- Add Relation Dialog -->
  <Dialog v-model:open="isAddRelDialogOpen">
    <DialogContent class="sm:max-w-lg overflow-hidden">
      <DialogHeader>
        <DialogTitle>Add a relation</DialogTitle>
        <DialogDescription>
          From <span class="font-mono text-sky-400">{{ addRelIssueId }}</span>
          <br />
          <span class="text-muted-foreground">{{ availableIssuesForDeps.find(i => i.id === addRelIssueId)?.title }}</span>
        </DialogDescription>
      </DialogHeader>
      <div class="space-y-4 py-2 overflow-hidden">
        <!-- Relation type selector -->
        <div class="space-y-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wide">Type</label>
          <select
            v-model="addRelSelectedType"
            class="h-9 w-full text-sm px-3 rounded-md border border-border bg-background text-foreground focus:outline-none focus:ring-2 focus:ring-ring"
          >
            <option v-for="rt in availableRelationTypes" :key="rt.value" :value="rt.value">{{ rt.label }}</option>
          </select>
        </div>
        <!-- Target issue search -->
        <div class="space-y-1.5 overflow-hidden">
          <div class="flex items-center justify-between">
            <label class="text-xs font-medium text-muted-foreground uppercase tracking-wide">Target issue</label>
            <button
              type="button"
              class="flex items-center gap-1 text-[11px] px-2 py-0.5 rounded-full border transition-colors select-none"
              :class="addRelFilterClosed
                ? 'border-sky-500/50 bg-sky-500/10 text-sky-400'
                : 'border-border text-muted-foreground hover:border-muted-foreground/50'"
              @click="addRelFilterClosed = !addRelFilterClosed"
            >
              <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 3H2l8 9.46V19l4 2v-8.54L22 3z" /></svg>
              Exclude closed
            </button>
          </div>
          <input
            v-model="addRelSearchQuery"
            type="text"
            class="h-9 w-full text-sm px-3 rounded-md border border-border bg-background text-foreground focus:outline-none focus:ring-2 focus:ring-ring"
            placeholder="Search by ID or title..."
          />
          <div class="max-h-64 overflow-y-auto rounded-md border border-border">
            <button
              v-for="opt in addRelFilteredOptions"
              :key="opt.id"
              class="flex items-center gap-2 w-full text-left px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground transition-colors min-w-0"
              :class="{ 'bg-accent': addRelSelectedTarget === opt.id }"
              @click="addRelSelectedTarget = opt.id"
            >
              <span :class="['font-medium shrink-0', priorityTextColor(opt.priority)]">{{ opt.id }}</span>
              <span class="truncate text-muted-foreground flex-1 min-w-0">{{ opt.title }}</span>
              <StatusBadge :status="opt.status" class="shrink-0 scale-75 origin-right" />
            </button>
            <p v-if="!addRelFilteredOptions.length && addRelSearchQuery" class="px-3 py-2 text-sm text-muted-foreground">
              No matching issues found
            </p>
            <p v-if="!addRelFilteredOptions.length && !addRelSearchQuery" class="px-3 py-2 text-sm text-muted-foreground">
              Type to search for issues...
            </p>
          </div>
        </div>
      </div>
      <DialogFooter class="gap-3">
        <Button variant="outline" @click="isAddRelDialogOpen = false">Cancel</Button>
        <Button
          :disabled="!addRelSelectedTarget || isAddingRel"
          @click="handleAddRelation"
        >
          <svg
            v-if="isAddingRel"
            class="animate-spin -ml-1 mr-2 h-4 w-4"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
          >
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          Add relation
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>

  <!-- Remove Relation Confirmation Dialog -->
  <ConfirmDialog
    v-model:open="isRemoveRelDialogOpen"
    title="Remove relation"
    confirm-text="Remove"
    cancel-text="Cancel"
    variant="destructive"
    :is-loading="isRemovingRel"
    @confirm="handleRemoveRelation"
  >
    <template #description>
      <p class="text-sm text-muted-foreground">
        Are you sure you want to remove this relation?
      </p>
      <div v-if="pendingRelRemoval" class="mt-2 space-y-2">
        <div>
          <p class="text-xs text-muted-foreground uppercase tracking-wide">Issue</p>
          <p class="text-sm font-mono text-sky-400">{{ pendingRelRemoval.issueId }}</p>
          <p class="text-sm text-muted-foreground">{{ availableIssuesForDeps.find(i => i.id === pendingRelRemoval!.issueId)?.title }}</p>
        </div>
        <div>
          <p class="text-xs text-muted-foreground uppercase tracking-wide">Related to</p>
          <p class="text-sm font-mono text-sky-400">{{ pendingRelRemoval.targetId }}</p>
          <p class="text-sm text-muted-foreground">{{ availableIssuesForDeps.find(i => i.id === pendingRelRemoval!.targetId)?.title }}</p>
        </div>
      </div>
    </template>
  </ConfirmDialog>

  <!-- Image Preview Dialog -->
  <ImagePreviewDialog
    v-model:open="imagePreview.isOpen.value"
    :image-src="imagePreview.imageSrc.value"
    :image-alt="imagePreview.imageAlt.value"
    :has-multiple-images="imagePreview.hasMultipleImages.value"
    :can-go-next="imagePreview.canGoNext.value"
    :can-go-prev="imagePreview.canGoPrev.value"
    :image-counter="imagePreview.imageCounter.value"
    @next="imagePreview.goNext"
    @prev="imagePreview.goPrev"
  />

  <!-- Markdown Preview Dialog -->
  <MarkdownPreviewDialog
    v-model:open="markdownPreview.isOpen.value"
    :markdown-content="markdownPreview.markdownContent.value"
    :markdown-title="markdownPreview.markdownTitle.value"
    :is-loading="markdownPreview.isLoading.value"
    :has-multiple-files="markdownPreview.hasMultipleFiles.value"
    :can-go-next="markdownPreview.canGoNext.value"
    :can-go-prev="markdownPreview.canGoPrev.value"
    :file-counter="markdownPreview.fileCounter.value"
    :is-edit-mode="markdownPreview.isEditMode.value"
    :edited-content="markdownPreview.editedContent.value"
    :is-saving="markdownPreview.isSaving.value"
    @next="markdownPreview.goNext"
    @prev="markdownPreview.goPrev"
    @toggle-edit="markdownPreview.toggleEdit"
    @save="markdownPreview.requestSave"
    @cancel-edit="markdownPreview.cancelEdit"
    @update:edited-content="markdownPreview.editedContent.value = $event"
  />

  <!-- Markdown Save Confirmation -->
  <ConfirmDialog
    v-model:open="markdownPreview.showSaveConfirm.value"
    title="Save changes"
    description="Save the changes to this markdown file?"
    confirm-text="Save"
    cancel-text="Cancel"
    :is-loading="markdownPreview.isSaving.value"
    @confirm="markdownPreview.confirmSave"
    @cancel="markdownPreview.cancelSave"
  />
</template>
