export interface AppItem {
  id: number
  name: string
  description: string | null
  shortcutCount: number
}

export interface ShortcutItem {
  id: number
  appId: number
  title: string
  combo: string
  notes: string | null
  isPinned: boolean
}

export interface UpsertAppPayload {
  name: string
  description: string | null
}

export interface CreateShortcutPayload {
  appId: number
  title: string
  combo: string
  notes: string | null
}

export interface UpdateShortcutPayload {
  title: string
  combo: string
  notes: string | null
}

export interface AppSettings {
  toggleShortcut: string
  windowPositionMode:
    | 'left_top'
    | 'top_center'
    | 'right_top'
    | 'left_center'
    | 'center'
    | 'right_center'
    | 'left_bottom'
    | 'bottom_center'
    | 'right_bottom'
}
