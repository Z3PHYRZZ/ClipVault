//! Single-instance guard (named mutex) and "show existing window" signal.
//!
//! Windows limitation: two instances would each register a clipboard listener
//! and a global hotkey for the same combo, racing for clipboard ownership. A
//! named mutex is the standard, kernel-lifetime solution. `ERROR_ALREADY_EXISTS`
//! means a first instance is running — the second should signal it and exit.

#[cfg(windows)]
mod imp {
    use crate::platform::PlatformError;
    use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, HANDLE, LPARAM, WPARAM};
    use windows::Win32::System::Threading::CreateMutexW;
    use windows::Win32::UI::WindowsAndMessaging::{
        HWND_BROADCAST, PostMessageW, RegisterWindowMessageW,
    };
    use windows::core::w;

    /// Holds the named mutex for the process lifetime.
    pub struct SingletonGuard {
        handle: HANDLE,
    }

    // SAFETY: a Win32 mutex HANDLE is process-wide and safe to move between threads.
    unsafe impl Send for SingletonGuard {}

    impl Drop for SingletonGuard {
        fn drop(&mut self) {
            let _ = unsafe { CloseHandle(self.handle) };
        }
    }

    /// Returns `Ok(None)` when another instance already holds the mutex.
    pub fn try_acquire() -> Result<Option<SingletonGuard>, PlatformError> {
        // CreateMutexW(SECURITY_ATTRIBUTES*, BOOL initial_owner, PCWSTR name).
        let handle = unsafe { CreateMutexW(None, true, w!("ClipVaultSingleton")) }
            .map_err(|e| PlatformError::Windows(format!("CreateMutexW: {e}")))?;
        let err = unsafe { windows::Win32::Foundation::GetLastError() };
        if err == ERROR_ALREADY_EXISTS {
            // Ownership was NOT granted (we asked for it, but the existing owner
            // keeps it); close our handle and report the duplicate instance.
            let _ = unsafe { CloseHandle(handle) };
            return Ok(None);
        }
        Ok(Some(SingletonGuard { handle }))
    }

    /// Ask an already-running instance to show its window.
    ///
    /// Practical limitation: a message-only window does not receive HWND_BROADCAST
    /// messages automatically; the running instance must register a listener for
    /// this registered message. The integrator wires that up. This call is still
    /// safe and useful (it reaches top-level windows) and never fails hard.
    pub fn signal_existing() -> Result<(), PlatformError> {
        let msg = unsafe { RegisterWindowMessageW(w!("ClipVaultShow")) };
        if msg == 0 {
            return Err(PlatformError::Other("RegisterWindowMessageW failed".into()));
        }
        unsafe {
            PostMessageW(Some(HWND_BROADCAST), msg, WPARAM(0), LPARAM(0))
                .map_err(|e| PlatformError::Windows(format!("PostMessageW: {e}")))?;
        }
        Ok(())
    }
}

#[cfg(not(windows))]
mod imp {
    use crate::platform::PlatformError;

    pub struct SingletonGuard;

    pub fn try_acquire() -> Result<Option<SingletonGuard>, PlatformError> {
        Ok(Some(SingletonGuard))
    }

    pub fn signal_existing() -> Result<(), PlatformError> {
        Ok(())
    }
}

pub use imp::{SingletonGuard, signal_existing, try_acquire};
