<script setup lang="ts">
import type { Issue } from '~/types/issue'
import type { DashboardStats } from '~/types/issue'
import KpiCard from '~/components/dashboard/KpiCard.vue'
import StatusChart from '~/components/dashboard/StatusChart.vue'
import PriorityChart from '~/components/dashboard/PriorityChart.vue'
import QuickList from '~/components/dashboard/QuickList.vue'
import PinnedList from '~/components/dashboard/PinnedList.vue'
import OnboardingCard from '~/components/dashboard/OnboardingCard.vue'
import type { PinnedSortMode } from '~/composables/usePinnedIssues'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '~/components/ui/tooltip'

type KpiFilter = 'total' | 'open' | 'in_progress' | 'blocked' | 'workflow'

const props = withDefaults(defineProps<{
  stats: DashboardStats | null
  readyIssues: Issue[]
  inProgressIssues: Issue[]
  blockedIssues: Issue[]
  pinnedIssues: Issue[]
  pinnedSortMode?: PinnedSortMode
  kpiGridCols?: 2 | 5
  activeKpiFilter: KpiFilter | null
  showOnboarding?: boolean
  hideKpis?: boolean
}>(), {
  kpiGridCols: 5,
  showOnboarding: false,
  hideKpis: false,
})

const emit = defineEmits<{
  'select-issue': [issue: Issue]
  'kpi-click': [kpi: KpiFilter]
  'reorder-pinned': [newOrder: string[]]
  'unpin': [issueId: string]
  'toggle-pinned-sort': []
  browse: []
}>()

// Collapsible state (per-project, singleton)
const isChartsCollapsed = useProjectStorage('chartsCollapsed', true)
const isInProgressCollapsed = useProjectStorage('inProgressCollapsed', true)
const isBlockedCollapsed = useProjectStorage('blockedCollapsed', true)
const isPinnedCollapsed = useProjectStorage('pinnedCollapsed', false)
const isReadyCollapsed = useProjectStorage('readyCollapsed', true)
</script>

<template>
  <template v-if="stats">
    <!-- KPI cards (hidden in desktop scrollable section where KPIs are in the fixed section) -->
    <div v-if="!hideKpis" :class="['flex flex-wrap p-0.5 -m-0.5', kpiGridCols === 5 ? 'gap-1.5' : 'gap-3']">
      <KpiCard title="Workflow" :value="stats.workflow" color="var(--color-status-deferred)" :active="activeKpiFilter === 'workflow'" @click="emit('kpi-click', 'workflow')" />
      <KpiCard title="Open" :value="stats.open" color="var(--color-status-open)" :active="activeKpiFilter === 'open'" @click="emit('kpi-click', 'open')" />
      <KpiCard title="In Progress" :value="stats.inProgress" color="var(--color-status-in-progress)" :active="activeKpiFilter === 'in_progress'" @click="emit('kpi-click', 'in_progress')" />
      <KpiCard title="Blocked" :value="stats.blocked" color="var(--color-status-blocked)" :active="activeKpiFilter === 'blocked'" @click="emit('kpi-click', 'blocked')" />
      <KpiCard title="All" :value="stats.total" :active="activeKpiFilter === 'total'" @click="emit('kpi-click', 'total')" />
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

    <!-- Collapsible In Progress Section -->
    <div v-if="inProgressIssues.length > 0" class="space-y-2">
      <button
        class="flex items-center gap-2 text-xs text-muted-foreground hover:text-foreground transition-colors w-full"
        @click="isInProgressCollapsed = !isInProgressCollapsed"
      >
        <svg
          class="w-3 h-3 transition-transform"
          :class="{ '-rotate-90': isInProgressCollapsed }"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
        <span class="uppercase tracking-wide">In Progress</span>
        <span class="text-[10px] ml-auto">({{ inProgressIssues.length }})</span>
      </button>
      <div v-show="!isInProgressCollapsed" class="pl-5">
        <QuickList :issues="inProgressIssues" @select="emit('select-issue', $event)" />
      </div>
    </div>

    <!-- Collapsible Blocked Section -->
    <div v-if="blockedIssues.length > 0" class="space-y-2">
      <button
        class="flex items-center gap-2 text-xs text-muted-foreground hover:text-foreground transition-colors w-full"
        @click="isBlockedCollapsed = !isBlockedCollapsed"
      >
        <svg
          class="w-3 h-3 transition-transform"
          :class="{ '-rotate-90': isBlockedCollapsed }"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
        <span class="uppercase tracking-wide">Blocked</span>
        <span class="text-[10px] ml-auto">({{ blockedIssues.length }})</span>
      </button>
      <div v-show="!isBlockedCollapsed" class="pl-5">
        <QuickList :issues="blockedIssues" @select="emit('select-issue', $event)" />
      </div>
    </div>

    <!-- Collapsible Pinned Section -->
    <div v-if="pinnedIssues.length > 0" class="space-y-2">
      <div class="flex items-center gap-2 text-xs text-muted-foreground w-full">
        <button
          class="flex items-center gap-2 hover:text-foreground transition-colors"
          @click="isPinnedCollapsed = !isPinnedCollapsed"
        >
          <svg
            class="w-3 h-3 transition-transform"
            :class="{ '-rotate-90': isPinnedCollapsed }"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <polyline points="6 9 12 15 18 9" />
          </svg>
          <span class="uppercase tracking-wide">Check This Out</span>
        </button>
        <span class="text-[10px] ml-auto">({{ pinnedIssues.length }})</span>
        <!-- Sort mode toggle -->
        <template v-if="!isPinnedCollapsed">
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger as-child>
                <button
                  class="text-muted-foreground hover:text-foreground transition-colors p-0.5 rounded"
                  @click.stop="emit('toggle-pinned-sort')"
                >
                  <!-- Pin icon (by date added) -->
                  <svg v-if="pinnedSortMode === 'added'" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M9 4v6l-2 4h10l-2-4V4" /><line x1="12" y1="16" x2="12" y2="21" /><line x1="8" y1="4" x2="16" y2="4" />
                  </svg>
                  <!-- Clock icon (by date updated) -->
                  <svg v-else-if="pinnedSortMode === 'updated'" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" />
                  </svg>
                  <!-- Grip icon for manual mode -->
                  <svg v-else class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="9" cy="6" r="1" fill="currentColor" /><circle cx="15" cy="6" r="1" fill="currentColor" />
                    <circle cx="9" cy="12" r="1" fill="currentColor" /><circle cx="15" cy="12" r="1" fill="currentColor" />
                    <circle cx="9" cy="18" r="1" fill="currentColor" /><circle cx="15" cy="18" r="1" fill="currentColor" />
                  </svg>
                </button>
              </TooltipTrigger>
              <TooltipContent>
                {{ pinnedSortMode === 'added' ? 'By pin date (click for updated)' : pinnedSortMode === 'updated' ? 'By updated (click for manual)' : 'Manual order (click for pin date)' }}
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </template>
      </div>
      <div v-show="!isPinnedCollapsed" class="pl-5">
        <PinnedList
          :issues="pinnedIssues"
          :drag-enabled="pinnedSortMode === 'manual'"
          @select="emit('select-issue', $event)"
          @reorder="emit('reorder-pinned', $event)"
          @unpin="emit('unpin', $event)"
        />
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
        <QuickList :issues="readyIssues" @select="emit('select-issue', $event)" />
      </div>
    </div>
  </template>

  <div v-else class="flex items-center justify-center py-8">
    <OnboardingCard v-if="showOnboarding" @browse="emit('browse')" />
    <span v-else class="text-muted-foreground text-sm">Loading...</span>
  </div>
</template>
