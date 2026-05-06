use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub fn is_mouse_shortcut(s: &str) -> bool {
    matches!(s, "Mouse4" | "Mouse5")
}

pub fn to_button_id(s: &str) -> u8 {
    match s {
        "Mouse4" => 4,
        "Mouse5" => 5,
        _ => panic!("not a mouse shortcut"),
    }
}

pub fn start(app: tauri::AppHandle, shortcut: &str) -> Arc<AtomicBool> {
    let target_id = to_button_id(shortcut);
    let stop = Arc::new(AtomicBool::new(false));
    platform::start_hook(app, target_id, stop.clone());
    stop
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use tauri::{Emitter, Manager};
    use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, PostThreadMessageW, SetWindowsHookExW,
        TranslateMessage, UnhookWindowsHookEx, MSLLHOOKSTRUCT, MSG, WH_MOUSE_LL,
        WM_QUIT, WM_XBUTTONDOWN, WM_XBUTTONUP,
    };

    use std::sync::mpsc::{self, SyncSender};

    static TARGET_XBUTTON: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
    static STOP_FLAG: OnceLock<Mutex<Option<Arc<AtomicBool>>>> = OnceLock::new();
    static TX: OnceLock<Mutex<Option<SyncSender<()>>>> = OnceLock::new();

    unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code >= 0 {
            let msg = wparam.0 as u32;
            if msg == WM_XBUTTONDOWN || msg == WM_XBUTTONUP {
                let data = &*(lparam.0 as *const MSLLHOOKSTRUCT);
                let xbutton = (data.mouseData >> 16) as u8;
                let target = TARGET_XBUTTON.load(Ordering::Relaxed);
                let stopped = STOP_FLAG.get()
                    .and_then(|m| m.lock().ok())
                    .and_then(|g| g.as_ref().map(|s| s.load(Ordering::Relaxed)))
                    .unwrap_or(true);

                if !stopped && xbutton == target {
                    if msg == WM_XBUTTONUP {
                        if let Some(tx) = TX.get().and_then(|m| m.lock().ok()).and_then(|g| g.clone()) {
                            let _ = tx.try_send(());
                        }
                    }
                    return LRESULT(1); // swallow
                }
            }
        }
        CallNextHookEx(None, code, wparam, lparam)
    }

    pub fn start_hook(app: tauri::AppHandle, target_id: u8, stop: Arc<AtomicBool>) {
        let target_xbutton = target_id - 3;

        TARGET_XBUTTON.store(target_xbutton, Ordering::Relaxed);
        *STOP_FLAG.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(stop.clone());

        let (tx, rx) = mpsc::sync_channel::<()>(4);
        *TX.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(tx);

        // worker thread: does the blocking get_text() work
        std::thread::spawn(move || {
            while rx.recv().is_ok() {
                let text = selection::get_text();
                if !text.is_empty() {
                    if let Some(panel) = app.get_webview_window("panel") {
                        let _ = panel.emit("get_text", &text);
                        let _ = panel.show();
                        let _ = panel.set_focus();
                    }
                }
            }
        });

        std::thread::spawn(move || unsafe {
            let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(hook_proc), None, 0)
                .expect("SetWindowsHookExW failed");

            let thread_id = windows::Win32::System::Threading::GetCurrentThreadId();
            std::thread::spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                let _ = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
            });

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let _ = UnhookWindowsHookEx(hook);
        });
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod platform {
    use super::*;
    use rdev::{grab, Button, Event, EventType};
    use tauri::{Emitter, Manager};

    pub fn start_hook(app: tauri::AppHandle, target_id: u8, stop: Arc<AtomicBool>) {
        let button = match target_id {
            4 => Button::Unknown(1),
            5 => Button::Unknown(2),
            _ => return,
        };

        std::thread::spawn(move || {
            let _ = grab(move |event: Event| {
                if stop.load(Ordering::Relaxed) {
                    return Some(event);
                }
                match event.event_type {
                    EventType::ButtonPress(b) if b == button => None,
                    EventType::ButtonRelease(b) if b == button => {
                        let text = selection::get_text();
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
    }
}
