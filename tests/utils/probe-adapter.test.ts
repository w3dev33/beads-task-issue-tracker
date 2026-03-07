import { describe, it, expect } from 'vitest'
import { matchProbeProject } from '../../app/utils/probe-adapter'
import type { ProbeProject } from '../../app/utils/probe-adapter'

describe('matchProbeProject', () => {
  const projects: ProbeProject[] = [
    { name: 'my-project', path: '/home/user/my-project/.beads' },
    { name: 'other', path: '/home/user/other/.beads' },
    { name: 'raw-path', path: '/tmp/raw-path' },
  ]

  it('matches exact .beads path', () => {
    const result = matchProbeProject(projects, '/home/user/my-project/.beads')
    expect(result?.name).toBe('my-project')
  })

  it('matches project path by appending /.beads', () => {
    const result = matchProbeProject(projects, '/home/user/my-project')
    expect(result?.name).toBe('my-project')
  })

  it('matches when probe stores path without .beads suffix', () => {
    const result = matchProbeProject(projects, '/tmp/raw-path')
    expect(result?.name).toBe('raw-path')
  })

  it('returns undefined when no match found', () => {
    const result = matchProbeProject(projects, '/home/user/unknown')
    expect(result).toBeUndefined()
  })

  it('returns undefined for empty project list', () => {
    const result = matchProbeProject([], '/home/user/my-project')
    expect(result).toBeUndefined()
  })

  it('does not double-append .beads if already present', () => {
    // Input already ends with .beads — should NOT try /path/.beads/.beads
    const result = matchProbeProject(projects, '/home/user/my-project/.beads')
    expect(result?.name).toBe('my-project')
  })

  it('distinguishes between similar paths', () => {
    const result = matchProbeProject(projects, '/home/user/other')
    expect(result?.name).toBe('other')
  })

  it('does not match partial path overlap', () => {
    const result = matchProbeProject(projects, '/home/user/my')
    expect(result).toBeUndefined()
  })
})
