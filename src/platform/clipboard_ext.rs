//! Full Windows clipboard backend: text, images, and file drop lists.
//!
//! Constraints handled (windows-rs 0.62, integrator compile-proven):
//! - Clipboard is thread-affine and contended. `ClipboardGuard::open(5, 15)`
//!   retries OpenClipboard with backoff and closes it on Drop — never spins.
//! - `GetClipboardData(fmt) -> Result<HANDLE>`; the returned memory is owned by
//!   the SYSTEM. We only GlobalLock/GlobalSize/copy/GlobalUnlock — never free it.
//! - Writes allocate HGLOBAL(GMEM_MOVEABLE), copy bytes, then
//!   `SetClipboardData(fmt, Some(HANDLE))` transfers ownership to the system.
//!   On success we must NOT GlobalFree; on failure we do.
//! - Text is CF_UNICODETEXT (UTF-16LE). Images use CF_DIB (device-independent,
//!   not the device-dependent CF_BITMAP handle). Files use CF_HDROP (DROPFILES).
//! - We hash the ORIGINAL DIB bytes (not the normalized PNG) for stable dedup.

use crate::platform::{ClipboardAccess, PlatformError};
use crate::workers::{ClipCapture, ClipPayload};

#[cfg(windows)]
mod imp {
    use super::*;
    use crate::domain::hash_bytes;
    use crate::platform::format::{self, CF_DIB, CF_HDROP, CF_UNICODETEXT, read_priority};
    use crate::platform::hdrop;
    use crate::platform::winutil::win32::ClipboardGuard;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Foundation::HGLOBAL;
    use windows::Win32::System::DataExchange::{
        EmptyClipboard, GetClipboardData, SetClipboardData,
    };
    use windows::Win32::System::Memory::{
        GLOBAL_ALLOC_FLAGS, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock,
    };

    const GMEM_MOVEABLE: u32 = 0x0002;

    /// Convert a Rust String to NUL-terminated UTF-16LE bytes.
    fn string_to_utf16(s: &str) -> Vec<u8> {
        let mut out = Vec::with_capacity(s.len() * 2 + 2);
        for u in s.encode_utf16() {
            out.extend_from_slice(&u.to_le_bytes());
        }
        out.extend_from_slice(&0u16.to_le_bytes());
        out
    }

    /// Decode UTF-16LE bytes (with trailing NULs) to a Rust String.
    fn utf16_to_string(bytes: &[u8]) -> String {
        let mut units = Vec::with_capacity(bytes.len() / 2);
        for chunk in bytes.chunks_exact(2) {
            units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
        }
        while units.last() == Some(&0) {
            units.pop();
        }
        String::from_utf16_lossy(&units)
    }

    /// Copy a clipboard format's global-memory bytes into an owned Vec.
    /// Returns Ok(None) if the format is absent or the block is empty.
    fn get_bytes(fmt: u32, _guard: &ClipboardGuard) -> Result<Option<Vec<u8>>, PlatformError> {
        unsafe {
            let handle: HANDLE = match GetClipboardData(fmt) {
                Ok(h) => h,
                Err(_) => return Ok(None),
            };
            if handle.0.is_null() {
                return Ok(None);
            }
            let hglobal = HGLOBAL(handle.0);
            let ptr = GlobalLock(hglobal) as *const u8;
            if ptr.is_null() {
                return Ok(None);
            }
            let size = GlobalSize(hglobal);
            let bytes = if size == 0 {
                Vec::new()
            } else {
                std::slice::from_raw_parts(ptr, size).to_vec()
            };
            let _ = GlobalUnlock(hglobal);
            if bytes.is_empty() {
                Ok(None)
            } else {
                Ok(Some(bytes))
            }
        }
    }

    /// Publish bytes under `fmt`, transferring HGLOBAL ownership to the system.
    fn set_bytes(fmt: u32, data: &[u8], _guard: &ClipboardGuard) -> Result<(), PlatformError> {
        if data.is_empty() {
            return Ok(());
        }
        unsafe {
            let hglobal: HGLOBAL = GlobalAlloc(GLOBAL_ALLOC_FLAGS(GMEM_MOVEABLE), data.len())
                .map_err(|e| PlatformError::Windows(format!("GlobalAlloc: {e}")))?;
            let ptr = GlobalLock(hglobal) as *mut u8;
            if ptr.is_null() {
                let _ = windows::Win32::Foundation::GlobalFree(Some(hglobal));
                return Err(PlatformError::Other("GlobalLock failed".into()));
            }
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
            let _ = GlobalUnlock(hglobal);
            EmptyClipboard().map_err(|e| PlatformError::Windows(format!("EmptyClipboard: {e}")))?;
            match SetClipboardData(fmt, Some(HANDLE(hglobal.0))) {
                Ok(_) => Ok(()), // ownership transferred — do not GlobalFree
                Err(e) => {
                    let _ = windows::Win32::Foundation::GlobalFree(Some(hglobal));
                    Err(PlatformError::Windows(format!("SetClipboardData: {e}")))
                }
            }
        }
    }

    pub struct FullClipboard;

    impl ClipboardAccess for FullClipboard {
        fn read(&self) -> Result<Option<ClipCapture>, PlatformError> {
            let guard = ClipboardGuard::open(5, 15)?;
            // Provenance must be captured while the clipboard is still open:
            // GetClipboardData(CF_HTML) and GetClipboardOwner need the open lock.
            let (source_app, html_url) = crate::platform::source::capture();
            let mut source_url = html_url;
            for &fmt in read_priority() {
                let bytes = match get_bytes(fmt, &guard)? {
                    Some(b) => b,
                    None => continue,
                };
                match fmt {
                    CF_UNICODETEXT => {
                        let text = utf16_to_string(&bytes);
                        if !text.is_empty() {
                            if source_url.is_none() {
                                source_url = crate::platform::source::url_from_text(&text);
                            }
                            return Ok(Some(
                                ClipCapture::new(ClipPayload::Text(text))
                                    .with_source(source_app, source_url),
                            ));
                        }
                    }
                    CF_HDROP => {
                        let paths = hdrop::parse_dropfiles(&bytes);
                        if !paths.is_empty() {
                            let metas = hdrop::query_metadata(&paths);
                            return Ok(Some(
                                ClipCapture::new(ClipPayload::Files(metas))
                                    .with_source(source_app, source_url),
                            ));
                        }
                    }
                    f if f == format::CF_DIBV5 || f == format::CF_DIB => {
                        if let Ok((png, w, h)) = crate::platform::dib::dib_to_png(&bytes) {
                            let hash = hash_bytes(&bytes);
                            return Ok(Some(
                                ClipCapture::new(ClipPayload::Image { png, w, h, hash })
                                    .with_source(source_app, source_url),
                            ));
                        }
                    }
                    // CF_BITMAP is a device-dependent HBITMAP handle, not a memory
                    // block; converting it requires a device context (GetDIBits).
                    // DIB/DIBV5 are tried first, so this is an intentional skip.
                    format::CF_BITMAP => {}
                    _ => {}
                }
            }
            Ok(None)
        }

        fn write_text(&self, s: &str) -> Result<(), PlatformError> {
            let guard = ClipboardGuard::open(5, 15)?;
            set_bytes(CF_UNICODETEXT, &string_to_utf16(s), &guard)
        }

        fn write_image(&self, png: &[u8]) -> Result<(), PlatformError> {
            let dib = crate::platform::dib::png_to_dib(png)?;
            let guard = ClipboardGuard::open(5, 15)?;
            set_bytes(CF_DIB, &dib, &guard)
        }

        fn write_files(&self, paths: &[String]) -> Result<(), PlatformError> {
            if paths.is_empty() {
                return Ok(());
            }
            let guard = ClipboardGuard::open(5, 15)?;
            set_bytes(CF_HDROP, &hdrop::build_dropfiles_wide(paths), &guard)
        }
    }

    pub fn create() -> Box<dyn ClipboardAccess> {
        Box::new(FullClipboard)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn utf16_roundtrip() {
            for s in ["hello", "héllo 世界", "", "a\0b"] {
                let back = utf16_to_string(&string_to_utf16(s));
                assert_eq!(back, s.trim_end_matches('\0'));
            }
        }
        #[test]
        fn hash_is_stable() {
            assert_eq!(
                crate::domain::hash_text("abc"),
                crate::domain::hash_text("abc")
            );
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use super::*;
    pub struct FullClipboard;
    impl ClipboardAccess for FullClipboard {
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
        Box::new(FullClipboard)
    }
}

pub use imp::{FullClipboard, create};
