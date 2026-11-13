//! Shared Win32 helpers: message-only windows and RAII clipboard guards.
//!
//! Windows limitation: there is NO global clipboard-change broadcast to arbitrary
//! processes. The correct mechanism is a message-only window (HWND_MESSAGE parent)
//! registered with AddClipboardFormatListener, pumped by a dedicated thread's
//! GetMessageW loop. That loop is event-driven (zero CPU idle) — no polling.
//!
//! Clipboard access is thread-affine and contended: OpenClipboard can fail transiently
//! with ERROR_ACCESS_DENIED while another process holds the lock. All reads use
//! bounded retries (5 attempts, 15 ms backoff), never a spin loop.

#[cfg(windows)]
pub mod win32 {
    use crate::platform::PlatformError;
    use windows::Win32::Foundation::*;
    use windows::Win32::System::DataExchange::*;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::core::PCWSTR;

    /// RAII guard: opens the clipboard with bounded retries, closes on Drop.
    pub struct ClipboardGuard;

    impl ClipboardGuard {
        pub fn open(retries: u32, backoff_ms: u64) -> Result<Self, PlatformError> {
            let mut last = String::new();
            for attempt in 0..retries {
                // OpenClipboard(Option<HWND>) -> Result<()> in windows-rs 0.62.
                match unsafe { OpenClipboard(None) } {
                    Ok(()) => return Ok(Self),
                    Err(e) => {
                        last = format!("{e}");
                        if attempt + 1 < retries {
                            std::thread::sleep(std::time::Duration::from_millis(backoff_ms));
                        }
                    }
                }
            }
            Err(PlatformError::Other(format!(
                "OpenClipboard failed after {retries} attempts: {last}"
            )))
        }
    }

    impl Drop for ClipboardGuard {
        fn drop(&mut self) {
            let _ = unsafe { CloseClipboard() };
        }
    }

    /// Forward all messages to the default window procedure.
    /// WNDPROC requires the `system` ABI; DefWindowProcW is a Rust-ABI fn in
    /// windows-rs 0.62, so we wrap it explicitly instead of casting.
    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }

    /// Create a message-only window on the calling thread.
    /// Returns the HWND; the thread must run a GetMessageW loop and DestroyWindow it.
    pub fn create_message_window() -> Result<HWND, PlatformError> {
        unsafe {
            let module: HMODULE = GetModuleHandleW(PCWSTR::null())
                .map_err(|e| PlatformError::Windows(format!("GetModuleHandleW: {e}")))?;
            // HMODULE and HINSTANCE are distinct newtypes over the same pointer.
            let instance = HINSTANCE(module.0);
            let class_name = windows::core::w!("ClipVaultMsgWindow");
            let wc = WNDCLASSW {
                lpfnWndProc: Some(wnd_proc),
                hInstance: instance,
                lpszClassName: class_name,
                ..Default::default()
            };
            // RegisterClassW takes *const WNDCLASSW and returns u16 atom (0 = fail).
            let atom = RegisterClassW(&wc);
            if atom == 0 {
                let err = GetLastError();
                if err != ERROR_CLASS_ALREADY_EXISTS {
                    return Err(PlatformError::Windows(format!("RegisterClassW: {err:?}")));
                }
            }
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class_name,
                windows::core::w!("ClipVault"),
                WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                Some(instance),
                None,
            )
            .map_err(|e| PlatformError::Windows(format!("CreateWindowExW: {e}")))?;
            Ok(hwnd)
        }
    }
}

#[cfg(not(windows))]
pub mod win32 {
    use crate::platform::PlatformError;
    /// Opaque stand-in for a native window handle on non-Windows targets.
    #[derive(Clone, Copy)]
    pub struct HWND(pub isize);
    pub struct ClipboardGuard;
    impl ClipboardGuard {
        pub fn open(_r: u32, _b: u64) -> Result<Self, PlatformError> {
            Ok(Self)
        }
    }
    pub fn create_message_window() -> Result<HWND, PlatformError> {
        Ok(HWND(0))
    }
}
