import { ref } from 'vue'

export type Locale = 'zh' | 'en'

const STORAGE_KEY = 'shortcut_key_map_locale'

function resolveInitialLocale(): Locale {
  if (typeof window === 'undefined') {
    return 'zh'
  }

  const stored = window.localStorage.getItem(STORAGE_KEY)
  if (stored === 'zh' || stored === 'en') {
    return stored
  }

  const language = window.navigator.language.toLowerCase()
  return language.startsWith('zh') ? 'zh' : 'en'
}

export const locale = ref<Locale>(resolveInitialLocale())

export function setLocale(next: Locale) {
  locale.value = next
  if (typeof window !== 'undefined') {
    window.localStorage.setItem(STORAGE_KEY, next)
  }
}

const messages: Record<Locale, Record<string, string>> = {
  zh: {
    appSubtitle: '记录、编辑并快速回顾各软件快捷键，支持组合键监听和下拉录入。',
    globalToggleShortcut: '全局切换快捷键',
    globalSettings: '全局设置',
    loading: '加载中...',
    saving: '保存中...',
    unknownError: '发生未知错误',

    appsPanelTitle: '应用列表',
    appsCount: '{count} 个应用',
    shortcutsCount: '{count} 个快捷键',
    searchAppsPlaceholder: '搜索应用名称或说明',
    noAppsYet: '还没有应用，先在下方创建一个。',
    noAppsMatched: '没有匹配到应用。',
    newApp: '新建应用',
    editApp: '编辑应用',
    appName: '应用名称',
    appNamePlaceholder: '例如: VS Code',
    appDescription: '说明',
    appDescriptionPlaceholder: '可选，例如: 日常开发工具',
    addApp: '添加应用',
    saveApp: '保存应用',
    appCreated: '应用已创建',
    appUpdated: '应用已更新',
    appDeleted: '应用已删除',

    shortcutsPanelTitle: '快捷键',
    searchShortcutsPlaceholder: '搜索快捷键名称、组合或备注',
    noShortcutsYet: '该应用还没有快捷键，先添加一条。',
    noShortcutsMatched: '没有匹配到快捷键。',
    selectAppFirst: '请先在左侧选择或创建一个应用。',
    selectAppRequired: '请先选择应用',
    newShortcut: '新建快捷键',
    editShortcut: '编辑快捷键',
    shortcutName: '快捷键名称',
    shortcutNamePlaceholder: '例如: 打开命令面板',
    shortcutCombo: '组合键',
    shortcutNotes: '备注',
    shortcutNotesPlaceholder: '可选，例如: 在编辑器内调用命令面板',
    addShortcut: '添加快捷键',
    saveShortcut: '保存快捷键',
    shortcutCreated: '快捷键已创建',
    shortcutUpdated: '快捷键已更新',
    shortcutDeleted: '快捷键已删除',
    pin: '置顶',
    unpin: '取消置顶',

    processing: '处理中...',
    deleting: '删除中...',
    delete: '删除',
    confirmDelete: '确认删除',
    edit: '编辑',
    cancelEdit: '取消编辑',
    cancel: '取消',

    settingsTitle: '全局设置',
    settingsSubtitle: '菜单栏与唤起行为',
    toggleShortcutField: '唤起/收起窗口快捷键',
    windowPositionField: '窗口展示位置',
    languageField: '界面语言',
    langZh: '中文',
    langEn: 'English',
    saveSettings: '保存设置',
    settingsUpdated: '窗口设置已更新',

    positionLeftTop: '左上',
    positionTopCenter: '中上',
    positionRightTop: '右上',
    positionLeftCenter: '左中',
    positionCenter: '中间',
    positionRightCenter: '右中',
    positionLeftBottom: '左下',
    positionBottomCenter: '中下',
    positionRightBottom: '右下',

    inputModeRecord: '监听输入',
    inputModeSelect: '下拉选择',
    clear: '清空',
    recordingPlaceholder: '请按下组合键...',
    idlePlaceholder: '点击后按下快捷键，例如 Command+Shift+K',
    selectPrimaryKey: '选择主键',
  },
  en: {
    appSubtitle: 'Record, edit, and quickly review app shortcuts with keyboard capture and dropdown input.',
    globalToggleShortcut: 'Global Toggle Shortcut',
    globalSettings: 'Settings',
    loading: 'Loading...',
    saving: 'Saving...',
    unknownError: 'Unknown error occurred',

    appsPanelTitle: 'Apps',
    appsCount: '{count} apps',
    shortcutsCount: '{count} shortcuts',
    searchAppsPlaceholder: 'Search app name or description',
    noAppsYet: 'No apps yet. Create one below.',
    noAppsMatched: 'No matching apps.',
    newApp: 'New App',
    editApp: 'Edit App',
    appName: 'App Name',
    appNamePlaceholder: 'e.g. VS Code',
    appDescription: 'Description',
    appDescriptionPlaceholder: 'Optional, e.g. Daily dev tool',
    addApp: 'Add App',
    saveApp: 'Save App',
    appCreated: 'App created',
    appUpdated: 'App updated',
    appDeleted: 'App deleted',

    shortcutsPanelTitle: 'Shortcuts',
    searchShortcutsPlaceholder: 'Search by title, combo, or note',
    noShortcutsYet: 'No shortcuts yet for this app. Add one below.',
    noShortcutsMatched: 'No matching shortcuts.',
    selectAppFirst: 'Select or create an app on the left first.',
    selectAppRequired: 'Please select an app first',
    newShortcut: 'New Shortcut',
    editShortcut: 'Edit Shortcut',
    shortcutName: 'Shortcut Name',
    shortcutNamePlaceholder: 'e.g. Open Command Palette',
    shortcutCombo: 'Key Combo',
    shortcutNotes: 'Notes',
    shortcutNotesPlaceholder: 'Optional, e.g. Open command palette in editor',
    addShortcut: 'Add Shortcut',
    saveShortcut: 'Save Shortcut',
    shortcutCreated: 'Shortcut created',
    shortcutUpdated: 'Shortcut updated',
    shortcutDeleted: 'Shortcut deleted',
    pin: 'Pin',
    unpin: 'Unpin',

    processing: 'Processing...',
    deleting: 'Deleting...',
    delete: 'Delete',
    confirmDelete: 'Confirm Delete',
    edit: 'Edit',
    cancelEdit: 'Cancel',
    cancel: 'Cancel',

    settingsTitle: 'Settings',
    settingsSubtitle: 'Tray and window behavior',
    toggleShortcutField: 'Toggle window shortcut',
    windowPositionField: 'Window position',
    languageField: 'Language',
    langZh: '中文',
    langEn: 'English',
    saveSettings: 'Save Settings',
    settingsUpdated: 'Settings updated',

    positionLeftTop: 'Top Left',
    positionTopCenter: 'Top Center',
    positionRightTop: 'Top Right',
    positionLeftCenter: 'Center Left',
    positionCenter: 'Center',
    positionRightCenter: 'Center Right',
    positionLeftBottom: 'Bottom Left',
    positionBottomCenter: 'Bottom Center',
    positionRightBottom: 'Bottom Right',

    inputModeRecord: 'Capture Input',
    inputModeSelect: 'Dropdown Select',
    clear: 'Clear',
    recordingPlaceholder: 'Press a key combination...',
    idlePlaceholder: 'Click and press keys, e.g. Command+Shift+K',
    selectPrimaryKey: 'Select primary key',
  },
}

export function t(key: string, params: Record<string, string | number> = {}) {
  const template = messages[locale.value][key] ?? messages.zh[key] ?? key
  return template.replace(/\{(\w+)\}/g, (_, token: string) => String(params[token] ?? `{${token}}`))
}
