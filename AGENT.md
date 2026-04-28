# AGENT.md — Translate 项目指南

## 项目概述

**Translate**（划词翻译）是一个基于 Tauri 2 的桌面划词翻译应用，通过可自定义的全局快捷键（默认 Alt+F1）获取屏幕选中文本，利用 LLM API（DeepSeek / 豆包）进行流式翻译，结果展示在无边框悬浮面板中。

- **版本**：0.3.1
- **标识符**：`com.translate.qiumo`
- **作者**：PengWee Wang
- **主要平台**：Windows（交叉编译），支持 Linux / macOS

---

## 项目结构

```
/workspace/
├── config/
│   └── config.toml              # 示例/默认运行时配置（TOML 格式）
├── src/                          # 前端（无构建工具链，纯 HTML/JS/CSS）
│   ├── index.html                # 主页面：拖拽栏 + 翻译内容区
│   ├── main.js                   # 核心前端逻辑：翻译、SSE 流式渲染
│   ├── styles.css                # 暗色主题样式
│   └── assets/
│       ├── marked.esm.js         # Markdown 解析器（ESM）
│       └── katex/                # KaTeX 数学公式渲染
│           ├── katex.min.css / .js / .mjs
│           ├── contrib/
│           │   └── auto-render.mjs  # 自动渲染 LaTeX
│           └── fonts/            # 20 个 woff2 字体文件
├── src-tauri/                    # Tauri 后端（Rust）
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── capabilities/
│   │   ├── default.json          # panel 窗口权限
│   │   └── desktop.json          # main 窗口桌面权限
│   ├── icons/
│   └── src/
│       ├── main.rs               # 入口：调用 translate_lib::run()
│       ├── lib.rs                # 应用启动、模块编排、插件注册
│       ├── config.rs             # 配置数据结构、文件 I/O、模型切换
│       ├── prompt.rs             # 文本类型检测 + prompt 构建
│       ├── commands.rs           # Tauri 命令（前端可调用）
│       ├── tray.rs               # 系统托盘菜单
│       ├── shortcut.rs           # 全局快捷键
│       └── windows.rs            # 面板窗口创建
├── .github/workflows/release.yml # GitHub Actions 发布流程
├── .ide/Dockerfile               # 开发容器（Rust + Node 18 + MinGW 交叉编译）
├── .cnb.yml                      # CNB CI 配置：开发环境 + push 自动构建发布
└── package.json                  # 前端依赖管理
```

---

## 架构设计

### 数据流

```
用户选中文本 → 快捷键（默认 Alt+F1）释放 →
  Rust 全局快捷键处理器：selection::get_text() 获取文本 →
  emit("get_text") 到 panel 窗口 →
  JS 收到事件 → 并行调用 get_config + get_translate_prompt →
  Rust prompt.rs 自动判定文本类型（单词/句子）→ 选择对应 prompt 模板 →
  JS 构建 SSE 请求到 LLM API →
  流式解析 SSE chunks → 增量 Markdown + KaTeX 渲染
```

### 文本类型判定逻辑（prompt.rs）

| 条件 | 类型 | 使用的 Prompt |
|------|------|--------------|
| 去空白后仅含 ASCII 字母且长度 ≤ 50 | Word | `select.word_prompt`（词典释义格式） |
| 其他（含空格/标点/数字/中文等） | Sentence | `select.sentence_prompt`（翻译格式） |

Prompt 模板使用 `${text}` 占位符，由 `build_prompt()` 替换为实际文本。

---

## Rust 后端模块职责

### `lib.rs` — 应用入口与编排

- 声明所有子模块
- 注册 Tauri 插件：`autostart`、`opener`（global-shortcut 在 `setup` 中通过 `app.handle().plugin()` 注册）
- 读取配置中的快捷键 → 创建 `AppState` 共享状态 → 调用 `tray::init_tray`、`windows::panel`、`shortcut::setup_shortcut`
- 将 `AppState` 通过 `app.manage()` 注入 Tauri 状态管理
- 注册 invoke handler：`send_text`, `hide_panel`, `start_drag`, `get_config`, `get_translate_prompt`, `set_shortcut`
- 共享状态：`AppState`（`shortcut_enabled` + `shortcut_key`），通过 `Arc<Mutex>` 在模块间共享

### `config.rs` — 配置管理

- **数据结构**：`Config` → `SelectConfig` + `DeepSeekConfig` + `DoubaoConfig`
- `SelectConfig` 字段：`llm`（当前模型名）、`shortcut`（全局快捷键，默认 `"Alt+F1"`）、`word_prompt`、`sentence_prompt`
- 配置文件路径：`~/.local/translate/config.toml`
- 核心函数：`read_or_create_config()`、`open_config_file()`、`switch_model()`、`update_shortcut()`
- 旧配置（单个 `prompt` 字段）会因 `#[serde(default)]` 自动回退到默认值

### `prompt.rs` — 文本分类与 Prompt 构建

- `TextType` 枚举：`Word` / `Sentence`
- `detect_text_type(text)` — 纯逻辑判定
- `build_prompt(text, word_prompt, sentence_prompt)` — 根据类型选择模板并替换 `${text}`
- 包含单元测试

### `commands.rs` — Tauri 命令层

| 命令 | 参数 | 说明 |
|------|------|------|
| `send_text` | 无 | 获取当前选中文本 |
| `hide_panel` | window | 隐藏面板窗口 |
| `start_drag` | window | 开始窗口拖拽（仅桌面） |
| `get_config` | 无 | 返回完整 Config |
| `get_translate_prompt` | text: String | 自动判定文本类型并返回构建好的 prompt |
| `set_shortcut` | app, shortcut_str: String, state | 设置自定义快捷键，更新配置并重新注册 |

### `tray.rs` — 系统托盘

- 定义 `AppState` 结构体（`shortcut_enabled: bool`, `shortcut_key: String`），通过 `Arc<Mutex>` 共享
- 菜单项：Config / Model 子菜单（DeepSeek √ / Doubao √）/ Shortcut 开关 / Autostart / Quit
- 事件处理：打开配置、切换模型、开关快捷键、开关自启动

### `shortcut.rs` — 全局快捷键

- 从配置文件读取快捷键字符串，利用 `tauri-plugin-global-shortcut` 原生字符串解析（`TryFrom<&str>`）
- `setup_shortcut(app, shortcut_str)` — 初始化注册，释放时获取选中文本 → emit → 显示并聚焦面板
- `reregister_shortcut(app, old, new)` — 注销旧快捷键并注册新快捷键
- 支持格式：`"Alt+F1"`, `"Ctrl+Shift+A"`, `"Super+Space"` 等

### `windows.rs` — 窗口创建

- 面板窗口：400×300，无边框，可调整大小，不可最小化/最大化，跳过任务栏，初始隐藏，有阴影，居中显示

---

## 关键依赖

### Rust（Cargo.toml）

| 依赖 | 用途 |
|------|------|
| `tauri` 2 | 应用框架，启用 `tray-icon` 特性 |
| `tauri-plugin-global-shortcut` 2 | 全局快捷键注册/注销，原生字符串解析 |
| `tauri-plugin-autostart` 2 | 开机自启动 |
| `tauri-plugin-opener` 2 | 使用系统默认程序打开文件 |
| `tauri-plugin-clipboard-manager` 2 | 剪贴板操作 |
| `selection` 1.2 | 获取屏幕选中文本 |
| `mouse_position` 0.1 | 获取鼠标位置 |
| `toml` 0.9 | TOML 配置序列化/反序列化 |
| `dirs` 6.0 | 获取系统目录（如 home） |
| `serde` / `serde_json` | 数据序列化 |

### JavaScript（package.json）

| 依赖 | 用途 |
|------|------|
| `@tauri-apps/cli` ^2 | Tauri 开发/构建 CLI |
| `@tauri-apps/plugin-global-shortcut` ^2 | 前端快捷键 API |
| `@tauri-apps/plugin-http` ^2 | HTTP 请求（SSE 流式 fetch） |
| `@tauri-apps/plugin-clipboard-manager` ^2 | 前端剪贴板 API |
| `@tauri-apps/plugin-autostart` ^2 | 前端自启动 API |
| `katex` ^0.16 | 数学公式渲染 |

---

## 前端设计

### 技术栈

- 无构建工具链，纯 HTML/JS/CSS 通过 `withGlobalTauri: true` 访问 `window.__TAURI__`
- Markdown 渲染：Marked.js（GFM + breaks）
- 数学公式：KaTeX auto-render（`$$`, `$`, `\(`, `\[` 定界符）
- CSP：`null`（允许 fetch 外部 API）

### main.js 核心流程

1. 监听 `get_text` 事件
2. 并行调用 `get_config` + `get_translate_prompt`
3. 使用 AbortController 取消前次请求
4. SSE 流式 fetch → 逐 chunk 解析 `delta.content`
5. 增量拼接 Markdown → `marked.parse()` + `renderMathInElement()` 渲染

### UI 交互

- 拖拽栏：mousedown → `start_drag`
- 关闭按钮：调用 `hide_panel`
- 置顶按钮：切换 `alwaysOnTop`
- ESC 键：隐藏面板

---

## 配置格式

运行时配置文件位于 `~/.local/translate/config.toml`：

```toml
[select]
llm = "deepseek"
shortcut = "Alt+F1"
word_prompt = "词典释义 prompt...${text}"
sentence_prompt = "翻译 prompt...${text}"

[deepseek]
api_base = "https://api.deepseek.com/v1"
api_key = "sk-..."
default_model = "deepseek-chat"
temperature = 1.3
thinking = "disabled"

[doubao]
api_base = "https://ark.cn-beijing.volces.com/api/v3"
api_key = "..."
default_model = "doubao-seed-1-6-251015"
temperature = 1.3
thinking = "disabled"
```

---

## 代码规范

### Rust

- 模块按职责拆分：`config` / `prompt` / `commands` / `tray` / `shortcut` / `windows`
- 公开函数使用 `pub`，内部函数保持私有
- 配置结构体使用 `serde` derive，可选字段用 `#[serde(default = "...")]` 提供默认值
- 命令函数使用 `#[tauri::command]` 注解，集中在 `commands.rs`
- 桌面限定代码使用 `#[cfg(desktop)]`
- 使用 `result` 处理错误，不使用 `unwrap()` 在生产路径中

### JavaScript

- ES Module 风格（`type: "module"`）
- 异步操作使用 `async/await`
- 使用 `AbortController` 实现请求取消
- 通过 `window.__TAURI__` 全局对象访问 Tauri API

### CSS

- 暗色主题（`#2a2a2a` 主背景）
- 隐藏滚动条（webkit + Firefox）
- 翻译内容区 Markdown 样式（标题、代码、引用等）

---

## 已实现功能

1. **全局划词翻译**：可自定义快捷键（默认 Alt+F1）获取选中文本，自动翻译
2. **双 LLM 提供商**：DeepSeek / 豆包，托盘菜单切换
3. **智能文本分类**：程序内置判定单词/句子，使用不同 prompt
   - 单词 → 词典释义格式（Markdown：词性缩写 + 中文释义）
   - 句子/短语 → 准确流畅翻译为中文
4. **流式输出**：SSE 实时渲染翻译结果
5. **Markdown + LaTeX 渲染**：支持 GFM、行内/块级数学公式
6. **无边框悬浮面板**：可拖拽、可调整大小、可置顶
7. **系统托盘**：配置、模型切换、快捷键开关、自启动、退出
8. **配置管理**：TOML 格式，自动创建，系统编辑器打开
9. **开机自启动**
10. **请求取消**：新翻译自动取消前次未完成的请求
11. **自定义快捷键**：通过配置文件 `shortcut` 字段或 `set_shortcut` 命令动态修改，支持 Alt/Ctrl/Shift/Super + 按键组合

---

## 构建与部署

### 本地开发

```bash
npm install
npm run tauri dev
```

### 交叉编译到 Windows（Linux 环境）

```bash
npm run tauri build -- --target x86_64-pc-windows-gnu
# 或使用 MSVC target（需要 link.exe）
npm run tauri build -- --target x86_64-pc-windows-msvc
```

### CI/CD

#### GitHub Actions

- Push 到 master 自动构建 Windows 版本并发布 Release
- 配置文件：`.github/workflows/release.yml`

#### CNB（云原生构建）

- 配置文件：`.cnb.yml`
- **开发环境**：使用 `.ide/Dockerfile` 构建 VS Code 开发容器（Rust + Node 18 + MinGW）
- **发布管道**（push 到 master 触发）：
  1. `npm install` 安装前端依赖
  2. `npm run tauri build -- --target x86_64-pc-windows-gnu` 交叉编译 Windows 版本
  3. 扫描 `src-tauri/target/x86_64-pc-windows-gnu/release/bundle/` 下的 `.exe` / `.msi` 产物
  4. 使用 `cnbcool/changelog` 插件生成变更日志
  5. `git:release` 阶段创建 Release，标题为分支名
  6. `cnbcool/attachments` 插件上传 `.msi` 和 `.exe` 到 Release

---

## 扩展指南

### 添加新的 LLM 提供商

1. 在 `config.rs` 中添加新的 `XxxConfig` 结构体（含 `api_base`, `api_key`, `default_model`, `temperature`, `thinking`）
2. 在 `Config` 中添加新字段
3. 在 `tray.rs` 中添加新的 CheckMenuItem 和菜单事件处理
4. 前端无需修改（自动根据 `config[llm]` 读取配置）

### 添加新的 Tauri 命令

1. 在 `commands.rs` 中添加 `#[tauri::command]` 函数
2. 在 `lib.rs` 的 `generate_handler!` 宏中注册

### 修改文本分类逻辑

1. 修改 `prompt.rs` 中的 `detect_text_type()` 函数
2. 如需新增文本类型，扩展 `TextType` 枚举和 `SelectConfig` 的 prompt 字段
3. 同步更新 `build_prompt()` 函数

### 更新版本号

需要同步修改以下 4 个文件中的版本号：

| 文件 | 字段 |
|------|------|
| `src-tauri/Cargo.toml` | `version = "x.y.z"` |
| `src-tauri/tauri.conf.json` | `"version": "x.y.z"` |
| `package.json` | `"version": "x.y.z"` |
| `AGENT.md` | `**版本**：x.y.z` |

---

## 开发踩坑

> 详见 [pitfalls.md](./pitfalls.md)
