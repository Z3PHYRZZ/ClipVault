//! Motion helpers ported from animate-ui's published principles.
//!
//! animate-ui is React/Radix and cannot run in Iced. What ports is its motion
//! contract: springs and short tweens on strong ease-out, exits quicker than
//! entrances, transforms/opacity only, interruptible, reduced-motion respected.
//! Only easing + reduced-motion query are needed now (the wave was reverted at
//! Manuel's request); press feedback in theme.rs uses `ease_out_cubic`.

/// Entrances: 260ms strong ease-out. Exits: 160ms (quicker, per animate-ui).
pub const ENTER_MS: u64 = 260;
pub const EXIT_MS: u64 = 160;

/// cubic ease-out: fast start, gentle settle. Faithful to "strong ease-out".
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Linear interpolation of scalars.
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Reduced-motion query. Windows SPI_GETCLIENTAREAANIMATION (0x1042): when the
/// user has disabled animations we skip all motion and jump to final state.
#[cfg(target_os = "windows")]
pub fn reduced_motion() -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{
        SPI_GETCLIENTAREAANIMATION, SystemParametersInfoW,
    };
    let mut enabled: i32 = 1;
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            Some(&mut enabled as *mut i32 as *mut std::ffi::c_void),
            windows::Win32::UI::WindowsAndMessaging::SPIF_UPDATEINIFILE
                | windows::Win32::UI::WindowsAndMessaging::SPIF_SENDCHANGE,
        )
    };
    ok.is_ok() && enabled == 0
}

#[cfg(not(target_os = "windows"))]
pub fn reduced_motion() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ease_out_is_monotonic_and_bounded() {
        let mut prev = -1.0;
        for i in 0..=20 {
            let v = ease_out_cubic(i as f32 / 20.0);
            assert!((0.0..=1.0).contains(&v));
            assert!(v >= prev);
            prev = v;
        }
        assert_eq!(ease_out_cubic(0.0), 0.0);
        assert_eq!(ease_out_cubic(1.0), 1.0);
    }

    #[test]
    fn exit_is_quicker_than_entrance() {
        const { assert!(EXIT_MS < ENTER_MS) };
    }
}
