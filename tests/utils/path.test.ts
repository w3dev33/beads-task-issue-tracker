import { describe, it, expect } from 'vitest'
import { splitPath, getPathSeparator, getFolderName, getParentPath } from '~/utils/path'

// ---------------------------------------------------------------------------
// splitPath
// ---------------------------------------------------------------------------
describe('splitPath', () => {
  it('splits Unix paths', () => {
    expect(splitPath('/home/dev/project')).toEqual(['', 'home', 'dev', 'project'])
  })

  it('splits Windows paths', () => {
    expect(splitPath('C:\\dev\\my-app')).toEqual(['C:', 'dev', 'my-app'])
  })

  it('splits mixed separators', () => {
    expect(splitPath('/path\\to/folder')).toEqual(['', 'path', 'to', 'folder'])
  })

  it('handles single component', () => {
    expect(splitPath('folder')).toEqual(['folder'])
  })

  it('handles trailing separator', () => {
    expect(splitPath('/path/to/')).toEqual(['', 'path', 'to', ''])
  })

  it('handles root path', () => {
    expect(splitPath('/')).toEqual(['', ''])
  })
})

// ---------------------------------------------------------------------------
// getPathSeparator
// ---------------------------------------------------------------------------
describe('getPathSeparator', () => {
  it('detects Unix separator', () => {
    expect(getPathSeparator('/home/dev')).toBe('/')
  })

  it('detects Windows separator', () => {
    expect(getPathSeparator('C:\\dev\\project')).toBe('\\')
  })

  it('defaults to / when no separator present', () => {
    expect(getPathSeparator('folder')).toBe('/')
  })

  it('prefers \\ when both are present', () => {
    expect(getPathSeparator('/path\\to')).toBe('\\')
  })
})

// ---------------------------------------------------------------------------
// getFolderName
// ---------------------------------------------------------------------------
describe('getFolderName', () => {
  it('extracts last component from Unix path', () => {
    expect(getFolderName('/home/dev/project')).toBe('project')
  })

  it('extracts last component from Windows path', () => {
    expect(getFolderName('C:\\dev\\my-app')).toBe('my-app')
  })

  it('returns the string itself for single component', () => {
    expect(getFolderName('project')).toBe('project')
  })

  it('returns the full path for trailing separator', () => {
    // splitPath('/path/to/') → ['', 'path', 'to', ''] → pop() returns ''
    // fallback to path since '' is falsy
    expect(getFolderName('/path/to/')).toBe('/path/to/')
  })
})

// ---------------------------------------------------------------------------
// getParentPath
// ---------------------------------------------------------------------------
describe('getParentPath', () => {
  it('returns parent for Unix path', () => {
    expect(getParentPath('/home/dev/project')).toBe('/home/dev')
  })

  it('returns parent for Windows path', () => {
    expect(getParentPath('C:\\dev\\my-app')).toBe('C:\\dev')
  })

  it('returns separator for root-level path', () => {
    expect(getParentPath('/folder')).toBe('/')
  })

  it('preserves Windows separator in result', () => {
    expect(getParentPath('C:\\folder')).toBe('C:')
  })

  it('returns separator for single component', () => {
    expect(getParentPath('folder')).toBe('/')
  })
})
