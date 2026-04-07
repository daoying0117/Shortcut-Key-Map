const MODIFIER_ORDER = ['Command', 'Ctrl', 'Option', 'Shift'] as const
type Modifier = (typeof MODIFIER_ORDER)[number]

const DISPLAY_KEYS = [
  'A',
  'B',
  'C',
  'D',
  'E',
  'F',
  'G',
  'H',
  'I',
  'J',
  'K',
  'L',
  'M',
  'N',
  'O',
  'P',
  'Q',
  'R',
  'S',
  'T',
  'U',
  'V',
  'W',
  'X',
  'Y',
  'Z',
  '0',
  '1',
  '2',
  '3',
  '4',
  '5',
  '6',
  '7',
  '8',
  '9',
  'F1',
  'F2',
  'F3',
  'F4',
  'F5',
  'F6',
  'F7',
  'F8',
  'F9',
  'F10',
  'F11',
  'F12',
  'Enter',
  'Escape',
  'Tab',
  'Space',
  'Backspace',
  'Delete',
  'Home',
  'End',
  'PageUp',
  'PageDown',
  'ArrowUp',
  'ArrowDown',
  'ArrowLeft',
  'ArrowRight',
] as const

const MODIFIER_KEYS = new Set(['Meta', 'Control', 'Alt', 'Shift'])

export const selectablePrimaryKeys = [...DISPLAY_KEYS]
export const selectableModifiers = [...MODIFIER_ORDER]

export function splitCombo(combo: string): { modifiers: Modifier[]; key: string } {
  const parts = combo
    .split('+')
    .map((part) => part.trim())
    .filter(Boolean)

  const modifiers = MODIFIER_ORDER.filter((modifier) => parts.includes(modifier))
  const key = parts.find((part) => !MODIFIER_ORDER.includes(part as Modifier)) ?? ''

  return { modifiers, key }
}

export function normalizeCombo(modifiers: string[], key: string) {
  const modifierSet = new Set<Modifier>()
  for (const raw of modifiers) {
    const normalized = normalizeModifier(raw)
    if (normalized) {
      modifierSet.add(normalized)
    }
  }

  const sortedModifiers = MODIFIER_ORDER.filter((modifier) => modifierSet.has(modifier))
  const primaryKey = normalizePrimaryKey(key)

  if (!primaryKey) {
    return ''
  }

  return [...sortedModifiers, primaryKey].join('+')
}

export function normalizeFromKeyboardEvent(event: KeyboardEvent) {
  const primaryKey = keyFromKeyboardEvent(event)
  if (!primaryKey) {
    return ''
  }

  const modifiers: Modifier[] = []
  if (event.metaKey) modifiers.push('Command')
  if (event.ctrlKey) modifiers.push('Ctrl')
  if (event.altKey) modifiers.push('Option')
  if (event.shiftKey) modifiers.push('Shift')

  return normalizeCombo(modifiers, primaryKey)
}

function normalizeModifier(raw: string): Modifier | null {
  switch (raw.toLowerCase()) {
    case 'command':
    case 'cmd':
    case 'meta':
      return 'Command'
    case 'ctrl':
    case 'control':
      return 'Ctrl'
    case 'option':
    case 'alt':
      return 'Option'
    case 'shift':
      return 'Shift'
    default:
      return null
  }
}

function normalizePrimaryKey(raw: string) {
  const trimmed = raw.trim()
  if (!trimmed) return ''

  const upper = trimmed.toUpperCase()
  if (/^[A-Z0-9]$/.test(upper)) return upper
  if (/^F([1-9]|1[0-2])$/.test(upper)) return upper

  const normalizedMap: Record<string, string> = {
    esc: 'Escape',
    escape: 'Escape',
    enter: 'Enter',
    return: 'Enter',
    tab: 'Tab',
    space: 'Space',
    ' ': 'Space',
    backspace: 'Backspace',
    delete: 'Delete',
    del: 'Delete',
    home: 'Home',
    end: 'End',
    pageup: 'PageUp',
    pagedown: 'PageDown',
    arrowup: 'ArrowUp',
    arrowdown: 'ArrowDown',
    arrowleft: 'ArrowLeft',
    arrowright: 'ArrowRight',
    up: 'ArrowUp',
    down: 'ArrowDown',
    left: 'ArrowLeft',
    right: 'ArrowRight',
  }

  return normalizedMap[trimmed.toLowerCase()] ?? trimmed
}

function keyFromKeyboardEvent(event: KeyboardEvent) {
  if (MODIFIER_KEYS.has(event.key)) {
    return ''
  }

  if (event.code.startsWith('Key')) {
    return event.code.slice(3).toUpperCase()
  }
  if (event.code.startsWith('Digit')) {
    return event.code.slice(5)
  }
  if (/^F([1-9]|1[0-2])$/.test(event.code)) {
    return event.code
  }

  return normalizePrimaryKey(event.key)
}
