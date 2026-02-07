import type { Issue, DashboardStats, IssueType, IssuePriority } from '~/types/issue'
import { bdReady } from '~/utils/bd-api'

export function useDashboard() {
  const stats = ref<DashboardStats | null>(null)
  const readyIssues = ref<Issue[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const { beadsPath } = useBeadsPath()

  // Helper to get the current path
  const getPath = () => beadsPath.value && beadsPath.value !== '.' ? beadsPath.value : undefined

  // Calculate stats from issues array (no API call needed)
  const computeStatsFromIssues = (issues: Issue[]): DashboardStats => {
    const stats: DashboardStats = {
      total: issues.length,
      open: 0,
      inProgress: 0,
      blocked: 0,
      closed: 0,
      ready: 0,
      byType: {
        bug: 0,
        task: 0,
        feature: 0,
        epic: 0,
        chore: 0,
      },
      byPriority: {
        p0: 0,
        p1: 0,
        p2: 0,
        p3: 0,
        p4: 0,
      },
    }

    for (const issue of issues) {
      // Count by status
      switch (issue.status) {
        case 'open':
          stats.open++
          break
        case 'in_progress':
          stats.inProgress++
          break
        case 'blocked':
          stats.blocked++
          break
        case 'closed':
          stats.closed++
          break
      }

      // Count by type
      if (issue.type in stats.byType) {
        stats.byType[issue.type]++
      }

      // Count by priority
      if (issue.priority in stats.byPriority) {
        stats.byPriority[issue.priority]++
      }
    }

    return stats
  }

  // Fetch stats - now accepts issues to avoid extra API calls
  const fetchStats = async (issues?: Issue[]) => {
    isLoading.value = true
    error.value = null

    try {
      const path = getPath()

      // Preserve current ready count to avoid flash
      const currentReady = stats.value?.ready ?? 0

      // Compute stats from issues (even if empty array)
      if (issues !== undefined) {
        stats.value = computeStatsFromIssues(issues)
      } else {
        // Fallback: initialize with empty stats if no issues provided
        stats.value = computeStatsFromIssues([])
      }

      // Restore ready count while waiting for bdReady
      stats.value.ready = currentReady

      // Only fetch ready issues from API (specific filtering logic)
      const readyData = await bdReady(path)
      readyIssues.value = readyData || []

      // Update ready count in stats
      stats.value.ready = readyIssues.value.length
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch dashboard stats'
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Update dashboard from pre-fetched poll data (no API calls needed).
   * Used by the batched polling system to avoid separate bdReady call.
   */
  const updateFromPollData = (issues: Issue[], readyData: Issue[]) => {
    stats.value = computeStatsFromIssues(issues)
    readyIssues.value = readyData || []
    stats.value.ready = readyIssues.value.length
  }

  // Clear all stats data (used when removing last favorite)
  const clearStats = () => {
    stats.value = null
    readyIssues.value = []
    error.value = null
  }

  return {
    stats,
    readyIssues,
    isLoading,
    error,
    fetchStats,
    computeStatsFromIssues,
    updateFromPollData,
    clearStats,
  }
}
