import type { Issue, ChildIssue } from '~/types/issue'
import { bdAvailableRelationTypes, checkBdCompatibility } from '~/utils/bd-api'

// Singleton state — shared across all callers

// Edit/create mode
const isEditMode = ref(false)
const isCreatingNew = ref(false)
const multiSelectMode = ref(false)
const selectedIds = ref<string[]>([])

// Delete confirmation dialog
const isDeleteDialogOpen = ref(false)
const deleteTargetTitles = ref<string[]>([])
const isDeleting = ref(false)

// Epic delete dialog state (for issues with children)
const isEpicDeleteDialogOpen = ref(false)
const epicToDelete = ref<Issue | null>(null)
const epicChildren = ref<ChildIssue[]>([])
const isDeletingEpic = ref(false)
const remainingDeleteIds = ref<string[]>([])

// Close confirmation dialog
const isCloseDialogOpen = ref(false)
const isClosing = ref(false)

// Detach image confirmation dialog
const isDetachDialogOpen = ref(false)
const detachImagePath = ref<string | null>(null)
const isDetaching = ref(false)

// Remove dependency confirmation dialog
const isRemoveDepDialogOpen = ref(false)
const pendingDepRemoval = ref<{ issueId: string; blockerId: string } | null>(null)
const isRemovingDep = ref(false)

// Relation types (fetched once on mount)
const availableRelationTypes = ref<Array<{ value: string; label: string }>>([])
// bd >= 0.50: parent-child is structural via dot notation (not explicit field)
const bdDotNotationParent = ref(false)

// Add relation dialog
const isAddRelDialogOpen = ref(false)
const addRelIssueId = ref('')
const addRelSelectedType = ref('')
const addRelSearchQuery = ref('')
const addRelSelectedTarget = ref('')
const addRelFilterClosed = ref(true)
const isAddingRel = ref(false)

// Add blocker dialog
const isAddBlockerDialogOpen = ref(false)
const addBlockerIssueId = ref('')
const addBlockerSearchQuery = ref('')
const addBlockerSelectedTarget = ref('')
const isAddingBlocker = ref(false)

// Remove relation confirmation dialog
const isRemoveRelDialogOpen = ref(false)
const pendingRelRemoval = ref<{ issueId: string; targetId: string } | null>(null)
const isRemovingRel = ref(false)

export function useIssueDialogs() {
  const { issues, filteredIssues, selectedIssue, isUpdating, error: issueError, fetchIssues, fetchIssue, updateIssue, closeIssue, deleteIssue, addDependency, removeDependency, addRelation, removeRelation, selectIssue } = useIssues()
  const { beadsPath } = useBeadsPath()
  const { notify, success: notifySuccess, error: notifyError } = useNotification()
  const { stats, fetchStats } = useDashboard()

  // Computed: available issues for dependency/relation dialogs
  const availableIssuesForDeps = computed(() =>
    issues.value.map(i => ({ id: i.id, title: i.title, priority: i.priority, status: i.status })),
  )

  const addRelFilteredOptions = computed(() => {
    if (!addRelIssueId.value) return []
    const issue = issues.value.find(i => i.id === addRelIssueId.value)
    const existingRelated = new Set([
      addRelIssueId.value,
      ...(issue?.relations?.map(r => r.id) || []),
    ])
    const query = addRelSearchQuery.value.toLowerCase()
    const showClosed = query || !addRelFilterClosed.value
    return availableIssuesForDeps.value
      .filter(i => !existingRelated.has(i.id))
      .filter(i => showClosed || i.status !== 'closed')
      .filter(i => !query || i.id.toLowerCase().includes(query) || i.title.toLowerCase().includes(query))
      .slice(0, 15)
  })

  const addBlockerFilteredOptions = computed(() => {
    if (!addBlockerIssueId.value) return []
    const existing = new Set([
      addBlockerIssueId.value,
      ...(issues.value.find(i => i.id === addBlockerIssueId.value)?.blockedBy || []),
    ])
    const query = addBlockerSearchQuery.value.toLowerCase()
    return availableIssuesForDeps.value
      .filter(i => !existing.has(i.id) && i.status !== 'closed')
      .filter(i => !query || i.id.toLowerCase().includes(query) || i.title.toLowerCase().includes(query))
      .slice(0, 15)
  })

  // Toggle multi-select mode
  const toggleMultiSelect = () => {
    multiSelectMode.value = !multiSelectMode.value
    if (!multiSelectMode.value) {
      selectedIds.value = []
    }
  }

  // Priority text color helper
  const priorityTextColor = (priority?: string) => {
    if (!priority) return 'text-sky-400'
    const colors: Record<string, string> = {
      p0: 'text-[#ef4444]',
      p1: 'text-[#ef4444]',
      p2: 'text-[#f59e0b]',
      p3: 'text-[#b8860b]',
      p4: 'text-[#6b7280]',
    }
    return colors[priority] || 'text-sky-400'
  }

  // Close issue
  const handleCloseIssue = () => {
    if (selectedIssue.value) {
      isCloseDialogOpen.value = true
    }
  }

  const confirmClose = async () => {
    if (!selectedIssue.value) return
    const issueId = selectedIssue.value.id
    const issueTitle = selectedIssue.value.title
    isClosing.value = true
    try {
      const result = await closeIssue(issueId)
      await fetchStats(issues.value)

      // br --suggest-next: enrich the close notification with unblocked issues
      let closeDesc = issueTitle
      let hasSuggestions = false
      if (result && typeof result === 'object') {
        const data = result as Record<string, unknown>
        const suggested = data.unblocked as Array<{ id: string; title?: string }> | undefined
        if (suggested?.length) {
          hasSuggestions = true
          const unblockedList = suggested.map(s => s.id).join(', ')
          const unblockedMsg = suggested.length === 1
            ? `Unblocked: ${unblockedList}`
            : `${suggested.length} unblocked: ${unblockedList}`
          closeDesc = closeDesc ? `${closeDesc} — ${unblockedMsg}` : unblockedMsg
        }
      }
      // Longer duration when showing unblocked issues so user has time to read
      notifySuccess(`Issue ${issueId} closed`, closeDesc, hasSuggestions ? 6000 : undefined)
    } catch {
      notifyError(`Failed to close ${issueId}`, issueTitle)
    } finally {
      isClosing.value = false
      isCloseDialogOpen.value = false
    }
  }

  // Reopen issue
  const handleReopenIssue = async () => {
    if (!selectedIssue.value) return
    const issueId = selectedIssue.value.id
    const issueTitle = selectedIssue.value.title
    try {
      await updateIssue(issueId, { status: 'open' })
      await fetchStats(issues.value)
      notifySuccess(`Issue ${issueId} reopened`, issueTitle)
    } catch {
      notifyError(`Failed to reopen ${issueId}`, issueTitle)
    }
  }

  // Attach files — copy to filesystem, no external_ref modification
  const handleAttachImage = async (paths: string | string[]) => {
    if (!selectedIssue.value) return

    const filePaths = Array.isArray(paths) ? paths : [paths]
    const { invoke } = await import('@tauri-apps/api/core')

    for (const sourcePath of filePaths) {
      try {
        await invoke<string>('copy_file_to_attachments', {
          projectPath: beadsPath.value,
          sourcePath,
          issueId: selectedIssue.value.id,
        })
      } catch (error) {
        console.error('Failed to copy file:', sourcePath, error)
      }
    }

    // Clear attachment cache so IssuePreview reloads from filesystem
    const { clearCache } = useAttachments()
    clearCache(selectedIssue.value.id)

    // Refresh issue to show the new attachments
    await fetchIssue(selectedIssue.value.id)
  }

  // Detach image — delete file from filesystem, no external_ref modification
  const confirmDetachImage = (filename: string) => {
    detachImagePath.value = filename
    isDetachDialogOpen.value = true
  }

  const handleDetachImage = async () => {
    if (!selectedIssue.value || !detachImagePath.value) return

    isDetaching.value = true
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('delete_attachment', {
        projectPath: beadsPath.value,
        issueId: selectedIssue.value.id,
        filename: detachImagePath.value,
      })

      // Clear attachment cache so IssuePreview reloads from filesystem
      const { clearCache } = useAttachments()
      clearCache(selectedIssue.value.id)

      // Refresh issue to update the attachments
      await fetchIssue(selectedIssue.value.id)
    } catch (error) {
      console.error('Failed to delete attachment:', error)
    } finally {
      isDetaching.value = false
      isDetachDialogOpen.value = false
      detachImagePath.value = null
    }
  }

  // Delete issue
  const handleDeleteIssue = async () => {
    let idsToDelete: string[] = []
    if (multiSelectMode.value && selectedIds.value.length > 0) {
      idsToDelete = [...selectedIds.value]
    } else if (selectedIssue.value) {
      idsToDelete = [selectedIssue.value.id]
    } else {
      return
    }

    // For each ID, fetch full issue to check for children
    const issuesToCheck: Issue[] = []
    for (const id of idsToDelete) {
      const fullIssue = await fetchIssue(id)
      if (fullIssue) {
        issuesToCheck.push(fullIssue)
      } else {
        const issueFromList = filteredIssues.value.find(i => i.id === id)
        if (issueFromList) {
          issuesToCheck.push(issueFromList)
        }
      }
    }

    // Check if any selected issue has children
    const issuesWithChildren = issuesToCheck.filter(
      (issue): issue is Issue => !!issue.children?.length
    )

    const firstEpic = issuesWithChildren[0]
    if (firstEpic) {
      epicToDelete.value = firstEpic
      epicChildren.value = firstEpic.children || []
      remainingDeleteIds.value = idsToDelete.filter(id => id !== firstEpic.id)
      isEpicDeleteDialogOpen.value = true
    } else {
      deleteTargetTitles.value = issuesToCheck
        .map(issue => issue.title)
        .filter((t): t is string => !!t)
      isDeleteDialogOpen.value = true
    }
  }

  const confirmDelete = async () => {
    isDeleting.value = true
    try {
      if (multiSelectMode.value && selectedIds.value.length > 0) {
        const successfullyDeleted: string[] = []
        for (const id of selectedIds.value) {
          const success = await deleteIssue(id)
          if (!success) {
            notifyError('Failed to delete issue', issueError.value || `Could not delete ${id}`)
          } else {
            successfullyDeleted.push(id)
          }
        }
        selectedIds.value = selectedIds.value.filter(id => !successfullyDeleted.includes(id))
        if (successfullyDeleted.length > 0) {
          notifySuccess(`${successfullyDeleted.length} issue(s) deleted`)
        }
      } else if (selectedIssue.value) {
        const issueId = selectedIssue.value.id
        const issueTitle = selectedIssue.value.title
        const success = await deleteIssue(issueId)
        if (!success) {
          notifyError('Failed to delete issue', issueError.value || 'Could not delete the issue')
        } else {
          isEditMode.value = false
          isCreatingNew.value = false
          notifySuccess(`Issue ${issueId} deleted`, issueTitle)
        }
      }
      await fetchIssues()
      await fetchStats(issues.value)
    } finally {
      isDeleting.value = false
      isDeleteDialogOpen.value = false
    }
  }

  const confirmEpicDelete = async (mode: 'delete-all' | 'detach') => {
    if (!epicToDelete.value) return
    isDeletingEpic.value = true

    try {
      if (mode === 'detach') {
        for (const child of epicChildren.value) {
          await updateIssue(child.id, { parent: '' })
        }
      } else {
        for (const child of epicChildren.value) {
          const success = await deleteIssue(child.id)
          if (!success) {
            notifyError('Failed to delete child issue', issueError.value || `Could not delete ${child.id}`)
          }
        }
      }
      const epicId = epicToDelete.value.id
      const epicTitle = epicToDelete.value.title
      const epicSuccess = await deleteIssue(epicId)
      if (!epicSuccess) {
        notifyError('Failed to delete issue', issueError.value || `Could not delete ${epicId}`)
      } else {
        notifySuccess(`Epic ${epicId} deleted`, epicTitle)
      }

      if (epicSuccess && selectedIssue.value?.id === epicToDelete.value.id) {
        isEditMode.value = false
        isCreatingNew.value = false
      }

      if (epicSuccess && multiSelectMode.value) {
        selectedIds.value = selectedIds.value.filter(id => id !== epicToDelete.value?.id)
      }

      // Check if there are more issues with children to process
      if (remainingDeleteIds.value.length > 0) {
        const remainingIssues = remainingDeleteIds.value
          .map(id => issues.value.find(i => i.id === id))
          .filter((issue): issue is Issue => !!issue)

        const nextIssueWithChildren = remainingIssues.find(issue => issue.children?.length)

        if (nextIssueWithChildren) {
          epicToDelete.value = nextIssueWithChildren
          epicChildren.value = nextIssueWithChildren.children || []
          remainingDeleteIds.value = remainingDeleteIds.value.filter(id => id !== nextIssueWithChildren.id)
          isDeletingEpic.value = false
          return
        } else {
          const successfullyDeleted: string[] = []
          for (const id of remainingDeleteIds.value) {
            const success = await deleteIssue(id)
            if (!success) {
              notifyError('Failed to delete issue', issueError.value || `Could not delete ${id}`)
            } else {
              successfullyDeleted.push(id)
            }
          }
          selectedIds.value = selectedIds.value.filter(id => !successfullyDeleted.includes(id))
        }
      }

      await fetchIssues()
      await fetchStats(issues.value)
    } finally {
      isDeletingEpic.value = false
      isEpicDeleteDialogOpen.value = false
      epicToDelete.value = null
      epicChildren.value = []
      remainingDeleteIds.value = []
    }
  }

  // Add blocker
  const openAddBlockerDialog = (issueId: string) => {
    addBlockerIssueId.value = issueId
    addBlockerSearchQuery.value = ''
    addBlockerSelectedTarget.value = ''
    isAddBlockerDialogOpen.value = true
  }

  const handleAddBlocker = async () => {
    if (!addBlockerIssueId.value || !addBlockerSelectedTarget.value) return
    isAddingBlocker.value = true
    try {
      await addDependency(addBlockerIssueId.value, addBlockerSelectedTarget.value)
      notifySuccess('Dependency added', `${addBlockerIssueId.value} is now blocked by ${addBlockerSelectedTarget.value}`)
      isAddBlockerDialogOpen.value = false
    } catch {
      notifyError('Failed to add dependency')
    } finally {
      isAddingBlocker.value = false
    }
  }

  // Remove dependency
  const confirmRemoveDependency = (issueId: string, blockerId: string) => {
    pendingDepRemoval.value = { issueId, blockerId }
    isRemoveDepDialogOpen.value = true
  }

  const handleRemoveDependency = async () => {
    if (!pendingDepRemoval.value) return
    isRemovingDep.value = true
    try {
      await removeDependency(pendingDepRemoval.value.issueId, pendingDepRemoval.value.blockerId)
      notifySuccess('Dependency removed')
    } catch {
      notifyError('Failed to remove dependency')
    } finally {
      isRemovingDep.value = false
      isRemoveDepDialogOpen.value = false
      pendingDepRemoval.value = null
    }
  }

  // Add relation
  const openAddRelationDialog = (issueId: string) => {
    addRelIssueId.value = issueId
    addRelSelectedType.value = availableRelationTypes.value[0]?.value || 'relates-to'
    addRelSearchQuery.value = ''
    addRelSelectedTarget.value = ''
    addRelFilterClosed.value = true
    isAddRelDialogOpen.value = true
  }

  const handleAddRelation = async () => {
    if (!addRelIssueId.value || !addRelSelectedTarget.value || !addRelSelectedType.value) return
    isAddingRel.value = true
    try {
      await addRelation(addRelIssueId.value, addRelSelectedTarget.value, addRelSelectedType.value)
      const typeLabel = availableRelationTypes.value.find(t => t.value === addRelSelectedType.value)?.label || addRelSelectedType.value
      notifySuccess('Relation added', `${addRelIssueId.value} → ${typeLabel} → ${addRelSelectedTarget.value}`)
      isAddRelDialogOpen.value = false
    } catch {
      notifyError('Failed to add relation')
    } finally {
      isAddingRel.value = false
    }
  }

  // Remove relation
  const confirmRemoveRelation = (issueId: string, targetId: string) => {
    pendingRelRemoval.value = { issueId, targetId }
    isRemoveRelDialogOpen.value = true
  }

  const handleRemoveRelation = async () => {
    if (!pendingRelRemoval.value) return
    isRemovingRel.value = true
    try {
      await removeRelation(pendingRelRemoval.value.issueId, pendingRelRemoval.value.targetId)
      notifySuccess('Relation removed')
    } catch {
      notifyError('Failed to remove relation')
    } finally {
      isRemovingRel.value = false
      isRemoveRelDialogOpen.value = false
      pendingRelRemoval.value = null
    }
  }

  // Init relation types (called from onMounted)
  const initRelationTypes = () => {
    bdAvailableRelationTypes().then(types => { availableRelationTypes.value = types }).catch(() => {})
    checkBdCompatibility().then(info => { bdDotNotationParent.value = info.usesDoltBackend }).catch(() => {})
  }

  return {
    // Edit/create mode
    isEditMode,
    isCreatingNew,
    multiSelectMode,
    selectedIds,
    toggleMultiSelect,

    // Delete dialog
    isDeleteDialogOpen,
    deleteTargetTitles,
    isDeleting,
    handleDeleteIssue,
    confirmDelete,

    // Epic delete dialog
    isEpicDeleteDialogOpen,
    epicToDelete,
    epicChildren,
    isDeletingEpic,
    confirmEpicDelete,

    // Close dialog
    isCloseDialogOpen,
    isClosing,
    handleCloseIssue,
    confirmClose,

    // Reopen
    handleReopenIssue,

    // Detach dialog
    isDetachDialogOpen,
    detachImagePath,
    isDetaching,
    handleAttachImage,
    confirmDetachImage,
    handleDetachImage,

    // Remove dependency dialog
    isRemoveDepDialogOpen,
    pendingDepRemoval,
    isRemovingDep,
    confirmRemoveDependency,
    handleRemoveDependency,

    // Relation types
    availableRelationTypes,
    bdDotNotationParent,

    // Add blocker dialog
    isAddBlockerDialogOpen,
    addBlockerIssueId,
    addBlockerSearchQuery,
    addBlockerSelectedTarget,
    addBlockerFilteredOptions,
    isAddingBlocker,
    openAddBlockerDialog,
    handleAddBlocker,

    // Add relation dialog
    isAddRelDialogOpen,
    addRelIssueId,
    addRelSelectedType,
    addRelSearchQuery,
    addRelSelectedTarget,
    addRelFilterClosed,
    addRelFilteredOptions,
    isAddingRel,
    openAddRelationDialog,
    handleAddRelation,

    // Remove relation dialog
    isRemoveRelDialogOpen,
    pendingRelRemoval,
    isRemovingRel,
    confirmRemoveRelation,
    handleRemoveRelation,

    // Helpers
    priorityTextColor,
    availableIssuesForDeps,
    initRelationTypes,
  }
}
