# Shortcut Key Map

一个基于 **Tauri 2 + Vue 3 + SQLite** 的桌面快捷键记忆工具，支持菜单栏常驻、全局唤起、快捷键录入与分类管理。

![alt text](VIEW.png)

## 功能特性

- 应用管理：新增、编辑、删除应用（删除含二次确认）
- 快捷键管理：新增、编辑、删除快捷键（删除含二次确认）
- 快捷键置顶：可将不熟悉的快捷键置顶优先展示
- 两种快捷键录入方式：
  - 监听输入：直接按下组合键录入
  - 下拉选择：修饰键 + 主键组合
- 本地 SQLite 持久化存储
- macOS 菜单栏托盘常驻：点击图标快速显示/隐藏窗口
- 全局快捷键唤起/收起窗口（默认：`CmdOrCtrl+Shift+I`，可在设置中修改）
- 多显示器支持：窗口根据鼠标所在屏幕弹出
- 窗口位置模式：支持九宫格位置（左上/中上/右上/左中/中间/右中/左下/中下/右下）
- 窗口行为：始终置顶、全工作区可见、失焦自动隐藏
- 苹果风界面：圆角窗口、毛玻璃面板、轻量 toast 提示

## 技术栈

- Tauri 2
- Vue 3 + TypeScript
- Vite
- rusqlite（SQLite）

## 本地开发

```bash
npm install
npm run tauri dev
```

## 构建

```bash
npm run tauri build
```

构建产物（macOS）默认路径：

- `src-tauri/target/release/bundle/macos/Shortcut Key Map.app`

## 常用脚本

```bash
npm run build           # 前端构建
npm run check:backend   # Rust 后端检查
npm run test:backend    # Rust 后端测试
npm run verify          # 一键回归检查（含 debug 构建）
```

## 配置说明

当前窗口配置位于：

- `src-tauri/tauri.conf.json`

默认使用：

- `1280 x 960` 固定窗口
- `transparent + decorations: false`
- macOS `windowEffects.radius` 圆角效果

## 数据库

- 文件名：`shortcut-key-map.sqlite3`
- 存储位置：Tauri `app_data_dir`（按操作系统用户目录自动解析）

## 开源协议

本项目采用 [MIT License](./LICENSE)。
