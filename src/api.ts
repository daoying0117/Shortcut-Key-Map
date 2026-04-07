import { invoke } from '@tauri-apps/api/core'
import type {
  AppItem,
  AppSettings,
  CreateShortcutPayload,
  ShortcutItem,
  UpdateShortcutPayload,
  UpsertAppPayload,
} from './types'

export async function getSettings() {
  return invoke<AppSettings>('get_settings')
}

export async function updateToggleShortcut(shortcut: string) {
  return invoke<AppSettings>('update_toggle_shortcut', { shortcut })
}

export async function updateWindowPositionMode(mode: AppSettings['windowPositionMode']) {
  return invoke<AppSettings>('update_window_position_mode', { mode })
}

export async function listApps() {
  return invoke<AppItem[]>('list_apps')
}

export async function createApp(payload: UpsertAppPayload) {
  return invoke<AppItem>('create_app', { payload })
}

export async function updateApp(appId: number, payload: UpsertAppPayload) {
  return invoke<AppItem>('update_app', { appId, payload })
}

export async function deleteApp(appId: number) {
  return invoke<void>('delete_app', { appId })
}

export async function listShortcuts(appId: number) {
  return invoke<ShortcutItem[]>('list_shortcuts', { appId })
}

export async function createShortcut(payload: CreateShortcutPayload) {
  return invoke<ShortcutItem>('create_shortcut', { payload })
}

export async function updateShortcut(shortcutId: number, payload: UpdateShortcutPayload) {
  return invoke<ShortcutItem>('update_shortcut', { shortcutId, payload })
}

export async function updateShortcutPin(shortcutId: number, pinned: boolean) {
  return invoke<ShortcutItem>('update_shortcut_pin', { shortcutId, pinned })
}

export async function deleteShortcut(shortcutId: number) {
  return invoke<void>('delete_shortcut', { shortcutId })
}
