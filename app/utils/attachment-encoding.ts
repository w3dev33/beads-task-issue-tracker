/**
 * External reference encoding utilities
 *
 * Uses `|` as separator instead of `\n` to avoid br JSONL validation errors.
 * splitRefs() handles both `|` (new) and `\n` (legacy) transparently.
 */

export const REF_SEPARATOR = '|'

/**
 * Split an external_ref field into individual refs.
 * Handles both `|` (new format) and `\n` (legacy format).
 */
export function splitRefs(externalRef: string | undefined): string[] {
  if (!externalRef) return []
  return externalRef
    .split(/[|\n]/)
    .map(r => r.trim())
    .filter(Boolean)
}

/**
 * Join refs into a single external_ref string using `|` separator.
 */
export function joinRefs(refs: string[]): string {
  return refs.filter(Boolean).join(REF_SEPARATOR)
}
