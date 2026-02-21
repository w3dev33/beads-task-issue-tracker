import { describe, it, expect } from 'vitest'
import {
  extractImagesFromMarkdown,
  isImagePath,
  isMarkdownPath,
  isUrl,
  extractImagesFromExternalRef,
  extractMarkdownFromExternalRef,
  extractNonImageRefs,
  renderMarkdown,
} from '~/utils/markdown'

// ---------------------------------------------------------------------------
// isImagePath
// ---------------------------------------------------------------------------
describe('isImagePath', () => {
  it('returns false for empty/falsy input', () => {
    expect(isImagePath('')).toBe(false)
    expect(isImagePath(undefined as unknown as string)).toBe(false)
  })

  it.each([
    'photo.png', 'photo.jpg', 'photo.jpeg', 'photo.gif',
    'photo.webp', 'photo.bmp', 'photo.svg', 'photo.ico',
    'photo.tiff', 'photo.tif',
  ])('recognises .%s extension', (file) => {
    expect(isImagePath(file)).toBe(true)
  })

  it('is case-insensitive', () => {
    expect(isImagePath('PHOTO.PNG')).toBe(true)
    expect(isImagePath('Photo.JpEg')).toBe(true)
  })

  it('handles URLs with query strings and fragments', () => {
    expect(isImagePath('https://example.com/img.png?w=100')).toBe(true)
    expect(isImagePath('https://example.com/img.jpg#section')).toBe(true)
    expect(isImagePath('https://example.com/img.png?w=100#top')).toBe(true)
  })

  it('rejects non-image extensions', () => {
    expect(isImagePath('file.txt')).toBe(false)
    expect(isImagePath('file.pdf')).toBe(false)
    expect(isImagePath('file.md')).toBe(false)
  })

  it('handles paths with directories', () => {
    expect(isImagePath('/path/to/image.png')).toBe(true)
    expect(isImagePath('.beads/attachments/abc/screenshot.jpg')).toBe(true)
  })
})

// ---------------------------------------------------------------------------
// isMarkdownPath
// ---------------------------------------------------------------------------
describe('isMarkdownPath', () => {
  it('returns false for empty/falsy input', () => {
    expect(isMarkdownPath('')).toBe(false)
  })

  it('recognises .md and .markdown', () => {
    expect(isMarkdownPath('notes.md')).toBe(true)
    expect(isMarkdownPath('notes.markdown')).toBe(true)
  })

  it('is case-insensitive', () => {
    expect(isMarkdownPath('README.MD')).toBe(true)
  })

  it('handles URLs with query/hash', () => {
    expect(isMarkdownPath('https://example.com/doc.md?v=2')).toBe(true)
  })

  it('rejects non-markdown extensions', () => {
    expect(isMarkdownPath('file.txt')).toBe(false)
    expect(isMarkdownPath('file.png')).toBe(false)
  })
})

// ---------------------------------------------------------------------------
// isUrl
// ---------------------------------------------------------------------------
describe('isUrl', () => {
  it('returns false for empty/falsy input', () => {
    expect(isUrl('')).toBe(false)
  })

  it('recognises http and https', () => {
    expect(isUrl('http://example.com')).toBe(true)
    expect(isUrl('https://example.com')).toBe(true)
  })

  it('rejects non-URL strings', () => {
    expect(isUrl('/path/to/file')).toBe(false)
    expect(isUrl('ftp://server.com')).toBe(false)
    expect(isUrl('www.example.com')).toBe(false)
  })
})

// ---------------------------------------------------------------------------
// extractImagesFromMarkdown
// ---------------------------------------------------------------------------
describe('extractImagesFromMarkdown', () => {
  it('returns empty array for empty/falsy input', () => {
    expect(extractImagesFromMarkdown('')).toEqual([])
    expect(extractImagesFromMarkdown(undefined as unknown as string)).toEqual([])
  })

  it('extracts single image', () => {
    const result = extractImagesFromMarkdown('![alt text](image.png)')
    expect(result).toEqual([{ alt: 'alt text', src: 'image.png' }])
  })

  it('extracts multiple images', () => {
    const text = '![a](one.png) some text ![b](two.jpg)'
    const result = extractImagesFromMarkdown(text)
    expect(result).toHaveLength(2)
    expect(result[0]).toEqual({ alt: 'a', src: 'one.png' })
    expect(result[1]).toEqual({ alt: 'b', src: 'two.jpg' })
  })

  it('defaults alt to "Image" when empty', () => {
    const result = extractImagesFromMarkdown('![](photo.png)')
    expect(result[0]?.alt).toBe('Image')
  })

  it('ignores regular links (not images)', () => {
    const result = extractImagesFromMarkdown('[click here](https://example.com)')
    expect(result).toEqual([])
  })

  it('handles text with no images', () => {
    expect(extractImagesFromMarkdown('just plain text')).toEqual([])
  })
})

// ---------------------------------------------------------------------------
// extractImagesFromExternalRef
// ---------------------------------------------------------------------------
describe('extractImagesFromExternalRef', () => {
  it('returns empty array for undefined/empty', () => {
    expect(extractImagesFromExternalRef(undefined)).toEqual([])
    expect(extractImagesFromExternalRef('')).toEqual([])
  })

  it('extracts single image path', () => {
    const result = extractImagesFromExternalRef('/path/to/screenshot.png')
    expect(result).toEqual([{ src: '/path/to/screenshot.png', alt: 'screenshot.png' }])
  })

  it('extracts multiple image paths (newline-separated)', () => {
    const ref = '/path/one.png\n/path/two.jpg'
    const result = extractImagesFromExternalRef(ref)
    expect(result).toHaveLength(2)
    expect(result[0]?.alt).toBe('one.png')
    expect(result[1]?.alt).toBe('two.jpg')
  })

  it('filters out cleared: prefixes', () => {
    const ref = 'cleared:abc123\n/path/image.png'
    const result = extractImagesFromExternalRef(ref)
    expect(result).toHaveLength(1)
    expect(result[0]?.src).toBe('/path/image.png')
  })

  it('filters out non-image lines', () => {
    const ref = '/path/image.png\nhttps://redmine.example.com/issues/42\nnotes.md'
    const result = extractImagesFromExternalRef(ref)
    expect(result).toHaveLength(1)
    expect(result[0]?.src).toBe('/path/image.png')
  })

  it('trims whitespace from lines', () => {
    const ref = '  /path/image.png  \n  /path/photo.jpg  '
    const result = extractImagesFromExternalRef(ref)
    expect(result).toHaveLength(2)
  })

  it('skips empty lines', () => {
    const ref = '/path/image.png\n\n\n/path/photo.jpg'
    const result = extractImagesFromExternalRef(ref)
    expect(result).toHaveLength(2)
  })

  it('handles only cleared: values', () => {
    const ref = 'cleared:abc\ncleared:def'
    expect(extractImagesFromExternalRef(ref)).toEqual([])
  })
})

// ---------------------------------------------------------------------------
// extractMarkdownFromExternalRef
// ---------------------------------------------------------------------------
describe('extractMarkdownFromExternalRef', () => {
  it('returns empty array for undefined', () => {
    expect(extractMarkdownFromExternalRef(undefined)).toEqual([])
  })

  it('extracts markdown paths', () => {
    const ref = '/path/notes.md\n/path/image.png'
    const result = extractMarkdownFromExternalRef(ref)
    expect(result).toHaveLength(1)
    expect(result[0]).toEqual({ src: '/path/notes.md', alt: 'notes.md' })
  })

  it('recognises .markdown extension', () => {
    const result = extractMarkdownFromExternalRef('/doc/readme.markdown')
    expect(result).toHaveLength(1)
  })

  it('filters out cleared: prefixes', () => {
    const ref = 'cleared:xyz\n/path/notes.md'
    const result = extractMarkdownFromExternalRef(ref)
    expect(result).toHaveLength(1)
  })
})

// ---------------------------------------------------------------------------
// extractNonImageRefs
// ---------------------------------------------------------------------------
describe('extractNonImageRefs', () => {
  it('returns empty array for undefined', () => {
    expect(extractNonImageRefs(undefined)).toEqual([])
  })

  it('returns non-image, non-markdown references', () => {
    const ref = '/path/image.png\nhttps://redmine.example.com/issues/42\nnotes.md\nREDMINE-123'
    const result = extractNonImageRefs(ref)
    expect(result).toEqual(['https://redmine.example.com/issues/42', 'REDMINE-123'])
  })

  it('filters out cleared: prefixes', () => {
    const ref = 'cleared:abc\nhttps://example.com'
    const result = extractNonImageRefs(ref)
    expect(result).toEqual(['https://example.com'])
  })

  it('returns empty when all refs are images/markdown', () => {
    const ref = '/path/image.png\n/doc/notes.md'
    expect(extractNonImageRefs(ref)).toEqual([])
  })

  it('trims whitespace', () => {
    const ref = '  https://example.com  '
    expect(extractNonImageRefs(ref)).toEqual(['https://example.com'])
  })
})

// ---------------------------------------------------------------------------
// renderMarkdown
// ---------------------------------------------------------------------------
describe('renderMarkdown', () => {
  it('returns empty string for empty/falsy input', () => {
    expect(renderMarkdown('')).toBe('')
  })

  it('renders basic markdown', () => {
    const html = renderMarkdown('**bold** and *italic*')
    expect(html).toContain('<strong>bold</strong>')
    expect(html).toContain('<em>italic</em>')
  })

  it('converts newlines to <br>', () => {
    const html = renderMarkdown('line1\nline2')
    expect(html).toContain('<br>')
  })

  it('auto-links URLs', () => {
    const html = renderMarkdown('visit https://example.com today')
    expect(html).toContain('href="https://example.com"')
    expect(html).toContain('target="_blank"')
    expect(html).toContain('rel="noopener noreferrer"')
  })

  it('strips images from output', () => {
    const html = renderMarkdown('text ![alt](img.png) more')
    expect(html).not.toContain('<img')
    expect(html).not.toContain('img.png')
  })

  it('sanitizes HTML injection', () => {
    const html = renderMarkdown('<script>alert("xss")</script>')
    expect(html).not.toContain('<script>')
  })

  it('renders code blocks', () => {
    const html = renderMarkdown('`inline code`')
    expect(html).toContain('<code>inline code</code>')
  })

  it('renders lists', () => {
    const html = renderMarkdown('- item 1\n- item 2')
    expect(html).toContain('<ul>')
    expect(html).toContain('<li>')
  })
})
