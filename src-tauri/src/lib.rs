use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, sync::Mutex};
use tauri::{
  menu::{Menu, MenuItem},
  tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
  AppHandle, Manager, PhysicalPosition, Position, State, WindowEvent,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

const DB_FILE_NAME: &str = "shortcut-key-map.sqlite3";
const DEFAULT_TOGGLE_SHORTCUT: &str = "CmdOrCtrl+Shift+I";
// Backward compatibility for databases created before the default shortcut changed.
const LEGACY_DEFAULT_TOGGLE_SHORTCUT: &str = "CmdOrCtrl+Shift+K";
const DEFAULT_WINDOW_POSITION_MODE: &str = "top_center";
const FLOATING_MARGIN_X: i32 = 16;
const FLOATING_MARGIN_Y: i32 = 16;

type CommandResult<T> = Result<T, String>;

struct DatabaseState {
  conn: Mutex<Connection>,
}

struct SettingsState {
  toggle_shortcut: Mutex<String>,
  window_position_mode: Mutex<WindowPositionMode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowPositionMode {
  LeftTop,
  TopCenter,
  RightTop,
  LeftCenter,
  RightBottom,
  RightCenter,
  LeftBottom,
  BottomCenter,
  Center,
}

impl WindowPositionMode {
  fn parse(raw: &str) -> Option<Self> {
    match raw.trim().to_lowercase().as_str() {
      "left_top" => Some(Self::LeftTop),
      "top_center" => Some(Self::TopCenter),
      "right_top" => Some(Self::RightTop),
      "left_center" => Some(Self::LeftCenter),
      "right_bottom" => Some(Self::RightBottom),
      "right_center" => Some(Self::RightCenter),
      "left_bottom" => Some(Self::LeftBottom),
      "bottom_center" => Some(Self::BottomCenter),
      "center" => Some(Self::Center),
      _ => None,
    }
  }

  fn as_str(self) -> &'static str {
    match self {
      Self::LeftTop => "left_top",
      Self::TopCenter => "top_center",
      Self::RightTop => "right_top",
      Self::LeftCenter => "left_center",
      Self::RightBottom => "right_bottom",
      Self::RightCenter => "right_center",
      Self::LeftBottom => "left_bottom",
      Self::BottomCenter => "bottom_center",
      Self::Center => "center",
    }
  }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppItem {
  id: i64,
  name: String,
  description: Option<String>,
  shortcut_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShortcutItem {
  id: i64,
  app_id: i64,
  title: String,
  combo: String,
  notes: Option<String>,
  is_pinned: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpsertAppPayload {
  name: String,
  description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateShortcutPayload {
  app_id: i64,
  title: String,
  combo: String,
  notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateShortcutPayload {
  title: String,
  combo: String,
  notes: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
  toggle_shortcut: String,
  window_position_mode: String,
}

fn normalize_optional_text(text: Option<String>) -> Option<String> {
  text.and_then(|value| {
    let trimmed = value.trim().to_owned();
    if trimmed.is_empty() {
      None
    } else {
      Some(trimmed)
    }
  })
}

fn require_non_empty(value: &str, field_name: &str) -> CommandResult<String> {
  let trimmed = value.trim();
  if trimmed.is_empty() {
    Err(format!("{field_name}不能为空"))
  } else {
    Ok(trimmed.to_owned())
  }
}

fn map_db_error(error: rusqlite::Error) -> String {
  match error {
    rusqlite::Error::SqliteFailure(_, Some(message)) => {
      let lower = message.to_lowercase();
      if lower.contains("unique") {
        "已存在重复记录，请更换名称或快捷键组合".to_owned()
      } else if lower.contains("foreign key") {
        "关联数据不存在或已被删除".to_owned()
      } else {
        format!("数据库错误: {message}")
      }
    }
    other => format!("数据库错误: {other}"),
  }
}

fn map_shortcut_error(error: tauri_plugin_global_shortcut::Error) -> String {
  format!("全局快捷键注册失败: {error}")
}

fn lock_conn<'a>(
  state: &'a State<'a, DatabaseState>,
) -> CommandResult<std::sync::MutexGuard<'a, Connection>> {
  state
    .conn
    .lock()
    .map_err(|_| "数据库连接锁获取失败".to_owned())
}

fn lock_shortcut<'a>(
  state: &'a State<'a, SettingsState>,
) -> CommandResult<std::sync::MutexGuard<'a, String>> {
  state
    .toggle_shortcut
    .lock()
    .map_err(|_| "设置状态锁获取失败".to_owned())
}

fn lock_position_mode<'a>(
  state: &'a State<'a, SettingsState>,
) -> CommandResult<std::sync::MutexGuard<'a, WindowPositionMode>> {
  state
    .window_position_mode
    .lock()
    .map_err(|_| "设置状态锁获取失败".to_owned())
}

fn upsert_setting(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
  conn.execute(
    r#"
    INSERT INTO app_settings (key, value)
    VALUES (?1, ?2)
    ON CONFLICT(key) DO UPDATE SET value = excluded.value
    "#,
    params![key, value],
  )?;
  Ok(())
}

fn read_setting(conn: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
  conn
    .query_row(
      "SELECT value FROM app_settings WHERE key = ?1",
      params![key],
      |row| row.get(0),
    )
    .optional()
}

fn app_row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<AppItem> {
  Ok(AppItem {
    id: row.get(0)?,
    name: row.get(1)?,
    description: row.get(2)?,
    shortcut_count: row.get(3)?,
  })
}

fn shortcut_row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<ShortcutItem> {
  Ok(ShortcutItem {
    id: row.get(0)?,
    app_id: row.get(1)?,
    title: row.get(2)?,
    combo: row.get(3)?,
    notes: row.get(4)?,
    is_pinned: row.get::<_, i64>(5)? != 0,
  })
}

fn fetch_app_by_id(conn: &Connection, app_id: i64) -> CommandResult<AppItem> {
  conn
    .query_row(
      r#"
      SELECT
        a.id,
        a.name,
        a.description,
        COUNT(s.id) AS shortcut_count
      FROM apps a
      LEFT JOIN shortcuts s ON s.app_id = a.id
      WHERE a.id = ?1
      GROUP BY a.id
      "#,
      params![app_id],
      app_row_mapper,
    )
    .optional()
    .map_err(map_db_error)?
    .ok_or_else(|| format!("应用不存在: {app_id}"))
}

fn fetch_shortcut_by_id(conn: &Connection, shortcut_id: i64) -> CommandResult<ShortcutItem> {
  conn
    .query_row(
      "SELECT id, app_id, title, combo, notes, is_pinned FROM shortcuts WHERE id = ?1",
      params![shortcut_id],
      shortcut_row_mapper,
    )
    .optional()
    .map_err(map_db_error)?
    .ok_or_else(|| format!("快捷键不存在: {shortcut_id}"))
}

#[tauri::command]
fn list_apps(state: State<'_, DatabaseState>) -> CommandResult<Vec<AppItem>> {
  let conn = lock_conn(&state)?;
  let mut statement = conn
    .prepare(
      r#"
      SELECT
        a.id,
        a.name,
        a.description,
        COUNT(s.id) AS shortcut_count
      FROM apps a
      LEFT JOIN shortcuts s ON s.app_id = a.id
      GROUP BY a.id
      ORDER BY a.name COLLATE NOCASE ASC
      "#,
    )
    .map_err(map_db_error)?;

  let rows = statement
    .query_map([], app_row_mapper)
    .map_err(map_db_error)?;

  let apps: Result<Vec<_>, _> = rows.collect();
  apps.map_err(map_db_error)
}

#[tauri::command]
fn create_app(payload: UpsertAppPayload, state: State<'_, DatabaseState>) -> CommandResult<AppItem> {
  let name = require_non_empty(&payload.name, "应用名称")?;
  let description = normalize_optional_text(payload.description);
  let conn = lock_conn(&state)?;

  conn
    .execute(
      r#"
      INSERT INTO apps (name, description, created_at, updated_at)
      VALUES (?1, ?2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
      "#,
      params![name, description],
    )
    .map_err(map_db_error)?;

  fetch_app_by_id(&conn, conn.last_insert_rowid())
}

#[tauri::command]
fn update_app(
  app_id: i64,
  payload: UpsertAppPayload,
  state: State<'_, DatabaseState>,
) -> CommandResult<AppItem> {
  let name = require_non_empty(&payload.name, "应用名称")?;
  let description = normalize_optional_text(payload.description);
  let conn = lock_conn(&state)?;

  let changed = conn
    .execute(
      r#"
      UPDATE apps
      SET name = ?1, description = ?2, updated_at = CURRENT_TIMESTAMP
      WHERE id = ?3
      "#,
      params![name, description, app_id],
    )
    .map_err(map_db_error)?;

  if changed == 0 {
    return Err(format!("应用不存在: {app_id}"));
  }

  fetch_app_by_id(&conn, app_id)
}

#[tauri::command]
fn delete_app(app_id: i64, state: State<'_, DatabaseState>) -> CommandResult<()> {
  let conn = lock_conn(&state)?;
  let changed = conn
    .execute("DELETE FROM apps WHERE id = ?1", params![app_id])
    .map_err(map_db_error)?;

  if changed == 0 {
    Err(format!("应用不存在: {app_id}"))
  } else {
    Ok(())
  }
}

#[tauri::command]
fn list_shortcuts(app_id: i64, state: State<'_, DatabaseState>) -> CommandResult<Vec<ShortcutItem>> {
  let conn = lock_conn(&state)?;

  let app_exists: Option<i64> = conn
    .query_row("SELECT id FROM apps WHERE id = ?1", params![app_id], |row| {
      row.get::<_, i64>(0)
    })
    .optional()
    .map_err(map_db_error)?;

  if app_exists.is_none() {
    return Err(format!("应用不存在: {app_id}"));
  }

  let mut statement = conn
    .prepare(
      r#"
      SELECT id, app_id, title, combo, notes, is_pinned
      FROM shortcuts
      WHERE app_id = ?1
      ORDER BY is_pinned DESC, title COLLATE NOCASE ASC
      "#,
    )
    .map_err(map_db_error)?;

  let rows = statement
    .query_map(params![app_id], shortcut_row_mapper)
    .map_err(map_db_error)?;

  let shortcuts: Result<Vec<_>, _> = rows.collect();
  shortcuts.map_err(map_db_error)
}

#[tauri::command]
fn create_shortcut(
  payload: CreateShortcutPayload,
  state: State<'_, DatabaseState>,
) -> CommandResult<ShortcutItem> {
  let title = require_non_empty(&payload.title, "快捷键名称")?;
  let combo = require_non_empty(&payload.combo, "快捷键组合")?;
  let notes = normalize_optional_text(payload.notes);
  let conn = lock_conn(&state)?;

  conn
    .execute(
      r#"
      INSERT INTO shortcuts (app_id, title, combo, notes, created_at, updated_at)
      VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
      "#,
      params![payload.app_id, title, combo, notes],
    )
    .map_err(map_db_error)?;

  fetch_shortcut_by_id(&conn, conn.last_insert_rowid())
}

#[tauri::command]
fn update_shortcut(
  shortcut_id: i64,
  payload: UpdateShortcutPayload,
  state: State<'_, DatabaseState>,
) -> CommandResult<ShortcutItem> {
  let title = require_non_empty(&payload.title, "快捷键名称")?;
  let combo = require_non_empty(&payload.combo, "快捷键组合")?;
  let notes = normalize_optional_text(payload.notes);
  let conn = lock_conn(&state)?;

  let changed = conn
    .execute(
      r#"
      UPDATE shortcuts
      SET title = ?1, combo = ?2, notes = ?3, updated_at = CURRENT_TIMESTAMP
      WHERE id = ?4
      "#,
      params![title, combo, notes, shortcut_id],
    )
    .map_err(map_db_error)?;

  if changed == 0 {
    return Err(format!("快捷键不存在: {shortcut_id}"));
  }

  fetch_shortcut_by_id(&conn, shortcut_id)
}

#[tauri::command]
fn update_shortcut_pin(
  shortcut_id: i64,
  pinned: bool,
  state: State<'_, DatabaseState>,
) -> CommandResult<ShortcutItem> {
  let conn = lock_conn(&state)?;
  let changed = conn
    .execute(
      r#"
      UPDATE shortcuts
      SET is_pinned = ?1, updated_at = CURRENT_TIMESTAMP
      WHERE id = ?2
      "#,
      params![if pinned { 1 } else { 0 }, shortcut_id],
    )
    .map_err(map_db_error)?;

  if changed == 0 {
    return Err(format!("快捷键不存在: {shortcut_id}"));
  }

  fetch_shortcut_by_id(&conn, shortcut_id)
}

#[tauri::command]
fn delete_shortcut(shortcut_id: i64, state: State<'_, DatabaseState>) -> CommandResult<()> {
  let conn = lock_conn(&state)?;
  let changed = conn
    .execute("DELETE FROM shortcuts WHERE id = ?1", params![shortcut_id])
    .map_err(map_db_error)?;

  if changed == 0 {
    Err(format!("快捷键不存在: {shortcut_id}"))
  } else {
    Ok(())
  }
}

#[tauri::command]
fn get_settings(settings: State<'_, SettingsState>) -> CommandResult<AppSettings> {
  let toggle_shortcut = lock_shortcut(&settings)?.to_string();
  let window_position_mode = lock_position_mode(&settings)?.as_str().to_owned();
  Ok(AppSettings {
    toggle_shortcut,
    window_position_mode,
  })
}

#[tauri::command]
fn update_toggle_shortcut(
  shortcut: String,
  app: AppHandle,
  db: State<'_, DatabaseState>,
  settings: State<'_, SettingsState>,
) -> CommandResult<AppSettings> {
  let target_shortcut = require_non_empty(&shortcut, "全局唤起快捷键")?;
  let current_shortcut = {
    let guard = lock_shortcut(&settings)?;
    guard.to_string()
  };

  if target_shortcut == current_shortcut {
    let window_position_mode = lock_position_mode(&settings)?.as_str().to_owned();
    return Ok(AppSettings {
      toggle_shortcut: target_shortcut,
      window_position_mode,
    });
  }

  let shortcut_manager = app.global_shortcut();
  let _ = shortcut_manager.unregister(current_shortcut.as_str());
  if let Err(error) = shortcut_manager.register(target_shortcut.as_str()) {
    let _ = shortcut_manager.register(current_shortcut.as_str());
    return Err(map_shortcut_error(error));
  }

  {
    let conn = lock_conn(&db)?;
    if let Err(error) = upsert_setting(&conn, "toggle_shortcut", &target_shortcut) {
      let _ = shortcut_manager.unregister(target_shortcut.as_str());
      let _ = shortcut_manager.register(current_shortcut.as_str());
      return Err(map_db_error(error));
    }
  }

  {
    let mut guard = lock_shortcut(&settings)?;
    *guard = target_shortcut.clone();
  }
  let window_position_mode = lock_position_mode(&settings)?.as_str().to_owned();
  Ok(AppSettings {
    toggle_shortcut: target_shortcut,
    window_position_mode,
  })
}

#[tauri::command]
fn update_window_position_mode(
  mode: String,
  app: AppHandle,
  db: State<'_, DatabaseState>,
  settings: State<'_, SettingsState>,
) -> CommandResult<AppSettings> {
  let target_mode = WindowPositionMode::parse(&mode).ok_or_else(|| {
    "窗口位置参数无效，可选值: left_top, top_center, right_top, left_center, center, right_center, left_bottom, bottom_center, right_bottom".to_owned()
  })?;
  let current_mode = *lock_position_mode(&settings)?;
  if current_mode == target_mode {
    let toggle_shortcut = lock_shortcut(&settings)?.to_string();
    return Ok(AppSettings {
      toggle_shortcut,
      window_position_mode: target_mode.as_str().to_owned(),
    });
  }

  {
    let conn = lock_conn(&db)?;
    upsert_setting(&conn, "window_position_mode", target_mode.as_str()).map_err(map_db_error)?;
  }

  {
    let mut guard = lock_position_mode(&settings)?;
    *guard = target_mode;
  }

  if let Some(window) = app.get_webview_window("main") {
    if window.is_visible().unwrap_or(false) {
      keep_main_window_floating(&app);
    }
  }

  let toggle_shortcut = lock_shortcut(&settings)?.to_string();
  Ok(AppSettings {
    toggle_shortcut,
    window_position_mode: target_mode.as_str().to_owned(),
  })
}

fn position_main_window(app: &AppHandle) {
  let Some(window) = app.get_webview_window("main") else {
    return;
  };

  let monitor = window.cursor_position().ok().and_then(|cursor| {
    window
      .available_monitors()
      .ok()
      .and_then(|monitors| {
        monitors.into_iter().find(|monitor| {
          let position = monitor.position();
          let size = monitor.size();

          let left = position.x as f64;
          let top = position.y as f64;
          let right = left + size.width as f64;
          let bottom = top + size.height as f64;

          cursor.x >= left && cursor.x < right && cursor.y >= top && cursor.y < bottom
        })
      })
      .or_else(|| {
        window
          .monitor_from_point(cursor.x, cursor.y)
          .ok()
          .flatten()
      })
  })
  .or_else(|| window.current_monitor().ok().flatten())
  .or_else(|| window.primary_monitor().ok().flatten());
  let Some(monitor) = monitor else {
    return;
  };

  let window_size = window
    .outer_size()
    .or_else(|_| window.inner_size())
    .ok()
    .map(|size| (size.width as i32, size.height as i32))
    .unwrap_or((900, 700));

  let work_area = monitor.work_area();
  let right_edge = work_area.position.x + work_area.size.width as i32;
  let left_edge = work_area.position.x;
  let top_edge = work_area.position.y;
  let bottom_edge = work_area.position.y + work_area.size.height as i32;

  let mode = app
    .try_state::<SettingsState>()
    .and_then(|settings| settings.window_position_mode.lock().ok().map(|mode| *mode))
    .unwrap_or(WindowPositionMode::TopCenter);

  let x = match mode {
    WindowPositionMode::LeftTop
    | WindowPositionMode::LeftCenter
    | WindowPositionMode::LeftBottom => left_edge + FLOATING_MARGIN_X,
    WindowPositionMode::TopCenter | WindowPositionMode::Center | WindowPositionMode::BottomCenter => {
      left_edge + ((work_area.size.width as i32 - window_size.0) / 2)
    }
    WindowPositionMode::RightTop | WindowPositionMode::RightCenter | WindowPositionMode::RightBottom => {
      (right_edge - window_size.0 - FLOATING_MARGIN_X).max(left_edge)
    }
  };
  let y = match mode {
    WindowPositionMode::LeftTop | WindowPositionMode::TopCenter | WindowPositionMode::RightTop => {
      top_edge + FLOATING_MARGIN_Y
    }
    WindowPositionMode::LeftCenter | WindowPositionMode::Center | WindowPositionMode::RightCenter => {
      top_edge + ((work_area.size.height as i32 - window_size.1) / 2)
    }
    WindowPositionMode::LeftBottom
    | WindowPositionMode::BottomCenter
    | WindowPositionMode::RightBottom => {
      (bottom_edge - window_size.1 - FLOATING_MARGIN_Y).max(top_edge)
    }
  };

  let _ = window.set_position(Position::Physical(PhysicalPosition::new(x, y)));
}

fn keep_main_window_floating(app: &AppHandle) {
  if let Some(window) = app.get_webview_window("main") {
    let _ = window.set_always_on_top(true);
    let _ = window.set_visible_on_all_workspaces(true);
  }
  position_main_window(app);
}

fn toggle_main_window(app: &AppHandle) {
  if let Some(window) = app.get_webview_window("main") {
    let is_visible = window.is_visible().unwrap_or(false);
    if is_visible {
      let _ = window.hide();
      return;
    }

    let _ = window.hide();
    keep_main_window_floating(app);
    let _ = window.show();
    let _ = window.set_focus();
  }
}

fn setup_tray(app: &AppHandle, toggle_shortcut: &str) -> Result<(), Box<dyn Error>> {
  let toggle_item =
    MenuItem::with_id(app, "toggle-window", "显示/隐藏窗口", true, Some(toggle_shortcut))?;
  let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
  let menu = Menu::with_items(app, &[&toggle_item, &quit_item])?;

  let mut tray_builder = TrayIconBuilder::with_id("main-tray")
    .menu(&menu)
    .show_menu_on_left_click(false)
    .on_menu_event(|app, event| {
      if event.id() == "toggle-window" {
        toggle_main_window(app);
      } else if event.id() == "quit" {
        app.exit(0);
      }
    })
    .on_tray_icon_event(|tray: &tauri::tray::TrayIcon<_>, event: TrayIconEvent| {
      if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
      } = event
      {
        toggle_main_window(tray.app_handle());
      }
    });

  if let Some(icon) = app.default_window_icon() {
    tray_builder = tray_builder.icon(icon.clone());
  }

  let _ = tray_builder.build(app)?;
  Ok(())
}

fn initialize_schema(conn: &Connection) -> rusqlite::Result<()> {
  conn.execute_batch(
    r#"
    PRAGMA foreign_keys = ON;

    CREATE TABLE IF NOT EXISTS app_meta (
      key TEXT PRIMARY KEY,
      value TEXT NOT NULL
    );

    INSERT OR IGNORE INTO app_meta (key, value) VALUES ('schema_version', '1');

    CREATE TABLE IF NOT EXISTS app_settings (
      key TEXT PRIMARY KEY,
      value TEXT NOT NULL
    );

    INSERT OR IGNORE INTO app_settings (key, value)
    VALUES ('toggle_shortcut', 'CmdOrCtrl+Shift+I');

    INSERT OR IGNORE INTO app_settings (key, value)
    VALUES ('window_position_mode', 'top_center');

    CREATE TABLE IF NOT EXISTS apps (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT NOT NULL UNIQUE,
      description TEXT,
      created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
      updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS shortcuts (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      app_id INTEGER NOT NULL,
      title TEXT NOT NULL,
      combo TEXT NOT NULL,
      notes TEXT,
      is_pinned INTEGER NOT NULL DEFAULT 0,
      created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
      updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
      UNIQUE(app_id, combo),
      FOREIGN KEY(app_id) REFERENCES apps(id) ON DELETE CASCADE
    );
    "#,
  )?;
  ensure_shortcuts_is_pinned_column(conn)?;
  Ok(())
}

fn ensure_shortcuts_is_pinned_column(conn: &Connection) -> rusqlite::Result<()> {
  // Backward compatibility for existing databases created before `is_pinned` was introduced.
  let mut statement = conn.prepare("PRAGMA table_info(shortcuts)")?;
  let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
  let mut has_is_pinned = false;
  for column in columns {
    if column?.eq_ignore_ascii_case("is_pinned") {
      has_is_pinned = true;
      break;
    }
  }

  if !has_is_pinned {
    conn.execute(
      "ALTER TABLE shortcuts ADD COLUMN is_pinned INTEGER NOT NULL DEFAULT 0",
      [],
    )?;
  }

  Ok(())
}

fn load_toggle_shortcut(conn: &Connection) -> Result<String, Box<dyn Error>> {
  let value = read_setting(conn, "toggle_shortcut")?;
  if let Some(shortcut) = value {
    let trimmed = shortcut.trim();
    if trimmed.is_empty() {
      upsert_setting(conn, "toggle_shortcut", DEFAULT_TOGGLE_SHORTCUT)?;
      return Ok(DEFAULT_TOGGLE_SHORTCUT.to_owned());
    }
    // Upgrade old default values to the current default shortcut.
    if trimmed == LEGACY_DEFAULT_TOGGLE_SHORTCUT {
      upsert_setting(conn, "toggle_shortcut", DEFAULT_TOGGLE_SHORTCUT)?;
      return Ok(DEFAULT_TOGGLE_SHORTCUT.to_owned());
    }
    return Ok(trimmed.to_owned());
  }

  upsert_setting(conn, "toggle_shortcut", DEFAULT_TOGGLE_SHORTCUT)?;
  Ok(DEFAULT_TOGGLE_SHORTCUT.to_owned())
}

fn load_window_position_mode(conn: &Connection) -> Result<WindowPositionMode, Box<dyn Error>> {
  let value = read_setting(conn, "window_position_mode")?;
  if let Some(raw) = value {
    if let Some(mode) = WindowPositionMode::parse(&raw) {
      return Ok(mode);
    }
  }

  upsert_setting(
    conn,
    "window_position_mode",
    DEFAULT_WINDOW_POSITION_MODE,
  )?;
  Ok(WindowPositionMode::parse(DEFAULT_WINDOW_POSITION_MODE).unwrap_or(WindowPositionMode::TopCenter))
}

fn setup_database(
  app: &AppHandle,
) -> Result<(DatabaseState, String, WindowPositionMode), Box<dyn Error>> {
  let mut app_data_dir = app.path().app_data_dir()?;
  fs::create_dir_all(&app_data_dir)?;
  app_data_dir.push(DB_FILE_NAME);

  let conn = Connection::open(app_data_dir)?;
  initialize_schema(&conn)?;
  let toggle_shortcut = load_toggle_shortcut(&conn)?;
  let window_position_mode = load_window_position_mode(&conn)?;

  Ok((
    DatabaseState {
      conn: Mutex::new(conn),
    },
    toggle_shortcut,
    window_position_mode,
  ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(
      tauri_plugin_log::Builder::default()
        .level(log::LevelFilter::Info)
        .build(),
    )
    .plugin(
      tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
          if event.state == ShortcutState::Pressed {
            toggle_main_window(app);
          }
        })
        .build(),
    )
    .setup(|app| {
      let (database, toggle_shortcut, window_position_mode) = setup_database(app.handle())?;
      app.manage(database);
      app.manage(SettingsState {
        toggle_shortcut: Mutex::new(toggle_shortcut.clone()),
        window_position_mode: Mutex::new(window_position_mode),
      });
      setup_tray(app.handle(), &toggle_shortcut)?;
      keep_main_window_floating(app.handle());

      let shortcut_manager = app.handle().global_shortcut();
      if let Err(error) = shortcut_manager.register(toggle_shortcut.as_str()) {
        if toggle_shortcut != DEFAULT_TOGGLE_SHORTCUT {
          log::warn!(
            "register custom shortcut `{}` failed: {}. fallback to default",
            toggle_shortcut,
            error
          );
          shortcut_manager.register(DEFAULT_TOGGLE_SHORTCUT)?;

          {
            let db_state = app.state::<DatabaseState>();
            let conn_lock = db_state.conn.lock();
            if let Ok(conn) = conn_lock {
              let _ = upsert_setting(&conn, "toggle_shortcut", DEFAULT_TOGGLE_SHORTCUT);
            }
          }
          {
            let settings_state = app.state::<SettingsState>();
            let shortcut_lock = settings_state.toggle_shortcut.lock();
            if let Ok(mut shortcut) = shortcut_lock {
              *shortcut = DEFAULT_TOGGLE_SHORTCUT.to_owned();
            }
          }
        } else {
          return Err(Box::new(error));
        }
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      get_settings,
      update_toggle_shortcut,
      update_window_position_mode,
      list_apps,
      create_app,
      update_app,
      delete_app,
      list_shortcuts,
      create_shortcut,
      update_shortcut,
      update_shortcut_pin,
      delete_shortcut
    ])
    .on_window_event(|window, event| {
      if window.label() == "main" {
        match event {
          WindowEvent::CloseRequested { api, .. } => {
            let _ = window.hide();
            api.prevent_close();
          }
          WindowEvent::Focused(false) => {
            let _ = window.hide();
          }
          _ => {}
        }
      }
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn normalize_optional_text_should_trim_and_drop_empty() {
    assert_eq!(normalize_optional_text(None), None);
    assert_eq!(normalize_optional_text(Some("   ".to_owned())), None);
    assert_eq!(
      normalize_optional_text(Some("  hello  ".to_owned())),
      Some("hello".to_owned())
    );
  }

  #[test]
  fn initialize_schema_should_create_default_toggle_shortcut() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    initialize_schema(&conn).expect("initialize schema");

    let toggle_shortcut = load_toggle_shortcut(&conn).expect("load default toggle shortcut");
    assert_eq!(toggle_shortcut, DEFAULT_TOGGLE_SHORTCUT);
  }

  #[test]
  fn shortcut_combo_should_be_unique_per_app() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    initialize_schema(&conn).expect("initialize schema");

    conn
      .execute(
        "INSERT INTO apps (name, description) VALUES (?1, ?2)",
        params!["VS Code", Option::<String>::None],
      )
      .expect("insert app");
    let app_id = conn.last_insert_rowid();

    conn
      .execute(
        "INSERT INTO shortcuts (app_id, title, combo, notes) VALUES (?1, ?2, ?3, ?4)",
        params![app_id, "命令面板", "Command+Shift+P", Option::<String>::None],
      )
      .expect("insert shortcut");

    let duplicated = conn.execute(
      "INSERT INTO shortcuts (app_id, title, combo, notes) VALUES (?1, ?2, ?3, ?4)",
      params![app_id, "重复命令", "Command+Shift+P", Option::<String>::None],
    );

    assert!(duplicated.is_err());
  }
}
