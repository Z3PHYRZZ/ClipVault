//! Event-driven clipboard change listener (no polling).
//!
//! Windows limitation: there is NO global clipboard-change broadcast to arbitrary
//! processes. The correct mechanism is AddClipboardFormatListener on a message-only
//! window owned by a dedicated thread with a GetMessageW loop. On WM_CLIPBOARDUPDATE
//! the thread reads the clipboard with bounded retries and emits the payload.
//! In windows-rs 0.62, AddClipboardFormatListener lives in
//! Win32::System::DataExchange and GetMessageW returns a BOOL (compare `r.0`).

use crate::platform::{winutil, ListenerHandle, PlatformError};
use crate::workers::WorkerEvent;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

#[cfg(windows)]
pub fn start(
    tx: flume::Sender<WorkerEvent>,
    pause: Arc<AtomicBool>,
) -> Result<ListenerHandle, PlatformError> {
    use windows::Win32::System::DataExchange::{
        AddClipboardFormatListener, RemoveClipboardFormatListener,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, TranslateMessage, MSG, WM_CLIPBOARDUPDATE,
    };
    let _ = thread::Builder::new()
        .name("clipvault-clipboard".into())
        .spawn(move || {
            let hwnd = match winutil::win32::create_message_window() {
                Ok(h) => h,
                Err(e) => {
                    let _ = tx.send(WorkerEvent::Error(format!("listener window: {e}")));
                    return;
                }
            };
            // AddClipboardFormatListener opts this window into WM_CLIPBOARDUPDATE.
            if unsafe { AddClipboardFormatListener(hwnd) }.is_err() {
                let _ = tx.send(WorkerEvent::Error("AddClipboardFormatListener failed".into()));
                return;
            }
            let clip = crate::platform::clipboard();
            let mut msg = MSG::default();
            loop {
                // GetMessageW returns BOOL (a struct wrapping i32): 0 = WM_QUIT, -1 = error.
                let r = unsafe { GetMessageW(&mut msg, None, 0, 0) };
                if r.0 == 0 || r.0 == -1 {
                    break;
                }
                if msg.message == WM_CLIPBOARDUPDATE && !pause.load(Ordering::SeqCst) {
                    match clip.read() {
                        Ok(Some(payload)) => {
                            if tx.send(WorkerEvent::NewClipboard(Box::new(payload))).is_err() {
                                break;
                            }
                        }
                        Ok(None) => {}
                        Err(e) => {
                            let _ = tx.send(WorkerEvent::Error(format!("clipboard read: {e}")));
                        }
                    }
                }
                unsafe {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
            unsafe {
                let _ = RemoveClipboardFormatListener(hwnd);
                let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd);
            }
        })
        .map_err(|e| PlatformError::Other(format!("spawn listener: {e}")))?;
    Ok(ListenerHandle { _inner: () })
}

#[cfg(not(windows))]
pub fn start(
    _tx: flume::Sender<WorkerEvent>,
    _pause: Arc<AtomicBool>,
) -> Result<ListenerHandle, PlatformError> {
    // Headless stub: no OS clipboard to observe. Tests inject payloads directly.
    Ok(ListenerHandle { _inner: () })
}
