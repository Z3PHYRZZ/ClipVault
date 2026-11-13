//! Windows clipboard read/write via `windows-rs` 0.62.
//!
//! Key Win32 constraints handled here:
//! - Clipboard is thread-affine and contended: OpenClipboard may fail transiently.
//!   `ClipboardGuard::open` retries 5x with 15ms backoff, never spins.
//! - Text uses CF_UNICODETEXT (UTF-16); we convert to/from Rust UTF-8.
//! - Restoration uses HGLOBAL-owned memory (GMEM_MOVEABLE) + SetClipboardData, which
//!   transfers ownership to the system — no delayed rendering, no owner HWND needed.
//!   After a successful SetClipboardData we must NOT GlobalFree the memory.
//! - windows-rs 0.62 types: GetClipboardData -> Result<HANDLE>,
//!   SetClipboardData(u32, Option<HANDLE>) -> Result<HANDLE>,
//!   GlobalAlloc -> Result<HGLOBAL> (HANDLE and HGLOBAL are distinct newtypes).

#[cfg(windows)]
mod imp {
    use crate::platform::{ClipboardAccess, PlatformError, winutil::win32::ClipboardGuard};
    use crate::workers::{ClipCapture, ClipPayload};
    use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
    use windows::Win32::System::DataExchange::{
        EmptyClipboard, GetClipboardData, SetClipboardData,
    };
    use windows::Win32::System::Memory::{
        GLOBAL_ALLOC_FLAGS, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock,
    };

    const CF_UNICODETEXT: u32 = 13;
    const GMEM_MOVEABLE: u32 = 0x0002;

    pub struct WinClipboard;

    impl ClipboardAccess for WinClipboard {
        fn read(&self) -> Result<Option<ClipCapture>, PlatformError> {
            let _guard = ClipboardGuard::open(5, 15)?;
            unsafe {
                let handle: HANDLE = GetClipboardData(CF_UNICODETEXT)
                    .map_err(|e| PlatformError::Windows(format!("GetClipboardData: {e}")))?;
                if handle.0.is_null() {
                    return Ok(None);
                }
                // GlobalLock accepts HGLOBAL; HANDLE and HGLOBAL share the pointer layout.
                let hglobal = HGLOBAL(handle.0);
                let ptr = GlobalLock(hglobal) as *const u16;
                if ptr.is_null() {
                    return Ok(None);
                }
                let bytes = GlobalSize(hglobal);
                let len = bytes / std::mem::size_of::<u16>();
                let slice = std::slice::from_raw_parts(ptr, len);
                let mut end = len;
                while end > 0 && slice[end - 1] == 0 {
                    end -= 1;
                }
                let text = String::from_utf16_lossy(&slice[..end]);
                let _ = GlobalUnlock(hglobal);
                if text.is_empty() {
                    return Ok(None);
                }
                Ok(Some(ClipCapture::new(ClipPayload::Text(text))))
            }
        }

        fn write_text(&self, s: &str) -> Result<(), PlatformError> {
            let _guard = ClipboardGuard::open(5, 15)?;
            let mut wide: Vec<u16> = s.encode_utf16().collect();
            wide.push(0);
            let byte_len = wide.len() * std::mem::size_of::<u16>();
            unsafe {
                let hglobal: HGLOBAL = GlobalAlloc(GLOBAL_ALLOC_FLAGS(GMEM_MOVEABLE), byte_len)
                    .map_err(|e| PlatformError::Windows(format!("GlobalAlloc: {e}")))?;
                let ptr = GlobalLock(hglobal) as *mut u16;
                if ptr.is_null() {
                    let _ = GlobalFree(Some(hglobal));
                    return Err(PlatformError::Other("GlobalLock failed".into()));
                }
                std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
                let _ = GlobalUnlock(hglobal);
                EmptyClipboard()
                    .map_err(|e| PlatformError::Windows(format!("EmptyClipboard: {e}")))?;
                // On success, ownership transfers to the system. Do not GlobalFree.
                let handle = HANDLE(hglobal.0);
                match SetClipboardData(CF_UNICODETEXT, Some(handle)) {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        let _ = GlobalFree(Some(hglobal));
                        Err(PlatformError::Windows(format!("SetClipboardData: {e}")))
                    }
                }
            }
        }

        fn write_image(&self, _png: &[u8]) -> Result<(), PlatformError> {
            Err(PlatformError::Other(
                "image write not available (phase 5)".into(),
            ))
        }

        fn write_files(&self, _paths: &[String]) -> Result<(), PlatformError> {
            Err(PlatformError::Other(
                "file write not available (phase 5)".into(),
            ))
        }
    }

    pub fn create() -> Box<dyn ClipboardAccess> {
        Box::new(WinClipboard)
    }
}

#[cfg(not(windows))]
mod imp {
    use crate::platform::{ClipboardAccess, PlatformError};
    use crate::workers::{ClipCapture, ClipPayload};
    pub struct StubClipboard;
    impl ClipboardAccess for StubClipboard {
        fn read(&self) -> Result<Option<ClipCapture>, PlatformError> {
            Ok(None)
        }
        fn write_text(&self, _s: &str) -> Result<(), PlatformError> {
            Ok(())
        }
        fn write_image(&self, _png: &[u8]) -> Result<(), PlatformError> {
            Ok(())
        }
        fn write_files(&self, _paths: &[String]) -> Result<(), PlatformError> {
            Ok(())
        }
    }
    pub fn create() -> Box<dyn ClipboardAccess> {
        Box::new(StubClipboard)
    }
}

pub fn platform_clipboard() -> Box<dyn crate::platform::ClipboardAccess> {
    imp::create()
}
