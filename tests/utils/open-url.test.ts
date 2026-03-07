import { describe, it, expect } from 'vitest'
import { isValidUrl, isLocalPath, normalizeUrl } from '~/utils/open-url'

// ---------------------------------------------------------------------------
// isValidUrl
// ---------------------------------------------------------------------------
describe('isValidUrl', () => {
  it('accepts http URLs', () => {
    expect(isValidUrl('http://example.com')).toBe(true)
  })

  it('accepts https URLs', () => {
    expect(isValidUrl('https://example.com')).toBe(true)
  })

  it('accepts URLs with paths and query params', () => {
    expect(isValidUrl('https://example.com/path?q=1&r=2#hash')).toBe(true)
  })

  it('rejects ftp protocol', () => {
    expect(isValidUrl('ftp://files.example.com')).toBe(false)
  })

  it('rejects javascript protocol', () => {
    expect(isValidUrl('javascript:alert(1)')).toBe(false)
  })

  it('rejects file protocol', () => {
    expect(isValidUrl('file:///etc/passwd')).toBe(false)
  })

  it('rejects malformed URLs', () => {
    expect(isValidUrl('not a url')).toBe(false)
    expect(isValidUrl('')).toBe(false)
  })

  it('rejects bare domains (no protocol)', () => {
    expect(isValidUrl('example.com')).toBe(false)
    expect(isValidUrl('www.example.com')).toBe(false)
  })
})

// ---------------------------------------------------------------------------
// isLocalPath
// ---------------------------------------------------------------------------
describe('isLocalPath', () => {
  it('detects absolute Unix paths', () => {
    expect(isLocalPath('/home/dev/file.png')).toBe(true)
  })

  it('detects relative paths with ./', () => {
    expect(isLocalPath('./image.png')).toBe(true)
  })

  it('detects parent-relative paths with ../', () => {
    expect(isLocalPath('../assets/logo.png')).toBe(true)
  })

  it('detects screenshots/ prefix', () => {
    expect(isLocalPath('screenshots/capture.png')).toBe(true)
  })

  it('rejects URLs', () => {
    expect(isLocalPath('https://example.com/img.png')).toBe(false)
    expect(isLocalPath('http://example.com')).toBe(false)
  })

  it('rejects bare filenames', () => {
    expect(isLocalPath('image.png')).toBe(false)
  })

  it('rejects empty string', () => {
    expect(isLocalPath('')).toBe(false)
  })
})

// ---------------------------------------------------------------------------
// normalizeUrl
// ---------------------------------------------------------------------------
describe('normalizeUrl', () => {
  it('adds https:// to www. URLs', () => {
    expect(normalizeUrl('www.example.com')).toBe('https://www.example.com')
  })

  it('leaves http:// URLs unchanged', () => {
    expect(normalizeUrl('http://example.com')).toBe('http://example.com')
  })

  it('leaves https:// URLs unchanged', () => {
    expect(normalizeUrl('https://example.com')).toBe('https://example.com')
  })

  it('leaves non-www strings unchanged', () => {
    expect(normalizeUrl('example.com')).toBe('example.com')
    expect(normalizeUrl('/local/path')).toBe('/local/path')
  })
})
