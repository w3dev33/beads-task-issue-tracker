import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

const readSource = (relativePath: string) => readFileSync(resolve(process.cwd(), relativePath), 'utf8')

describe('floating overlay zoom contract', () => {
  it('provides an unzoomed overlay root outside the zoomed app content', () => {
    const page = readSource('app/pages/index.vue')

    expect(page).toContain('id="floating-overlay-root"')
  })

  it('routes all shared floating primitives through the unzoomed overlay root', () => {
    const dropdown = readSource('app/components/ui/dropdown-menu/DropdownMenuContent.vue')
    const tooltip = readSource('app/components/ui/tooltip/TooltipContent.vue')
    const select = readSource('app/components/ui/select/SelectContent.vue')

    expect(dropdown).toContain('to="#floating-overlay-root"')
    expect(tooltip).toContain('to="#floating-overlay-root"')
    expect(select).toContain('to="#floating-overlay-root"')
    expect(dropdown).toContain('var(--app-zoom, 1)')
    expect(tooltip).toContain('var(--app-zoom, 1)')
    expect(select).toContain('var(--app-zoom, 1)')
    expect(readSource('app/composables/useZoom.ts')).toContain("setProperty('--app-zoom'")
  })
})
