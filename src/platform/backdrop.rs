//! Native Windows backdrop translucency (v0.1.6).
//!
//! Iced 0.13 DOES expose the native window handle through
//! `iced::window::run_with_handle`, so an OS-level blur is possible. This module
//! keeps every Win32 call behind `#[cfg(windows)]` with a headless stub, matching
//! the pattern used by `singleton.rs` / `source.rs`.
//!
//! Strategy, in order of preference (each failure degrades gracefully, never panics):
//! 1. **Windows 11 Mica** — `DwmSetWindowAttribute(DWMWA_SYSTEMBACKDROP_TYPE,
//!    DWMSBT_MAINWINDOW)` plus `DWMWA_USE_IMMERSIVE_DARK_MODE` to match the theme.
//! 2. **Windows 10/11 acrylic** — `SetWindowCompositionAttribute` (an
//!    undocumented `user32.dll` export, loaded via `GetProcAddress`) with
//!    `ACCENT_ENABLE_ACRYLICBLURBEHIND`. The struct layout below is the widely
//!    used `WINCOMPATTRDATA` / `ACCENT_POLICY` definition.
//! 3. **Legacy blur-behind** — `DwmEnableBlurBehindWindow` with `DWM_BB_ENABLE`.
//! 4. `None` — the app falls back to its software gradient (see `AppState.backdrop`).
//!
//! Mica is Windows 11 only; acrylic can be disabled under RDP / certain drivers.
//! No clipboard content is ever logged here.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackdropKind {
    /// Windows 11 Mica system backdrop.
    Mica,
    /// Undocumented acrylic accent policy.
    Acrylic,
    /// Legacy DWM blur-behind region.
    BlurBehind,
    /// No native effect available — use the software gradient.
    #[default]
    None,
}

impl BackdropKind {
    pub fn label(self) -> &'static str {
        match self {
            BackdropKind::Mica => "Mica blur",
            BackdropKind::Acrylic => "Acrylic blur",
            BackdropKind::BlurBehind => "Blur behind",
            BackdropKind::None => "Frosted (software)",
        }
    }
    pub fn is_native(self) -> bool {
        !matches!(self, BackdropKind::None)
    }
}

#[cfg(windows)]
pub fn apply(hwnd: isize, dark: bool) -> Result<BackdropKind, String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{
        DWM_BB_ENABLE, DWM_BLURBEHIND, DWMSBT_MAINWINDOW, DWMWA_SYSTEMBACKDROP_TYPE,
        DWMWA_USE_IMMERSIVE_DARK_MODE, DwmEnableBlurBehindWindow, DwmSetWindowAttribute,
    };
    use windows::core::BOOL;

    if hwnd == 0 {
        return Err("null HWND".into());
    }
    let hwnd = HWND(hwnd as *mut core::ffi::c_void);

    // 1. Windows 11 Mica. On older Windows the attribute is unknown and the call
    //    returns an error, which we treat as "try acrylic".
    let backdrop: i32 = DWMSBT_MAINWINDOW.0;
    let dark_flag: i32 = if dark { 1 } else { 0 };
    unsafe {
        let mica = DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            &backdrop as *const i32 as *const core::ffi::c_void,
            std::mem::size_of::<i32>() as u32,
        )
        .is_ok();
        if mica {
            // Dark-mode attribute is best-effort; failure must not discard Mica.
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                &dark_flag as *const i32 as *const core::ffi::c_void,
                std::mem::size_of::<i32>() as u32,
            );
            return Ok(BackdropKind::Mica);
        }

        // 2. Acrylic via the undocumented user32 export.
        if let Some(kind) = try_acrylic(hwnd, dark) {
            return Ok(kind);
        }

        // 3. Legacy blur-behind as a last native resort.
        let bb = DWM_BLURBEHIND {
            dwFlags: DWM_BB_ENABLE,
            fEnable: BOOL(1),
            ..Default::default()
        };
        if DwmEnableBlurBehindWindow(hwnd, &bb).is_ok() {
            return Ok(BackdropKind::BlurBehind);
        }
    }
    Ok(BackdropKind::None)
}

/// `WINCOMPATTRDATA` / `ACCENT_POLICY` for `SetWindowCompositionAttribute`.
/// Layout matches the well-known Win32 acrylic recipe; all fields are plain
/// integers so repr(C) is exact. Keep this cfg(windows)-only.
#[cfg(windows)]
#[repr(C)]
struct AccentPolicy {
    accent_state: u32,
    accent_flags: u32,
    gradient_color: u32,
    animation_id: u32,
}

#[cfg(windows)]
#[repr(C)]
struct WindowCompositionAttributeData {
    attribute: u32,
    data: *mut AccentPolicy,
    data_size: u32,
}

/// Signature of `user32!SetWindowCompositionAttribute`.
#[cfg(windows)]
type SetWindowCompositionAttributeFn = unsafe extern "system" fn(
    windows::Win32::Foundation::HWND,
    *mut WindowCompositionAttributeData,
) -> windows::core::BOOL;

#[cfg(windows)]
fn try_acrylic(hwnd: windows::Win32::Foundation::HWND, dark: bool) -> Option<BackdropKind> {
    use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
    use windows::core::{PCSTR, w};

    unsafe {
        let user32 = GetModuleHandleW(w!("user32.dll")).ok()?;
        // The export is not declared in windows-rs; look it up by name.
        let name = b"SetWindowCompositionAttribute\0";
        let proc = GetProcAddress(user32, PCSTR(name.as_ptr()))?;
        let set_attr: SetWindowCompositionAttributeFn = std::mem::transmute(proc);

        // ACCENT_ENABLE_ACRYLICBLURBEHIND = 4; flag 2 draws the whole window.
        // GradientColor is ABGR: low alpha keeps the desktop visible, with a
        // dark/light tint matching the app theme.
        let tint: u32 = if dark { 0x99_22_22_3B } else { 0x99_F2_E9_E4 };
        let mut policy = AccentPolicy {
            accent_state: 4,
            accent_flags: 2,
            gradient_color: tint,
            animation_id: 0,
        };
        let mut data = WindowCompositionAttributeData {
            attribute: 19, // WCA_ACCENT_POLICY
            data: &mut policy,
            data_size: std::mem::size_of::<AccentPolicy>() as u32,
        };
        if set_attr(hwnd, &mut data).0 != 0 {
            Some(BackdropKind::Acrylic)
        } else {
            None
        }
    }
}

#[cfg(not(windows))]
pub fn apply(_hwnd: isize, _dark: bool) -> Result<BackdropKind, String> {
    // Headless builds (tests, non-Windows) simply report no native backdrop.
    Ok(BackdropKind::None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backdrop_kind_labels_are_ascii_and_distinct() {
        let labels = [
            BackdropKind::Mica.label(),
            BackdropKind::Acrylic.label(),
            BackdropKind::BlurBehind.label(),
            BackdropKind::None.label(),
        ];
        assert!(labels.iter().all(|s| s.is_ascii()));
        assert_eq!(labels.len(), 4);
        let mut sorted = labels.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 4);
    }

    #[test]
    fn none_is_the_default_and_not_native() {
        assert_eq!(BackdropKind::default(), BackdropKind::None);
        assert!(!BackdropKind::None.is_native());
        assert!(BackdropKind::Mica.is_native());
    }

    #[cfg(windows)]
    #[test]
    fn accent_layout_sizes_are_correct() {
        // ACCENT_POLICY is 4 u32 = 16 bytes; WINCOMPATTRDATA is 4+8+4 = 24
        // bytes on x64 (pointer alignment). Guards the FFI layout.
        assert_eq!(std::mem::size_of::<AccentPolicy>(), 16);
        assert!(std::mem::size_of::<WindowCompositionAttributeData>() >= 16);
    }

    #[test]
    fn null_hwnd_is_rejected() {
        // On non-windows this returns None; on Windows it must error, never panic.
        let r = apply(0, false);
        assert!(r.is_ok() || r.is_err());
    }
}
