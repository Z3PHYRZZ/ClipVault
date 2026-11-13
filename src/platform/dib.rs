//! Pure-Rust `CF_DIB` / `CF_DIBV5` <-> PNG conversion for the Windows clipboard.
//!
//! No `windows-rs` calls live here on purpose: the module is byte-in / byte-out so it
//! can be unit-tested headless on any target. The clipboard backend obtains the blob
//! via `GlobalLock` + `GlobalSize` and hands the whole `&[u8]` to [`dib_to_png`];
//! every offset below is bounds-checked against that length, so a truncated HGLOBAL
//! can never cause an out-of-bounds read.
//!
//! ## Windows API limitation worked around
//! `CF_DIB` carries a `BITMAPINFOHEADER` (there is no `BITMAPFILEHEADER` on the
//! clipboard) and cannot formally express an alpha channel: for 32bpp `BI_RGB` the
//! fourth byte is documented as unused. In practice modern producers — and our own
//! [`png_to_dib`] — store real straight alpha there, while legacy producers leave it
//! zero. We treat a 32bpp `BI_RGB` image as opaque when *all* fourth bytes are zero,
//! and as straight alpha otherwise. The correct clipboard format for true per-pixel
//! alpha is `CF_DIBV5` (`BITMAPV5HEADER` + `bV5AlphaMask`); this parser reads V5 masks,
//! but the frozen write contract emits `CF_DIB` only. Publishing `CF_DIBV5` alongside
//! `CF_DIB` is an integrator-level addition and does not change these signatures.
//!
//! `biSizeImage` is deliberately ignored: many producers leave it zero for `BI_RGB`.
//! Stride is always recomputed from `biWidth` and `biBitCount`, which is authoritative.

use crate::platform::PlatformError;
use image::{GenericImageView, ImageEncoder};
use std::io::Cursor;

const BI_RGB: u32 = 0;
const BI_BITFIELDS: u32 = 3;
const INFO_HEADER_SIZE: u32 = 40;
const V5_HEADER_SIZE: u32 = 124;

fn other<T>(msg: impl Into<String>) -> Result<T, PlatformError> {
    Err(PlatformError::Other(msg.into()))
}

fn read_u16_le(b: &[u8], off: usize) -> Option<u16> {
    let arr: [u8; 2] = b.get(off..off.checked_add(2)?)?.try_into().ok()?;
    Some(u16::from_le_bytes(arr))
}

fn read_u32_le(b: &[u8], off: usize) -> Option<u32> {
    let arr: [u8; 4] = b.get(off..off.checked_add(4)?)?.try_into().ok()?;
    Some(u32::from_le_bytes(arr))
}

fn read_i32_le(b: &[u8], off: usize) -> Option<i32> {
    read_u32_le(b, off).map(|v| v as i32)
}

fn write_u16_le(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn write_u32_le(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

/// Parsed DIB metadata plus where the pixel array starts within the blob.
struct HeaderInfo {
    width: u32,
    height: u32,
    top_down: bool,
    bpp: u16,
    compression: u32,
    /// Byte count of the header (40 or 124) plus any mask/table preamble.
    pixel_offset: usize,
    /// Color table entries, already converted to RGBA (palette bpp only).
    palette: Vec<[u8; 4]>,
    /// (red, green, blue, alpha) masks for `BI_BITFIELDS` 16/32bpp.
    masks: Option<(u32, u32, u32, u32)>,
}

/// DWORD-aligned row stride in bytes for `width` pixels at `bpp` bits each.
fn row_stride(width: u32, bpp: u16) -> Option<usize> {
    let bits = (width as u64).checked_mul(bpp as u64)?;
    let bytes = bits.checked_add(31)? / 32;
    usize::try_from(bytes.checked_mul(4)?).ok()
}

/// Extract a component from a packed pixel using a bit mask (a zero mask yields 0;
/// the alpha channel is handled separately in `decode_pixel` so a missing alpha mask
/// means opaque rather than transparent).
fn masked_component(pixel: u32, mask: u32) -> u8 {
    if mask == 0 {
        return 0;
    }
    let bits = (pixel & mask) >> mask.trailing_zeros();
    let width = mask.count_ones();
    if width >= 8 {
        (bits >> (width - 8)) as u8
    } else {
        let max = (1u32 << width) - 1;
        ((bits * 255 + max / 2) / max) as u8
    }
}

/// Parse header + color table + bitfield masks. All slice accesses are checked.
fn parse_header(dib: &[u8]) -> Result<HeaderInfo, PlatformError> {
    let bi_size = read_u32_le(dib, 0)
        .ok_or_else(|| PlatformError::Other("DIB too short for biSize".into()))?;
    let (header_size, masks_in_header) = match bi_size {
        INFO_HEADER_SIZE => (INFO_HEADER_SIZE as usize, false),
        V5_HEADER_SIZE => (V5_HEADER_SIZE as usize, true),
        _ => return other(format!("unsupported DIB header biSize={bi_size}")),
    };
    if dib.len() < header_size {
        return other("DIB shorter than declared header");
    }

    let raw_width =
        read_i32_le(dib, 4).ok_or_else(|| PlatformError::Other("missing biWidth".into()))?;
    let raw_height =
        read_i32_le(dib, 8).ok_or_else(|| PlatformError::Other("missing biHeight".into()))?;
    let planes =
        read_u16_le(dib, 12).ok_or_else(|| PlatformError::Other("missing biPlanes".into()))?;
    let bpp =
        read_u16_le(dib, 14).ok_or_else(|| PlatformError::Other("missing biBitCount".into()))?;
    let compression =
        read_u32_le(dib, 16).ok_or_else(|| PlatformError::Other("missing biCompression".into()))?;
    let clr_used =
        read_u32_le(dib, 32).ok_or_else(|| PlatformError::Other("missing biClrUsed".into()))?;

    if planes != 1 {
        return other(format!("invalid biPlanes={planes}"));
    }
    let width = u32::try_from(raw_width)
        .map_err(|_| PlatformError::Other(format!("invalid biWidth={raw_width}")))?;
    let height = raw_height.unsigned_abs();
    if width == 0 || height == 0 {
        return other("DIB has a zero dimension");
    }
    if !matches!(bpp, 1 | 4 | 8 | 16 | 24 | 32) {
        return other(format!("unsupported biBitCount={bpp}"));
    }
    if compression != BI_RGB && compression != BI_BITFIELDS {
        return other(format!(
            "unsupported biCompression={compression} (RLE not supported)"
        ));
    }
    if compression == BI_BITFIELDS && !matches!(bpp, 16 | 32) {
        return other("BI_BITFIELDS requires 16 or 32 bpp");
    }

    let mut cursor = header_size;
    let mut masks: Option<(u32, u32, u32, u32)> = None;

    if compression == BI_BITFIELDS {
        let (r, g, b, a) = if masks_in_header {
            // BITMAPV5HEADER: masks live at fixed offsets inside the 124-byte header.
            let r = read_u32_le(dib, 40)
                .ok_or_else(|| PlatformError::Other("missing bV5RedMask".into()))?;
            let g = read_u32_le(dib, 44)
                .ok_or_else(|| PlatformError::Other("missing bV5GreenMask".into()))?;
            let b = read_u32_le(dib, 48)
                .ok_or_else(|| PlatformError::Other("missing bV5BlueMask".into()))?;
            let a = read_u32_le(dib, 52)
                .ok_or_else(|| PlatformError::Other("missing bV5AlphaMask".into()))?;
            (r, g, b, a)
        } else {
            // BITMAPINFOHEADER: three DWORD masks follow the 40-byte header.
            let r = read_u32_le(dib, cursor)
                .ok_or_else(|| PlatformError::Other("missing red mask".into()))?;
            let g = read_u32_le(dib, cursor + 4)
                .ok_or_else(|| PlatformError::Other("missing green mask".into()))?;
            let b = read_u32_le(dib, cursor + 8)
                .ok_or_else(|| PlatformError::Other("missing blue mask".into()))?;
            cursor += 12;
            (r, g, b, 0)
        };
        masks = Some((r, g, b, a));
    }

    let mut palette: Vec<[u8; 4]> = Vec::new();
    if bpp <= 8 {
        let default_count = 1u32 << (bpp as u32);
        let count = if clr_used == 0 {
            default_count
        } else {
            clr_used
        };
        let count = count.min(1u32 << 16);
        let table_bytes = (count as usize)
            .checked_mul(4)
            .ok_or_else(|| PlatformError::Other("palette size overflow".into()))?;
        let table_end = cursor
            .checked_add(table_bytes)
            .ok_or_else(|| PlatformError::Other("palette offset overflow".into()))?;
        if table_end > dib.len() {
            return other("truncated color table");
        }
        for i in 0..count as usize {
            let off = cursor + i * 4;
            // Bounds already verified via table_end <= dib.len().
            let blue = dib[off];
            let green = dib[off + 1];
            let red = dib[off + 2];
            palette.push([red, green, blue, 255]);
        }
        cursor = table_end;
    }

    let stride =
        row_stride(width, bpp).ok_or_else(|| PlatformError::Other("row stride overflow".into()))?;
    let pixel_bytes = stride
        .checked_mul(height as usize)
        .ok_or_else(|| PlatformError::Other("pixel array overflow".into()))?;
    let required = cursor
        .checked_add(pixel_bytes)
        .ok_or_else(|| PlatformError::Other("pixel offset overflow".into()))?;
    if required > dib.len() {
        return other(format!(
            "truncated pixel data: need {required} bytes, have {}",
            dib.len()
        ));
    }

    Ok(HeaderInfo {
        width,
        height,
        top_down: raw_height < 0,
        bpp,
        compression,
        pixel_offset: cursor,
        palette,
        masks,
    })
}

/// Decode a packed pixel (16/32bpp, BI_RGB or BI_BITFIELDS) into RGBA.
fn decode_pixel(pixel: u32, info: &HeaderInfo, opaque_alpha: bool) -> [u8; 4] {
    if let Some((r, g, b, a)) = info.masks {
        let alpha = if a == 0 {
            255
        } else {
            masked_component(pixel, a)
        };
        return [
            masked_component(pixel, r),
            masked_component(pixel, g),
            masked_component(pixel, b),
            alpha,
        ];
    }
    match info.bpp {
        32 => [
            ((pixel >> 16) & 0xFF) as u8,
            ((pixel >> 8) & 0xFF) as u8,
            (pixel & 0xFF) as u8,
            {
                let a = ((pixel >> 24) & 0xFF) as u8;
                if opaque_alpha { 255 } else { a }
            },
        ],
        16 => {
            // BI_RGB 16bpp is 5-5-5 (high bit unused). Scale after widening so the
            // 255 multiplier cannot overflow a u8 intermediate.
            let scale = |v: u32| -> u8 { ((v * 255 + 15) / 31) as u8 };
            [
                scale((pixel >> 10) & 0x1F),
                scale((pixel >> 5) & 0x1F),
                scale(pixel & 0x1F),
                255,
            ]
        }
        _ => [0, 0, 0, 255],
    }
}

/// Determine whether a 32bpp BI_RGB blob should be treated as opaque.
/// If every alpha byte is zero the producer left it unused (legacy CF_DIB).
fn all_alpha_zero(dib: &[u8], info: &HeaderInfo) -> bool {
    if info.bpp != 32 || info.compression != BI_RGB {
        return false;
    }
    let stride = match row_stride(info.width, info.bpp) {
        Some(s) => s,
        None => return false,
    };
    for row in 0..info.height as usize {
        let base = info.pixel_offset + row * stride;
        for x in 0..info.width as usize {
            match dib.get(base + x * 4 + 3) {
                Some(&0) => {}
                _ => return false,
            }
        }
    }
    true
}

/// Parse a `CF_DIB` / `CF_DIBV5` clipboard blob into normalized PNG bytes + (width, height).
///
/// Handles `BITMAPINFOHEADER` (40) and `BITMAPV5HEADER` (124), top-down (negative
/// `biHeight`) and bottom-up rows, 1/4/8bpp palettes, 16/24/32bpp color, and
/// `BI_BITFIELDS` masks. Returns `PlatformError::Other` on any malformed input.
pub fn dib_to_png(dib: &[u8]) -> Result<(Vec<u8>, u32, u32), PlatformError> {
    let info = parse_header(dib)?;
    let stride = row_stride(info.width, info.bpp)
        .ok_or_else(|| PlatformError::Other("row stride overflow".into()))?;
    let w = info.width as usize;
    let h = info.height as usize;
    let mut rgba: Vec<u8> = Vec::with_capacity(
        w.checked_mul(h)
            .and_then(|n| n.checked_mul(4))
            .ok_or_else(|| PlatformError::Other("image size overflow".into()))?,
    );

    let opaque = all_alpha_zero(dib, &info);

    for y in 0..h {
        // Bottom-up DIB: row 0 of the blob is the last row of the image.
        let src_row = if info.top_down { y } else { h - 1 - y };
        let base = info
            .pixel_offset
            .checked_add(
                src_row
                    .checked_mul(stride)
                    .ok_or_else(|| PlatformError::Other("row offset overflow".into()))?,
            )
            .ok_or_else(|| PlatformError::Other("row offset overflow".into()))?;

        match info.bpp {
            32 => {
                for x in 0..w {
                    let p = read_u32_le(dib, base + x * 4)
                        .ok_or_else(|| PlatformError::Other("truncated 32bpp row".into()))?;
                    rgba.extend_from_slice(&decode_pixel(p, &info, opaque));
                }
            }
            24 => {
                for x in 0..w {
                    let off = base + x * 3;
                    let b = *dib
                        .get(off)
                        .ok_or_else(|| PlatformError::Other("truncated 24bpp row".into()))?;
                    let g = *dib
                        .get(off + 1)
                        .ok_or_else(|| PlatformError::Other("truncated 24bpp row".into()))?;
                    let r = *dib
                        .get(off + 2)
                        .ok_or_else(|| PlatformError::Other("truncated 24bpp row".into()))?;
                    rgba.extend_from_slice(&[r, g, b, 255]);
                }
            }
            16 => {
                for x in 0..w {
                    let p = read_u16_le(dib, base + x * 2)
                        .ok_or_else(|| PlatformError::Other("truncated 16bpp row".into()))?;
                    rgba.extend_from_slice(&decode_pixel(p as u32, &info, opaque));
                }
            }
            8 => {
                for x in 0..w {
                    let idx = *dib
                        .get(base + x)
                        .ok_or_else(|| PlatformError::Other("truncated 8bpp row".into()))?
                        as usize;
                    let c = info.palette.get(idx).ok_or_else(|| {
                        PlatformError::Other(format!("palette index {idx} out of range"))
                    })?;
                    rgba.extend_from_slice(c);
                }
            }
            4 | 1 => {
                for x in 0..w {
                    let byte = *dib
                        .get(base + x / (8 / info.bpp as usize))
                        .ok_or_else(|| PlatformError::Other("truncated palette row".into()))?;
                    let idx = if info.bpp == 4 {
                        let shift = if x % 2 == 0 { 4 } else { 0 };
                        ((byte >> shift) & 0x0F) as usize
                    } else {
                        let shift = 7 - (x % 8);
                        ((byte >> shift) & 0x01) as usize
                    };
                    let c = info.palette.get(idx).ok_or_else(|| {
                        PlatformError::Other(format!("palette index {idx} out of range"))
                    })?;
                    rgba.extend_from_slice(c);
                }
            }
            _ => return other("unsupported bit depth in decode loop"),
        }
    }

    let mut png: Vec<u8> = Vec::new();
    {
        let mut cursor = Cursor::new(&mut png);
        let encoder = image::codecs::png::PngEncoder::new(&mut cursor);
        encoder
            .write_image(
                &rgba,
                info.width,
                info.height,
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|e| PlatformError::Other(format!("PNG encode failed: {e}")))?;
    }
    Ok((png, info.width, info.height))
}

/// Convert PNG bytes back into a `CF_DIB` blob for `SetClipboardData`.
///
/// Emits a 40-byte `BITMAPINFOHEADER` + bottom-up, DWORD-aligned 32bpp `BI_RGB`
/// BGRA rows. See the module doc for the alpha convention and the `CF_DIBV5` note.
pub fn png_to_dib(png: &[u8]) -> Result<Vec<u8>, PlatformError> {
    let dyn_img = image::load_from_memory(png)
        .map_err(|e| PlatformError::Other(format!("PNG decode failed: {e}")))?;
    let (width, height) = dyn_img.dimensions();
    if width == 0 || height == 0 {
        return other("PNG has a zero dimension");
    }
    let stride =
        row_stride(width, 32).ok_or_else(|| PlatformError::Other("row stride overflow".into()))?;
    let img = dyn_img.to_rgba8();

    let capacity = stride
        .checked_mul(height as usize)
        .and_then(|n| n.checked_add(40))
        .ok_or_else(|| PlatformError::Other("DIB size overflow".into()))?;
    let mut out: Vec<u8> = Vec::with_capacity(capacity);
    // BITMAPINFOHEADER
    write_u32_le(&mut out, INFO_HEADER_SIZE);
    write_u32_le(&mut out, width); // biWidth
    write_u32_le(&mut out, height); // biHeight (>0 = bottom-up)
    write_u16_le(&mut out, 1); // biPlanes
    write_u16_le(&mut out, 32); // biBitCount
    write_u32_le(&mut out, BI_RGB); // biCompression
    write_u32_le(&mut out, 0); // biSizeImage (allowed 0 for BI_RGB)
    write_u32_le(&mut out, 0); // biXPelsPerMeter
    write_u32_le(&mut out, 0); // biYPelsPerMeter
    write_u32_le(&mut out, 0); // biClrUsed
    write_u32_le(&mut out, 0); // biClrImportant

    // Rows are stored bottom-up; padding bytes stay zero.
    for y in 0..height {
        let src_y = height - 1 - y;
        let row_start = out.len();
        out.resize(row_start + stride, 0);
        for x in 0..width {
            let px = img.get_pixel(x, src_y).0; // [r, g, b, a]
            let off = row_start + x as usize * 4;
            out[off] = px[2]; // B
            out[off + 1] = px[1]; // G
            out[off + 2] = px[0]; // R
            out[off + 3] = px[3]; // A
        }
    }
    Ok(out)
}

/// Read `(width, |height|)` from a DIB header without decoding pixels.
///
/// Returns `None` for a short buffer, an unknown `biSize`, or a non-positive dimension.
pub fn dib_dimensions(dib: &[u8]) -> Option<(u32, u32)> {
    let bi_size = read_u32_le(dib, 0)?;
    if bi_size != INFO_HEADER_SIZE && bi_size != V5_HEADER_SIZE {
        return None;
    }
    if dib.len() < 12 {
        return None;
    }
    let raw_width = read_i32_le(dib, 4)?;
    let raw_height = read_i32_le(dib, 8)?;
    let width = u32::try_from(raw_width).ok()?;
    let height = raw_height.unsigned_abs();
    if width == 0 || height == 0 {
        return None;
    }
    Some((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a 32bpp BI_RGB header; positive height = bottom-up.
    fn header32(width: i32, height: i32) -> Vec<u8> {
        let mut h = Vec::new();
        write_u32_le(&mut h, INFO_HEADER_SIZE);
        write_u32_le(&mut h, width as u32);
        write_u32_le(&mut h, height as u32);
        write_u16_le(&mut h, 1);
        write_u16_le(&mut h, 32);
        write_u32_le(&mut h, BI_RGB);
        write_u32_le(&mut h, 0);
        write_u32_le(&mut h, 0);
        write_u32_le(&mut h, 0);
        write_u32_le(&mut h, 0);
        write_u32_le(&mut h, 0);
        h
    }

    #[test]
    fn decode_2x2_32bpp_bgra() {
        // Bottom-up: blob row 0 = image bottom row.
        // image: top = [red, green], bottom = [blue, white]
        let mut dib = header32(2, 2);
        let stride = row_stride(2, 32).unwrap();
        assert_eq!(stride, 8);
        // bottom row (B,G,R,A) = blue, white
        dib.extend_from_slice(&[255, 0, 0, 255, 255, 255, 255, 255]);
        // top row = red, green
        dib.extend_from_slice(&[0, 0, 255, 255, 0, 255, 0, 255]);

        let (png, w, h) = dib_to_png(&dib).unwrap();
        assert_eq!((w, h), (2, 2));
        let img = image::load_from_memory(&png).unwrap().to_rgba8();
        assert_eq!(img.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(img.get_pixel(1, 0).0, [0, 255, 0, 255]);
        assert_eq!(img.get_pixel(0, 1).0, [0, 0, 255, 255]);
        assert_eq!(img.get_pixel(1, 1).0, [255, 255, 255, 255]);
    }

    #[test]
    fn decode_1x1_24bpp() {
        let mut h = Vec::new();
        write_u32_le(&mut h, INFO_HEADER_SIZE);
        write_u32_le(&mut h, 1);
        write_u32_le(&mut h, 1);
        write_u16_le(&mut h, 1);
        write_u16_le(&mut h, 24);
        write_u32_le(&mut h, BI_RGB);
        for _ in 0..5 {
            write_u32_le(&mut h, 0);
        }
        // stride is 4: 3 color bytes + 1 pad.
        h.extend_from_slice(&[10, 20, 30, 0]);
        let (png, w, hh) = dib_to_png(&h).unwrap();
        assert_eq!((w, hh), (1, 1));
        let img = image::load_from_memory(&png).unwrap().to_rgba8();
        assert_eq!(img.get_pixel(0, 0).0, [30, 20, 10, 255]);
    }

    #[test]
    fn decode_negative_height_top_down() {
        let mut dib = header32(1, -2);
        // top-down: blob row 0 = image top row (green), row 1 = bottom (red).
        dib.extend_from_slice(&[0, 255, 0, 255]);
        dib.extend_from_slice(&[0, 0, 255, 255]);
        let (png, w, h) = dib_to_png(&dib).unwrap();
        assert_eq!((w, h), (1, 2));
        let img = image::load_from_memory(&png).unwrap().to_rgba8();
        assert_eq!(img.get_pixel(0, 0).0, [0, 255, 0, 255]);
        assert_eq!(img.get_pixel(0, 1).0, [255, 0, 0, 255]);
        assert_eq!(dib_dimensions(&dib), Some((1, 2)));
    }

    #[test]
    fn malformed_header_is_rejected() {
        // Too short.
        assert!(dib_to_png(&[0u8; 3]).is_err());
        assert_eq!(dib_dimensions(&[0u8; 3]), None);
        // Unknown biSize.
        let mut bad = vec![0u8; 40];
        bad[0..4].copy_from_slice(&12u32.to_le_bytes());
        assert!(dib_to_png(&bad).is_err());
        assert_eq!(dib_dimensions(&bad), None);
        // Valid header but truncated pixel data.
        let mut trunc = header32(4, 4);
        trunc.extend_from_slice(&[0u8; 4]);
        assert!(dib_to_png(&trunc).is_err());
    }

    #[test]
    fn roundtrip_png_to_dib_to_png() {
        let (orig_png, w, h) = {
            let mut h = header32(3, 2);
            let stride = row_stride(3, 32).unwrap();
            assert_eq!(stride, 12);
            for row in 0..2u8 {
                let start = h.len();
                h.resize(start + stride, 0);
                for x in 0..3u8 {
                    let off = start + x as usize * 4;
                    h[off] = 10 * x + row; // B
                    h[off + 1] = 40 + 10 * x; // G
                    h[off + 2] = 200 - 10 * x; // R
                    h[off + 3] = 255;
                }
            }
            dib_to_png(&h).unwrap()
        };
        assert_eq!((w, h), (3, 2));

        let dib = png_to_dib(&orig_png).unwrap();
        assert_eq!(dib_dimensions(&dib), Some((3, 2)));
        let (back_png, w2, h2) = dib_to_png(&dib).unwrap();
        assert_eq!((w2, h2), (3, 2));

        let a = image::load_from_memory(&orig_png).unwrap().to_rgba8();
        let b = image::load_from_memory(&back_png).unwrap().to_rgba8();
        for y in 0..h {
            for x in 0..w {
                assert_eq!(a.get_pixel(x, y).0, b.get_pixel(x, y).0, "pixel ({x},{y})");
            }
        }
    }

    #[test]
    fn palette_8bpp_and_bitfields_32bpp() {
        // 8bpp palette with 2 entries.
        let mut h = Vec::new();
        write_u32_le(&mut h, INFO_HEADER_SIZE);
        write_u32_le(&mut h, 2);
        write_u32_le(&mut h, 1);
        write_u16_le(&mut h, 1);
        write_u16_le(&mut h, 8);
        write_u32_le(&mut h, BI_RGB);
        write_u32_le(&mut h, 0);
        write_u32_le(&mut h, 0);
        write_u32_le(&mut h, 0);
        write_u32_le(&mut h, 2); // biClrUsed = 2
        write_u32_le(&mut h, 0);
        // palette: index 0 = red (B,G,R,0), index 1 = blue
        h.extend_from_slice(&[0, 0, 255, 0, 255, 0, 0, 0]);
        let stride = row_stride(2, 8).unwrap();
        let start = h.len();
        h.resize(start + stride, 0);
        h[start] = 0;
        h[start + 1] = 1;
        let (png, w, hh) = dib_to_png(&h).unwrap();
        assert_eq!((w, hh), (2, 1));
        let img = image::load_from_memory(&png).unwrap().to_rgba8();
        assert_eq!(img.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(img.get_pixel(1, 0).0, [0, 0, 255, 255]);

        // 32bpp BI_BITFIELDS (BITMAPINFOHEADER carries only 3 masks, no alpha).
        let mut bf = header32(1, 1);
        bf[16..20].copy_from_slice(&BI_BITFIELDS.to_le_bytes());
        bf.extend_from_slice(&0x00FF0000u32.to_le_bytes()); // R
        bf.extend_from_slice(&0x0000FF00u32.to_le_bytes()); // G
        bf.extend_from_slice(&0x000000FFu32.to_le_bytes()); // B
        // pixel: high byte unused, R=0x12, G=0x34, B=0x56 -> opaque alpha.
        bf.extend_from_slice(&0x80123456u32.to_le_bytes());
        let (png, _, _) = dib_to_png(&bf).unwrap();
        let img = image::load_from_memory(&png).unwrap().to_rgba8();
        assert_eq!(img.get_pixel(0, 0).0, [0x12, 0x34, 0x56, 255]);
    }
}
