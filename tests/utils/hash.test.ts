import { describe, it, expect } from 'vitest'
import { hashPath } from '~/utils/hash'

describe('hashPath', () => {
  it('returns an 8-character hex string', () => {
    const result = hashPath('/home/dev/project')
    expect(result).toMatch(/^[0-9a-f]{8}$/)
  })

  it('is deterministic (same input → same output)', () => {
    expect(hashPath('/home/dev/project')).toBe(hashPath('/home/dev/project'))
  })

  it('produces different hashes for different paths', () => {
    const a = hashPath('/home/dev/project-a')
    const b = hashPath('/home/dev/project-b')
    expect(a).not.toBe(b)
  })

  it('handles empty string', () => {
    const result = hashPath('')
    expect(result).toMatch(/^[0-9a-f]{8}$/)
  })

  it('handles long paths', () => {
    const long = '/a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z'
    const result = hashPath(long)
    expect(result).toMatch(/^[0-9a-f]{8}$/)
  })

  it('handles special characters', () => {
    const result = hashPath('/home/dev/my project (2)')
    expect(result).toMatch(/^[0-9a-f]{8}$/)
  })
})
