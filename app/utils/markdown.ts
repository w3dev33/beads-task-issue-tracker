/**
 * Markdown rendering utility with security sanitization
 */
import MarkdownIt from 'markdown-it'
import DOMPurify from 'dompurify'

// Configure markdown-it
const md = new MarkdownIt({
  html: false, // Disable HTML tags in source
  breaks: true, // Convert \n to <br>
  linkify: true, // Auto-convert URLs to links
})

// Configure links to open in new tab with security attributes
const defaultRender =
  md.renderer.rules.link_open ||
  function (tokens, idx, options, _env, self) {
    return self.renderToken(tokens, idx, options)
  }

md.renderer.rules.link_open = function (tokens, idx, options, env, self) {
  const token = tokens[idx]
  if (token) {
    // Add target="_blank" and rel="noopener noreferrer" to all links
    token.attrSet('target', '_blank')
    token.attrSet('rel', 'noopener noreferrer')
    // Add a data attribute to identify links for click handling
    token.attrSet('data-external-link', 'true')
  }
  return defaultRender(tokens, idx, options, env, self)
}

// Remove images from rendered output (they'll be shown separately)
md.renderer.rules.image = function () {
  return ''
}

/**
 * Extract image references from markdown text
 * Returns array of { src, alt } objects
 */
export function extractImagesFromMarkdown(text: string): { src: string; alt: string }[] {
  if (!text) return []
  const regex = /!\[([^\]]*)\]\(([^)]+)\)/g
  const images: { src: string; alt: string }[] = []
  let match
  while ((match = regex.exec(text)) !== null) {
    images.push({
      alt: match[1] || 'Image',
      src: match[2] || '',
    })
  }
  return images
}

/**
 * Image file extensions supported for attachment preview
 */
const IMAGE_EXTENSIONS = ['.png', '.jpg', '.jpeg', '.gif', '.webp', '.bmp', '.svg', '.ico', '.tiff', '.tif']

/**
 * Markdown file extensions supported for attachment preview
 */
const MARKDOWN_EXTENSIONS = ['.md', '.markdown']

/**
 * Check if a path or URL is an image based on extension
 * Supports both local paths and URLs (http/https)
 */
export function isImagePath(path: string): boolean {
  if (!path) return false
  // Remove query string and hash for URL checking
  const cleanPath = (path.split('?')[0] ?? '').split('#')[0]?.toLowerCase() ?? ''
  return IMAGE_EXTENSIONS.some(ext => cleanPath.endsWith(ext))
}

/**
 * Check if a path is a markdown file based on extension
 */
export function isMarkdownPath(path: string): boolean {
  if (!path) return false
  const cleanPath = (path.split('?')[0] ?? '').split('#')[0]?.toLowerCase() ?? ''
  return MARKDOWN_EXTENSIONS.some(ext => cleanPath.endsWith(ext))
}

/**
 * Check if a string is a URL (http or https)
 */
export function isUrl(path: string): boolean {
  if (!path) return false
  return path.startsWith('http://') || path.startsWith('https://')
}

/**
 * Extract image paths from externalRef field
 * externalRef can contain multiple values separated by newlines
 * Returns array of { src, alt } objects for image paths only
 */
export function extractImagesFromExternalRef(externalRef: string | undefined): { src: string; alt: string }[] {
  if (!externalRef) return []

  return externalRef
    .split('\n')
    .map(line => line.trim())
    // Filter out cleared placeholders and non-image lines
    .filter(line => line && !line.startsWith('cleared:') && isImagePath(line))
    .map(path => ({
      src: path,
      alt: path.split('/').pop() || 'Image',
    }))
}

/**
 * Extract markdown file paths from externalRef field
 * Returns array of { src, alt } objects for markdown paths only
 */
export function extractMarkdownFromExternalRef(externalRef: string | undefined): { src: string; alt: string }[] {
  if (!externalRef) return []

  return externalRef
    .split('\n')
    .map(line => line.trim())
    .filter(line => line && !line.startsWith('cleared:') && isMarkdownPath(line))
    .map(path => ({
      src: path,
      alt: path.split('/').pop() || 'Markdown',
    }))
}

/**
 * Extract non-image, non-markdown references from externalRef field
 * Returns array of URLs/IDs that are not image or markdown paths
 */
export function extractNonImageRefs(externalRef: string | undefined): string[] {
  if (!externalRef) return []

  return externalRef
    .split('\n')
    .map(line => line.trim())
    // Filter out cleared placeholders, image paths, and markdown paths
    .filter(line => line && !line.startsWith('cleared:') && !isImagePath(line) && !isMarkdownPath(line))
}

// Configure DOMPurify to allow our custom attributes
const purifyConfig = {
  ALLOWED_TAGS: [
    'p',
    'br',
    'strong',
    'b',
    'em',
    'i',
    'u',
    's',
    'del',
    'a',
    'ul',
    'ol',
    'li',
    'code',
    'pre',
    'blockquote',
    'h1',
    'h2',
    'h3',
    'h4',
    'h5',
    'h6',
    'hr',
    'table',
    'thead',
    'tbody',
    'tr',
    'th',
    'td',
  ],
  ALLOWED_ATTR: ['href', 'target', 'rel', 'data-external-link'],
}

/**
 * Render Markdown text to sanitized HTML
 * @param text - Markdown text to render
 * @returns Sanitized HTML string
 */
export function renderMarkdown(text: string): string {
  if (!text) return ''
  const html = md.render(text)
  return DOMPurify.sanitize(html, purifyConfig) as string
}
