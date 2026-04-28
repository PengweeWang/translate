# Translate

基于 Tauri v2 的划词翻译桌面应用，支持 DeepSeek / 豆包大模型 API，流式输出 Markdown 渲染。

## 功能

- **划词翻译** — 设置全局快捷键（默认 `Alt+F1`），选中文本后释放快捷键即可翻译
- **智能识别** — 自动区分英文单词（查词典释义）与句子/段落（翻译），支持自定义 Prompt 模板
- **多模型支持** — 已集成 DeepSeek 与火山引擎豆包，系统托盘一键切换
- **流式输出** — 基于 SSE 实时渲染翻译结果，支持 Markdown 与 LaTeX 公式（KaTeX）
- **系统托盘** — 托盘菜单提供配置管理、模型切换、快捷键开关、开机自启等功能
- **窗口操作** — 无边框可拖拽面板，支持置顶切换（🔼 按钮），按 `Esc` 关闭，`Alt + 拖拽` 调整位置
- **快捷键重载** — 可在配置文件中自定义快捷键，支持运行时热重载
- **配置管理** — 配置文件位于 `~/.local/translate/config.toml`，支持通过系统编辑器直接编辑

## 使用教程

### 设置 API Key

应用支持 DeepSeek 和火山引擎豆包两种模型。

| 平台 | 申请地址 |
|------|---------|
| DeepSeek | https://platform.deepseek.com/api_keys |
| 火山引擎 | https://console.volcengine.com/ark/region:ark+cn-beijing/apiKey |

> 火山引擎新用户通常会赠送 50 万 token 额度，可供试用。

配置文件首次启动时自动生成于 `~/.local/translate/config.toml`，编辑对应模型的 `api_key` 即可。

### 快捷键

| 操作 | 按键 |
|------|------|
| 划词翻译 | `Alt+F1`（默认，可自定义） |
| 关闭窗口 | `Esc` |
| 拖拽窗口 | `Alt + 鼠标拖拽` |

### 系统托盘

| 菜单项 | 功能 |
|--------|------|
| Config | 用系统编辑器打开配置文件 |
| Model | 切换 DeepSeek / Doubao 模型 |
| Enable/Disable Shortcut | 启用/关闭全局快捷键 |
| Autostart | 开机自启（切换后即时生效） |
| Quit | 完全退出应用 |

### 智能 Prompt 切换

选定文本后自动判断类型：

- **单词**（纯字母，长度 ≤ 50）→ 使用词典释义模版
- **句子/段落/含数字或符号** → 使用翻译模版

可在配置文件 `config.toml` 中自定义 `word_prompt` 和 `sentence_prompt`，模板中使用 `${text}` 占位。

## 开发

```bash
# 安装前端依赖
npm install

# 启动开发模式
npm run tauri dev

# 构建发布版本
npm run tauri build
```

### 技术栈

| 层 | 技术 |
|---|------|
| 桌面框架 | Tauri v2 (Rust) |
| 前端 | 原生 JavaScript + HTML + CSS |
| Markdown 渲染 | marked |
| 公式渲染 | KaTeX (本地) |
| 大模型 API | DeepSeek API / 火山引擎 Ark API |

## Todo

- [ ] 更加完善的错误处理
- [ ] 更多 API 支持（OpenAI、Claude 等）
- [ ] 自定义主题 / 配色
- [ ] 翻译历史记录
