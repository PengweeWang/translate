# 开发踩坑

## 1. `tauri_plugin_global_shortcut::Error` 无法直接转为 `tauri::Error`

`Builder::with_shortcut()` 返回 `Result<_, tauri_plugin_global_shortcut::Error>`，该错误类型未实现 `From<T> for tauri::Error`，因此不能用 `?` 直接抛出到 `tauri::Result`。

**正确做法**：手动转为 `Box<dyn std::error::Error>` 再构造 `SetupError`：

```rust
.with_shortcut(shortcut_str)
    .map_err(|e| tauri::Error::Setup((Box::new(e) as Box<dyn std::error::Error>).into()))?
```

**错误链路**：`plugin::Error` → `Box<dyn Error>` → `SetupError::from()` → `tauri::Error::Setup`

---

## 2. `app.manage()` 需要 `use tauri::Manager`

`manage()` 是 `Manager` trait 的方法，而非 `App` 的固有方法。如果未导入 `use tauri::Manager`，编译器只会提示"no method named `manage` found"，不会自动提示缺少 trait 导入。

**正确做法**：在 `lib.rs` 中添加：

```rust
use tauri::Manager;
```

---

## 3. `register`/`unregister` 直接接受字符串

`tauri-plugin-global-shortcut` 的 `GlobalShortcut::register()` 和 `unregister()` 的参数类型为 `impl TryInto<ShortcutWrapper>`，而 `ShortcutWrapper` 实现了 `TryFrom<&str>`（内部调用 `global-hotkey` 的 `HotKey::from_str`）。

**无需手动解析**：直接传入 `"Alt+F1"`、`"Ctrl+Shift+A"` 等字符串即可，插件内部会自动解析。不要自己写 `parse_shortcut()` 之类的解析函数。

---

## 4. `with_handler` 是全局处理器

`Builder::with_handler()` 注册的是所有快捷键的统一处理器，而非单个快捷键的处理器。在只注册一个快捷键时，处理器内无需比对 `shortcut` 参数；但若注册多个快捷键，需要根据 `shortcut` 参数区分处理逻辑。
