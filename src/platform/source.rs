//! Clipboard provenance: where a copy came from.
//!
//! Windows has no "copy source" API, so we reconstruct it from two signals:
//!
//! - **source_url**: browsers publish a `CF_HTML` clipboard format whose header
//!   contains `SourceURL:<page url>`. We register the format by name and parse
//!   that header. Fallback: a bare `http(s)://` URL in copied text.
//! - **source_app**: `GetClipboardOwner()` -> `GetWindowThreadProcessId` ->
//!   `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` ->
//!   `QueryFullProcessImageNameW`, reduced to the executable basename.
//!
//! Both are best-effort: the owner window can vanish between the change
//! notification and the read, and not every app publishes CF_HTML. [cfg]-gated
//! with a headless stub so `cargo test` does not touch the real clipboard.

/// Guess a source URL from a clipboard payload's text (bare http/https link).
pub fn url_from_text(text: &str) -> Option<String> {
    let t = text.trim();
    if (t.starts_with("http://") || t.starts_with("https://"))
        && !t.chars().any(|c| c.is_whitespace())
        && t.len() > 12
    {
        return Some(t.trim_end_matches(&['/', ','][..]).to_string());
    }
    // Otherwise find the first URL token inside the text.
    for word in t.split_whitespace() {
        let w = word.trim_matches(&['"', '\'', '<', '>', ')', ']', ',', '.'][..]);
        if (w.starts_with("http://") || w.starts_with("https://")) && w.len() > 12 {
            return Some(w.to_string());
        }
    }
    None
}

/// Parse `SourceURL:` from a CF_HTML header (ASCII header before the HTML body).
pub fn parse_cf_html_source_url(cf_html: &[u8]) -> Option<String> {
    // CF_HTML is UTF-8/ASCII; lossy decode is safe for the header.
    let s = String::from_utf8_lossy(cf_html);
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("SourceURL:") {
            let url = rest.trim();
            if url.starts_with("http://") || url.starts_with("https://") {
                return Some(url.to_string());
            }
        }
    }
    None
}

#[cfg(windows)]
pub fn capture() -> (Option<String>, Option<String>) {
    let app = owner_process_name();
    let url = html_source_url();
    (app, url)
}

#[cfg(windows)]
fn html_source_url() -> Option<String> {
    use windows::Win32::Foundation::{HANDLE, HGLOBAL};
    use windows::Win32::System::DataExchange::{GetClipboardData, RegisterClipboardFormatW};
    use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
    use windows::core::w;

    let fmt = unsafe { RegisterClipboardFormatW(w!("HTML Format")) };
    if fmt == 0 {
        return None;
    }
    unsafe {
        let h: HANDLE = match GetClipboardData(fmt) {
            Ok(h) => h,
            Err(_) => return None,
        };
        if h.0.is_null() {
            return None;
        }
        let hg = HGLOBAL(h.0);
        let ptr = GlobalLock(hg) as *const u8;
        if ptr.is_null() {
            return None;
        }
        let size = GlobalSize(hg);
        // Only the header matters; cap to avoid a huge copy for big HTML bodies.
        let n = size.min(8192);
        let bytes = std::slice::from_raw_parts(ptr, n).to_vec();
        let _ = GlobalUnlock(hg);
        parse_cf_html_source_url(&bytes)
    }
}

#[cfg(windows)]
fn owner_process_name() -> Option<String> {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::DataExchange::GetClipboardOwner;
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    };
    use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
    use windows::core::PWSTR;

    unsafe {
        let owner = match GetClipboardOwner() {
            Ok(h) => h,
            Err(_) => return None,
        };
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(owner, Some(&mut pid as *mut u32));
        if pid == 0 {
            return None;
        }
        let process: HANDLE = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => return None,
        };
        let mut buf = [0u16; 32768];
        let mut len: u32 = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut len as *mut u32,
        )
        .is_ok();
        let _ = windows::Win32::Foundation::CloseHandle(process);
        if !ok || len == 0 {
            return None;
        }
        let full = String::from_utf16_lossy(&buf[..len as usize]);
        // Executable basename only (no full path) — no private directories stored.
        std::path::Path::new(&full)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .filter(|s| !s.is_empty())
    }
}

#[cfg(not(windows))]
pub fn capture() -> (Option<String>, Option<String>) {
    (None, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cf_html_source_url() {
        let html = b"Version:0.9\r\nStartHTML:00000097\r\nSourceURL:https://example.com/page?q=1\r\n<html><body>x</body></html>";
        assert_eq!(
            parse_cf_html_source_url(html),
            Some("https://example.com/page?q=1".to_string())
        );
    }

    #[test]
    fn cf_html_without_source_url() {
        assert_eq!(
            parse_cf_html_source_url(b"Version:0.9\r\n<html>hi</html>"),
            None
        );
    }

    #[test]
    fn bare_url_and_embedded_url() {
        assert_eq!(
            url_from_text("https://a.b/c"),
            Some("https://a.b/c".to_string())
        );
        assert_eq!(
            url_from_text("look at https://example.org/x and more"),
            Some("https://example.org/x".to_string())
        );
        assert_eq!(url_from_text("just plain text"), None);
    }
}
