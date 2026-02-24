import { describe, it, expect } from 'vitest'
import { splitRefs, joinRefs, REF_SEPARATOR } from '~/utils/attachment-encoding'

describe('splitRefs', () => {
  it('splits pipe-separated paths', () => {
    expect(splitRefs('/path/a.png|/path/b.jpg')).toEqual(['/path/a.png', '/path/b.jpg'])
  })

  it('splits legacy newline-separated paths', () => {
    expect(splitRefs('/path/a.png\n/path/b.jpg')).toEqual(['/path/a.png', '/path/b.jpg'])
  })

  it('handles mixed separators', () => {
    expect(splitRefs('/path/a.png\n/path/b.jpg|/path/c.gif')).toEqual([
      '/path/a.png',
      '/path/b.jpg',
      '/path/c.gif',
    ])
  })

  it('trims whitespace around refs', () => {
    expect(splitRefs(' /path/a.png | /path/b.jpg ')).toEqual(['/path/a.png', '/path/b.jpg'])
  })

  it('filters out empty entries', () => {
    expect(splitRefs('|/path/a.png||/path/b.jpg|')).toEqual(['/path/a.png', '/path/b.jpg'])
    expect(splitRefs('\n/path/a.png\n\n/path/b.jpg\n')).toEqual(['/path/a.png', '/path/b.jpg'])
  })

  it('returns empty array for undefined', () => {
    expect(splitRefs(undefined)).toEqual([])
  })

  it('returns empty array for empty string', () => {
    expect(splitRefs('')).toEqual([])
  })

  it('handles single ref', () => {
    expect(splitRefs('/path/a.png')).toEqual(['/path/a.png'])
  })

  it('preserves cleared: sentinels', () => {
    expect(splitRefs('cleared:abc-123|/path/a.png')).toEqual(['cleared:abc-123', '/path/a.png'])
  })
})

describe('joinRefs', () => {
  it('joins with pipe separator', () => {
    expect(joinRefs(['/path/a.png', '/path/b.jpg'])).toBe('/path/a.png|/path/b.jpg')
  })

  it('filters out empty strings', () => {
    expect(joinRefs(['/path/a.png', '', '/path/b.jpg'])).toBe('/path/a.png|/path/b.jpg')
  })

  it('returns empty string for empty array', () => {
    expect(joinRefs([])).toBe('')
  })

  it('returns single ref without separator', () => {
    expect(joinRefs(['/path/a.png'])).toBe('/path/a.png')
  })
})

describe('REF_SEPARATOR', () => {
  it('is pipe character', () => {
    expect(REF_SEPARATOR).toBe('|')
  })
})
