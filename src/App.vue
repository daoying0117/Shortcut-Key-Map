<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import {
  createApp,
  createShortcut,
  deleteApp,
  deleteShortcut,
  getSettings,
  listApps,
  listShortcuts,
  updateToggleShortcut,
  updateWindowPositionMode,
  updateApp,
  updateShortcutPin,
  updateShortcut,
} from './api'
import ShortcutInput from './components/ShortcutInput.vue'
import { locale, setLocale, t, type Locale } from './i18n'
import type { AppItem, AppSettings, ShortcutItem } from './types'

const apps = ref<AppItem[]>([])
const shortcuts = ref<ShortcutItem[]>([])
const selectedAppId = ref<number | null>(null)

const appForm = ref({
  name: '',
  description: '',
})
const editingAppId = ref<number | null>(null)
const savingApp = ref(false)
const deletingAppId = ref<number | null>(null)
const pendingDeleteAppId = ref<number | null>(null)
let pendingDeleteAppTimer: number | null = null

const shortcutForm = ref({
  title: '',
  combo: '',
  notes: '',
})
const editingShortcutId = ref<number | null>(null)
const savingShortcut = ref(false)
const pinningShortcutId = ref<number | null>(null)
const deletingShortcutId = ref<number | null>(null)
const pendingDeleteShortcutId = ref<number | null>(null)
let pendingDeleteShortcutTimer: number | null = null

const loadingApps = ref(false)
const loadingShortcuts = ref(false)
const notice = ref('')
const appQuery = ref('')
const shortcutQuery = ref('')
const appSettings = ref({
  toggleShortcut: 'CmdOrCtrl+Shift+I',
  windowPositionMode: 'top_center' as AppSettings['windowPositionMode'],
})
const settingsForm = ref({
  toggleShortcut: '',
  windowPositionMode: 'top_center' as AppSettings['windowPositionMode'],
})
const settingsLanguage = ref<Locale>(locale.value)
const savingSettings = ref(false)
const showSettingsDialog = ref(false)

const selectedApp = computed(() => {
  if (selectedAppId.value === null) {
    return null
  }
  return apps.value.find((app) => app.id === selectedAppId.value) ?? null
})

const filteredApps = computed(() => {
  const keyword = appQuery.value.trim().toLowerCase()
  if (!keyword) {
    return apps.value
  }

  return apps.value.filter((app) => {
    return (
      app.name.toLowerCase().includes(keyword) ||
      (app.description ?? '').toLowerCase().includes(keyword)
    )
  })
})

const filteredShortcuts = computed(() => {
  const keyword = shortcutQuery.value.trim().toLowerCase()
  if (!keyword) {
    return shortcuts.value
  }

  return shortcuts.value.filter((shortcut) => {
    return (
      shortcut.title.toLowerCase().includes(keyword) ||
      shortcut.combo.toLowerCase().includes(keyword) ||
      (shortcut.notes ?? '').toLowerCase().includes(keyword)
    )
  })
})

const appsCountLabel = computed(() => {
  if (loadingApps.value) {
    return t('loading')
  }
  return t('appsCount', { count: apps.value.length })
})

watch(selectedAppId, (appId) => {
  clearPendingAppDelete()
  clearPendingShortcutDelete()
  void loadShortcuts(appId)
})

onMounted(() => {
  void initializePage()
})

async function initializePage() {
  await Promise.all([loadSettings(), loadApps()])
}

async function loadSettings() {
  try {
    const settings = await getSettings()
    appSettings.value = settings
    settingsForm.value.toggleShortcut = settings.toggleShortcut
    settingsForm.value.windowPositionMode = settings.windowPositionMode
  } catch (error) {
    showNotice(parseError(error))
  }
}

function openSettingsDialog() {
  settingsForm.value.toggleShortcut = appSettings.value.toggleShortcut
  settingsForm.value.windowPositionMode = appSettings.value.windowPositionMode
  settingsLanguage.value = locale.value
  showSettingsDialog.value = true
}

function closeSettingsDialog() {
  showSettingsDialog.value = false
}

async function loadApps(preferredAppId?: number | null) {
  loadingApps.value = true
  try {
    const data = await listApps()
    apps.value = data

    if (data.length === 0) {
      selectedAppId.value = null
      return
    }

    const targetId = preferredAppId ?? selectedAppId.value
    if (targetId !== null && data.some((item) => item.id === targetId)) {
      selectedAppId.value = targetId
    } else {
      selectedAppId.value = data[0].id
    }
  } catch (error) {
    showNotice(parseError(error))
  } finally {
    loadingApps.value = false
  }
}

async function loadShortcuts(appId: number | null) {
  if (appId === null) {
    shortcuts.value = []
    return
  }

  loadingShortcuts.value = true
  try {
    shortcuts.value = await listShortcuts(appId)
  } catch (error) {
    shortcuts.value = []
    showNotice(parseError(error))
  } finally {
    loadingShortcuts.value = false
  }
}

function resetAppForm() {
  appForm.value = { name: '', description: '' }
  editingAppId.value = null
}

function startEditApp(app: AppItem) {
  clearPendingAppDelete()
  editingAppId.value = app.id
  appForm.value = {
    name: app.name,
    description: app.description ?? '',
  }
}

async function submitApp() {
  if (savingApp.value) return
  savingApp.value = true

  try {
    let targetId: number
    if (editingAppId.value !== null) {
      const updated = await updateApp(editingAppId.value, {
        name: appForm.value.name,
        description: appForm.value.description || null,
      })
      targetId = updated.id
      showNotice(t('appUpdated'))
    } else {
      const created = await createApp({
        name: appForm.value.name,
        description: appForm.value.description || null,
      })
      targetId = created.id
      showNotice(t('appCreated'))
    }
    resetAppForm()
    await loadApps(targetId)
  } catch (error) {
    showNotice(parseError(error))
  } finally {
    savingApp.value = false
  }
}

async function removeApp(app: AppItem) {
  if (deletingAppId.value !== null) return
  deletingAppId.value = app.id
  try {
    await deleteApp(app.id)
    if (selectedAppId.value === app.id) {
      selectedAppId.value = null
    }
    await loadApps()
    clearPendingAppDelete()
    showNotice(t('appDeleted'))
  } catch (error) {
    showNotice(parseError(error))
  } finally {
    deletingAppId.value = null
  }
}

function clearPendingAppDelete() {
  pendingDeleteAppId.value = null
  if (pendingDeleteAppTimer !== null) {
    window.clearTimeout(pendingDeleteAppTimer)
    pendingDeleteAppTimer = null
  }
}

function requestRemoveApp(app: AppItem) {
  if (deletingAppId.value !== null) return
  if (pendingDeleteAppId.value === app.id) {
    void removeApp(app)
    return
  }

  pendingDeleteAppId.value = app.id
  if (pendingDeleteAppTimer !== null) {
    window.clearTimeout(pendingDeleteAppTimer)
  }
  pendingDeleteAppTimer = window.setTimeout(() => {
    pendingDeleteAppId.value = null
    pendingDeleteAppTimer = null
  }, 3500)
}

function resetShortcutForm() {
  shortcutForm.value = { title: '', combo: '', notes: '' }
  editingShortcutId.value = null
}

function startEditShortcut(shortcut: ShortcutItem) {
  clearPendingShortcutDelete()
  editingShortcutId.value = shortcut.id
  shortcutForm.value = {
    title: shortcut.title,
    combo: shortcut.combo,
    notes: shortcut.notes ?? '',
  }
}

async function submitShortcut() {
  if (savingShortcut.value) return
  if (selectedAppId.value === null) {
    showNotice(t('selectAppRequired'))
    return
  }

  savingShortcut.value = true
  try {
    if (editingShortcutId.value !== null) {
      await updateShortcut(editingShortcutId.value, {
        title: shortcutForm.value.title,
        combo: shortcutForm.value.combo,
        notes: shortcutForm.value.notes || null,
      })
      showNotice(t('shortcutUpdated'))
    } else {
      await createShortcut({
        appId: selectedAppId.value,
        title: shortcutForm.value.title,
        combo: shortcutForm.value.combo,
        notes: shortcutForm.value.notes || null,
      })
      showNotice(t('shortcutCreated'))
    }

    resetShortcutForm()
    await loadShortcuts(selectedAppId.value)
    await loadApps(selectedAppId.value)
  } catch (error) {
    showNotice(parseError(error))
  } finally {
    savingShortcut.value = false
  }
}

async function removeShortcut(shortcut: ShortcutItem) {
  if (deletingShortcutId.value !== null) return
  deletingShortcutId.value = shortcut.id
  try {
    await deleteShortcut(shortcut.id)
    if (selectedAppId.value !== null) {
      await loadShortcuts(selectedAppId.value)
      await loadApps(selectedAppId.value)
    }
    clearPendingShortcutDelete()
    showNotice(t('shortcutDeleted'))
  } catch (error) {
    showNotice(parseError(error))
  } finally {
    deletingShortcutId.value = null
  }
}

function clearPendingShortcutDelete() {
  pendingDeleteShortcutId.value = null
  if (pendingDeleteShortcutTimer !== null) {
    window.clearTimeout(pendingDeleteShortcutTimer)
    pendingDeleteShortcutTimer = null
  }
}

function requestRemoveShortcut(shortcut: ShortcutItem) {
  if (deletingShortcutId.value !== null) return
  if (pendingDeleteShortcutId.value === shortcut.id) {
    void removeShortcut(shortcut)
    return
  }

  pendingDeleteShortcutId.value = shortcut.id
  if (pendingDeleteShortcutTimer !== null) {
    window.clearTimeout(pendingDeleteShortcutTimer)
  }
  pendingDeleteShortcutTimer = window.setTimeout(() => {
    pendingDeleteShortcutId.value = null
    pendingDeleteShortcutTimer = null
  }, 3500)
}

async function toggleShortcutPinned(shortcut: ShortcutItem, pinned: boolean) {
  if (pinningShortcutId.value !== null) return
  clearPendingAppDelete()
  clearPendingShortcutDelete()
  pinningShortcutId.value = shortcut.id
  try {
    await updateShortcutPin(shortcut.id, pinned)
    if (selectedAppId.value !== null) {
      await loadShortcuts(selectedAppId.value)
    }
  } catch (error) {
    showNotice(parseError(error))
  } finally {
    pinningShortcutId.value = null
  }
}

async function submitSettings() {
  if (savingSettings.value) return
  savingSettings.value = true
  try {
    let latest = appSettings.value
    if (settingsForm.value.toggleShortcut !== appSettings.value.toggleShortcut) {
      latest = await updateToggleShortcut(settingsForm.value.toggleShortcut)
    }
    if (settingsForm.value.windowPositionMode !== latest.windowPositionMode) {
      latest = await updateWindowPositionMode(settingsForm.value.windowPositionMode)
    }

    appSettings.value = latest
    settingsForm.value.toggleShortcut = latest.toggleShortcut
    settingsForm.value.windowPositionMode = latest.windowPositionMode
    if (settingsLanguage.value !== locale.value) {
      setLocale(settingsLanguage.value)
    }
    showNotice(t('settingsUpdated'))
    showSettingsDialog.value = false
  } catch (error) {
    showNotice(parseError(error))
    settingsForm.value.toggleShortcut = appSettings.value.toggleShortcut
    settingsForm.value.windowPositionMode = appSettings.value.windowPositionMode
    settingsLanguage.value = locale.value
  } finally {
    savingSettings.value = false
  }
}

function parseError(error: unknown) {
  if (typeof error === 'string') return error
  if (error && typeof error === 'object' && 'toString' in error) {
    return String(error)
  }
  return t('unknownError')
}

function showNotice(message: string) {
  notice.value = message
  window.setTimeout(() => {
    if (notice.value === message) {
      notice.value = ''
    }
  }, 2500)
}

onUnmounted(() => {
  clearPendingAppDelete()
  clearPendingShortcutDelete()
})
</script>

<template>
  <div class="shell">
    <header class="topbar">
      <div>
        <h1>Shortcut Key Map</h1>
        <p>{{ t('appSubtitle') }}</p>
      </div>
      <div class="topbar-actions">
        <div class="hint">
          <span class="hint-label">{{ t('globalToggleShortcut') }}</span>
          <span class="hint-value">{{ appSettings.toggleShortcut }}</span>
        </div>
        <button type="button" class="ghost-btn topbar-btn" @click="openSettingsDialog">
          {{ t('globalSettings') }}
        </button>
      </div>
    </header>

    <main class="layout">
      <section class="panel apps-panel">
        <div class="panel-header">
          <h2>{{ t('appsPanelTitle') }}</h2>
          <span>{{ appsCountLabel }}</span>
        </div>

        <input
          v-model="appQuery"
          class="search-input"
          type="text"
          :placeholder="t('searchAppsPlaceholder')"
        />

        <ul v-if="filteredApps.length > 0" class="app-list">
          <li
            v-for="app in filteredApps"
            :key="app.id"
            :class="{ active: app.id === selectedAppId }"
            @click="selectedAppId = app.id"
          >
            <div class="app-main">
              <div class="app-title">{{ app.name }}</div>
              <div class="app-meta">{{ t('shortcutsCount', { count: app.shortcutCount }) }}</div>
            </div>
            <div class="row-actions">
              <button type="button" class="ghost-btn" @click.stop="startEditApp(app)">{{ t('edit') }}</button>
              <button
                type="button"
                class="ghost-btn danger"
                :class="{ active: pendingDeleteAppId === app.id }"
                :disabled="deletingAppId === app.id"
                @click.stop="requestRemoveApp(app)"
              >
                {{
                  deletingAppId === app.id
                    ? t('deleting')
                    : pendingDeleteAppId === app.id
                      ? t('confirmDelete')
                      : t('delete')
                }}
              </button>
            </div>
          </li>
        </ul>
        <div v-else class="empty">
          {{ apps.length === 0 ? t('noAppsYet') : t('noAppsMatched') }}
        </div>

        <form class="form" @submit.prevent="submitApp">
          <h3>{{ editingAppId === null ? t('newApp') : t('editApp') }}</h3>
          <label>
            <span>{{ t('appName') }}</span>
            <input v-model="appForm.name" type="text" :placeholder="t('appNamePlaceholder')" />
          </label>
          <label>
            <span>{{ t('appDescription') }}</span>
            <input v-model="appForm.description" type="text" :placeholder="t('appDescriptionPlaceholder')" />
          </label>
          <div class="form-actions">
            <button type="submit" :disabled="savingApp">
              {{ savingApp ? t('saving') : editingAppId === null ? t('addApp') : t('saveApp') }}
            </button>
            <button v-if="editingAppId !== null" type="button" class="ghost-btn" @click="resetAppForm">
              {{ t('cancelEdit') }}
            </button>
          </div>
        </form>
      </section>

      <section class="panel shortcuts-panel">
        <div class="panel-header">
          <h2>{{ t('shortcutsPanelTitle') }}</h2>
          <span v-if="selectedApp">{{ selectedApp.name }}</span>
        </div>

        <template v-if="selectedApp">
          <input
            v-model="shortcutQuery"
            class="search-input"
            type="text"
            :placeholder="t('searchShortcutsPlaceholder')"
          />

          <ul v-if="filteredShortcuts.length > 0" class="shortcut-list">
            <li v-for="shortcut in filteredShortcuts" :key="shortcut.id">
              <div class="shortcut-main">
                <div class="shortcut-title">{{ shortcut.title }}</div>
                <div class="shortcut-combo">{{ shortcut.combo }}</div>
                <p v-if="shortcut.notes" class="shortcut-notes">{{ shortcut.notes }}</p>
              </div>
              <div class="row-actions">
                <button
                  type="button"
                  class="ghost-btn"
                  :class="{ active: shortcut.isPinned }"
                  :disabled="pinningShortcutId === shortcut.id"
                  @click="toggleShortcutPinned(shortcut, !shortcut.isPinned)"
                >
                  {{
                    pinningShortcutId === shortcut.id
                      ? t('processing')
                      : shortcut.isPinned
                        ? t('unpin')
                        : t('pin')
                  }}
                </button>
                <button type="button" class="ghost-btn" @click="startEditShortcut(shortcut)">
                  {{ t('edit') }}
                </button>
                <button
                  type="button"
                  class="ghost-btn danger"
                  :class="{ active: pendingDeleteShortcutId === shortcut.id }"
                  :disabled="deletingShortcutId === shortcut.id"
                  @click="requestRemoveShortcut(shortcut)"
                >
                  {{
                    deletingShortcutId === shortcut.id
                      ? t('deleting')
                      : pendingDeleteShortcutId === shortcut.id
                        ? t('confirmDelete')
                        : t('delete')
                  }}
                </button>
              </div>
            </li>
          </ul>
          <div v-else class="empty">
            {{
              loadingShortcuts
                ? t('loading')
                : shortcuts.length === 0
                  ? t('noShortcutsYet')
                  : t('noShortcutsMatched')
            }}
          </div>

          <form class="form" @submit.prevent="submitShortcut">
            <h3>{{ editingShortcutId === null ? t('newShortcut') : t('editShortcut') }}</h3>
            <label>
              <span>{{ t('shortcutName') }}</span>
              <input v-model="shortcutForm.title" type="text" :placeholder="t('shortcutNamePlaceholder')" />
            </label>

            <label>
              <span>{{ t('shortcutCombo') }}</span>
              <ShortcutInput v-model="shortcutForm.combo" />
            </label>

            <label>
              <span>{{ t('shortcutNotes') }}</span>
              <textarea
                v-model="shortcutForm.notes"
                rows="3"
                :placeholder="t('shortcutNotesPlaceholder')"
              />
            </label>

            <div class="form-actions">
              <button type="submit" :disabled="savingShortcut">
                {{
                  savingShortcut
                    ? t('saving')
                    : editingShortcutId === null
                      ? t('addShortcut')
                      : t('saveShortcut')
                }}
              </button>
              <button
                v-if="editingShortcutId !== null"
                type="button"
                class="ghost-btn"
                @click="resetShortcutForm"
              >
                {{ t('cancelEdit') }}
              </button>
            </div>
          </form>
        </template>

        <div v-else class="empty">{{ t('selectAppFirst') }}</div>
      </section>
    </main>

    <transition name="toast-fade">
      <div v-if="notice" class="toast" role="status" aria-live="polite">
        {{ notice }}
      </div>
    </transition>

    <div
      v-if="showSettingsDialog"
      class="settings-overlay"
      role="dialog"
      aria-modal="true"
      @click.self="closeSettingsDialog"
    >
      <section class="settings-dialog panel">
        <div class="panel-header">
          <h2>{{ t('settingsTitle') }}</h2>
          <span>{{ t('settingsSubtitle') }}</span>
        </div>
        <form class="form" @submit.prevent="submitSettings">
          <label>
            <span>{{ t('toggleShortcutField') }}</span>
            <ShortcutInput v-model="settingsForm.toggleShortcut" />
          </label>
          <label>
            <span>{{ t('windowPositionField') }}</span>
            <select v-model="settingsForm.windowPositionMode" class="position-select">
              <option value="left_top">{{ t('positionLeftTop') }}</option>
              <option value="top_center">{{ t('positionTopCenter') }}</option>
              <option value="right_top">{{ t('positionRightTop') }}</option>
              <option value="left_center">{{ t('positionLeftCenter') }}</option>
              <option value="center">{{ t('positionCenter') }}</option>
              <option value="right_center">{{ t('positionRightCenter') }}</option>
              <option value="left_bottom">{{ t('positionLeftBottom') }}</option>
              <option value="bottom_center">{{ t('positionBottomCenter') }}</option>
              <option value="right_bottom">{{ t('positionRightBottom') }}</option>
            </select>
          </label>
          <label>
            <span>{{ t('languageField') }}</span>
            <select v-model="settingsLanguage" class="position-select">
              <option value="zh">{{ t('langZh') }}</option>
              <option value="en">{{ t('langEn') }}</option>
            </select>
          </label>
          <div class="form-actions">
            <button type="submit" :disabled="savingSettings">
              {{ savingSettings ? t('saving') : t('saveSettings') }}
            </button>
            <button type="button" class="ghost-btn" @click="closeSettingsDialog">{{ t('cancel') }}</button>
          </div>
        </form>
      </section>
    </div>
  </div>
</template>
