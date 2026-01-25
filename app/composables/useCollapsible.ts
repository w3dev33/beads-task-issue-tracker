import type { CollapsibleState } from '~/types/issue'

const defaultState: CollapsibleState = {
  dashboard: true,
  issues: true,
  details: true,
}

export function useCollapsible() {
  const state = useLocalStorage<CollapsibleState>('beads:collapsed', defaultState)

  const toggle = (section: keyof CollapsibleState) => {
    state.value[section] = !state.value[section]
  }

  const expand = (section: keyof CollapsibleState) => {
    state.value[section] = true
  }

  const collapse = (section: keyof CollapsibleState) => {
    state.value[section] = false
  }

  const expandAll = () => {
    state.value = { dashboard: true, issues: true, details: true }
  }

  const collapseAll = () => {
    state.value = { dashboard: false, issues: false, details: false }
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
