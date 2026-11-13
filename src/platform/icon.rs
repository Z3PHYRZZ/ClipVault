//! Procedurally generated brand icon (no external asset dependency).
//!
//! The icon uses ONLY the five official brand hexes (plus neutral alpha):
//! Sage background, Peach/Pink card, Dusty Rose "V" mark, Muted Mauve outline.
//! Generated at runtime so the tray icon and the .ico/embedded resource always
//! share one design. Deterministic — no randomness.

use crate::contracts::{MAUVE_RGB, PEACH_RGB, PINK_RGB, ROSE_RGB, SAGE_RGB};
use crate::platform::PlatformError;

const SIZE: usize = 32;

fn px(buf: &mut [u8], x: usize, y: usize, c: (u8, u8, u8), a: u8) {
    if x < SIZE && y < SIZE {
        let i = (y * SIZE + x) * 4;
        buf[i] = c.0;
        buf[i + 1] = c.1;
        buf[i + 2] = c.2;
        buf[i + 3] = a;
    }
}

fn fill_rect(buf: &mut [u8], x0: usize, y0: usize, x1: usize, y1: usize, c: (u8, u8, u8), a: u8) {
    for y in y0..y1 {
        for x in x0..x1 {
            px(buf, x, y, c, a);
        }
    }
}

fn inside_rounded(
    x: usize,
    y: usize,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    r: usize,
) -> bool {
    let (cx0, cy0, cx1, cy1) = (x0 + r, y0 + r, x1 - r, y1 - r);
    let (fx, fy) = (x as f64 + 0.5, y as f64 + 0.5);
    if x < cx0 && y < cy0 {
        (fx - cx0 as f64).powi(2) + (fy - cy0 as f64).powi(2) <= (r as f64).powi(2)
    } else if x >= cx1 && y < cy0 {
        (fx - cx1 as f64).powi(2) + (fy - cy0 as f64).powi(2) <= (r as f64).powi(2)
    } else if x < cx0 && y >= cy1 {
        (fx - cx0 as f64).powi(2) + (fy - cy1 as f64).powi(2) <= (r as f64).powi(2)
    } else if x >= cx1 && y >= cy1 {
        (fx - cx1 as f64).powi(2) + (fy - cy1 as f64).powi(2) <= (r as f64).powi(2)
    } else {
        x >= x0 && x < x1 && y >= y0 && y < y1
    }
}

/// 32x32 RGBA row-major buffer (len = 4096).
pub fn rgba_icon() -> Vec<u8> {
    let mut buf = vec![0u8; SIZE * SIZE * 4];

    // Sage rounded-square background with a Muted Mauve 1px outline.
    for y in 0..SIZE {
        for x in 0..SIZE {
            if inside_rounded(x, y, 1, 1, 31, 31, 6) {
                let border = !inside_rounded(x, y, 2, 2, 30, 30, 5);
                px(
                    &mut buf,
                    x,
                    y,
                    if border { MAUVE_RGB } else { SAGE_RGB },
                    255,
                );
            }
        }
    }

    // Peach clipboard card.
    fill_rect(&mut buf, 8, 7, 24, 26, PEACH_RGB, 255);
    // Pink header strip + Dusty Rose clip tab.
    fill_rect(&mut buf, 8, 7, 24, 11, PINK_RGB, 255);
    fill_rect(&mut buf, 13, 5, 19, 9, ROSE_RGB, 255);
    // Muted Mauve card outline (thin).
    for x in 8..24 {
        px(&mut buf, x, 7, MAUVE_RGB, 120);
        px(&mut buf, x, 25, MAUVE_RGB, 120);
    }
    for y in 7..26 {
        px(&mut buf, 8, y, MAUVE_RGB, 120);
        px(&mut buf, 23, y, MAUVE_RGB, 120);
    }

    // Dusty Rose "V" (two diagonal strokes) — the ClipVault mark.
    for t in 0..7 {
        let y = 13 + t;
        let left = 12 + t / 2;
        let right = 20 - t / 2;
        for dx in 0..2 {
            px(&mut buf, left + dx, y, ROSE_RGB, 255);
            px(&mut buf, right - dx, y, ROSE_RGB, 255);
        }
    }
    buf
}

/// A valid .ico file: ICONDIR + one ICONDIRENTRY + a PNG payload.
pub fn ico_bytes() -> Vec<u8> {
    let png = encode_png(&rgba_icon());
    let mut out = Vec::with_capacity(22 + png.len());
    // ICONDIR: reserved(0), type(1 = icon), count(1).
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    // ICONDIRENTRY: w, h, colors, reserved, planes, bitcount, bytesInRes, offset.
    out.push(32);
    out.push(32);
    out.push(0);
    out.push(0);
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&32u16.to_le_bytes());
    out.extend_from_slice(&(png.len() as u32).to_le_bytes());
    out.extend_from_slice(&22u32.to_le_bytes());
    out.extend_from_slice(&png);
    out
}

fn encode_png(rgba: &[u8]) -> Vec<u8> {
    use image::{ExtendedColorType, ImageEncoder};
    let mut out = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut out);
    let enc = image::codecs::png::PngEncoder::new(&mut cursor);
    if enc
        .write_image(rgba, SIZE as u32, SIZE as u32, ExtendedColorType::Rgba8)
        .is_ok()
    {
        out
    } else {
        Vec::new()
    }
}

/// Write the .ico to `path` (creates parent directories).
pub fn write_ico(path: &std::path::Path) -> Result<(), PlatformError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, ico_bytes())?;
    Ok(())
}

/// Build a Win32 HICON from `rgba_icon()` via CreateIconIndirect.
#[cfg(windows)]
pub fn make_hicon() -> Result<isize, PlatformError> {
    use windows::Win32::Graphics::Gdi::{CreateBitmap, DeleteObject, HBITMAP, HGDIOBJ};
    use windows::Win32::UI::WindowsAndMessaging::{CreateIconIndirect, ICONINFO};
    use windows::core::BOOL;

    let rgba = rgba_icon();
    // Convert RGBA -> bottom-up BGRA for the color bitmap (CreateBitmap bits).
    let mut bgra = vec![0u8; SIZE * SIZE * 4];
    for y in 0..SIZE {
        let src = (SIZE - 1 - y) * SIZE * 4;
        let dst = y * SIZE * 4;
        for x in 0..SIZE {
            let i = src + x * 4;
            let j = dst + x * 4;
            bgra[j] = rgba[i + 2];
            bgra[j + 1] = rgba[i + 1];
            bgra[j + 2] = rgba[i];
            bgra[j + 3] = 255;
        }
    }
    // Monochrome AND mask: all-zero = fully opaque icon.
    let mask = [0u8; SIZE * (SIZE / 8)];

    unsafe {
        let hbm_color: HBITMAP = CreateBitmap(
            SIZE as i32,
            SIZE as i32,
            1,
            32,
            Some(bgra.as_ptr() as *const core::ffi::c_void),
        );
        let hbm_mask: HBITMAP = CreateBitmap(
            SIZE as i32,
            SIZE as i32,
            1,
            1,
            Some(mask.as_ptr() as *const core::ffi::c_void),
        );
        if hbm_color.0.is_null() || hbm_mask.0.is_null() {
            if !hbm_color.0.is_null() {
                let _ = DeleteObject(HGDIOBJ(hbm_color.0));
            }
            if !hbm_mask.0.is_null() {
                let _ = DeleteObject(HGDIOBJ(hbm_mask.0));
            }
            return Err(PlatformError::Other("CreateBitmap failed".into()));
        }
        let info = ICONINFO {
            fIcon: BOOL(1),
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: hbm_mask,
            hbmColor: hbm_color,
        };
        let hicon = CreateIconIndirect(&info)
            .map_err(|e| PlatformError::Windows(format!("CreateIconIndirect: {e}")))?;
        // The system copies the bitmaps; release our references.
        let _ = DeleteObject(HGDIOBJ(hbm_color.0));
        let _ = DeleteObject(HGDIOBJ(hbm_mask.0));

        Ok(hicon.0 as isize)
    }
}

#[cfg(not(windows))]
pub fn make_hicon() -> Result<isize, PlatformError> {
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_icon_is_32x32_rgba() {
        assert_eq!(rgba_icon().len(), 32 * 32 * 4);
    }

    #[test]
    fn ico_has_valid_header_and_png() {
        let ico = ico_bytes();
        assert_eq!(&ico[0..6], &[0, 0, 1, 0, 1, 0]);
        assert!(ico.len() > 22);
        assert_eq!(&ico[22..26], b"\x89PNG");
    }

    #[test]
    fn write_ico_roundtrips() {
        let mut path = std::env::temp_dir();
        path.push(format!("clipvault_icon_test_{}.ico", std::process::id()));
        assert!(write_ico(&path).is_ok());
        assert!(std::fs::read(&path).is_ok_and(|b| b.starts_with(&[0, 0, 1, 0, 1, 0])));
        let _ = std::fs::remove_file(&path);
    }
}
