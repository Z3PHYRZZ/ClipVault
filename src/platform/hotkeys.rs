//! Global hotkeys via RegisterHotKey on a dedicated message-loop thread.
//!
//! Limitation: only ONE process can own a given global combo. Registration can fail
//! with ERROR_HOTKEY_ALREADY_REGISTERED. We surface that as a clear error event
//! (Settings offers a rebind) rather than crashing. In windows-rs 0.62,
//! RegisterHotKey lives in Win32::UI::Input::KeyboardAndMouse, takes
//! (Option<HWND>, i32, HOT_KEY_MODIFIERS, u32), and GetMessageW returns BOOL.

use crate::domain::ShortcutAction;
use crate::platform::{HotkeyHandle, HotkeySpec, PlatformError};
use crate::workers::WorkerEvent;
use std::thread;

#[cfg(windows)]
pub fn start(
    tx: flume::Sender<WorkerEvent>,
    spec: HotkeySpec,
) -> Result<HotkeyHandle, PlatformError> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        DestroyWindow, DispatchMessageW, GetMessageW, TranslateMessage, MSG, WM_HOTKEY,
    };
    let _ = thread::Builder::new()
        .name("clipvault-hotkeys".into())
        .spawn(move || {
            let hwnd = match crate::platform::winutil::win32::create_message_window() {
                Ok(h) => h,
                Err(e) => {
                    let _ = tx.send(WorkerEvent::Error(format!("hotkey window: {e}")));
                    return;
                }
            };
            let id = crate::contracts::HOTKEY_ID_SHOW;
            let mods = HOT_KEY_MODIFIERS(spec.modifiers);
            if unsafe { RegisterHotKey(Some(hwnd), id, mods, spec.vk) }.is_err() {
                let _ = tx.send(WorkerEvent::Error(format!(
                    "Global hotkey {} is already registered by another app - rebind in Settings",
                    spec.display()
                )));
                unsafe { let _ = DestroyWindow(hwnd); }
                return;
            }
            let mut msg = MSG::default();
            loop {
                let r = unsafe { GetMessageW(&mut msg, None, 0, 0) };
                if r.0 == 0 || r.0 == -1 { break; }
                if msg.message == WM_HOTKEY {
                    let _ = tx.send(WorkerEvent::Hotkey(ShortcutAction::ShowHide));
                }
                unsafe { let _ = TranslateMessage(&msg); DispatchMessageW(&msg); }
            }
            unsafe {
                let _ = UnregisterHotKey(Some(hwnd), id);
                let _ = DestroyWindow(hwnd);
            }
        })
        .map_err(|e| PlatformError::Other(format!("spawn hotkeys: {e}")))?;
    Ok(HotkeyHandle { _inner: () })
}

#[cfg(not(windows))]
pub fn start(
    _tx: flume::Sender<WorkerEvent>,
    _spec: HotkeySpec,
) -> Result<HotkeyHandle, PlatformError> {
    Ok(HotkeyHandle { _inner: () })
}
