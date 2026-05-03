use rdev::{grab, Button, Event, EventType};
use selection::get_text;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::{Emitter, Manager};

pub fn is_mouse_shortcut(s: &str) -> bool {
    matches!(s, "Mouse4" | "Mouse5")
}

fn to_button_id(s: &str) -> u8 {
    match s {
        "Mouse4" => 1,
        "Mouse5" => 2,
        _ => panic!("not a mouse shortcut"),
    }
}

pub fn start(app: tauri::AppHandle, shortcut: &str) -> Arc<AtomicBool> {
    let target_id = to_button_id(shortcut);
    let stop = Arc::new(AtomicBool::new(false));
    let stop2 = stop.clone();

    std::thread::spawn(move || {
        let _ = grab(move |event: Event| {
            if stop2.load(Ordering::Relaxed) {
                return Some(event);
            }
            match event.event_type {
                EventType::ButtonPress(Button::Unknown(id)) if id == target_id => None,
                EventType::ButtonRelease(Button::Unknown(id)) if id == target_id => {
                    let text = get_text();
                    if !text.is_empty() {
                        if let Some(panel) = app.get_webview_window("panel") {
                            let _ = panel.emit("get_text", &text);
                            let _ = panel.show();
                            let _ = panel.set_focus();
                        }
                    }
                    None
                }
                _ => Some(event),
            }
        });
    });

    stop
}
