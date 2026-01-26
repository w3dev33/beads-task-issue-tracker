<script setup lang="ts">
import type { Issue, UpdateIssuePayload, IssueStatus, IssueType, IssuePriority } from '~/types/issue'

// Layout components
import AppHeader from '~/components/layout/AppHeader.vue'
import UpdateIndicator from '~/components/layout/UpdateIndicator.vue'

// Dashboard components
import PathSelector from '~/components/dashboard/PathSelector.vue'
import FolderPicker from '~/components/dashboard/FolderPicker.vue'
import KpiCard from '~/components/dashboard/KpiCard.vue'
import StatusChart from '~/components/dashboard/StatusChart.vue'
import PriorityChart from '~/components/dashboard/PriorityChart.vue'
import QuickList from '~/components/dashboard/QuickList.vue'
import OnboardingCard from '~/components/dashboard/OnboardingCard.vue'
import PrerequisitesCard from '~/components/dashboard/PrerequisitesCard.vue'

// Issues components
import IssuesToolbar from '~/components/issues/IssuesToolbar.vue'
import FilterChips from '~/components/issues/FilterChips.vue'
import IssueTable from '~/components/issues/IssueTable.vue'

// Details components
import IssuePreview from '~/components/details/IssuePreview.vue'
import IssueForm from '~/components/details/IssueForm.vue'
import CommentSection from '~/components/details/CommentSection.vue'

// Badge components
import TypeBadge from '~/components/issues/TypeBadge.vue'
import StatusBadge from '~/components/issues/StatusBadge.vue'
import PriorityBadge from '~/components/issues/PriorityBadge.vue'

// UI components
import { Button } from '~/components/ui/button'
import { ScrollArea } from '~/components/ui/scroll-area'
import { ConfirmDialog } from '~/components/ui/confirm-dialog'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '~/components/ui/tooltip'

// Composables
const { filters, toggleStatus, toggleType, togglePriority, clearFilters, setStatusFilter, setSearch } = useFilters()
const { columns, toggleColumn, setColumns, resetColumns } = useColumnConfig()
const { beadsPath, hasStoredPath } = useBeadsPath()
const { favorites } = useFavorites()
const {
  issues,
  filteredIssues,
  selectedIssue,
  isLoading,
  isUpdating,
  fetchIssues,
  fetchIssue,
  createIssue,
  updateIssue,
  closeIssue,
  deleteIssue,
  selectIssue,
  addComment,
  checkForChanges,
  clearIssues,
} = useIssues()
const { stats, readyIssues, fetchStats, clearStats } = useDashboard()
const { check: checkForUpdates } = useUpdateChecker()

// Sidebar states (persisted)
const isLeftSidebarOpen = useLocalStorage('beads:leftSidebar', true)
const isRightSidebarOpen = useLocalStorage('beads:rightSidebar', true)

// Close right sidebar on init if no issue selected
if (import.meta.client && !selectedIssue.value) {
  isRightSidebarOpen.value = false
}
const isChartsCollapsed = useLocalStorage('beads:chartsCollapsed', true)
const isReadyCollapsed = useLocalStorage('beads:readyCollapsed', true)
const leftSidebarWidth = useLocalStorage('beads:leftSidebarWidth', 360)
const rightSidebarWidth = useLocalStorage('beads:rightSidebarWidth', 484)

// Sidebar resize
const isResizingLeft = ref(false)
const isResizingRight = ref(false)
const startX = ref(0)
const startWidth = ref(0)

const startResizeLeft = (e: MouseEvent) => {
  e.preventDefault()
  // Clear any existing text selection
  window.getSelection()?.removeAllRanges()
  isResizingLeft.value = true
  startX.value = e.clientX
  startWidth.value = leftSidebarWidth.value
  document.addEventListener('mousemove', onResizeLeft)
  document.addEventListener('mouseup', stopResizeLeft)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  document.body.style.webkitUserSelect = 'none'
}

const onResizeLeft = (e: MouseEvent) => {
  if (!isResizingLeft.value) return
  const diff = e.clientX - startX.value
  const newWidth = Math.min(Math.max(startWidth.value + diff, 280), 500)
  leftSidebarWidth.value = newWidth
}

const stopResizeLeft = () => {
  isResizingLeft.value = false
  document.removeEventListener('mousemove', onResizeLeft)
  document.removeEventListener('mouseup', stopResizeLeft)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
  document.body.style.webkitUserSelect = ''
}

const startResizeRight = (e: MouseEvent) => {
  e.preventDefault()
  // Clear any existing text selection
  window.getSelection()?.removeAllRanges()
  isResizingRight.value = true
  startX.value = e.clientX
  startWidth.value = rightSidebarWidth.value
  document.addEventListener('mousemove', onResizeRight)
  document.addEventListener('mouseup', stopResizeRight)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  document.body.style.webkitUserSelect = 'none'
}

const onResizeRight = (e: MouseEvent) => {
  if (!isResizingRight.value) return
  const diff = startX.value - e.clientX
  const newWidth = Math.min(Math.max(startWidth.value + diff, 300), 800)
  rightSidebarWidth.value = newWidth
}

const stopResizeRight = () => {
  isResizingRight.value = false
  document.removeEventListener('mousemove', onResizeRight)
  document.removeEventListener('mouseup', stopResizeRight)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
  document.body.style.webkitUserSelect = ''
}

const isResizing = computed(() => isResizingLeft.value || isResizingRight.value)

// Current favorite name for header subtitle
const currentFavoriteName = computed(() => {
  const favorite = favorites.value.find(f => f.path === beadsPath.value)
  return favorite?.name
})

// Show onboarding when no project is selected (no stored path and no favorites)
const showOnboarding = computed(() => {
  return favorites.value.length === 0 && !hasStoredPath.value
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
  await fetchStats(issues.value)
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

// Polling for external changes (replaces file watcher for lower CPU usage)
const POLLING_INTERVAL = 5000 // 5 seconds
let pollingInterval: ReturnType<typeof setInterval> | null = null
let isPollingPaused = false
const isSyncing = ref(false)

// Fetch latest data on each poll
const pollForChanges = async () => {
  // Don't poll if no active project
  if (isPollingPaused || isLoading.value || isUpdating.value || showOnboarding.value || !beadsPath.value || favorites.value.length === 0) {
    return
  }

  try {
    isSyncing.value = true
    // Fetch issues first, then compute stats from them (reduces API calls from 6 to 3)
    await fetchIssues(!!filters.value.search?.trim(), true)
    await fetchStats(issues.value)
  } catch {
    // Ignore polling errors
  } finally {
    isSyncing.value = false
  }
}

// Start polling
const startPolling = () => {
  if (pollingInterval) return
  pollingInterval = setInterval(pollForChanges, POLLING_INTERVAL)
}

// Stop polling
const stopPolling = () => {
  if (pollingInterval) {
    clearInterval(pollingInterval)
    pollingInterval = null
  }
}

// Handle visibility change - pause polling when app is hidden
const handleVisibilityChange = () => {
  if (document.hidden) {
    isPollingPaused = true
  } else {
    isPollingPaused = false
    // Immediately check for changes when app becomes visible
    pollForChanges()
  }
}

onMounted(async () => {
  checkViewport()
  if (import.meta.client) {
    window.addEventListener('resize', checkViewport)
    document.addEventListener('visibilitychange', handleVisibilityChange)

    // Start polling for changes
    startPolling()

    // Check for updates after initial load
    checkForUpdates()
  }
  // Only fetch data if not in onboarding mode
  if (!showOnboarding.value) {
    fetchIssues().then(() => fetchStats(issues.value))
  }
})

onUnmounted(() => {
  if (import.meta.client) {
    window.removeEventListener('resize', checkViewport)
    document.removeEventListener('visibilitychange', handleVisibilityChange)
    stopPolling()
  }
})

// Local state
const isEditMode = ref(false)
const isCreatingNew = ref(false)
const multiSelectMode = ref(false)
const selectedIds = ref<string[]>([])
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

// Delete confirmation dialog
const isDeleteDialogOpen = ref(false)
const deleteTargetTitles = ref<string[]>([])
const isDeleting = ref(false)

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

const toggleMultiSelect = () => {
  multiSelectMode.value = !multiSelectMode.value
  if (!multiSelectMode.value) {
    selectedIds.value = []
  }
}

// Handlers
const handleRefresh = () => {
  // Full page reload to reset all state (like reopening the app)
  window.location.reload()
}

const handlePathChange = async () => {
  selectIssue(null)
  isEditMode.value = false
  isCreatingNew.value = false
  await fetchIssues()
  await fetchStats(issues.value)
}

const handleReset = () => {
  // Last favorite removed - clear all data to show onboarding
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
  }
  isEditMode.value = false
  isCreatingNew.value = false
}

const handleSaveIssue = async (payload: UpdateIssuePayload) => {
  if (isCreatingNew.value) {
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
    })
    if (result) {
      selectIssue(result)
      // Fetch full issue details to get all fields
      await fetchIssue(result.id)
    }
  } else if (selectedIssue.value) {
    await updateIssue(selectedIssue.value.id, payload)
    // Fetch full issue details to get comments and all fields
    await fetchIssue(selectedIssue.value.id)
  }
  isEditMode.value = false
  isCreatingNew.value = false
  await fetchStats(issues.value)
}

const handleCloseIssue = async () => {
  if (selectedIssue.value) {
    await closeIssue(selectedIssue.value.id)
    await fetchStats(issues.value)
  }
}

const handleAddComment = async (content: string) => {
  if (selectedIssue.value) {
    await addComment(selectedIssue.value.id, content)
  }
}

const handleDeleteIssue = () => {
  if (multiSelectMode.value && selectedIds.value.length > 0) {
    // Get titles of selected issues
    deleteTargetTitles.value = selectedIds.value
      .map(id => filteredIssues.value.find(i => i.id === id)?.title)
      .filter((t): t is string => !!t)
  } else if (selectedIssue.value) {
    deleteTargetTitles.value = [selectedIssue.value.title]
  } else {
    return
  }
  isDeleteDialogOpen.value = true
}

const confirmDelete = async () => {
  isDeleting.value = true
  try {
    if (multiSelectMode.value && selectedIds.value.length > 0) {
      for (const id of selectedIds.value) {
        await deleteIssue(id)
      }
      selectedIds.value = []
    } else if (selectedIssue.value) {
      await deleteIssue(selectedIssue.value.id)
      isEditMode.value = false
      isCreatingNew.value = false
    }
    await fetchStats(issues.value)
  } finally {
    isDeleting.value = false
    isDeleteDialogOpen.value = false
  }
}

// Search handler - search is prioritary over filters (always starts empty)
const searchValue = ref('')
const isSearchActive = computed(() => !!searchValue.value?.trim())

watch(searchValue, async (value) => {
  setSearch(value)
  // When search changes, refetch with ignoreFilters if search is active
  await fetchIssues(!!value.trim())
})

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

// Filter handlers for toolbar
const handleAddStatusFilter = (status: IssueStatus) => {
  if (!filters.value.status.includes(status)) {
    toggleStatus(status)
  }
}

const handleAddTypeFilter = (type: IssueType) => {
  if (!filters.value.type.includes(type)) {
    toggleType(type)
  }
}

const handleAddPriorityFilter = (priority: IssuePriority) => {
  if (!filters.value.priority.includes(priority)) {
    togglePriority(priority)
  }
}

// Watch filters to refetch issues (only when no active search)
watch(
  () => [filters.value.status, filters.value.type, filters.value.priority],
  () => {
    // Don't refetch if search is active (search ignores filters)
    if (!filters.value.search?.trim()) {
      fetchIssues()
    }
  },
  { deep: true }
)
</script>

<template>
  <div class="fixed inset-0 grid grid-rows-[1fr_auto] bg-background">
    <!-- Zoomable content (header + panels) -->
    <div id="zoomable-content" class="grid grid-rows-[auto_1fr] overflow-hidden">
      <!-- Header -->
      <AppHeader
        :favorite-name="currentFavoriteName"
        :edit-context="editContext"
        :edit-id="editId"
        :show-refresh="!showOnboarding"
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
            <!-- Collapsible Charts Section -->
            <div class="space-y-2">
              <button
                class="flex items-center gap-2 text-xs text-muted-foreground hover:text-foreground transition-colors w-full"
                @click="isChartsCollapsed = !isChartsCollapsed"
              >
                <svg
                  class="w-3 h-3 transition-transform"
                  :class="{ '-rotate-90': isChartsCollapsed }"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <polyline points="6 9 12 15 18 9" />
                </svg>
                <span class="uppercase tracking-wide">Charts</span>
              </button>
              <div v-show="!isChartsCollapsed" class="space-y-4 pl-5">
                <StatusChart :open="stats.open" :closed="stats.closed" />
                <PriorityChart :by-priority="stats.byPriority" />
              </div>
            </div>

            <!-- Collapsible Ready to Work Section -->
            <div class="space-y-2">
              <button
                class="flex items-center gap-2 text-xs text-muted-foreground hover:text-foreground transition-colors w-full"
                @click="isReadyCollapsed = !isReadyCollapsed"
              >
                <svg
                  class="w-3 h-3 transition-transform"
                  :class="{ '-rotate-90': isReadyCollapsed }"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <polyline points="6 9 12 15 18 9" />
                </svg>
                <span class="uppercase tracking-wide">Ready to Work</span>
                <span class="text-[10px] ml-auto">({{ readyIssues.length }})</span>
              </button>
              <div v-show="!isReadyCollapsed" class="pl-5">
                <QuickList :issues="readyIssues" @select="handleSelectIssue" />
              </div>
            </div>
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
        <div v-if="!showOnboarding" class="p-4 border-b border-border space-y-3">
          <IssuesToolbar
            v-model:search="searchValue"
            :active-status-filters="filters.status"
            :active-type-filters="filters.type"
            :active-priority-filters="filters.priority"
            :has-selection="multiSelectMode ? selectedIds.length > 0 : !!selectedIssue"
            :multi-select-mode="multiSelectMode"
            :selected-count="selectedIds.length"
            :columns="columns"
            @add="handleAddIssue"
            @delete="handleDeleteIssue"
            @toggle-multi-select="toggleMultiSelect"
            @update:columns="setColumns"
            @reset-columns="resetColumns"
            @add-status-filter="handleAddStatusFilter"
            @add-type-filter="handleAddTypeFilter"
            @add-priority-filter="handleAddPriorityFilter"
          />

          <FilterChips
            v-if="!isSearchActive"
            :status-filters="filters.status"
            :type-filters="filters.type"
            :priority-filters="filters.priority"
            @remove-status="toggleStatus"
            @remove-type="toggleType"
            @remove-priority="togglePriority"
            @clear-all="clearFilters"
          />
        </div>

        <!-- Onboarding: Prerequisites Card -->
        <PrerequisitesCard v-if="showOnboarding" @browse="openFolderPicker" />

        <!-- Normal: Issues Table -->
        <div v-else class="flex-1 overflow-auto p-4">
          <IssueTable
            v-model:selected-ids="selectedIds"
            :issues="filteredIssues"
            :columns="columns"
            :selected-id="selectedIssue?.id"
            :multi-select-mode="multiSelectMode"
            @select="handleSelectIssue"
            @edit="handleEditIssueFromTable"
            @deselect="handleDeselectIssue"
          />

          <div v-if="isLoading" class="text-center text-muted-foreground py-4">
            Loading...
          </div>
        </div>
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
          <div v-if="selectedIssue && !isEditMode && !isCreatingNew" class="p-4 pb-0 space-y-3 border-b border-border">
            <!-- Badges row -->
            <div class="flex items-center gap-1.5 flex-wrap">
              <CopyableId :value="selectedIssue.id" />
              <TypeBadge :type="selectedIssue.type" size="sm" />
              <StatusBadge :status="selectedIssue.status" size="sm" />
              <PriorityBadge :priority="selectedIssue.priority" size="sm" />
            </div>

            <!-- Title -->
            <h3 class="text-sm font-semibold line-clamp-2">{{ selectedIssue.title }}</h3>

            <!-- Action buttons -->
            <div class="flex items-center justify-between pb-3">
              <div class="flex items-center gap-1">
                <Button size="sm" class="h-7 text-xs px-2" @click="handleEditIssue">
                  <svg class="w-3 h-3 mr-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
                    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
                  </svg>
                  Edit
                </Button>
                <Button
                  v-if="selectedIssue.status !== 'closed'"
                  variant="outline"
                  size="sm"
                  class="h-7 text-xs px-2"
                  @click="handleCloseIssue"
                >
                  <svg class="w-3 h-3 mr-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                  Close
                </Button>
              </div>
              <Button
                variant="outline"
                size="sm"
                class="h-7 text-xs px-2 text-destructive hover:bg-destructive hover:text-destructive-foreground"
                @click="handleDeleteIssue"
              >
                <svg class="w-3 h-3 mr-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="3 6 5 6 21 6" />
                  <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                  <line x1="10" y1="11" x2="10" y2="17" />
                  <line x1="14" y1="11" x2="14" y2="17" />
                </svg>
                Delete
              </Button>
            </div>
          </div>

          <!-- Form mode: form gère son propre scroll -->
          <div v-if="isEditMode || isCreatingNew" class="flex-1 min-h-0 p-4 overflow-hidden">
            <IssueForm
              :issue="isCreatingNew ? null : selectedIssue"
              :is-new="isCreatingNew"
              :is-saving="isUpdating"
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
                />
                <CommentSection
                  :comments="selectedIssue.comments || []"
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

          <div v-if="stats" class="space-y-6">
            <div class="grid grid-cols-2 gap-3">
              <KpiCard title="Total" :value="stats.total" :active="activeKpiFilter === null && filters.status.length === 0" @click="handleKpiClick('total')" />
              <KpiCard title="Open" :value="stats.open" color="var(--color-status-open)" :active="activeKpiFilter === 'open'" @click="handleKpiClick('open')" />
              <KpiCard title="In Progress" :value="stats.inProgress" color="var(--color-status-in-progress)" :active="activeKpiFilter === 'in_progress'" @click="handleKpiClick('in_progress')" />
              <KpiCard title="Blocked" :value="stats.blocked" color="var(--color-status-blocked)" :active="activeKpiFilter === 'blocked'" @click="handleKpiClick('blocked')" />
            </div>

            <!-- Collapsible Charts Section -->
            <div class="space-y-2">
              <button
                class="flex items-center gap-2 text-xs text-muted-foreground hover:text-foreground transition-colors w-full"
                @click="isChartsCollapsed = !isChartsCollapsed"
              >
                <svg
                  class="w-3 h-3 transition-transform"
                  :class="{ '-rotate-90': isChartsCollapsed }"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <polyline points="6 9 12 15 18 9" />
                </svg>
                <span class="uppercase tracking-wide">Charts</span>
              </button>
              <div v-show="!isChartsCollapsed" class="space-y-4 pl-5">
                <StatusChart :open="stats.open" :closed="stats.closed" />
                <PriorityChart :by-priority="stats.byPriority" />
              </div>
            </div>

            <!-- Collapsible Ready to Work Section -->
            <div class="space-y-2">
              <button
                class="flex items-center gap-2 text-xs text-muted-foreground hover:text-foreground transition-colors w-full"
                @click="isReadyCollapsed = !isReadyCollapsed"
              >
                <svg
                  class="w-3 h-3 transition-transform"
                  :class="{ '-rotate-90': isReadyCollapsed }"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <polyline points="6 9 12 15 18 9" />
                </svg>
                <span class="uppercase tracking-wide">Ready to Work</span>
                <span class="text-[10px] ml-auto">({{ readyIssues.length }})</span>
              </button>
              <div v-show="!isReadyCollapsed" class="pl-5">
                <QuickList :issues="readyIssues" @select="handleSelectIssue" />
              </div>
            </div>
          </div>

          <div v-else class="flex items-center justify-center py-8">
            <OnboardingCard v-if="showOnboarding" @browse="openFolderPicker" />
            <span v-else class="text-muted-foreground text-sm">Loading...</span>
          </div>
        </div>
      </ScrollArea>

      <!-- Issues Panel -->
      <div v-else-if="mobilePanel === 'issues'" class="flex-1 flex flex-col overflow-hidden">
        <!-- Onboarding: Prerequisites Card -->
        <PrerequisitesCard v-if="showOnboarding" @browse="openFolderPicker" />

        <!-- Normal: Issues Toolbar and Table -->
        <template v-else>
          <div class="p-4 border-b border-border space-y-3">
            <IssuesToolbar
              v-model:search="searchValue"
              :active-status-filters="filters.status"
              :active-type-filters="filters.type"
              :active-priority-filters="filters.priority"
              :has-selection="multiSelectMode ? selectedIds.length > 0 : !!selectedIssue"
              :multi-select-mode="multiSelectMode"
              :selected-count="selectedIds.length"
              :columns="columns"
              @add="handleAddIssue"
              @delete="handleDeleteIssue"
              @toggle-multi-select="toggleMultiSelect"
              @update:columns="setColumns"
              @reset-columns="resetColumns"
              @add-status-filter="handleAddStatusFilter"
              @add-type-filter="handleAddTypeFilter"
              @add-priority-filter="handleAddPriorityFilter"
            />

            <FilterChips
              v-if="!isSearchActive"
              :status-filters="filters.status"
              :type-filters="filters.type"
              :priority-filters="filters.priority"
              @remove-status="toggleStatus"
              @remove-type="toggleType"
              @remove-priority="togglePriority"
              @clear-all="clearFilters"
            />
          </div>

          <div class="flex-1 overflow-auto p-4">
            <IssueTable
              v-model:selected-ids="selectedIds"
              :issues="filteredIssues"
              :columns="columns"
              :selected-id="selectedIssue?.id"
              :multi-select-mode="multiSelectMode"
              @select="handleSelectIssue"
              @edit="handleEditIssueFromTable"
              @deselect="handleDeselectIssue"
            />
          </div>
        </template>
      </div>

      <!-- Details Panel -->
      <div v-else-if="mobilePanel === 'details'" class="flex-1 flex flex-col overflow-hidden">
        <!-- Fixed header for issue preview -->
        <div v-if="selectedIssue && !isEditMode && !isCreatingNew" class="p-4 pb-0 space-y-3 border-b border-border">
          <!-- Badges row -->
          <div class="flex items-center gap-1.5 flex-wrap">
            <CopyableId :value="selectedIssue.id" />
            <TypeBadge :type="selectedIssue.type" size="sm" />
            <StatusBadge :status="selectedIssue.status" size="sm" />
            <PriorityBadge :priority="selectedIssue.priority" size="sm" />
          </div>

          <!-- Title -->
          <h3 class="text-sm font-semibold line-clamp-2">{{ selectedIssue.title }}</h3>

          <!-- Action buttons -->
          <div class="flex items-center justify-between pb-3">
            <div class="flex items-center gap-1">
              <Button size="sm" class="h-7 text-xs px-2" @click="handleEditIssue">
                <svg class="w-3 h-3 mr-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
                  <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
                </svg>
                Edit
              </Button>
              <Button
                v-if="selectedIssue.status !== 'closed'"
                variant="outline"
                size="sm"
                class="h-7 text-xs px-2"
                @click="handleCloseIssue"
              >
                <svg class="w-3 h-3 mr-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="20 6 9 17 4 12" />
                </svg>
                Close
              </Button>
            </div>
            <Button
              variant="outline"
              size="sm"
              class="h-7 text-xs px-2 text-destructive hover:bg-destructive hover:text-destructive-foreground"
              @click="handleDeleteIssue"
            >
              <svg class="w-3 h-3 mr-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6" />
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                <line x1="10" y1="11" x2="10" y2="17" />
                <line x1="14" y1="11" x2="14" y2="17" />
              </svg>
              Delete
            </Button>
          </div>
        </div>

        <!-- Form mode: form gère son propre scroll -->
        <div v-if="isEditMode || isCreatingNew" class="flex-1 min-h-0 p-4 overflow-hidden">
          <IssueForm
            :issue="isCreatingNew ? null : selectedIssue"
            :is-new="isCreatingNew"
            :is-saving="isUpdating"
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
              />
              <CommentSection
                :comments="selectedIssue.comments || []"
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

    <!-- Footer (outside zoomable content) -->
    <footer class="px-4 py-2 border-t border-border bg-card flex items-center justify-between text-xs text-muted-foreground font-mono">
      <!-- Sync indicator -->
      <div class="w-20 flex items-center gap-1.5">
        <svg
          :class="['w-3 h-3 transition-all', isSyncing ? 'animate-spin text-primary' : 'text-muted-foreground/50']"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" />
          <path d="M21 3v5h-5" />
        </svg>
        <span v-if="isSyncing" class="text-primary">Sync...</span>
      </div>

      <!-- Contenu centré -->
      <div class="flex items-center justify-center gap-1">
        <span>Powered by</span>
        <a
          href="https://github.com/steveyegge/beads"
          target="_blank"
          rel="noopener noreferrer"
          class="text-primary hover:underline inline-flex items-center gap-1"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
          </svg>
          beads
        </a>
        <span>by Steve Yegge / Inspired by</span>
        <a
          href="https://marketplace.visualstudio.com/items?itemName=planet57.vscode-beads"
          target="_blank"
          rel="noopener noreferrer"
          class="text-primary hover:underline"
        >vscode-beads</a>
        <span>plugin by Planet57 / Vibe coded with</span>
        <a
          href="https://claude.ai/code"
          target="_blank"
          rel="noopener noreferrer"
          class="text-primary hover:underline"
        >Claude Code</a>
        <span>by Laurent Chapin</span>
      </div>

      <!-- Version à droite -->
      <div class="w-20 text-right">
        <UpdateIndicator />
      </div>
    </footer>

    <!-- Delete Confirmation Dialog -->
    <ConfirmDialog
      v-model:open="isDeleteDialogOpen"
      title="Supprimer"
      confirm-text="Supprimer"
      cancel-text="Annuler"
      variant="destructive"
      :is-loading="isDeleting"
      @confirm="confirmDelete"
    >
      <template #description>
        <p class="text-sm text-muted-foreground">
          Vous êtes sur le point de supprimer définitivement
          {{ deleteTargetTitles.length > 1 ? 'les issues suivantes' : 'l\'issue' }} :
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
            ... et {{ deleteTargetTitles.length - 5 }} autre{{ deleteTargetTitles.length - 5 > 1 ? 's' : '' }} ({{ deleteTargetTitles.length }} au total)
          </p>
        </div>
        <p class="mt-3 text-sm text-muted-foreground">
          Cette action est irréversible.
        </p>
      </template>
    </ConfirmDialog>

    <!-- Onboarding Folder Picker -->
    <FolderPicker
      v-model:open="isOnboardingPickerOpen"
      current-path="~"
      @select="handleOnboardingFolderSelect"
    />
  </div>
</template>
