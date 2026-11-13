//! Global hotkeys — full implementation with quick-paste (Ctrl+1..9).
//!
//! Windows limitation: only ONE process can own a given global combo.
//! Registration can fail with ERROR_HOTKEY_ALREADY_REGISTERED. We surface a
//! clear WorkerEvent::Error with a rebind hint rather than crashing.
//! RegisterHotKey lives in Win32::UI::Input::KeyboardAndMouse and takes
//! (Option<HWND>, i32, HOT_KEY_MODIFIERS, u32); WM_HOTKEY's wParam is the id.

use crate::domain::ShortcutAction;
use crate::platform::handles::ThreadHandle;
use crate::platform::{HotkeyHandle, HotkeySpec, PlatformError};
use crate::workers::WorkerEvent;
use std::sync::mpsc;

#[cfg(windows)]
pub fn start(
    tx: flume::Sender<WorkerEvent>,
    spec: HotkeySpec,
) -> Result<HotkeyHandle, PlatformError> {
    use windows::Win32::System::Threading::GetCurrentThreadId;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        HOT_KEY_MODIFIERS, RegisterHotKey, UnregisterHotKey,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        DestroyWindow, DispatchMessageW, GetMessageW, MSG, TranslateMessage, WM_HOTKEY,
    };

    // Id map: 0xC100 = show/hide, 0xC101..0xC109 = QuickPaste(1..9).
    const BASE_SHOW: i32 = 0xC100;
    const BASE_QUICK: i32 = 0xC101;
    let ctrl_alt = HOT_KEY_MODIFIERS(0x0002 | 0x0004); // MOD_CONTROL|MOD_SHIFT

    let (id_tx, id_rx) = mpsc::channel::<u32>();
    let join = std::thread::Builder::new()
        .name("clipvault-hotkeys".into())
        .spawn(move || {
            let _ = id_tx.send(unsafe { GetCurrentThreadId() });
            let hwnd = match crate::platform::winutil::win32::create_message_window() {
                Ok(h) => h,
                Err(e) => {
                    let _ = tx.send(WorkerEvent::Error(format!("hotkey window: {e}")));
                    return;
                }
            };
            let mut registered: Vec<i32> = Vec::new();
            // Primary show/hide combo from settings.
            if unsafe {
                RegisterHotKey(
                    Some(hwnd),
                    BASE_SHOW,
                    HOT_KEY_MODIFIERS(spec.modifiers),
                    spec.vk,
                )
            }
            .is_ok()
            {
                registered.push(BASE_SHOW);
            } else {
                let _ = tx.send(WorkerEvent::Error(format!(
                    "Global hotkey {} is already registered by another app - rebind in Settings",
                    spec.display()
                )));
            }
            // Quick paste: Ctrl+Shift+1..9 (0x31..0x39).
            for n in 1..=9u32 {
                let id = BASE_QUICK + (n as i32) - 1;
                let vk = 0x30 + n;
                if unsafe { RegisterHotKey(Some(hwnd), id, ctrl_alt, vk) }.is_ok() {
                    registered.push(id);
                }
            }

            let mut msg = MSG::default();
            loop {
                let r = unsafe { GetMessageW(&mut msg, None, 0, 0) };
                if r.0 == 0 || r.0 == -1 {
                    break;
                }
                if msg.message == WM_HOTKEY {
                    let id = msg.wParam.0 as i32;
                    let action = if id == BASE_SHOW {
                        Some(ShortcutAction::ShowHide)
                    } else if (BASE_QUICK..BASE_QUICK + 9).contains(&id) {
                        Some(ShortcutAction::QuickPaste((id - BASE_QUICK + 1) as u8))
                    } else {
                        None
                    };
                    if let Some(a) = action {
                        let _ = tx.send(WorkerEvent::Hotkey(a));
                    }
                }
                unsafe {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
            for id in registered {
                unsafe {
                    let _ = UnregisterHotKey(Some(hwnd), id);
                }
            }
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
        })
        .map_err(|e| PlatformError::Other(format!("spawn hotkeys: {e}")))?;

    let thread_id = id_rx
        .recv()
        .map_err(|e| PlatformError::Other(format!("hotkey thread id: {e}")))?;
    Ok(HotkeyHandle::new(ThreadHandle::new(thread_id, join)))
}

#[cfg(not(windows))]
pub fn start(
    _tx: flume::Sender<WorkerEvent>,
    _spec: HotkeySpec,
) -> Result<HotkeyHandle, PlatformError> {
    Ok(HotkeyHandle::new(ThreadHandle::new(
        0,
        std::thread::spawn(|| {}),
    )))
}
