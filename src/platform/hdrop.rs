//! Pure-Rust `CF_HDROP` (DROPFILES) encode/decode and file metadata stamping.
//! No `windows-rs` here so the module stays headless-testable.

use crate::domain::FileMetadata;

/// Size of the Win32 `DROPFILES` header in bytes: `pFiles:u32` + `POINT(i32,i32)` + `fNC:i32` + `fWide:i32`.
const DROPFILES_HEADER_LEN: u32 = 20;

/// Read a little-endian `u32` at `off`, returning `None` on truncation instead of panicking.
fn read_u32_le(buf: &[u8], off: usize) -> Option<u32> {
    let a = *buf.get(off)?;
    let b = *buf.get(off.checked_add(1)?)?;
    let c = *buf.get(off.checked_add(2)?)?;
    let d = *buf.get(off.checked_add(3)?)?;
    Some(u32::from_le_bytes([a, b, c, d]))
}

/// Build a `CF_HDROP` blob: `DROPFILES` header (`fWide = true`) + UTF-16LE paths
/// separated by NUL and terminated by a double NUL. Returns raw bytes.
///
/// Layout is little-endian: `pFiles = 20`, `pt = (0, 0)`, `fNC = 0`, `fWide = 1`.
/// Win32 expects the file list to always end in two consecutive NUL `u16` units;
/// with zero paths there is no per-path terminator, so a second NUL is emitted
/// explicitly to keep the empty drop double-NUL terminated.
pub fn build_dropfiles_wide(paths: &[String]) -> Vec<u8> {
    let mut out = Vec::with_capacity(DROPFILES_HEADER_LEN as usize + paths.len() * 64);

    out.extend_from_slice(&DROPFILES_HEADER_LEN.to_le_bytes()); // pFiles
    out.extend_from_slice(&0i32.to_le_bytes()); // pt.x
    out.extend_from_slice(&0i32.to_le_bytes()); // pt.y
    out.extend_from_slice(&0i32.to_le_bytes()); // fNC
    out.extend_from_slice(&1i32.to_le_bytes()); // fWide = 1 (UTF-16)

    for path in paths {
        for unit in path.encode_utf16() {
            out.extend_from_slice(&unit.to_le_bytes());
        }
        out.extend_from_slice(&0u16.to_le_bytes()); // per-path NUL terminator
    }
    out.extend_from_slice(&0u16.to_le_bytes()); // final extra NUL -> double-NUL
    if paths.is_empty() {
        out.extend_from_slice(&0u16.to_le_bytes()); // empty drop still double-NUL
    }

    out
}

/// Decode the UTF-16LE unit stream after `pFiles`. Consecutive NUL units
/// (double-NUL) end the list; an unterminated trailing path is kept so truncated
/// input still yields the valid paths parsed so far.
fn parse_wide_list(list: &[u8]) -> Vec<String> {
    let mut paths = Vec::new();
    let mut current: Vec<u16> = Vec::new();
    for chunk in list.chunks_exact(2) {
        let unit = u16::from_le_bytes([chunk[0], chunk[1]]);
        if unit == 0 {
            if current.is_empty() {
                break; // double-NUL (or leading NUL) -> end of list
            }
            paths.push(String::from_utf16_lossy(&current));
            current.clear();
        } else {
            current.push(unit);
        }
    }
    if !current.is_empty() {
        paths.push(String::from_utf16_lossy(&current));
    }
    paths
}

/// Decode the ANSI/narrow byte stream after `pFiles`, using `String::from_utf8_lossy`
/// per path so non-UTF-8 bytes degrade gracefully instead of panicking.
fn parse_narrow_list(list: &[u8]) -> Vec<String> {
    let mut paths = Vec::new();
    let mut current: Vec<u8> = Vec::new();
    for &byte in list {
        if byte == 0 {
            if current.is_empty() {
                break; // double-NUL (or leading NUL) -> end of list
            }
            paths.push(String::from_utf8_lossy(&current).into_owned());
            current.clear();
        } else {
            current.push(byte);
        }
    }
    if !current.is_empty() {
        paths.push(String::from_utf8_lossy(&current).into_owned());
    }
    paths
}

/// Parse a `DROPFILES` blob back into paths (wide and narrow variants).
///
/// Reads `pFiles` (offset 0) and `fWide` (offset 16); the header is skipped via
/// `pFiles`. Malformed, short, or truncated blobs return whatever valid paths
/// were decoded and never panic.
pub fn parse_dropfiles(buf: &[u8]) -> Vec<String> {
    if buf.len() < DROPFILES_HEADER_LEN as usize {
        return Vec::new();
    }
    let p_files = match read_u32_le(buf, 0) {
        Some(v) => v as usize,
        None => return Vec::new(),
    };
    let f_wide = match read_u32_le(buf, 16) {
        Some(v) => v,
        None => return Vec::new(),
    };
    let list = match buf.get(p_files..) {
        Some(l) => l,
        None => return Vec::new(),
    };
    if f_wide != 0 {
        parse_wide_list(list)
    } else {
        parse_narrow_list(list)
    }
}

/// Stat each path into `FileMetadata` (size, is_dir, modified, exists).
/// On any stat error the entry falls back to safe defaults with `exists = false`.
/// `std::fs` is portable, so full metadata is produced on every platform.
pub fn query_metadata(paths: &[String]) -> Vec<FileMetadata> {
    paths
        .iter()
        .map(|path| match std::fs::metadata(path) {
            Ok(md) => {
                let modified_unix = md
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .and_then(|d| i64::try_from(d.as_secs()).ok())
                    .unwrap_or(0);
                FileMetadata {
                    path: path.clone(),
                    size_bytes: md.len(),
                    is_dir: md.is_dir(),
                    modified_unix,
                    exists: true,
                }
            }
            Err(_) => FileMetadata {
                path: path.clone(),
                size_bytes: 0,
                is_dir: false,
                modified_unix: 0,
                exists: false,
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hand-build a narrow (`fWide = 0`) header; caller appends raw list bytes.
    fn narrow_header() -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&20u32.to_le_bytes()); // pFiles
        v.extend_from_slice(&0i32.to_le_bytes()); // pt.x
        v.extend_from_slice(&0i32.to_le_bytes()); // pt.y
        v.extend_from_slice(&0i32.to_le_bytes()); // fNC
        v.extend_from_slice(&0i32.to_le_bytes()); // fWide = 0
        v
    }

    #[test]
    fn header_layout_and_empty_double_nul() {
        let blob = build_dropfiles_wide(&[]);
        // 20-byte header + two NUL u16 units = 24 bytes total.
        assert_eq!(blob.len(), 24);
        assert_eq!(read_u32_le(&blob, 0), Some(20)); // pFiles
        assert_eq!(read_u32_le(&blob, 12), Some(0)); // fNC
        assert_eq!(read_u32_le(&blob, 16), Some(1)); // fWide = 1
        assert_eq!(&blob[20..24], &[0, 0, 0, 0]); // double-NUL
    }

    #[test]
    fn roundtrip_empty() {
        let paths: Vec<String> = Vec::new();
        assert_eq!(parse_dropfiles(&build_dropfiles_wide(&paths)), paths);
    }

    #[test]
    fn roundtrip_single_ascii() {
        let paths = vec!["C:\\Users\\me\\file.txt".to_string()];
        assert_eq!(parse_dropfiles(&build_dropfiles_wide(&paths)), paths);
    }

    #[test]
    fn roundtrip_many_with_unicode() {
        let paths = vec![
            "C:\\a.txt".to_string(),
            "C:\\Users\\测试\\file.txt".to_string(),
            "D:\\日本 語\\x.png".to_string(),
            "E:\\ spaced \\report final.pdf".to_string(),
        ];
        let blob = build_dropfiles_wide(&paths);
        assert_eq!(parse_dropfiles(&blob), paths);
        // The list must end in two consecutive NUL u16 units.
        let tail = &blob[blob.len() - 4..];
        assert_eq!(tail, &[0, 0, 0, 0]);
    }

    #[test]
    fn parse_narrow_handbuilt() {
        let mut blob = narrow_header();
        blob.extend_from_slice(b"C:\\one.txt\0D:\\two.dat\0\0");
        assert_eq!(
            parse_dropfiles(&blob),
            vec!["C:\\one.txt".to_string(), "D:\\two.dat".to_string()]
        );
    }

    #[test]
    fn parse_narrow_lossy_high_bytes() {
        let mut blob = narrow_header();
        blob.extend_from_slice(b"C:\\caf");
        blob.push(0xE9); // lone latin1 'é' is invalid UTF-8 -> from_utf8_lossy yields U+FFFD
        blob.extend_from_slice(b".txt\0\0");
        assert_eq!(
            parse_dropfiles(&blob),
            vec!["C:\\caf\u{FFFD}.txt".to_string()]
        );
    }

    #[test]
    fn parse_stops_at_double_nul() {
        let mut blob = build_dropfiles_wide(&["C:\\a.txt".to_string()]);
        blob.extend_from_slice(
            &"GARBAGE"
                .encode_utf16()
                .flat_map(|u| u.to_le_bytes())
                .collect::<Vec<u8>>(),
        );
        assert_eq!(parse_dropfiles(&blob), vec!["C:\\a.txt".to_string()]);
    }

    #[test]
    fn parse_truncated_is_graceful() {
        let paths = vec!["C:\\first.txt".to_string(), "C:\\second.txt".to_string()];
        let blob = build_dropfiles_wide(&paths);
        // Cut inside the second path: first path must survive without panic.
        let cut = blob.len() - 6;
        let got = parse_dropfiles(&blob[..cut]);
        assert_eq!(got.first().map(String::as_str), Some("C:\\first.txt"));
        assert!(got.len() <= 2);
    }

    #[test]
    fn parse_short_buffer_returns_empty() {
        assert!(parse_dropfiles(&[0, 1, 2]).is_empty());
    }

    #[test]
    fn query_metadata_existing_file_and_missing() {
        let mut path = std::env::temp_dir();
        path.push(format!("clipvault_hdrop_{}.txt", std::process::id()));
        let data = b"clipvault-hdrop-test";
        std::fs::write(&path, data).unwrap();
        let path_str = path.to_string_lossy().into_owned();
        let metas = query_metadata(&[
            path_str,
            "Z:\\definitely_missing_clipvault_xyz\\nope".to_string(),
        ]);
        let _ = std::fs::remove_file(&path);
        assert_eq!(metas.len(), 2);
        assert!(metas[0].exists);
        assert_eq!(metas[0].size_bytes, data.len() as u64);
        assert!(!metas[0].is_dir);
        assert!(metas[0].modified_unix > 0);
        assert!(!metas[1].exists);
        assert_eq!(metas[1].size_bytes, 0);
        assert_eq!(metas[1].modified_unix, 0);
    }

    #[test]
    fn query_metadata_directory() {
        let dir = std::env::temp_dir();
        let path_str = dir.to_string_lossy().into_owned();
        let metas = query_metadata(&[path_str]);
        assert!(metas[0].exists);
        assert!(metas[0].is_dir);
    }
}
