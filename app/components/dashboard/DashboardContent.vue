<script setup lang="ts">
import type { Issue } from '~/types/issue'
import type { DashboardStats } from '~/types/issue'
import KpiCard from '~/components/dashboard/KpiCard.vue'
import StatusChart from '~/components/dashboard/StatusChart.vue'
import PriorityChart from '~/components/dashboard/PriorityChart.vue'
import QuickList from '~/components/dashboard/QuickList.vue'
import OnboardingCard from '~/components/dashboard/OnboardingCard.vue'

type KpiFilter = 'total' | 'open' | 'in_progress' | 'blocked'

const props = withDefaults(defineProps<{
  stats: DashboardStats | null
  readyIssues: Issue[]
  inProgressIssues: Issue[]
  kpiGridCols?: 2 | 4
  activeKpiFilter: KpiFilter | null
  statusFilters: string[]
  showOnboarding?: boolean
  hideKpis?: boolean
}>(), {
  kpiGridCols: 4,
  showOnboarding: false,
  hideKpis: false,
})

const emit = defineEmits<{
  'select-issue': [issue: Issue]
  'kpi-click': [kpi: KpiFilter]
  browse: []
}>()

// Collapsible state (per-project, singleton)
const isChartsCollapsed = useProjectStorage('chartsCollapsed', true)
const isInProgressCollapsed = useProjectStorage('inProgressCollapsed', true)
const isReadyCollapsed = useProjectStorage('readyCollapsed', true)
</script>

<template>
  <template v-if="stats">
    <!-- KPI cards (hidden in desktop scrollable section where KPIs are in the fixed section) -->
    <div v-if="!hideKpis" :class="['grid', kpiGridCols === 4 ? 'grid-cols-4 gap-1.5' : 'grid-cols-2 gap-3']">
      <KpiCard title="Total" :value="stats.total" :active="activeKpiFilter === null && statusFilters.length === 0" @click="emit('kpi-click', 'total')" />
      <KpiCard title="Open" :value="stats.open" color="var(--color-status-open)" :active="activeKpiFilter === 'open'" @click="emit('kpi-click', 'open')" />
      <KpiCard title="In Progress" :value="stats.inProgress" color="var(--color-status-in-progress)" :active="activeKpiFilter === 'in_progress'" @click="emit('kpi-click', 'in_progress')" />
      <KpiCard title="Blocked" :value="stats.blocked" color="var(--color-status-blocked)" :active="activeKpiFilter === 'blocked'" @click="emit('kpi-click', 'blocked')" />
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
