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
  // Add target="_blank" and rel="noopener noreferrer" to all links
  tokens[idx].attrSet('target', '_blank')
  tokens[idx].attrSet('rel', 'noopener noreferrer')
  // Add a data attribute to identify links for click handling
  tokens[idx].attrSet('data-external-link', 'true')
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
      src: match[2],
    })
  }
  return images
}

// Configure DOMPurify to allow our custom attributes
const purifyConfig: DOMPurify.Config = {
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
  return DOMPurify.sanitize(html, purifyConfig)
}
