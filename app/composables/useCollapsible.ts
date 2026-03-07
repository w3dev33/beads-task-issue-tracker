import type { CollapsibleState } from '~/types/issue'
import { useProjectStorage } from '~/composables/useProjectStorage'

// Per-project collapsed state (issues and details panels)
interface ProjectCollapsibleState {
  issues: boolean
  details: boolean
}

const defaultProjectState: ProjectCollapsibleState = {
  issues: true,
  details: true,
}

export function useCollapsible() {
  // Dashboard collapsed state is global (user preference)
  const dashboardState = useLocalStorage<boolean>('beads:collapsed:dashboard', true)

  // Issues and details collapsed states are per-project
  const projectState = useProjectStorage<ProjectCollapsibleState>('collapsed', defaultProjectState)

  // Combined state for backwards compatibility
  const state = computed<CollapsibleState>(() => ({
    dashboard: dashboardState.value,
    issues: projectState.value.issues,
    details: projectState.value.details,
  }))

  const toggle = (section: keyof CollapsibleState) => {
    if (section === 'dashboard') {
      dashboardState.value = !dashboardState.value
    } else {
      projectState.value[section] = !projectState.value[section]
    }
  }

  const expand = (section: keyof CollapsibleState) => {
    if (section === 'dashboard') {
      dashboardState.value = true
    } else {
      projectState.value[section] = true
    }
  }

  const collapse = (section: keyof CollapsibleState) => {
    if (section === 'dashboard') {
      dashboardState.value = false
    } else {
      projectState.value[section] = false
    }
  }

  const expandAll = () => {
    dashboardState.value = true
    projectState.value = { issues: true, details: true }
  }

  const collapseAll = () => {
    dashboardState.value = false
    projectState.value = { issues: false, details: false }
  }

  return {
    state,
    toggle,
    expand,
    collapse,
    expandAll,
    collapseAll,
  }
}
