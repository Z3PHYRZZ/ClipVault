//! Launch-at-startup via the HKCU Run registry value.
//! Idempotent and reversible: we only ever write/remove our own "ClipVault" value.
//! `create()` opens-or-creates the Run key so first-run enable always succeeds.

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "ClipVault";

#[cfg(windows)]
pub fn set_launch_at_startup(enable: bool) -> Result<(), crate::platform::PlatformError> {
    use windows_registry::CURRENT_USER;
    let key = CURRENT_USER
        .create(RUN_KEY)
        .map_err(|e| crate::platform::PlatformError::Other(format!("open Run key: {e}")))?;
    if enable {
        let exe = std::env::current_exe().map_err(crate::platform::PlatformError::Io)?;
        let quoted = format!("\"{}\"", exe.display());
        key.set_string(VALUE_NAME, &quoted)
            .map_err(|e| crate::platform::PlatformError::Other(format!("set Run value: {e}")))?;
    } else {
        // Removing a value that does not exist is fine; treat as success.
        let _ = key.remove_value(VALUE_NAME);
    }
    Ok(())
}

#[cfg(windows)]
pub fn launch_at_startup() -> bool {
    windows_registry::CURRENT_USER
        .open(RUN_KEY)
        .and_then(|k| k.get_string(VALUE_NAME))
        .map(|s| !s.is_empty())
        .unwrap_or(false)
}

#[cfg(not(windows))]
pub fn set_launch_at_startup(_enable: bool) -> Result<(), crate::platform::PlatformError> {
    Ok(())
}

#[cfg(not(windows))]
pub fn launch_at_startup() -> bool {
    false
}
