import { describe, it, expect, vi } from 'vitest'
import { computed, ref, nextTick } from 'vue'
import { useKeyboardNavigation } from '~/composables/useKeyboardNavigation'

function makeKeyEvent(key: string, target?: Partial<HTMLElement>): KeyboardEvent {
  const event = new KeyboardEvent('keydown', { key })
  Object.defineProperty(event, 'target', { value: { tagName: 'DIV', ...target } })
  Object.defineProperty(event, 'preventDefault', { value: vi.fn() })
  return event
}

describe('useKeyboardNavigation', () => {
  it('ArrowDown from no focus selects first item', () => {
    const ids = computed(() => ['a', 'b', 'c'])
    const { focusedId, handleKeydown } = useKeyboardNavigation({ itemIds: ids })

    expect(focusedId.value).toBeNull()
    handleKeydown(makeKeyEvent('ArrowDown'))
    expect(focusedId.value).toBe('a')
  })

  it('ArrowDown moves to next item', () => {
    const ids = computed(() => ['a', 'b', 'c'])
    const { focusedId, handleKeydown, setFocused } = useKeyboardNavigation({ itemIds: ids })

    setFocused('a')
    handleKeydown(makeKeyEvent('ArrowDown'))
    expect(focusedId.value).toBe('b')
  })

  it('ArrowDown clamps at end', () => {
    const ids = computed(() => ['a', 'b', 'c'])
    const { focusedId, handleKeydown, setFocused } = useKeyboardNavigation({ itemIds: ids })

    setFocused('c')
    handleKeydown(makeKeyEvent('ArrowDown'))
    expect(focusedId.value).toBe('c')
  })

  it('ArrowUp from no focus selects last item', () => {
    const ids = computed(() => ['a', 'b', 'c'])
    const { focusedId, handleKeydown } = useKeyboardNavigation({ itemIds: ids })

    handleKeydown(makeKeyEvent('ArrowUp'))
    expect(focusedId.value).toBe('c')
  })

  it('ArrowUp moves to previous item', () => {
    const ids = computed(() => ['a', 'b', 'c'])
    const { focusedId, handleKeydown, setFocused } = useKeyboardNavigation({ itemIds: ids })

    setFocused('b')
    handleKeydown(makeKeyEvent('ArrowUp'))
    expect(focusedId.value).toBe('a')
  })

  it('ArrowUp clamps at start', () => {
    const ids = computed(() => ['a', 'b', 'c'])
    const { focusedId, handleKeydown, setFocused } = useKeyboardNavigation({ itemIds: ids })

    setFocused('a')
    handleKeydown(makeKeyEvent('ArrowUp'))
    expect(focusedId.value).toBe('a')
  })

  it('Enter calls onSelect with focused id', () => {
    const onSelect = vi.fn()
    const ids = computed(() => ['a', 'b'])
    const { handleKeydown, setFocused } = useKeyboardNavigation({ itemIds: ids, onSelect })

    setFocused('b')
    handleKeydown(makeKeyEvent('Enter'))
    expect(onSelect).toHaveBeenCalledWith('b')
  })

  it('Enter does nothing when no focused item', () => {
    const onSelect = vi.fn()
    const ids = computed(() => ['a', 'b'])
    const { handleKeydown } = useKeyboardNavigation({ itemIds: ids, onSelect })

    handleKeydown(makeKeyEvent('Enter'))
    expect(onSelect).not.toHaveBeenCalled()
  })

  it('Space calls onAction with focused id', () => {
    const onAction = vi.fn()
    const ids = computed(() => ['a', 'b'])
    const { handleKeydown, setFocused } = useKeyboardNavigation({ itemIds: ids, onAction })

    setFocused('a')
    handleKeydown(makeKeyEvent(' '))
    expect(onAction).toHaveBeenCalledWith('a')
  })

  it('Space does nothing without onAction', () => {
    const ids = computed(() => ['a'])
    const { handleKeydown, setFocused } = useKeyboardNavigation({ itemIds: ids })

    setFocused('a')
    // Should not throw
    handleKeydown(makeKeyEvent(' '))
  })

  it('ignores keystrokes when target is INPUT', () => {
    const ids = computed(() => ['a', 'b'])
    const { focusedId, handleKeydown } = useKeyboardNavigation({ itemIds: ids })

    handleKeydown(makeKeyEvent('ArrowDown', { tagName: 'INPUT' }))
    expect(focusedId.value).toBeNull()
  })

  it('ignores keystrokes when target is TEXTAREA', () => {
    const ids = computed(() => ['a', 'b'])
    const { focusedId, handleKeydown } = useKeyboardNavigation({ itemIds: ids })

    handleKeydown(makeKeyEvent('ArrowDown', { tagName: 'TEXTAREA' }))
    expect(focusedId.value).toBeNull()
  })

  it('empty list is a no-op', () => {
    const ids = computed(() => [] as string[])
    const { focusedId, handleKeydown } = useKeyboardNavigation({ itemIds: ids })

    handleKeydown(makeKeyEvent('ArrowDown'))
    expect(focusedId.value).toBeNull()
  })

  it('resets focus to first when focused item is removed from list', async () => {
    const items = ref(['a', 'b', 'c'])
    const ids = computed(() => items.value)
    const { focusedId, setFocused } = useKeyboardNavigation({ itemIds: ids })

    setFocused('b')
    expect(focusedId.value).toBe('b')

    items.value = ['a', 'c']
    await nextTick()
    expect(focusedId.value).toBe('a')
  })

  it('keeps focus when focused item still present after list change', async () => {
    const items = ref(['a', 'b', 'c'])
    const ids = computed(() => items.value)
    const { focusedId, setFocused } = useKeyboardNavigation({ itemIds: ids })

    setFocused('b')
    items.value = ['b', 'c', 'd']
    await nextTick()
    expect(focusedId.value).toBe('b')
  })

  it('resets to null when list becomes empty', async () => {
    const items = ref(['a', 'b'])
    const ids = computed(() => items.value)
    const { focusedId, setFocused } = useKeyboardNavigation({ itemIds: ids })

    setFocused('a')
    items.value = []
    await nextTick()
    expect(focusedId.value).toBeNull()
  })

  it('isFocused returns correct value', () => {
    const ids = computed(() => ['a', 'b'])
    const { isFocused, setFocused } = useKeyboardNavigation({ itemIds: ids })

    setFocused('a')
    expect(isFocused('a')).toBe(true)
    expect(isFocused('b')).toBe(false)
  })
})
