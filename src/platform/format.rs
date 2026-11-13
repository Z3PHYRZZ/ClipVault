//! Standard Win32 clipboard format constants and clipboard-read helpers.
//!
//! The `CF_*` values below are stable Win32 defines from `winuser.h`. They are
//! numeric format identifiers passed to `GetClipboardData`/`SetClipboardData`
//! and never change across Windows versions, so they are safe to hard-code.

/// `CF_UNICODETEXT` (13): UTF-16LE text in a global memory block. The most
/// common clipboard payload and the cheapest to decode.
pub const CF_UNICODETEXT: u32 = 13;

/// `CF_HDROP` (15): a `DROPFILES` structure describing a file drop list.
pub const CF_HDROP: u32 = 15;

/// `CF_DIB` (8): a device-independent bitmap (`BITMAPINFOHEADER` + pixels)
/// held in global memory. Win32 synthesizes this from `CF_BITMAP` on read.
pub const CF_DIB: u32 = 8;

/// `CF_DIBV5` (17): device-independent bitmap with the extended
/// `BITMAPV5HEADER`. Preferred over `CF_DIB` because it carries color-space
/// and alpha information. Win32 also synthesizes this on read.
pub const CF_DIBV5: u32 = 17;

/// `CF_BITMAP` (2): a device-dependent `HBITMAP` handle (a GDI object, not a
/// memory block). Last resort: it is tied to a device context and requires GDI
/// conversion, unlike the self-contained DIB blobs above.
pub const CF_BITMAP: u32 = 2;

/// Formats a clipboard reader should probe, in priority order (text > files > image).
///
/// The order is deliberate:
/// - Unicode text is by far the most common payload and the cheapest to decode,
///   so probing it first resolves the large majority of reads with one call.
/// - File drop lists come next: a single `CF_HDROP` is more meaningful than an
///   image a file manager may also place on the clipboard as a thumbnail.
/// - Image fallbacks follow, richest first: `CF_DIBV5` (alpha + color space),
///   then `CF_DIB`, then the device-dependent `CF_BITMAP` handle last. DIB and
///   DIBV5 are device-independent memory blobs Win32 synthesizes from
///   `CF_BITMAP`, so reading them first avoids the device-context-bound handle.
///
/// Returns a borrowed static slice — no allocation.
pub fn read_priority() -> &'static [u32] {
    &READ_PRIORITY
}

/// Backing array for [`read_priority`]. A `static` (not a `const` slice) gives a
/// genuine `'static` borrow without promotion ambiguity.
static READ_PRIORITY: [u32; 5] = [CF_UNICODETEXT, CF_HDROP, CF_DIBV5, CF_DIB, CF_BITMAP];

/// Cheap change discriminator used to reject self-induced clipboard updates.
///
/// `GetClipboardSequenceNumber` returns a per-session DWORD that the system
/// increments on every `EmptyClipboard` and `SetClipboardData` call (i.e. every
/// clipboard write, including our own restores). Capturing it before a write
/// and comparing against the value seen in `WM_CLIPBOARDUPDATE` lets the
/// listener skip notifications caused by its own writes instead of re-reading
/// and re-inserting the content it just placed. It is a single user32 call with
/// no clipboard open/lock required, which is why it is the correct cheap
/// discriminator here. Returns `0` only on failure.
///
/// This function is `unsafe`-free on the outside; the raw Win32 call is
/// confined to the inner `unsafe` block.
#[cfg(windows)]
pub fn sequence_number() -> u32 {
    use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
    // SAFETY: GetClipboardSequenceNumber has no arguments, no preconditions,
    // and reads only a process-wide counter; it is always safe to call.
    unsafe { GetClipboardSequenceNumber() }
}

/// Headless counterpart of [`sequence_number`]. Returns a monotonically
/// increasing value from a process-wide `AtomicU32` so that self-filter logic
/// and tests can still exercise change detection on non-Windows targets where
/// there is no OS clipboard sequence counter.
#[cfg(not(windows))]
pub fn sequence_number() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static SEQ: AtomicU32 = AtomicU32::new(0);
    // fetch_add returns the previous value; +1 makes the first call return 1
    // (0 is reserved as the "no clipboard / failure" sentinel on Windows).
    SEQ.fetch_add(1, Ordering::SeqCst) + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_priority_contains_all_five_formats() {
        let p = read_priority();
        assert_eq!(p.len(), 5);
        for &c in &[CF_UNICODETEXT, CF_HDROP, CF_DIB, CF_DIBV5, CF_BITMAP] {
            assert!(p.contains(&c), "missing clipboard format {c}");
        }
    }

    #[test]
    fn read_priority_has_no_duplicates() {
        let p = read_priority();
        for (i, &a) in p.iter().enumerate() {
            for &b in &p[i + 1..] {
                assert_ne!(a, b, "duplicate clipboard format {a}");
            }
        }
    }

    #[test]
    fn read_priority_text_first() {
        assert_eq!(read_priority()[0], CF_UNICODETEXT);
    }

    #[cfg(not(windows))]
    #[test]
    fn sequence_number_is_non_decreasing() {
        let a = sequence_number();
        let b = sequence_number();
        assert!(b >= a, "sequence went backwards: {a} -> {b}");
    }
}
