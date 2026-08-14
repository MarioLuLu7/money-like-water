# Money Like Water

Money Like Water is a Windows desktop usage meter for AI services. It shows remaining usage, balance, or quota information in a compact taskbar overlay, while keeping detailed data-source configuration in a Tauri desktop app.

The app is built with Tauri 2, React, TypeScript, and Rust. It currently includes presets for ChatGPT, AI-MEMBER, Kimi Code, DeepSeek, and custom HTTP sources.

## Download

Download the latest Windows build from [GitHub Releases](https://github.com/MarioLuLu7/money-like-water/releases/latest).

[Download v0.1.3 for Windows x64](https://github.com/MarioLuLu7/money-like-water/releases/download/v0.1.3/Money.Like.Water_0.1.3_x64-setup.exe)

## Features

- Windows taskbar meter with configurable position, text size, colors, and reset-time display.
- Multi-source usage monitoring with selectable taskbar rotation.
- Built-in HTTP presets for common AI usage or balance endpoints.
- Custom HTTP data sources with editable headers, bearer token auth, timeout, and Transform JS response mapping.
- English and Chinese interface language setting.
- System tray integration for quick access and refresh.
- Local settings persistence through the Tauri backend.

## Requirements

- Windows 10 or later.
- Node.js and npm.
- Rust stable toolchain.
- Tauri 2 prerequisites for Windows development.

## Development

Install dependencies:

```bash
npm install
```

Run the frontend dev server:

```bash
npm run dev
```

Run the Tauri desktop app in development mode:

```bash
npm run tauri:dev
```

Build the frontend:

```bash
npm run build
```

Build the desktop app:

```bash
npm run tauri:build
```

## Release Workflow

Run the one-command release script from the repository root:

```powershell
npm run release
```

The script installs dependencies with `npm ci`, updates app metadata and README download links when `--version` is provided, builds the frontend, builds the Tauri installer bundle, copies installer artifacts into `releases/v<version>`, writes `SHA256SUMS.txt`, commits changes, pushes the current branch to `origin`, creates or updates the `v<version>` tag, and uploads the installer files to GitHub Releases.

GitHub Release publishing uses the GitHub CLI (`gh`). If `gh` is missing, the script tries to install it with `winget`; if it is not signed in yet, the script starts `gh auth login`.

Automatic updates use Tauri's signed updater. Keep the local private key at `.tauri/updater.key` or set `TAURI_SIGNING_PRIVATE_KEY_PATH` / `TAURI_SIGNING_PRIVATE_KEY` before releasing. The release script signs installer artifacts and uploads `latest.json` to GitHub Releases for the app's in-app update check.

To release a new version and update the app metadata at the same time:

```powershell
npm run release -- --version 0.1.1 --message "release: v0.1.1"
```

Useful options:

```powershell
npm run release -- --skip-git
npm run release -- --no-commit
npm run release -- --no-push
npm run release -- --no-github-release
npm run release -- --no-auto-install-github-cli
npm run release -- --no-interactive-github-login
npm run release -- --skip-install
```

## Configuration

Open the app settings to configure data sources and taskbar behavior. For HTTP sources, provide the service base URL, endpoint, token, optional headers, and Transform JS that converts the response JSON into normalized usage windows.

For ChatGPT, the default preset targets:

```text
https://chatgpt.com/backend-api/wham/usage
```

It expects a valid bearer token supplied by the user. Tokens and source settings are stored locally by the desktop app.

## Project Status

This project is in early development. APIs for unofficial service usage endpoints may change, so presets and Transform JS mappings may need updates over time.

## License

No license has been selected yet.

---

# Money Like Water 中文说明

Money Like Water 是一个面向 Windows 的 AI 服务用量监控桌面应用。它可以在任务栏中显示紧凑的用量条，展示剩余额度、余额或配额信息，同时通过 Tauri 桌面窗口管理更详细的数据源配置。

应用使用 Tauri 2、React、TypeScript 和 Rust 构建。目前内置 ChatGPT、AI-MEMBER、Kimi Code、DeepSeek 以及自定义 HTTP 数据源预设。

## 下载

请从 [GitHub Releases](https://github.com/MarioLuLu7/money-like-water/releases/latest) 下载最新版 Windows 构建。

[下载 v0.1.3 Windows x64 版本](https://github.com/MarioLuLu7/money-like-water/releases/download/v0.1.3/Money.Like.Water_0.1.3_x64-setup.exe)

## 功能

- 可嵌入 Windows 任务栏的用量条，支持位置、字号、颜色和重置时间显示配置。
- 支持多个数据源，并可选择哪些数据源在任务栏轮播展示。
- 内置常见 AI 用量或余额接口的 HTTP 预设。
- 支持自定义 HTTP 数据源，可配置请求头、Bearer Token、超时时间和 Transform JS 响应映射脚本。
- 支持英文和中文界面。
- 支持系统托盘快捷入口和刷新操作。
- 通过 Tauri 后端在本地持久化设置。

## 环境要求

- Windows 10 或更高版本。
- Node.js 和 npm。
- Rust stable 工具链。
- Windows 上的 Tauri 2 开发依赖。

## 本地开发

安装依赖：

```bash
npm install
```

启动前端开发服务器：

```bash
npm run dev
```

以开发模式启动 Tauri 桌面应用：

```bash
npm run tauri:dev
```

构建前端：

```bash
npm run build
```

构建桌面应用：

```bash
npm run tauri:build
```

## 配置说明

打开应用设置即可配置数据源和任务栏展示方式。HTTP 数据源需要填写服务地址、接口路径、Token、可选请求头，以及用于把响应 JSON 转成标准用量窗口的 Transform JS。

ChatGPT 默认预设使用：

```text
https://chatgpt.com/backend-api/wham/usage
```

该预设需要用户自行提供有效的 Bearer Token。Token 和数据源设置由桌面应用保存在本地。

## 项目状态

项目仍处于早期开发阶段。部分服务的非公开用量接口可能发生变化，因此预设和 Transform JS 映射可能需要随时调整。

## 许可证

尚未选择许可证。
