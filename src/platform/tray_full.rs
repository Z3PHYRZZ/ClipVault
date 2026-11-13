//! System tray icon via Shell_NotifyIconW — full implementation.
//!
//! Windows limitation: a tray icon needs a window + message loop, and Iced has
//! no built-in tray. We create a message-only window on a dedicated thread,
//! register a callback message (WM_APP+1 = 0x8001), and build the context menu
//! with CreatePopupMenu + TrackPopupMenu on right-click. NIM_DELETE runs on
//! shutdown, otherwise the icon lingers until explorer refreshes.
//!
//! The thread reports its OS thread id over a channel; `TrayHandle` owns a
//! `ThreadHandle` that posts WM_QUIT and joins on shutdown.

use crate::platform::PlatformError;
use crate::platform::TrayHandle;
use crate::workers::{TrayAction, WorkerEvent};

#[cfg(windows)]
pub fn start(tx: flume::Sender<WorkerEvent>) -> Result<TrayHandle, PlatformError> {
    use crate::platform::handles::ThreadHandle;
    use crate::platform::icon;
    use std::sync::mpsc;
    use windows::Win32::System::Threading::GetCurrentThreadId;
    use windows::Win32::UI::Shell::{
        NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        DestroyWindow, DispatchMessageW, GetMessageW, HICON, MSG, TranslateMessage, WM_APP,
        WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_RBUTTONUP,
    };

    const WM_TRAY: u32 = WM_APP + 1;
    const TRAY_UID: u32 = 1;

    let (id_tx, id_rx) = mpsc::channel::<u32>();
    let join = std::thread::Builder::new()
        .name("clipvault-tray".into())
        .spawn(move || {
            // Report our OS thread id so the handle can post WM_QUIT to us.
            let _ = id_tx.send(unsafe { GetCurrentThreadId() });

            let hwnd = match crate::platform::winutil::win32::create_message_window() {
                Ok(h) => h,
                Err(e) => {
                    let _ = tx.send(WorkerEvent::Error(format!("tray window: {e}")));
                    return;
                }
            };

            let hicon = match icon::make_hicon() {
                Ok(h) => HICON(h as *mut core::ffi::c_void),
                Err(_) => HICON(std::ptr::null_mut()),
            };

            let mut tip = [0u16; 128];
            for (i, u) in "ClipVault".encode_utf16().enumerate().take(127) {
                tip[i] = u;
            }
            let data = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: TRAY_UID,
                uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
                uCallbackMessage: WM_TRAY,
                hIcon: hicon,
                szTip: tip,
                ..Default::default()
            };
            let _ = unsafe { Shell_NotifyIconW(NIM_ADD, &data) };

            let mut msg = MSG::default();
            let mut paused = false;
            loop {
                let r = unsafe { GetMessageW(&mut msg, None, 0, 0) };
                if r.0 == 0 || r.0 == -1 {
                    break;
                }
                if msg.message == WM_TRAY {
                    // LOWORD(lParam) carries the mouse/event id.
                    let event = (msg.lParam.0 as u32) & 0xFFFF;
                    if event == WM_RBUTTONUP || event == WM_LBUTTONUP {
                        let action = show_menu(hwnd, &mut paused);
                        if let Some(a) = action {
                            let _ = tx.send(WorkerEvent::Tray(a));
                        }
                    } else if event == WM_LBUTTONDBLCLK {
                        let _ = tx.send(WorkerEvent::Tray(TrayAction::Show));
                    }
                }
                unsafe {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }

            let _ = unsafe { Shell_NotifyIconW(NIM_DELETE, &data) };
            let _ = unsafe { DestroyWindow(hwnd) };
        })
        .map_err(|e| PlatformError::Other(format!("spawn tray: {e}")))?;

    let thread_id = id_rx
        .recv()
        .map_err(|e| PlatformError::Other(format!("tray thread id: {e}")))?;
    Ok(TrayHandle::new(ThreadHandle::new(thread_id, join)))
}

/// Build + track the context menu; returns the chosen action (mutates paused state).
#[cfg(windows)]
fn show_menu(hwnd: windows::Win32::Foundation::HWND, paused: &mut bool) -> Option<TrayAction> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, MF_SEPARATOR, MF_STRING,
        SetForegroundWindow, TPM_LEFTALIGN, TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu,
    };
    use windows::core::w;
    unsafe {
        let menu = match CreatePopupMenu() {
            Ok(m) => m,
            Err(_) => return None,
        };
        let _ = AppendMenuW(menu, MF_STRING, 1, w!("Show ClipVault"));
        let _ = AppendMenuW(menu, MF_STRING, 2, w!("Settings"));
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, w!(""));
        let pause_label = if *paused {
            w!("Resume monitoring")
        } else {
            w!("Pause monitoring")
        };
        let _ = AppendMenuW(menu, MF_STRING, 3, pause_label);
        let _ = AppendMenuW(menu, MF_STRING, 4, w!("Quit"));
        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        let _ = SetForegroundWindow(hwnd);
        let cmd = TrackPopupMenu(
            menu,
            TPM_LEFTALIGN | TPM_RIGHTBUTTON | TPM_RETURNCMD,
            pt.x,
            pt.y,
            None,
            hwnd,
            None,
        );
        let _ = DestroyMenu(menu);
        match cmd.0 as u32 {
            1 => Some(TrayAction::Show),
            2 => Some(TrayAction::Settings),
            3 => {
                *paused = !*paused;
                Some(if *paused {
                    TrayAction::Pause
                } else {
                    TrayAction::Resume
                })
            }
            4 => Some(TrayAction::Quit),
            _ => None,
        }
    }
}

#[cfg(not(windows))]
pub fn start(_tx: flume::Sender<WorkerEvent>) -> Result<TrayHandle, PlatformError> {
    Ok(TrayHandle::new(
        crate::platform::handles::ThreadHandle::new(0, std::thread::spawn(|| {})),
    ))
}
