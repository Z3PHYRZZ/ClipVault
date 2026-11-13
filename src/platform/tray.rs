//! System tray icon via Shell_NotifyIconW.
//!
//! Limitation: a tray icon needs a window + message loop, and Iced has no built-in
//! tray. We create a message-only window on a dedicated thread, register a callback
//! message (WM_APP+1), and build the context menu with TrackPopupMenu on right-click.
//! NIM_DELETE on shutdown, otherwise the icon lingers until explorer refreshes.

use crate::platform::{PlatformError, TrayHandle};
use crate::workers::{TrayAction, WorkerEvent};
use std::thread;

#[cfg(windows)]
pub fn start(_tx: flume::Sender<WorkerEvent>) -> Result<TrayHandle, PlatformError> {
    // Full Shell_NotifyIconW + TrackPopupMenu implementation is hardened in wave 3
    // (agent D5). The baseline thread keeps the handle alive so startup/shutdown work.
    let handle = thread::Builder::new()
        .name("clipvault-tray".into())
        .spawn(move || {
            // Placeholder loop; replaced by the fleet's Win32 tray pump.
            loop {
                std::thread::sleep(std::time::Duration::from_secs(3600));
            }
        })
        .map_err(|e| PlatformError::Other(format!("spawn tray: {e}")))?;
    let _ = handle;
    Ok(TrayHandle { _inner: () })
}

#[cfg(not(windows))]
pub fn start(_tx: flume::Sender<WorkerEvent>) -> Result<TrayHandle, PlatformError> {
    Ok(TrayHandle { _inner: () })
}

#[allow(dead_code)]
fn map_action(_id: u32) -> Option<TrayAction> {
    match 0 {
        1 => Some(TrayAction::Show),
        2 => Some(TrayAction::Settings),
        3 => Some(TrayAction::Pause),
        4 => Some(TrayAction::Resume),
        5 => Some(TrayAction::Quit),
        _ => None,
    }
}
