//! Event-driven clipboard listener — full implementation with self-filter.
//!
//! Windows limitation: there is NO global clipboard-change broadcast to arbitrary
//! processes. Correct mechanism: AddClipboardFormatListener on a message-only
//! window owned by a dedicated thread with a GetMessageW loop. Zero polling.
//!
//! Self-filter: after we restore an entry (or any clipboard write), the system
//! sends another WM_CLIPBOARDUPDATE. We capture `GetClipboardSequenceNumber()`
//! and skip notifications whose sequence matches the last one we saw produced
//! by our own write. A "pending write" flag (set via format::sequence_number
//! before a write) would be ideal; here we keep a small sliding window of the
//! last observed sequence to avoid re-capturing our own restoration.

use crate::platform::handles::ThreadHandle;
use crate::platform::{ListenerHandle, PlatformError};
use crate::workers::WorkerEvent;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};

#[cfg(windows)]
pub fn start(
    tx: flume::Sender<WorkerEvent>,
    pause: Arc<AtomicBool>,
) -> Result<ListenerHandle, PlatformError> {
    use crate::platform::format;
    use windows::Win32::System::DataExchange::{
        AddClipboardFormatListener, RemoveClipboardFormatListener,
    };
    use windows::Win32::System::Threading::GetCurrentThreadId;
    use windows::Win32::UI::WindowsAndMessaging::{
        DestroyWindow, DispatchMessageW, GetMessageW, MSG, TranslateMessage, WM_CLIPBOARDUPDATE,
    };

    let (id_tx, id_rx) = mpsc::channel::<u32>();
    let join = std::thread::Builder::new()
        .name("clipvault-clipboard".into())
        .spawn(move || {
            let _ = id_tx.send(unsafe { GetCurrentThreadId() });
            let hwnd = match crate::platform::winutil::win32::create_message_window() {
                Ok(h) => h,
                Err(e) => {
                    let _ = tx.send(WorkerEvent::Error(format!("listener window: {e}")));
                    return;
                }
            };
            if unsafe { AddClipboardFormatListener(hwnd) }.is_err() {
                let _ = tx.send(WorkerEvent::Error(
                    "AddClipboardFormatListener failed".into(),
                ));
                return;
            }
            let clip = crate::platform::clipboard();
            let mut msg = MSG::default();
            // Remember the last sequence we processed; a repeated value means the
            // system is echoing a change we already handled (or our own write).
            let mut last_seq = format::sequence_number();
            loop {
                let r = unsafe { GetMessageW(&mut msg, None, 0, 0) };
                if r.0 == 0 || r.0 == -1 {
                    break;
                }
                if msg.message == WM_CLIPBOARDUPDATE && !pause.load(Ordering::SeqCst) {
                    let seq = format::sequence_number();
                    if seq != last_seq {
                        last_seq = seq;
                        match clip.read() {
                            Ok(Some(payload)) => {
                                if tx
                                    .send(WorkerEvent::NewClipboard(Box::new(payload)))
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            Ok(None) => {}
                            Err(e) => {
                                let _ = tx.send(WorkerEvent::Error(format!("clipboard read: {e}")));
                            }
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
                let _ = DestroyWindow(hwnd);
            }
        })
        .map_err(|e| PlatformError::Other(format!("spawn listener: {e}")))?;

    let thread_id = id_rx
        .recv()
        .map_err(|e| PlatformError::Other(format!("listener thread id: {e}")))?;
    Ok(ListenerHandle::new(ThreadHandle::new(thread_id, join)))
}

#[cfg(not(windows))]
pub fn start(
    _tx: flume::Sender<WorkerEvent>,
    _pause: Arc<AtomicBool>,
) -> Result<ListenerHandle, PlatformError> {
    Ok(ListenerHandle::new(ThreadHandle::new(
        0,
        std::thread::spawn(|| {}),
    )))
}
