use tauri::AppHandle;

pub fn panel(app: &AppHandle) {
    tauri::WebviewWindowBuilder::new(app, "panel", tauri::WebviewUrl::App("/".into()))
        .title("Tran")
        .inner_size(256.0, 200.0)
        .fullscreen(false)
        .resizable(true)
        .minimizable(false)
        .maximizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .shadow(true)
        .center()
        .build()
        .expect("Failed to create panel window");
}
