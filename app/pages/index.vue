<script setup lang="ts">
import type { Issue, UpdateIssuePayload } from '~/types/issue'

// Layout components
import AppHeader from '~/components/layout/AppHeader.vue'
import UpdateIndicator from '~/components/layout/UpdateIndicator.vue'
import DebugPanel from '~/components/layout/DebugPanel.vue'
import DialogsLayer from '~/components/layout/DialogsLayer.vue'

// Dashboard components
import PathSelector from '~/components/dashboard/PathSelector.vue'
import FolderPicker from '~/components/dashboard/FolderPicker.vue'
import KpiCard from '~/components/dashboard/KpiCard.vue'
import DashboardContent from '~/components/dashboard/DashboardContent.vue'
import OnboardingCard from '~/components/dashboard/OnboardingCard.vue'
import PrerequisitesCard from '~/components/dashboard/PrerequisitesCard.vue'


// Details components
import IssueDetailHeader from '~/components/details/IssueDetailHeader.vue'
import IssuePreview from '~/components/details/IssuePreview.vue'
import IssueForm from '~/components/details/IssueForm.vue'
import CommentSection from '~/components/details/CommentSection.vue'

// Issues components
import IssueListPanel from '~/components/issues/IssueListPanel.vue'

// UI components
import { Button } from '~/components/ui/button'
import { ScrollArea } from '~/components/ui/scroll-area'
import { ConfirmDialog } from '~/components/ui/confirm-dialog'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '~/components/ui/tooltip'

// Composables
const { filters, toggleStatus, toggleType, togglePriority, toggleAssignee, clearFilters, setStatusFilter, setSearch, toggleLabelFilter } = useFilters()
const { columns, toggleColumn, setColumns, resetColumns } = useColumnConfig()
const { beadsPath, hasStoredPath } = useBeadsPath()
const { success: notifySuccess, error: notifyError } = useNotification()
const { isBr, init: initCliClient } = useCliClient()
const { projects } = useProjects()
const {
  issues,
  filteredIssues,
  paginatedIssues,
  groupedIssues,
  selectedIssue,
  isLoading,
  isUpdating,
  // Pagination
  hasMore,
  loadMore,
  sortField,
  sortDirection,
  setSort,
  // Epic expand
  expandEpic,
  // Actions
  fetchIssues,
  fetchPollData,
  fetchIssue,
  createIssue,
  updateIssue,
  selectIssue,
  addComment,
  clearIssues,
  newlyAddedIds,
} = useIssues()
const { stats, readyIssues, fetchStats, updateFromPollData, clearStats } = useDashboard()
const { check: checkForUpdates, startPeriodicCheck, stopPeriodicCheck } = useUpdateChecker()
const { showDebugPanel, showSettingsDialog } = useAppMenu()
const { needsRepair, affectedProject, isRepairing, repairError, repairProgress, repair: repairDatabase, repairAll, dismiss: dismissRepair } = useRepairDatabase()
const { needsMigration, affectedProject: migrateAffectedProject, isMigrating, migrateError, migrate: migrateToDolt, checkProject: checkMigrationNeeded, dismiss: dismissMigration } = useMigrateToDolt()
const { needsMigration: needsRefsMigration, refCount: refsRefCount, isMigrating: isRefsMigrating, migrateError: refsMigrateError, checkProject: checkRefsMigration, migrate: migrateRefs, dismiss: dismissRefsMigration } = useMigrateRefs()

// Sidebar resize
const { isLeftSidebarOpen, isRightSidebarOpen, leftSidebarWidth, rightSidebarWidth, isResizing, startResizeLeft, startResizeRight } = useSidebarResize()

// Close right sidebar on init if no issue selected
if (import.meta.client && !selectedIssue.value) {
  isRightSidebarOpen.value = false
}

// Probe enabled toggle (dev-only — hidden in production until probe is a public feature)
const isDev = import.meta.dev
const probeEnabled = isDev ? useLocalStorage('beads:probeEnabled', false) : ref(false)

// React to probe toggle: launch/register or unregister
watch(probeEnabled, async (enabled) => {
  if (enabled) {
    await launchProbeIfNeeded()
    ensureProbeRegistration(beadsPath.value)
  } else {
    probeUnregisterProject(beadsPath.value)
  }
})

// Current project name for header subtitle
const currentProjectName = computed(() => {
  const project = projects.value.find(f => f.path === beadsPath.value)
  return project?.name
})

// Whether the current project is exposed to the probe (read from PathSelector)
const isCurrentProjectExposed = computed(() => {
  return pathSelectorRef.value?.isCurrentExposed || mobilePathSelectorRef.value?.isCurrentExposed || false
})

// Show onboarding when no project is selected (no stored path and no projects)
const showOnboarding = computed(() => {
  return projects.value.length === 0 && !hasStoredPath.value
})

// Refs to PathSelector to open folder picker (desktop and mobile)
const pathSelectorRef = ref<InstanceType<typeof PathSelector> | null>(null)
const mobilePathSelectorRef = ref<InstanceType<typeof PathSelector> | null>(null)

// Onboarding folder picker state
const isOnboardingPickerOpen = ref(false)
const { setPath } = useBeadsPath()

const openFolderPicker = () => {
  // Try PathSelector refs first, fallback to onboarding picker
  const ref = pathSelectorRef.value || mobilePathSelectorRef.value
  if (ref) {
    ref.isPickerOpen = true
  } else {
    isOnboardingPickerOpen.value = true
  }
}

const handleOnboardingFolderSelect = async (path: string) => {
  setPath(path)
  await fetchIssues()
  fetchStats(issues.value)
}

// Edit context for header
const editContext = computed(() => {
  if (isCreatingNew.value) {
    return 'New issue'
  }
  if (isEditMode.value && selectedIssue.value) {
    return 'Editing'
  }
  return undefined
})

const editId = computed(() => {
  if (isEditMode.value && selectedIssue.value && !isCreatingNew.value) {
    return selectedIssue.value.id
  }
  return undefined
})

// Mobile state
const isMobileView = ref(false)
const mobilePanel = ref<'dashboard' | 'issues' | 'details'>('issues')

// Check viewport size
const checkViewport = () => {
  if (import.meta.client) {
    isMobileView.value = window.innerWidth < 1024
  }
}

// Sync status composable (for auto-sync indicator and error dialog)
const { showErrorDialog: showSyncErrorDialog, lastSyncError, closeErrorDialog: closeSyncErrorDialog } = useSyncStatus()

// Change detection: native file watcher via Tauri events
const { active: changeDetectionActive, startListening, stopListening, notifySelfWrite } = useChangeDetection({
  onChanged: async () => {
    await pollForChanges()
  },
})

// Polling for external changes — optimized with 5 layers:
// 1. Native file watcher (0 CPU when idle, instant detection)
// 2. Sync cooldown (Rust backend skips redundant syncs within 10s)
// 3. Filesystem mtime check as fallback (zero bd processes if nothing changed)
// 4. Batched poll command (1 IPC call instead of 3 when changes detected)
// 5. Adaptive intervals (30s safety net when watcher active, 5s/1s fallback without watcher)
const isSyncing = ref(false)
let skipMtimeCheck = false // Set by watcher/fast check to avoid redundant bdCheckChanged in pollFn

// Fast change detection (cheap mtime stat, ~0ms) — runs every 1s when active
const checkMtimeChanged = async (): Promise<boolean> => {
  if (isLoading.value || isUpdating.value || showOnboarding.value || !beadsPath.value || projects.value.length === 0) {
    return false
  }
  const path = beadsPath.value && beadsPath.value !== '.' ? beadsPath.value : undefined
  const changed = await bdCheckChanged(path)
  if (changed) skipMtimeCheck = true // pollFn can skip the mtime check — we already consumed it
  return changed
}

const pollForChanges = async () => {
  // Don't poll if no active project
  if (isLoading.value || isUpdating.value || showOnboarding.value || !beadsPath.value || projects.value.length === 0) {
    return
  }

  try {
    isSyncing.value = true

    const path = beadsPath.value && beadsPath.value !== '.' ? beadsPath.value : undefined

    // Layer 2: Check filesystem mtime first — skip if fast check already detected change
    if (!skipMtimeCheck) {
      const changed = await bdCheckChanged(path)
      if (!changed) {
        // Nothing changed on disk — skip entire poll cycle
        return
      }
    }
    skipMtimeCheck = false

    // Layer 3: Changes detected — use batched command (1 IPC instead of 3)
    const readyData = await fetchPollData()

    // Update dashboard from pre-fetched data (no extra API call)
    if (readyData) {
      updateFromPollData(issues.value, readyData)
    }

    // Snapshot mtime AFTER all operations (including epic bd_show calls in fetchPollData)
    // so the next check ignores changes caused by our own poll cycle
    await bdCheckChanged(path)

    // Tell change detection backend to ignore self-triggered events
    notifySelfWrite()
  } catch {
    // Ignore polling errors
  } finally {
    isSyncing.value = false
  }
}

// Adaptive polling with fast mtime detection (degrades gracefully if watcher unavailable)
const { start: startPolling, stop: stopPolling } = useAdaptivePolling(pollForChanges, {
  checkFn: checkMtimeChanged,
  watcherActive: changeDetectionActive,
})

onMounted(async () => {
  checkViewport()
  if (import.meta.client) {
    window.addEventListener('resize', checkViewport)

    // Detect CLI client (br vs bd) for feature gating
    await initCliClient()

    // Check for updates after initial load + start periodic check (hourly)
    // (these don't call bd CLI, safe to run before migration check)
    checkForUpdates()
    startPeriodicCheck()
  }

  // Only fetch data if not in onboarding mode
  if (!showOnboarding.value) {
    // Check migration BEFORE any bd command — bd >= 0.52 auto-migrates on any
    // bd call (like `bd list`, `bd mtime`), which would bypass our migration modal
    // that preserves labels, deps, comments, and attachments.
    const migrationNeeded = await checkMigrationNeeded()
    if (!migrationNeeded) {
      if (import.meta.client) {
        // Start change detection — native file watcher
        if (beadsPath.value) {
          startListening(beadsPath.value)
        }

        // Start adaptive polling (handles visibility, focus, idle internally)
        // When change detection is active, polling uses 30s safety-net instead of 1s mtime checks
        startPolling()

        // Fetch available relation types + detect bd >= 0.50 for dot-notation parent-child
        initRelationTypes()
      }
      // Auto-launch probe if enabled (must complete before fetch)
      await launchProbeIfNeeded()
      // Auto-register with probe if enabled (fire-and-forget, never blocks UI)
      ensureProbeRegistration(beadsPath.value)

      // Check attachment refs migration (may have been auto-migrated before sync)
      const migrationResult = await checkRefsMigration()
      if (migrationResult === 'just_migrated') {
        notifySuccess('Attachments migrated', 'Attachment references and folders have been updated to the new format.')
      } else if (migrationResult) {
        await migrateRefs()
        notifySuccess('Attachments migrated', 'Attachment references and folders have been updated to the new format.')
      }

      // Sequential: bd commands can't run concurrently (Dolt SIGSEGV on parallel access)
      fetchIssues().then(() => fetchStats(issues.value))
    }
  }

  // Track current path for handlePathChange unregistration
  previousBeadsPath = beadsPath.value
})

onUnmounted(() => {
  if (import.meta.client) {
    window.removeEventListener('resize', checkViewport)
    stopListening()
    stopPolling()
    stopPeriodicCheck()
    // Auto-unregister from probe (fire-and-forget)
    probeUnregisterProject(beadsPath.value)
  }
})

// Issue dialogs composable (dialog-only state lives in DialogsLayer via singleton)
const {
  isEditMode, isCreatingNew, multiSelectMode, selectedIds, toggleMultiSelect,
  handleDeleteIssue, handleCloseIssue, handleReopenIssue,
  handleAttachImage, confirmDetachImage,
  confirmRemoveDependency, openAddBlockerDialog, openAddRelationDialog, confirmRemoveRelation,
  bdDotNotationParent, availableIssuesForDeps, initRelationTypes,
} = useIssueDialogs()

const leftSidebarStateBeforeEdit = ref<boolean | null>(null)

// Watch edit mode to manage left sidebar state
watch(
  () => isEditMode.value || isCreatingNew.value,
  (inEditMode) => {
    if (inEditMode) {
      // Save current state and close
      leftSidebarStateBeforeEdit.value = isLeftSidebarOpen.value
      isLeftSidebarOpen.value = false
    } else if (leftSidebarStateBeforeEdit.value !== null) {
      // Restore previous state
      isLeftSidebarOpen.value = leftSidebarStateBeforeEdit.value
      leftSidebarStateBeforeEdit.value = null
    }
  }
)

// Close and clear panel when issue transitions to closed (not when selecting an already closed issue)
watch(
  () => selectedIssue.value?.status,
  (newStatus, oldStatus) => {
    if (newStatus === 'closed' && oldStatus && oldStatus !== 'closed') {
      isEditMode.value = false
      selectIssue(null)
      isRightSidebarOpen.value = false
    }
  }
)

// Handlers
const handleRefresh = () => {
  // Full page reload to reset all state (like reopening the app)
  window.location.reload()
}

const handleRepair = async () => {
  const success = await repairDatabase()
  if (success) {
    notifySuccess('Database repaired', 'Your issues have been restored successfully.')
    // Reload data after repair
    await fetchIssues()
    await fetchStats(issues.value)
  }
}

const handleRepairAll = async () => {
  const projectPaths = projects.value.map(f => f.path)
  const results = await repairAll(projectPaths)

  if (results.failed === 0) {
    notifySuccess('All databases repaired', `${results.success} project(s) repaired successfully.`)
    // Reload data after repair
    await fetchIssues()
    await fetchStats(issues.value)
  } else {
    notifyError('Some repairs failed', `${results.success} succeeded, ${results.failed} failed.`)
  }
}

const handleMigrateToDolt = async () => {
  const success = await migrateToDolt()
  if (success) {
    notifySuccess('Migration complete', 'Project has been migrated to the Dolt backend.')

    // Start change detection + polling that were deferred during migration
    if (import.meta.client) {
      if (beadsPath.value) {
        await startListening(beadsPath.value)
      }
      startPolling()
      initRelationTypes()
    }

    // Reload data after migration
    await fetchIssues()
    await fetchStats(issues.value)
  }
}

const handleMigrateRefs = async () => {
  const success = await migrateRefs()
  if (success) {
    notifySuccess('Attachments updated', 'File references have been updated for br CLI compatibility.')
    // Reload data after migration
    await fetchIssues()
    await fetchStats(issues.value)
  }
}

let previousBeadsPath: string | undefined
let pathChangeGeneration = 0  // Guard against concurrent handlePathChange calls

const handlePathChange = async () => {
  // Increment generation — any in-flight handlePathChange with an older generation will bail out
  const thisGeneration = ++pathChangeGeneration

  // Show loading indicator immediately
  isLoading.value = true

  // Capture the old path before it changes (setPath was already called in PathSelector)
  const oldPath = previousBeadsPath
  previousBeadsPath = beadsPath.value

  // Unregister old project from probe (fire-and-forget)
  if (oldPath) probeUnregisterProject(oldPath)

  selectIssue(null)
  isEditMode.value = false
  isCreatingNew.value = false
  clearIssues()  // Reset issue list so new-issue detection doesn't flash all rows
  clearStats()   // Reset stats so previous project's ready work doesn't persist

  // Stop polling + change detection during project switch to prevent:
  // 1. Concurrent bd calls from old project's poll cycle
  // 2. Change detection events triggering stale refreshes
  stopPolling()
  await stopListening()

  // Bail out if another handlePathChange was triggered while we awaited
  if (thisGeneration !== pathChangeGeneration) {
return
  }

  try {
    // Pre-flight checks in parallel: cleanup stale locks + migration check + mtime reset
const [, , migrationNeeded] = await Promise.all([
      bdCleanupStaleLocks(beadsPath.value),
      bdResetMtime(),
      checkMigrationNeeded(),
    ])

    if (thisGeneration !== pathChangeGeneration) {
return
    }

if (!migrationNeeded) {
      // Register new project with probe before fetching (probe needs to know the project)
      await ensureProbeRegistration(beadsPath.value)

      if (thisGeneration !== pathChangeGeneration) {
return
      }

      // Check attachment refs migration (may have been auto-migrated before sync)
      const migrationResult2 = await checkRefsMigration()
      if (migrationResult2 === 'just_migrated') {
        notifySuccess('Attachments migrated', 'Attachment references and folders have been updated to the new format.')
      } else if (migrationResult2) {
        await migrateRefs()
        notifySuccess('Attachments migrated', 'Attachment references and folders have been updated to the new format.')
      }

      // IMPORTANT: bd commands must run sequentially — concurrent Dolt embedded access
      // causes SIGSEGV crashes (nil pointer dereference in dolthub/driver).
      await fetchIssues()
      // Fire-and-forget: stats update doesn't block issue list display
      fetchStats(issues.value)
    }
  } catch (e) {
    // Don't let pre-flight errors block the app — log and continue
    console.error('[handlePathChange] Error during project switch:', e)
  }

  if (thisGeneration !== pathChangeGeneration) {
return
  }

  // Resume change detection + polling AFTER data is loaded (avoids self-triggered cascade)
  if (beadsPath.value) {
    await startListening(beadsPath.value)
    notifySelfWrite()  // Arm cooldown so backend ignores bd's recent .beads/ writes
  }
  startPolling()
}

const handleReset = () => {
  // Last project removed - clear all data to show onboarding
  clearIssues()
  clearStats()
  isEditMode.value = false
  isCreatingNew.value = false
}

const handleAddIssue = () => {
  selectIssue(null)
  isCreatingNew.value = true
  isEditMode.value = true
  if (isMobileView.value) {
    mobilePanel.value = 'details'
  } else {
    isRightSidebarOpen.value = true
  }
}

const handleSelectIssue = async (issue: Issue) => {
  // First set the issue from list for immediate feedback
  selectIssue(issue)
  isEditMode.value = false
  isCreatingNew.value = false
  if (isMobileView.value) {
    mobilePanel.value = 'details'
  } else {
    isRightSidebarOpen.value = true
  }
  // Then fetch full details (including extended fields) in background
  await fetchIssue(issue.id)
}

const handleEditIssueFromTable = async (issue: Issue) => {
  // First set the issue from list for immediate feedback
  selectIssue(issue)
  isEditMode.value = true
  isCreatingNew.value = false
  if (isMobileView.value) {
    mobilePanel.value = 'details'
  } else {
    isRightSidebarOpen.value = true
  }
  // Then fetch full details (including extended fields) in background
  await fetchIssue(issue.id)
}

const handleDeselectIssue = () => {
  selectIssue(null)
  isEditMode.value = false
  isCreatingNew.value = false
}

const handleEditIssue = () => {
  isEditMode.value = true
  isCreatingNew.value = false
}

const handleCancelEdit = () => {
  // Si on était en mode création, fermer le panel
  if (isCreatingNew.value) {
    selectedIssue.value = null
    isRightSidebarOpen.value = false
    defaultParent.value = undefined
  }
  isEditMode.value = false
  isCreatingNew.value = false
}

const handleSaveIssue = async (payload: UpdateIssuePayload) => {
  try {
    if (isCreatingNew.value) {
      // Use the parent from payload (set in form) or from defaultParent (set via create-child)
      const parentId = payload.parent || defaultParent.value
      const result = await createIssue({
        title: payload.title || '',
        description: payload.description,
        type: payload.type,
        priority: payload.priority,
        assignee: payload.assignee,
        labels: payload.labels,
        externalRef: payload.externalRef,
        estimateMinutes: payload.estimateMinutes,
        designNotes: payload.designNotes,
        acceptanceCriteria: payload.acceptanceCriteria,
        workingNotes: payload.workingNotes,
        parent: parentId || undefined,
      })
      if (result) {
        selectIssue(result)
        // Fetch full issue details to get all fields
        await fetchIssue(result.id)
        notifySuccess('Issue created')
      }
      defaultParent.value = undefined
    } else if (selectedIssue.value) {
      await updateIssue(selectedIssue.value.id, payload)
      // Fetch full issue details to get comments and all fields
      await fetchIssue(selectedIssue.value.id)
      notifySuccess('Issue saved')
    }
    isEditMode.value = false
    isCreatingNew.value = false
    await fetchStats(issues.value)
  } catch {
    notifyError('Failed to save issue')
  }
}


const handleAddComment = async (content: string) => {
  if (!selectedIssue.value) return
  try {
    await addComment(selectedIssue.value.id, content)
    notifySuccess('Comment added')
  } catch {
    notifyError('Failed to add comment')
  }
}


const handleNavigateToIssue = async (id: string) => {
  // Check if this is a child issue (format: parent-id.number)
  // If so, expand the parent epic to make the child visible
  const lastDotIndex = id.lastIndexOf('.')
  if (lastDotIndex > 0) {
    const parentId = id.slice(0, lastDotIndex)
    expandEpic(parentId)
  }

  // Find the issue in the current list or fetch it
  const existingIssue = issues.value.find(i => i.id === id)
  if (existingIssue) {
    selectIssue(existingIssue)
  }
  // Fetch full details (including extended fields, parent, children)
  await fetchIssue(id)
}


// Search handler - search is prioritary over filters (always starts empty)
const searchValue = ref('')
const isSearchActive = computed(() => !!searchValue.value?.trim())

// Debounced br search to avoid spawning too many CLI processes
let searchTimeout: ReturnType<typeof setTimeout> | null = null

watch(searchValue, async (value) => {
  if (searchTimeout) clearTimeout(searchTimeout)

  const term = value.trim()
  if (isBr.value && term) {
    // br: delegate to full-text search via Tauri — skip client-side setSearch
    // to avoid flickering (client-side filter would render first, then br results replace)
    searchTimeout = setTimeout(async () => {
      const { searchIssues } = useIssues()
      await searchIssues(term)
    }, 300)
  } else if (isBr.value && !term) {
    // br: search cleared — restore the full list
    setSearch('')
    await fetchIssues()
  } else {
    // bd: client-side filtering (existing behavior)
    setSearch(value)
    await fetchIssues(!!term)
  }
})

// Available labels computed from all issues
const availableLabels = computed(() => {
  const labelSet = new Set<string>()
  issues.value.forEach(issue => {
    issue.labels?.forEach(label => labelSet.add(label))
  })
  return Array.from(labelSet).sort()
})

// Available assignees computed from all issues
const availableAssignees = computed(() => {
  const assigneeSet = new Set<string>()
  issues.value.forEach(issue => {
    if (issue.assignee) {
      assigneeSet.add(issue.assignee)
    }
  })
  return Array.from(assigneeSet).sort()
})

// Available epics for parent selector (only non-closed epics)
const availableEpics = computed(() => {
  return issues.value
    .filter(issue => issue.type === 'epic' && issue.status !== 'closed')
    .map(issue => ({ id: issue.id, title: issue.title }))
})

// In-progress issues for dashboard sidebar
const inProgressIssues = computed(() => {
  return issues.value.filter(issue => issue.status === 'in_progress')
})

// Default parent for new issues (set when creating child from epic)
const defaultParent = ref<string | undefined>(undefined)

const handleCreateChild = (parentId: string) => {
  defaultParent.value = parentId
  selectIssue(null)
  isCreatingNew.value = true
  isEditMode.value = true
  if (isMobileView.value) {
    mobilePanel.value = 'details'
  } else {
    isRightSidebarOpen.value = true
  }
}

const handleRemoveLabelFilter = (label: string) => {
  toggleLabelFilter(label)
}

// KPI filter handlers
type KpiFilter = 'total' | 'open' | 'in_progress' | 'blocked'
const activeKpiFilter = computed<KpiFilter | null>(() => {
  const statusFilters = filters.value.status
  if (statusFilters.length === 0) return null
  if (statusFilters.length === 1 && statusFilters[0] === 'open') return 'open'
  if (statusFilters.length === 1 && statusFilters[0] === 'in_progress') return 'in_progress'
  if (statusFilters.length === 1 && statusFilters[0] === 'blocked') return 'blocked'
  return null
})

const handleKpiClick = (kpi: KpiFilter) => {
  if (kpi === 'total') {
    clearFilters()
  } else if (kpi === 'open') {
    setStatusFilter(['open'])
  } else if (kpi === 'in_progress') {
    setStatusFilter(['in_progress'])
  } else if (kpi === 'blocked') {
    setStatusFilter(['blocked'])
  }
}

// Watch filters to refetch issues (only when no active search)
// Serialize values to avoid false triggers from deep watch when only search changes
watch(
  () => JSON.stringify([filters.value.status, filters.value.type, filters.value.priority]),
  () => {
    // Don't refetch if search is active (search ignores filters)
    if (!filters.value.search?.trim()) {
      fetchIssues()
    }
  }
)
</script>

<template>
  <div class="fixed inset-0 grid grid-rows-[1fr_auto] bg-background">
    <!-- Zoomable content (header + panels) -->
    <div id="zoomable-content" class="grid grid-rows-[auto_1fr] overflow-hidden">
      <!-- Header -->
      <AppHeader
        :project-name="currentProjectName"
        :edit-context="editContext"
        :edit-id="editId"
        :show-refresh="!showOnboarding"
        :is-exposed="isCurrentProjectExposed"
        @refresh="handleRefresh"
      />

    <!-- Desktop Layout (3 columns) -->
    <div v-if="!isMobileView" class="flex overflow-hidden">
      <!-- Left Sidebar - Dashboard (hidden in edit mode) -->
      <aside
        v-show="!(isEditMode || isCreatingNew)"
        class="border-r border-border bg-card flex flex-col relative"
        :class="{ 'transition-all duration-300': !isResizing }"
        :style="isLeftSidebarOpen ? { width: `${leftSidebarWidth}px` } : { width: '48px' }"
      >
        <!-- Resize handle -->
        <div
          v-if="isLeftSidebarOpen"
          class="absolute right-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-primary/50 transition-colors z-10"
          @mousedown="startResizeLeft"
        />
        <!-- Sidebar toggle -->
        <div class="p-2 border-b border-border flex items-center" :class="isLeftSidebarOpen ? 'justify-between' : 'justify-center'">
          <span v-if="isLeftSidebarOpen" class="text-sm font-medium px-2">Dashboard</span>
          <Tooltip>
            <TooltipTrigger as-child>
              <Button
                variant="ghost"
                size="icon"
                class="h-8 w-8"
                @click="isLeftSidebarOpen = !isLeftSidebarOpen"
              >
                <svg
                  class="w-4 h-4 transition-transform"
                  :class="{ 'rotate-180': !isLeftSidebarOpen }"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <polyline points="15 18 9 12 15 6" />
                </svg>
              </Button>
            </TooltipTrigger>
            <TooltipContent>{{ isLeftSidebarOpen ? 'Close dashboard' : 'Open dashboard' }}</TooltipContent>
          </Tooltip>
        </div>

        <!-- Sidebar content -->
        <div v-if="isLeftSidebarOpen" class="flex-1 flex flex-col overflow-hidden">
          <!-- Top section (fixed content) -->
          <div class="p-4 space-y-4 shrink-0">
            <PathSelector v-if="!showOnboarding" ref="pathSelectorRef" :is-loading="isLoading" @change="handlePathChange" @reset="handleReset" />

            <div v-if="stats" class="space-y-4 mt-6">
              <div class="grid grid-cols-4 gap-1.5">
                <KpiCard title="Total" :value="stats.total" :active="activeKpiFilter === null && filters.status.length === 0" @click="handleKpiClick('total')" />
                <KpiCard title="Open" :value="stats.open" color="var(--color-status-open)" :active="activeKpiFilter === 'open'" @click="handleKpiClick('open')" />
                <KpiCard title="In Progress" :value="stats.inProgress" color="var(--color-status-in-progress)" :active="activeKpiFilter === 'in_progress'" @click="handleKpiClick('in_progress')" />
                <KpiCard title="Blocked" :value="stats.blocked" color="var(--color-status-blocked)" :active="activeKpiFilter === 'blocked'" @click="handleKpiClick('blocked')" />
              </div>
            </div>

            <div v-if="!stats" class="flex items-center justify-center py-8">
              <OnboardingCard v-if="showOnboarding" @browse="openFolderPicker" />
              <span v-else class="text-muted-foreground text-sm">Loading...</span>
            </div>
          </div>

          <!-- Scrollable section for Charts and Ready to Work -->
          <div v-if="stats" class="flex-1 overflow-y-auto px-4 pb-4 space-y-4">
            <DashboardContent
              hide-kpis
              :stats="stats"
              :ready-issues="readyIssues"
              :in-progress-issues="inProgressIssues"
              :active-kpi-filter="activeKpiFilter"
              :status-filters="filters.status"
              @select-issue="handleSelectIssue"
              @kpi-click="handleKpiClick"
            />
          </div>
        </div>

        <!-- Collapsed state icon -->
        <div v-else class="flex-1 flex flex-col items-center pt-4 gap-4">
          <svg class="w-5 h-5 text-muted-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
            <line x1="9" y1="9" x2="15" y2="9" />
            <line x1="9" y1="13" x2="15" y2="13" />
            <line x1="9" y1="17" x2="11" y2="17" />
          </svg>
        </div>
      </aside>

      <!-- Center - Issues List -->
      <main
        v-show="!(isEditMode || isCreatingNew)"
        class="flex-1 flex flex-col overflow-hidden min-w-0"
      >
        <!-- Onboarding: Prerequisites Card -->
        <PrerequisitesCard v-if="showOnboarding" @browse="openFolderPicker" />

        <!-- Normal: Issues Toolbar + Table -->
        <template v-else>
          <IssueListPanel
            v-model:search="searchValue"
            v-model:selected-ids="selectedIds"
            :filters="{ status: filters.status, type: filters.type, priority: filters.priority, labels: filters.labels, assignee: filters.assignee }"
            :available-labels="availableLabels"
            :available-assignees="availableAssignees"
            :has-selection="multiSelectMode ? selectedIds.length > 0 : !!selectedIssue"
            :multi-select-mode="multiSelectMode"
            :selected-count="selectedIds.length"
            :columns="columns"
            :is-search-active="isSearchActive"
            :issues="paginatedIssues"
            :grouped-issues="groupedIssues"
            :selected-id="selectedIssue?.id"
            :has-more="hasMore"
            :total-count="filteredIssues.length"
            :sort-field="sortField"
            :sort-direction="sortDirection"
            :newly-added-ids="newlyAddedIds"
            @add="handleAddIssue"
            @delete="handleDeleteIssue"
            @toggle-multi-select="toggleMultiSelect"
            @update:columns="setColumns"
            @reset-columns="resetColumns"
            @toggle-status="toggleStatus"
            @toggle-type="toggleType"
            @toggle-priority="togglePriority"
            @toggle-label="toggleLabelFilter"
            @toggle-assignee="toggleAssignee"
            @remove-label="handleRemoveLabelFilter"
            @clear-filters="clearFilters"
            @select="handleSelectIssue"
            @edit="handleEditIssueFromTable"
            @deselect="handleDeselectIssue"
            @load-more="loadMore"
            @sort="setSort"
          />

          <div v-if="isLoading" class="text-center text-muted-foreground py-4">
            Loading...
          </div>
        </template>
      </main>

      <!-- Right Sidebar - Details (hidden when no selection and not in edit mode) -->
      <aside
        v-if="selectedIssue || isEditMode || isCreatingNew"
        class="bg-card flex flex-col relative overflow-hidden"
        :class="[
          { 'transition-all duration-300': !isResizing && !(isEditMode || isCreatingNew) },
          { 'border-l border-border': !(isEditMode || isCreatingNew) },
          { 'w-full lg:w-1/2 lg:min-w-2xl mx-auto my-4 border border-border rounded-lg': isEditMode || isCreatingNew }
        ]"
        :style="(isEditMode || isCreatingNew) ? {} : (isRightSidebarOpen ? { width: `${rightSidebarWidth}px` } : { width: '48px' })"
      >
        <!-- Resize handle -->
        <div
          v-if="isRightSidebarOpen && !(isEditMode || isCreatingNew)"
          class="absolute left-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-primary/50 transition-colors z-10"
          @mousedown="startResizeRight"
        />
        <!-- Sidebar toggle -->
        <div
          v-show="!(isEditMode || isCreatingNew)"
          class="p-2 border-b border-border flex items-center"
          :class="isRightSidebarOpen ? 'justify-between' : 'justify-center'"
        >
          <Button
            variant="ghost"
            size="icon"
            class="h-8 w-8"
            @click="isRightSidebarOpen = !isRightSidebarOpen"
          >
            <svg
              class="w-4 h-4 transition-transform"
              :class="{ 'rotate-180': isRightSidebarOpen }"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <polyline points="15 18 9 12 15 6" />
            </svg>
          </Button>
          <span v-if="isRightSidebarOpen" class="text-sm font-medium px-2">Details</span>
        </div>

        <!-- Sidebar content -->
        <template v-if="isRightSidebarOpen">
          <!-- Fixed header for issue preview -->
          <IssueDetailHeader
            v-if="selectedIssue && !isEditMode && !isCreatingNew"
            :selected-issue="selectedIssue"
            @edit="handleEditIssue"
            @reopen="handleReopenIssue"
            @close="handleCloseIssue"
            @delete="handleDeleteIssue"
          />

          <!-- Form mode: form gère son propre scroll -->
          <div v-if="isEditMode || isCreatingNew" class="flex-1 min-h-0 p-4 overflow-hidden">
            <IssueForm
              :issue="isCreatingNew ? null : selectedIssue"
              :is-new="isCreatingNew"
              :is-saving="isUpdating"
              :available-epics="availableEpics"
              :available-labels="availableLabels"
              :default-parent="defaultParent"
              :dot-notation-parent="bdDotNotationParent"
              @save="handleSaveIssue"
              @cancel="handleCancelEdit"
            />
          </div>

          <!-- Preview mode: ScrollArea pour le contenu -->
          <ScrollArea v-else class="flex-1 min-h-0">
            <div class="p-4">
              <div v-if="selectedIssue">
                <IssuePreview
                  :issue="selectedIssue"
                  :readonly="selectedIssue.status === 'closed'"
                  :available-issues="availableIssuesForDeps"
                  @navigate-to-issue="handleNavigateToIssue"
                  @attach-image="handleAttachImage"
                  @detach-image="confirmDetachImage"
                  @create-child="handleCreateChild"
                  @open-add-blocker="openAddBlockerDialog"
                  @remove-dependency="confirmRemoveDependency"
                  @open-add-relation="openAddRelationDialog"
                  @remove-relation="confirmRemoveRelation"
                />
                <CommentSection
                  class="mt-3"
                  :comments="selectedIssue.comments || []"
                  :readonly="selectedIssue.status === 'closed'"
                  @add-comment="handleAddComment"
                />
              </div>

              <div v-else class="text-center text-muted-foreground py-8">
                Select an issue to view details
              </div>
            </div>
          </ScrollArea>
        </template>

        <!-- Collapsed state icon -->
        <div v-else class="flex-1 flex flex-col items-center pt-4 gap-4">
          <svg class="w-5 h-5 text-muted-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
            <line x1="16" y1="13" x2="8" y2="13" />
            <line x1="16" y1="17" x2="8" y2="17" />
          </svg>
        </div>
      </aside>
    </div>

    <!-- Mobile Layout (tabs + stacked panels) -->
    <div v-else class="flex flex-col overflow-hidden">
      <!-- Mobile Navigation Tabs -->
      <div class="flex border-b border-border bg-card">
        <button
          class="flex-1 py-3 text-sm font-medium transition-colors"
          :class="mobilePanel === 'dashboard' ? 'text-primary border-b-2 border-primary' : 'text-muted-foreground'"
          @click="mobilePanel = 'dashboard'"
        >
          Dashboard
        </button>
        <button
          class="flex-1 py-3 text-sm font-medium transition-colors"
          :class="mobilePanel === 'issues' ? 'text-primary border-b-2 border-primary' : 'text-muted-foreground'"
          @click="mobilePanel = 'issues'"
        >
          Issues ({{ filteredIssues.length }})
        </button>
        <button
          class="flex-1 py-3 text-sm font-medium transition-colors"
          :class="mobilePanel === 'details' ? 'text-primary border-b-2 border-primary' : 'text-muted-foreground'"
          @click="mobilePanel = 'details'"
        >
          Details
        </button>
      </div>

      <!-- Mobile Panels -->
      <!-- Dashboard Panel -->
      <ScrollArea v-if="mobilePanel === 'dashboard'" class="flex-1">
        <div class="p-4 space-y-6">
          <PathSelector v-if="!showOnboarding" ref="mobilePathSelectorRef" :is-loading="isLoading" @change="handlePathChange" @reset="handleReset" />

          <DashboardContent
            class="space-y-6"
            :stats="stats"
            :ready-issues="readyIssues"
            :in-progress-issues="inProgressIssues"
            :kpi-grid-cols="2"
            :active-kpi-filter="activeKpiFilter"
            :status-filters="filters.status"
            :show-onboarding="showOnboarding"
            @select-issue="handleSelectIssue"
            @kpi-click="handleKpiClick"
            @browse="openFolderPicker"
          />
        </div>
      </ScrollArea>

      <!-- Issues Panel -->
      <div v-else-if="mobilePanel === 'issues'" class="flex-1 flex flex-col overflow-hidden">
        <!-- Onboarding: Prerequisites Card -->
        <PrerequisitesCard v-if="showOnboarding" @browse="openFolderPicker" />

        <!-- Normal: Issues Toolbar and Table -->
        <IssueListPanel
          v-if="!showOnboarding"
          v-model:search="searchValue"
          v-model:selected-ids="selectedIds"
          :filters="{ status: filters.status, type: filters.type, priority: filters.priority, labels: filters.labels, assignee: filters.assignee }"
          :available-labels="availableLabels"
          :available-assignees="availableAssignees"
          :has-selection="multiSelectMode ? selectedIds.length > 0 : !!selectedIssue"
          :multi-select-mode="multiSelectMode"
          :selected-count="selectedIds.length"
          :columns="columns"
          :is-search-active="isSearchActive"
          :issues="paginatedIssues"
          :grouped-issues="groupedIssues"
          :selected-id="selectedIssue?.id"
          :has-more="hasMore"
          :total-count="filteredIssues.length"
          :sort-field="sortField"
          :sort-direction="sortDirection"
          :newly-added-ids="newlyAddedIds"
          @add="handleAddIssue"
          @delete="handleDeleteIssue"
          @toggle-multi-select="toggleMultiSelect"
          @update:columns="setColumns"
          @reset-columns="resetColumns"
          @toggle-status="toggleStatus"
          @toggle-type="toggleType"
          @toggle-priority="togglePriority"
          @toggle-label="toggleLabelFilter"
          @toggle-assignee="toggleAssignee"
          @remove-label="handleRemoveLabelFilter"
          @clear-filters="clearFilters"
          @select="handleSelectIssue"
          @edit="handleEditIssueFromTable"
          @deselect="handleDeselectIssue"
          @load-more="loadMore"
          @sort="setSort"
        />
      </div>

      <!-- Details Panel -->
      <div v-else-if="mobilePanel === 'details'" class="flex-1 flex flex-col overflow-hidden">
        <!-- Fixed header for issue preview -->
        <IssueDetailHeader
          v-if="selectedIssue && !isEditMode && !isCreatingNew"
          :selected-issue="selectedIssue"
          @edit="handleEditIssue"
          @reopen="handleReopenIssue"
          @close="handleCloseIssue"
          @delete="handleDeleteIssue"
        />

        <!-- Form mode: form gère son propre scroll -->
        <div v-if="isEditMode || isCreatingNew" class="flex-1 min-h-0 p-4 overflow-hidden">
          <IssueForm
            :issue="isCreatingNew ? null : selectedIssue"
            :is-new="isCreatingNew"
            :is-saving="isUpdating"
            :available-epics="availableEpics"
            :available-labels="availableLabels"
            :default-parent="defaultParent"
            @save="handleSaveIssue"
            @cancel="handleCancelEdit"
          />
        </div>

        <!-- Preview mode: ScrollArea pour le contenu -->
        <ScrollArea v-else class="flex-1 min-h-0">
          <div class="p-4">
            <div v-if="selectedIssue">
              <IssuePreview
                :issue="selectedIssue"
                :readonly="selectedIssue.status === 'closed'"
                :available-issues="availableIssuesForDeps"
                @navigate-to-issue="handleNavigateToIssue"
                @attach-image="handleAttachImage"
                @detach-image="confirmDetachImage"
                @create-child="handleCreateChild"
                @open-add-blocker="openAddBlockerDialog"
                @remove-dependency="confirmRemoveDependency"
                @open-add-relation="openAddRelationDialog"
                @remove-relation="confirmRemoveRelation"
              />
              <CommentSection
                class="mt-3"
                :comments="selectedIssue.comments || []"
                :readonly="selectedIssue.status === 'closed'"
                @add-comment="handleAddComment"
              />
            </div>

            <div v-else class="text-center text-muted-foreground py-8">
              Select an issue to view details
            </div>
          </div>
        </ScrollArea>
      </div>
      </div>
    </div>

    <!-- Debug Panel (above footer) -->
    <DebugPanel v-model:is-open="showDebugPanel" />

    <!-- Footer (outside zoomable content) -->
    <footer class="px-4 py-2 border-t border-border bg-card flex items-center justify-between text-xs text-muted-foreground font-mono">
      <div class="flex items-center gap-2">
        <!-- Debug panel toggle -->
        <button
          class="flex items-center gap-1.5 hover:text-foreground transition-colors"
          :class="showDebugPanel ? 'text-foreground' : ''"
          title="Toggle Debug Panel (⌘⇧L)"
          @click="showDebugPanel = !showDebugPanel"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="m8 2 1.88 1.88" /><path d="M14.12 3.88 16 2" />
            <path d="M9 7.13v-1a3.003 3.003 0 1 1 6 0v1" />
            <path d="M12 20c-3.3 0-6-2.7-6-6v-3a4 4 0 0 1 4-4h4a4 4 0 0 1 4 4v3c0 3.3-2.7 6-6 6" />
            <path d="M12 20v-9" /><path d="M6.53 9C4.6 8.8 3 7.1 3 5" />
            <path d="M6 13H2" /><path d="M3 21c0-2.1 1.7-3.9 3.8-4" />
            <path d="M20.97 5c0 2.1-1.6 3.8-3.5 4" /><path d="M22 13h-4" />
            <path d="M17.2 17c2.1.1 3.8 1.9 3.8 4" />
          </svg>
        </button>

        <!-- Settings -->
        <button
          class="flex items-center hover:text-foreground transition-colors"
          title="Settings (⌘,)"
          @click="showSettingsDialog = true"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
            <circle cx="12" cy="12" r="3" />
          </svg>
        </button>

        <!-- Probe enabled indicator (dev-only) -->
        <span
          v-if="probeEnabled && isDev"
          class="flex items-center gap-1 text-green-500"
          title="Probe broadcasting enabled"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4.9 19.1C1 15.2 1 8.8 4.9 4.9" />
            <path d="M7.8 16.2c-2.3-2.3-2.3-6.1 0-8.4" />
            <circle cx="12" cy="12" r="2" fill="currentColor" />
            <path d="M16.2 7.8c2.3 2.3 2.3 6.1 0 8.4" />
            <path d="M19.1 4.9C23 8.8 23 15.2 19.1 19.1" />
          </svg>
          <span class="uppercase text-[10px] font-semibold tracking-wider">Probe</span>
        </span>
      </div>

      <!-- Center spacer -->
      <div></div>

      <!-- Version à droite -->
      <UpdateIndicator />
    </footer>

    <!-- Issue management dialogs + Image/Markdown Preview -->
    <DialogsLayer />

    <!-- Onboarding Folder Picker -->
    <FolderPicker
      v-model:open="isOnboardingPickerOpen"
      current-path="~"
      @select="handleOnboardingFolderSelect"
    />

    <!-- Sync Error Dialog -->
    <ConfirmDialog
      v-model:open="showSyncErrorDialog"
      title="Sync Error"
      confirm-text="OK"
      :show-cancel="false"
      @confirm="closeSyncErrorDialog"
    >
      <template #description>
        <p class="text-sm text-muted-foreground mb-2">
          The sync operation failed with the following error:
        </p>
        <pre class="text-sm text-destructive bg-muted p-3 rounded-md overflow-x-auto whitespace-pre-wrap break-words">{{ lastSyncError }}</pre>
      </template>
    </ConfirmDialog>

    <!-- Database Repair Dialog -->
    <Dialog :open="needsRepair" @update:open="(open) => !open && dismissRepair()">
      <DialogContent class="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle class="flex items-center gap-2 text-amber-500">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            Database Repair Required
          </DialogTitle>
          <DialogDescription class="text-left space-y-3 pt-2">
            <p>
              A database schema incompatibility was detected. This is caused by a bug in the bd CLI update (version 0.49.4).
            </p>
            <p v-if="affectedProject" class="text-sm bg-muted p-2 rounded font-mono break-all">
              {{ affectedProject }}
            </p>
            <p>
              <strong>What will happen:</strong>
            </p>
            <ul class="list-disc list-inside text-sm space-y-1 ml-2">
              <li>Your current database will be backed up</li>
              <li>The database will be recreated from your issues backup file</li>
              <li>All your issues will be preserved</li>
            </ul>
            <p v-if="repairProgress" class="text-sm text-muted-foreground">
              Repairing {{ repairProgress.current }}/{{ repairProgress.total }}:
              <span class="font-mono text-xs">{{ repairProgress.currentPath.split('/').pop() }}</span>
            </p>
            <p v-if="repairError" class="text-destructive text-sm">
              Error: {{ repairError }}
            </p>
          </DialogDescription>
        </DialogHeader>
        <div class="flex justify-between mt-4">
          <Button v-if="projects.length > 1" variant="secondary" :disabled="isRepairing" @click="handleRepairAll">
            <svg v-if="isRepairing && repairProgress" class="animate-spin -ml-1 mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            Repair All ({{ projects.length }})
          </Button>
          <div class="flex gap-2 ml-auto">
            <Button variant="outline" :disabled="isRepairing" @click="dismissRepair">
              Later
            </Button>
            <Button :disabled="isRepairing" @click="handleRepair">
              <svg v-if="isRepairing && !repairProgress" class="animate-spin -ml-1 mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              {{ isRepairing && !repairProgress ? 'Repairing...' : 'Repair This Project' }}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>

    <!-- Dolt Migration Dialog -->
    <Dialog :open="needsMigration" @update:open="(open) => !open && dismissMigration()">
      <DialogContent class="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle class="flex items-center gap-2 text-amber-500">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            Database Migration Required
          </DialogTitle>
          <DialogDescription class="text-left space-y-3 pt-2">
            <p>
              Your bd version (>= 0.50) can no longer read previous SQLite databases.
              This project needs to be migrated to the new Dolt backend. This is a one-time operation.
            </p>
            <p v-if="migrateAffectedProject" class="text-sm bg-muted p-2 rounded font-mono break-all">
              {{ migrateAffectedProject }}
            </p>
            <p>
              <strong>What will happen:</strong>
            </p>
            <ul class="list-disc list-inside text-sm space-y-1 ml-2">
              <li>A new Dolt database will be created (<code class="text-xs">bd init</code>)</li>
              <li>Your issues will be imported from the JSONL backup file (<code class="text-xs">bd import</code>)</li>
              <li>None of your issues will be lost — only previously deleted issues (tombstones) are skipped</li>
            </ul>
            <p v-if="migrateError" class="text-destructive text-sm">
              Error: {{ migrateError }}
            </p>
          </DialogDescription>
        </DialogHeader>
        <div class="flex justify-end gap-2 mt-4">
          <Button variant="outline" :disabled="isMigrating" @click="dismissMigration">
            Later
          </Button>
          <Button :disabled="isMigrating" class="bg-[#29E3C1] hover:bg-[#22c9aa] text-black" @click="handleMigrateToDolt">
            <svg v-if="isMigrating" class="animate-spin -ml-1 mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            {{ isMigrating ? 'Migrating...' : 'Migrate Now' }}
          </Button>
        </div>
      </DialogContent>
    </Dialog>

    <!-- Attachment Refs Migration Dialog -->
    <Dialog :open="needsRefsMigration" @update:open="(v: boolean) => { if (!v) dismissRefsMigration() }">
      <DialogContent class="sm:max-w-[480px]">
        <DialogHeader>
          <DialogTitle class="flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-blue-500" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
            </svg>
            Attachment Update Required
          </DialogTitle>
          <DialogDescription as="div" class="space-y-3 text-sm">
            <p>
              Attachments now use the filesystem directly.
              This cleanup removes old attachment paths from external references. One-time operation.
            </p>
            <p class="text-muted-foreground">
              A backup of your data will be created before any changes are made.
            </p>
            <p class="bg-muted p-2 rounded text-xs font-mono">
              {{ refsRefCount }} issue(s) with references to clean up
            </p>
            <p v-if="refsMigrateError" class="text-destructive text-sm">
              Error: {{ refsMigrateError }}
            </p>
          </DialogDescription>
        </DialogHeader>
        <div class="flex justify-end gap-2 mt-4">
          <Button variant="outline" :disabled="isRefsMigrating" @click="dismissRefsMigration">
            Later
          </Button>
          <Button :disabled="isRefsMigrating" class="bg-[#29E3C1] hover:bg-[#22c9aa] text-black" @click="handleMigrateRefs">
            <svg v-if="isRefsMigrating" class="animate-spin -ml-1 mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            {{ isRefsMigrating ? 'Updating...' : 'Update Now' }}
          </Button>
        </div>
      </DialogContent>
    </Dialog>

  </div>
</template>
