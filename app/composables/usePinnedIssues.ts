import type { Issue } from '~/types/issue'
import { useProjectStorage } from '~/composables/useProjectStorage'

export type PinnedSortMode = 'added' | 'updated' | 'manual'

// Singleton state — shared across all components that call usePinnedIssues()
const pinnedIssueIds = useProjectStorage<string[]>('pinnedIssueIds', [])
const pinnedSortMode = useProjectStorage<PinnedSortMode>('pinnedSortMode', 'added')

export function usePinnedIssues() {
  const isPinned = (issueId: string): boolean => {
    return pinnedIssueIds.value.includes(issueId)
  }

  const togglePin = (issueId: string): void => {
    if (isPinned(issueId)) {
      pinnedIssueIds.value = pinnedIssueIds.value.filter(id => id !== issueId)
    } else {
      pinnedIssueIds.value = [...pinnedIssueIds.value, issueId]
    }
  }

  const reorderPinned = (newOrder: string[]): void => {
    pinnedIssueIds.value = newOrder
    pinnedSortMode.value = 'manual'
  }

  const toggleSortMode = (): void => {
    if (pinnedSortMode.value === 'added') pinnedSortMode.value = 'updated'
    else if (pinnedSortMode.value === 'updated') pinnedSortMode.value = 'manual'
    else pinnedSortMode.value = 'added'
  }

  const getPinnedIssues = (allIssues: Issue[]): Issue[] => {
    const issueMap = new Map(allIssues.map(i => [i.id, i]))
    const filtered = pinnedIssueIds.value
      .map(id => issueMap.get(id))
      .filter((i): i is Issue => !!i && i.status !== 'closed' && i.status !== 'tombstone')

    if (pinnedSortMode.value === 'added') {
      // Reverse pinned order: most recently pinned first
      return [...filtered].reverse()
    }
    if (pinnedSortMode.value === 'updated') {
      return [...filtered].sort((a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime())
    }
    return filtered // manual: keep pinned order as-is
  }

  return {
    pinnedIssueIds,
    pinnedSortMode,
    isPinned,
    togglePin,
    reorderPinned,
    toggleSortMode,
    getPinnedIssues,
  }
}
