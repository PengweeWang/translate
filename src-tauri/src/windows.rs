use tauri::AppHandle;

pub fn panel(app: &AppHandle) {
    tauri::WebviewWindowBuilder::new(app, "panel", tauri::WebviewUrl::App("/".into()))
        .title("Tran")
        .inner_size(400.0, 300.0)
        .fullscreen(false)
        .resizable(true)
        .minimizable(false)
        .maximizable(false)
        .decorations(false)
        .always_on_top(false)
        .skip_taskbar(true)
        .visible(false)
        .shadow(true)
        .center()
        .build()
        .expect("Failed to create panel window");
}
