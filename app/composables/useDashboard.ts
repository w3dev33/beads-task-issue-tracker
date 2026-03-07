import type { Issue, DashboardStats } from '~/types/issue'
import { bdReady } from '~/utils/bd-api'
import { computeStatsFromIssues } from '~/utils/issue-helpers'

export function useDashboard() {
  const stats = ref<DashboardStats | null>(null)
  const readyIssues = ref<Issue[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const { beadsPath } = useBeadsPath()

  // Helper to get the current path
  const getPath = () => beadsPath.value && beadsPath.value !== '.' ? beadsPath.value : undefined

  // Prefetch bdReady data — call this before fetchIssues to overlap the two API calls
  const prefetchReady = () => bdReady(getPath()).catch(() => [] as Issue[])

  // Fetch stats - now accepts issues to avoid extra API calls
  // Optional prefetchedReady: a Promise<Issue[]> from prefetchReady() already in flight
  const fetchStats = async (issues?: Issue[], prefetchedReady?: Promise<Issue[]>) => {
    isLoading.value = true
    error.value = null

    try {
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

      // Use prefetched ready data if available, otherwise fetch now
      const readyData = prefetchedReady
        ? await prefetchedReady
        : await bdReady(getPath())
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
    prefetchReady,
    fetchStats,
    computeStatsFromIssues,
    updateFromPollData,
    clearStats,
  }
}
