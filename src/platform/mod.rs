//! Platform integration. All `windows-rs` usage is cfg-gated and lives here.
pub mod backdrop;
pub mod clipboard;
pub mod clipboard_ext;
pub mod dib;
pub mod format;
pub mod handles;
pub mod hdrop;
pub mod hotkeys_full;
pub mod icon;
pub mod listener_full;
pub mod singleton;
pub mod source;
pub mod startup;
pub mod tray_full;
pub mod winutil;

use crate::workers::WorkerEvent;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("windows: {0}")]
    Windows(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

/// OS clipboard access trait. Implemented by the Windows backend.
/// `read()` returns the payload plus best-effort provenance (captured while the
/// clipboard is still open, since `GetClipboardData`/`GetClipboardOwner` require it).
pub trait ClipboardAccess: Send {
    fn read(&self) -> Result<Option<crate::workers::ClipCapture>, PlatformError>;
    fn write_text(&self, s: &str) -> Result<(), PlatformError>;
    fn write_image(&self, png: &[u8]) -> Result<(), PlatformError>;
    fn write_files(&self, paths: &[String]) -> Result<(), PlatformError>;
}

/// A parsed global hotkey combination (Win32 MOD_* flags + virtual key).
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct HotkeySpec {
    pub modifiers: u32,
    pub vk: u32,
}

impl HotkeySpec {
    /// Parse a display string like "Ctrl+Shift+V" into MOD_* + VK.
    pub fn parse(s: &str) -> Self {
        let mut modifiers = 0u32;
        let mut vk = 0u32;
        for part in s.split('+') {
            match part.trim().to_ascii_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= 0x0002, // MOD_CONTROL
                "shift" => modifiers |= 0x0004,            // MOD_SHIFT
                "alt" => modifiers |= 0x0001,              // MOD_ALT
                "win" | "super" => modifiers |= 0x0008,    // MOD_WIN
                key if key.len() == 1 => {
                    vk = key.chars().next().unwrap_or('V').to_ascii_uppercase() as u32;
                }
                "v" => vk = 0x56,
                "c" => vk = 0x43,
                _ => {}
            }
        }
        if vk == 0 {
            vk = 0x56; // default 'V'
        }
        if modifiers == 0 {
            modifiers = 0x0002 | 0x0004; // Ctrl+Shift
        }
        Self { modifiers, vk }
    }

    pub fn display(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if self.modifiers & 0x0002 != 0 {
            parts.push("Ctrl".into());
        }
        if self.modifiers & 0x0004 != 0 {
            parts.push("Shift".into());
        }
        if self.modifiers & 0x0001 != 0 {
            parts.push("Alt".into());
        }
        if self.modifiers & 0x0008 != 0 {
            parts.push("Win".into());
        }
        parts.push((self.vk as u8 as char).to_string());
        parts.join("+")
    }
}

impl Default for HotkeySpec {
    fn default() -> Self {
        Self::parse("Ctrl+Shift+V")
    }
}

/// Owns a platform worker thread. `shutdown()` posts WM_QUIT and joins.
pub struct ListenerHandle {
    thread: handles::ThreadHandle,
}
impl ListenerHandle {
    pub fn new(thread: handles::ThreadHandle) -> Self {
        Self { thread }
    }
    pub fn shutdown(&mut self) {
        self.thread.shutdown();
    }
}

pub struct HotkeyHandle {
    thread: handles::ThreadHandle,
}
impl HotkeyHandle {
    pub fn new(thread: handles::ThreadHandle) -> Self {
        Self { thread }
    }
    pub fn shutdown(&mut self) {
        self.thread.shutdown();
    }
}

pub struct TrayHandle {
    thread: handles::ThreadHandle,
}
impl TrayHandle {
    pub fn new(thread: handles::ThreadHandle) -> Self {
        Self { thread }
    }
    pub fn shutdown(&mut self) {
        self.thread.shutdown();
    }
    pub fn set_pause(&self, _paused: bool) {}
}

/// Return the platform clipboard backend (full: text/image/files).
#[cfg(windows)]
pub fn clipboard() -> Box<dyn ClipboardAccess> {
    clipboard_ext::create()
}
#[cfg(not(windows))]
pub fn clipboard() -> Box<dyn ClipboardAccess> {
    clipboard::platform_clipboard()
}

pub fn start_listener(
    tx: flume::Sender<WorkerEvent>,
    pause: Arc<AtomicBool>,
) -> Result<ListenerHandle, PlatformError> {
    listener_full::start(tx, pause)
}

pub fn start_hotkeys(
    tx: flume::Sender<WorkerEvent>,
    spec: HotkeySpec,
) -> Result<HotkeyHandle, PlatformError> {
    hotkeys_full::start(tx, spec)
}

pub fn start_tray(tx: flume::Sender<WorkerEvent>) -> Result<TrayHandle, PlatformError> {
    tray_full::start(tx)
}

pub use startup::{launch_at_startup, set_launch_at_startup};
